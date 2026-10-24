//! A minimal native KDE Wayland popup adapted from RepoTicker's proven GTK3
//! Layer Shell approach. The existing Tauri WebView is reparented into a
//! bottom-right anchored layer surface before its first presentation.
use gtk::gdk::{Geometry, Gravity, WindowHints};
use gtk::prelude::*;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};
use tauri::{App, AppHandle, Manager};
use serde::{Deserialize, Serialize};

unsafe extern "C" {
    fn cfvigil_relative_init(callback: extern "C" fn(f64, f64)) -> i32;
    fn cfvigil_relative_begin() -> i32;
    fn cfvigil_relative_end();
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct CardPosition {
    bottom: i32,
    right: i32,
}
impl Default for CardPosition {
    fn default() -> Self { Self {bottom:46, right:10} }
}


const WIDTH: i32 = 390;
const BOTTOM_MARGIN: i32 = 46;
const RIGHT_MARGIN: i32 = 10;
static ENABLED: AtomicBool = AtomicBool::new(false);
static VISIBLE: AtomicBool = AtomicBool::new(false);
static HEIGHT: AtomicI32 = AtomicI32::new(122);
static DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);
static DRAG_MOVED: AtomicBool = AtomicBool::new(false);


thread_local! {
    static CARD: RefCell<Option<gtk::ApplicationWindow>> = const { RefCell::new(None) };
    static CONTENT: RefCell<Option<gtk::Box>> = const { RefCell::new(None) };
    static MARGINS: RefCell<(i32, i32)> = const { RefCell::new((46, 10)) };
    static FRACTIONAL_MOTION: RefCell<(f64, f64)> = const { RefCell::new((0.0, 0.0)) };
}

