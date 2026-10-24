//! Bounded WebKitGTK Settings surface; keeps the browser's natural height
//! from forcing a taller native window. Unlike the Layer Shell card, this is
//! the existing regular Tauri window and remains WM-managed and movable.
use gtk::gdk::{Geometry, Gravity, WindowHints};
use gtk::prelude::*;
use std::{cell::RefCell, sync::atomic::{AtomicBool, AtomicI32, Ordering}};
use tauri::{App, AppHandle, Manager};

const WIDTH: i32 = 460;
const INITIAL_HEIGHT: i32 = 660;
static INSTALLED: AtomicBool = AtomicBool::new(false);
static HEIGHT: AtomicI32 = AtomicI32::new(INITIAL_HEIGHT);
thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = const { RefCell::new(None) };
    static VIEWPORT: RefCell<Option<gtk::ScrolledWindow>> = const { RefCell::new(None) };
    static CONTENT: RefCell<Option<gtk::Box>> = const { RefCell::new(None) };
}

fn apply_size(window: &gtk::ApplicationWindow, height: i32) {
    let size = Geometry::new(
        WIDTH, height, WIDTH, height, WIDTH, height,
        1, 1, 0.0, 0.0, Gravity::NorthWest,
    );
    window.set_geometry_hints(None::<&gtk::Widget>, Some(&size),
        WindowHints::MIN_SIZE | WindowHints::MAX_SIZE);
    window.set_default_size(WIDTH, height);
    window.resize(WIDTH, height);
}

fn change_height(window: &gtk::ApplicationWindow, height: i32) {
    let previous = HEIGHT.load(Ordering::Acquire);
    if previous == height { return; }
    VIEWPORT.with(|slot| {
        if let Some(viewport) = slot.borrow().as_ref() {
            // GTK validates each setter immediately, min must never exceed max.
            if height > previous {
                viewport.set_max_content_height(height);
                viewport.set_min_content_height(height);
            } else {
                viewport.set_min_content_height(height);
                viewport.set_max_content_height(height);
            }
            viewport.set_size_request(WIDTH, height);
        }
    });
    CONTENT.with(|slot| {
        if let Some(content) = slot.borrow().as_ref() {
            content.set_size_request(WIDTH, height);
        }
    });
    apply_size(window, height);
    HEIGHT.store(height, Ordering::Release);
    if std::env::var_os("CFVIGIL_UI_DEBUG").is_some() {
        eprintln!(
            "CFVigil Settings GTK: requested={height} allocated={}x{}",
            window.allocated_width(), window.allocated_height()
        );
    }
}

pub fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let view = app.get_webview_window("settings")
        .ok_or("CFVigil Settings window missing")?;
    let window = view.gtk_window()?;
    let content = view.default_vbox()?;

    let _ = view.hide();
    window.remove(&content);
    // The Tauri/WebKit VBox can demand ~774px even when content is ~655px.
    // A bounded GtkScrolledWindow clips that natural minimum without
    // inventing extra space. WebView's own body handles long-locale scrolling.
    let viewport = gtk::ScrolledWindow::new(
        None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    viewport.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
    viewport.set_propagate_natural_height(false);
    viewport.set_propagate_natural_width(false);
    viewport.set_min_content_height(INITIAL_HEIGHT);
    viewport.set_max_content_height(INITIAL_HEIGHT);
    viewport.set_min_content_width(WIDTH);
    viewport.set_max_content_width(WIDTH);
    viewport.set_size_request(WIDTH, INITIAL_HEIGHT);
    content.set_size_request(WIDTH, INITIAL_HEIGHT);
    viewport.add(&content);
    window.add(&viewport);
    window.set_decorated(false);
    window.set_resizable(true); // Internal resize only; min/max are exact.
    apply_size(&window, INITIAL_HEIGHT);

    if std::env::var_os("CFVIGIL_UI_DEBUG").is_some() {
        window.connect_size_allocate(|_, allocation| {
            eprintln!(
                "CFVigil Settings allocation: requested={} actual={}x{}",
                HEIGHT.load(Ordering::Acquire),
                allocation.width(), allocation.height()
            );
        });
    }
    WINDOW.with(|slot| *slot.borrow_mut() = Some(window));
    VIEWPORT.with(|slot| *slot.borrow_mut() = Some(viewport));
    CONTENT.with(|slot| *slot.borrow_mut() = Some(content));
    INSTALLED.store(true, Ordering::Release);
    Ok(())
}

pub fn enabled() -> bool { INSTALLED.load(Ordering::Acquire) }

pub fn fit(app: &AppHandle, height: i32) -> Result<(), String> {
    let height = height.clamp(370, 740);
    app.run_on_main_thread(move || {
        WINDOW.with(|slot| {
            if let Some(window) = slot.borrow().as_ref() {
                change_height(window, height);
            }
        });
    }).map_err(|_| "Settings GTK event loop unavailable".into())
}
