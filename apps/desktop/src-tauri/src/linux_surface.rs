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

const WIDTH: i32 = 390;
const BOTTOM_MARGIN: i32 = 46;
const RIGHT_MARGIN: i32 = 10;
static ENABLED: AtomicBool = AtomicBool::new(false);
static VISIBLE: AtomicBool = AtomicBool::new(false);
static HEIGHT: AtomicI32 = AtomicI32::new(122);

thread_local! {
    static CARD: RefCell<Option<gtk::ApplicationWindow>> = const { RefCell::new(None) };
    static CONTENT: RefCell<Option<gtk::Box>> = const { RefCell::new(None) };
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
    layer.set_layer_shell_margin(Edge::Bottom, BOTTOM_MARGIN);
    layer.set_layer_shell_margin(Edge::Right, RIGHT_MARGIN);
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_mode(KeyboardMode::OnDemand);

    CARD.with(|slot| *slot.borrow_mut() = Some(layer));
    CONTENT.with(|slot| *slot.borrow_mut() = Some(content));
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
    if height == old { return Ok(()); }
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
    // Keep bottom and right margins invariant while growing upwards.
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    window.set_layer_shell_margin(Edge::Bottom, BOTTOM_MARGIN);
    window.set_layer_shell_margin(Edge::Right, RIGHT_MARGIN);
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
    on_main(app, |window| window.show_all())
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    VISIBLE.store(false, Ordering::Release);
    on_main(app, |window| window.hide())
}