/// Relative-pointer motion belongs to GDK's native Wayland event loop.
extern "C" fn on_relative_motion(dx: f64, dy: f64) {
    if !DRAG_ACTIVE.load(Ordering::Acquire) || !VISIBLE.load(Ordering::Acquire)
        || !dx.is_finite() || !dy.is_finite() { return; }
    FRACTIONAL_MOTION.with(|state| {
        let mut pending = state.borrow_mut();
        pending.0 += dx;
        pending.1 += dy;
        let x = pending.0.trunc().clamp(-5000.0, 5000.0) as i32;
        let y = pending.1.trunc().clamp(-5000.0, 5000.0) as i32;
        pending.0 -= f64::from(x);
        pending.1 -= f64::from(y);
        if x == 0 && y == 0 { return; }
        CARD.with(|slot| {
            if let Some(card) = slot.borrow().as_ref() {
                MARGINS.with(|state| {
                    let mut m = state.borrow_mut();
                    let previous = *m;
                    m.0 = (m.0 - y).clamp(0, 5000);
                    m.1 = (m.1 - x).clamp(0, 5000);
                    if *m != previous {
                        DRAG_MOVED.store(true, Ordering::Release);
                        card.set_layer_shell_margin(Edge::Bottom, m.0);
                        card.set_layer_shell_margin(Edge::Right, m.1);
                    }
                });
            }
        });
    });
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn visible() -> bool {
    VISIBLE.load(Ordering::Acquire)
}

fn apply_geometry(window: &gtk::ApplicationWindow, height: i32) {
    let geometry = Geometry::new(
        WIDTH, height, WIDTH, height, WIDTH, height,
        1, 1, 0.0, 0.0, Gravity::SouthEast,
    );
    window.set_geometry_hints(
        None::<&gtk::Widget>,
        Some(&geometry),
        WindowHints::MIN_SIZE | WindowHints::MAX_SIZE,
    );
    window.set_default_size(WIDTH, height);
    window.resize(WIDTH, height);
}

pub fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() || !gtk_layer_shell::is_supported() {
        return Ok(());
    }
    let view = app.get_webview_window("main").ok_or("CFVigil card window missing")?;
    let original = view.gtk_window()?;
    let application = original.application().ok_or("GTK application missing")?;
    let content = view.default_vbox()?;
    let _ = view.hide();
    original.remove(&content);

    let layer = gtk::ApplicationWindow::new(&application);
    layer.set_decorated(false);
    layer.set_resizable(true);
    layer.set_app_paintable(true);
    if let Some(screen) = gtk::prelude::WidgetExt::screen(&layer) {
        if let Some(visual) = screen.rgba_visual() {
            layer.set_visual(Some(&visual));
        }
        let style = gtk::CssProvider::new();
        style.load_from_data(b"window.cfvigil-layer {background-color: transparent; background-image:none;}")?;
        gtk::StyleContext::add_provider_for_screen(
            &screen, &style, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }
    layer.style_context().add_class("cfvigil-layer");

    let viewport = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    viewport.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
    viewport.set_propagate_natural_height(false);
    viewport.set_propagate_natural_width(false);
    viewport.set_min_content_height(122);
    viewport.set_max_content_height(122);
    viewport.set_min_content_width(WIDTH);
    viewport.set_max_content_width(WIDTH);
    viewport.set_size_request(WIDTH, 122);
    content.set_size_request(WIDTH, 122);
    viewport.add(&content);
    apply_geometry(&layer, 122);
    layer.add(&viewport);
    layer.init_layer_shell();
    layer.set_namespace("cfvigil");
    layer.set_layer(Layer::Top);
    layer.set_anchor(Edge::Bottom, true);
    layer.set_anchor(Edge::Right, true);
    let stored = crate::read_json::<CardPosition>(app.handle(), "card-position.json")
        .map_err(std::io::Error::other)?;
    let position = stored.value;
    let bottom = position.bottom.clamp(0, 5000);
    let right = position.right.clamp(0, 5000);
    MARGINS.with(|state| *state.borrow_mut() = (bottom, right));
    layer.set_layer_shell_margin(Edge::Bottom, bottom);
    layer.set_layer_shell_margin(Edge::Right, right);
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_mode(KeyboardMode::OnDemand);

    CARD.with(|slot| *slot.borrow_mut() = Some(layer));
    CONTENT.with(|slot| *slot.borrow_mut() = Some(content));
    if unsafe { cfvigil_relative_init(on_relative_motion) } == 0 {
        eprintln!("CFVigil: Wayland relative-pointer initialization unavailable");
    }
    ENABLED.store(true, Ordering::Release);
    Ok(())
}

fn on_main(app: &AppHandle, callback: impl FnOnce(&gtk::ApplicationWindow) + Send + 'static) -> Result<(), String> {
    app.run_on_main_thread(move || {
        CARD.with(|slot| {
            if let Some(card) = slot.borrow().as_ref() {
                callback(card);
            }
        });
    }).map_err(|_| "GTK event loop unavailable".into())
}

fn apply_height(window: &gtk::ApplicationWindow, height: i32) -> Result<(), String> {
    let old = HEIGHT.load(Ordering::Acquire);
    // Reapply exact geometry even when the requested mode did not change:
    // WebKitGTK may raise the native layer's allocation above its CSS height.
    let widget = window.child().ok_or("Popup viewport missing")?;
    let viewport = widget.downcast::<gtk::ScrolledWindow>()
        .map_err(|_| "Unexpected popup viewport")?;
    if height > old {
        viewport.set_max_content_height(height);
        viewport.set_min_content_height(height);
    } else {
        viewport.set_min_content_height(height);
        viewport.set_max_content_height(height);
    }
    viewport.set_size_request(WIDTH, height);
    CONTENT.with(|slot| {
        if let Some(content) = slot.borrow().as_ref() {
            content.set_size_request(WIDTH, height);
        }
    });
    apply_geometry(window, height);
    viewport.queue_resize();
    window.queue_resize();
    // Keep bottom and right margins invariant while growing upwards.
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    MARGINS.with(|state| {
        let (bottom, right) = *state.borrow();
        window.set_layer_shell_margin(Edge::Bottom, bottom);
        window.set_layer_shell_margin(Edge::Right, right);
    });
    HEIGHT.store(height, Ordering::Release);
    Ok(())
}

pub fn request_resize(app: &AppHandle, height: i32) -> Result<(), String> {
    if height != 122 && height != 510 { return Err("Invalid popup height".into()); }
    on_main(app, move |window| {
        if let Err(err) = apply_height(window, height) {
            eprintln!("CFVigil popup resize: {err}");
        }
    })
}

pub async fn resize(app: AppHandle, height: i32) -> Result<(), String> {
    if height != 122 && height != 510 { return Err("Invalid popup height".into()); }
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let value = CARD.with(|slot| {
            let borrowed = slot.borrow();
            let card = borrowed.as_ref().ok_or("Popup not initialized")?;
            apply_height(card, height)
        });
        let _ = send.send(value);
    }).map_err(|_| "GTK event loop unavailable")?;
    tauri::async_runtime::spawn_blocking(move || receive.recv())
        .await.map_err(|_| "GTK resize interrupted")?
        .map_err(|_| "GTK resize confirmation failed")?
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    VISIBLE.store(true, Ordering::Release);
    on_main(app, |window| {
        // A compact tray activation must not inherit the last expanded
        // native WebKit minimum. Expansion is an explicit user action.
        if let Err(error) = apply_height(window, 122) {
            eprintln!("CFVigil compact layer geometry: {error}");
        }
        window.show_all();
    })
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    VISIBLE.store(false, Ordering::Release);
    cancel_drag(app)?;
    on_main(app, |window| window.hide())
}

/// Store only the card's position, independently of Cloudflare credentials,
/// incident history and the other user settings.
fn save_position(app: &AppHandle) -> Result<(), String> {
    let value = MARGINS.with(|slot| {
        let (bottom, right) = *slot.borrow();
        CardPosition { bottom, right }
    });
    crate::write_json(app, "card-position.json", &value)
}

fn stop_drag_native(app: &AppHandle) -> Result<(), String> {
    unsafe { cfvigil_relative_end() };
    FRACTIONAL_MOTION.with(|slot| *slot.borrow_mut() = (0.0, 0.0));
    if DRAG_MOVED.swap(false, Ordering::AcqRel) { save_position(app)?; }
    Ok(())
}

pub fn cancel_drag(app: &AppHandle) -> Result<(), String> {
    DRAG_ACTIVE.store(false, Ordering::Release);
    let handle = app.clone();
    on_main(app, move |_| {
        if let Err(err) = stop_drag_native(&handle) {
            eprintln!("CFVigil could not store card position: {err}");
        }
    })
}

pub async fn start_drag(app: &AppHandle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    on_main(app, move |_| {
        let result = if !VISIBLE.load(Ordering::Acquire) {
            Err("Card is hidden".to_owned())
        } else if unsafe { cfvigil_relative_begin() } == 1 {
            FRACTIONAL_MOTION.with(|slot| *slot.borrow_mut() = (0.0, 0.0));
            DRAG_MOVED.store(false, Ordering::Release);
            DRAG_ACTIVE.store(true, Ordering::Release);
            Ok(())
        } else {
            Err("Wayland relative-pointer-v1 is unavailable".to_owned())
        };
        let _ = tx.send(result);
    })?;
    tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await.map_err(|_| "Native drag start interrupted".to_owned())?
        .map_err(|_| "Native drag start confirmation failed".to_owned())?
}

pub async fn stop_drag(app: &AppHandle) -> Result<(), String> {
    DRAG_ACTIVE.store(false, Ordering::Release);
    let handle = app.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    on_main(app, move |_| { let _ = tx.send(stop_drag_native(&handle)); })?;
    tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await.map_err(|_| "Native drag stop interrupted".to_owned())?
        .map_err(|_| "Native drag stop confirmation failed".to_owned())?
}

