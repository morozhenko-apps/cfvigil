//! Windows card geometry is a single bottom-anchored native state machine.
//! Persist the user's physical (x,bottom) only after an actual drag, never
//! the intermediate top-left produced by expanding/contracting a WebView.
use crate::{read_json, write_json, CARD_WIDTH, COMPACT_HEIGHT};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};
use tauri::{App, AppHandle, Manager, PhysicalPosition, WebviewWindow, Window, WindowEvent};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct Anchor {
    x: i32,
    bottom: i32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct SavedPosition {
    anchor: Option<Anchor>,
}

#[derive(Default)]
struct NativeCard {
    anchor: Mutex<Option<Anchor>>,
    dragging: AtomicBool,
    last_written: Mutex<Option<Instant>>,
}

fn physical_height(window: &WebviewWindow, logical_height: f64) -> i32 {
    (logical_height * window.scale_factor().unwrap_or(1.0)).round() as i32
}

fn current_anchor(window: &WebviewWindow) -> Option<Anchor> {
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let bottom = i64::from(pos.y) + i64::from(size.height);
    Some(Anchor {x: pos.x, bottom: i32::try_from(bottom).ok()?})
}

fn default_anchor(window: &WebviewWindow) -> Anchor {
    // Use the actual Windows work area, excluding the taskbar, instead of
    // placing an invisible Tauri window at its usual top-left default.
    let monitor = window.current_monitor().ok().flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let scale = window.scale_factor().unwrap_or(1.0);
        let width = (CARD_WIDTH * scale).round() as i64;
        let right = i64::from(area.position.x) + i64::from(area.size.width);
        let bottom = i64::from(area.position.y) + i64::from(area.size.height);
        return Anchor {
            x: (right - width - 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32,
            bottom: (bottom - 12).clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        };
    }
    current_anchor(window).unwrap_or(Anchor {x: 20, bottom: 300})
}

fn move_to_anchor(window: &WebviewWindow, anchor: Anchor, height: f64) -> Result<(), String> {
    let top = i64::from(anchor.bottom) - i64::from(physical_height(window, height));
    let top = i32::try_from(top).map_err(|_| "Card position out of bounds")?;
    window.set_position(PhysicalPosition::new(anchor.x, top))
        .map_err(|_| "Unable to position Windows card".to_owned())
}

pub fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = read_json::<SavedPosition>(app.handle(), "windows-card-position.json")
        .map_err(std::io::Error::other)?;
    let state = NativeCard {
        anchor: Mutex::new(loaded.value.anchor),
        dragging: AtomicBool::new(false),
        last_written: Mutex::new(None),
    };
    app.manage(state);
    Ok(())
}

pub fn restore_for_show(app: &AppHandle, window: &WebviewWindow) {
    // On Windows hiding a previously expanded WebView does not reset its
    // native size. Normalize the two before showing, without repositioning
    // relative to the old window's TOP edge.
    let state = app.state::<NativeCard>();
    let anchor = {
        let mut saved = state.anchor.lock().unwrap();
        *saved.get_or_insert_with(|| default_anchor(window))
    };
    let _ = window.set_size(tauri::LogicalSize::new(CARD_WIDTH, COMPACT_HEIGHT));
    if let Err(error) = move_to_anchor(window, anchor, COMPACT_HEIGHT) {
        eprintln!("CFVigil Windows card restore failed: {error}");
    }
}

pub fn resize_anchored(
    app: &AppHandle,
    window: &WebviewWindow,
    height: f64,
) -> Result<(), String> {
    let state = app.state::<NativeCard>();
    let anchor = {
        let mut saved = state.anchor.lock().unwrap();
        *saved.get_or_insert_with(|| {
            current_anchor(window).unwrap_or_else(|| default_anchor(window))
        })
    };
    window.set_size(tauri::LogicalSize::new(CARD_WIDTH, height))
        .map_err(|_| "Unable to resize Windows card".to_owned())?;
    move_to_anchor(window, anchor, height)
}

pub fn collapse_hidden(app: &AppHandle, window: &WebviewWindow) {
    if let Err(error) = resize_anchored(app, window, COMPACT_HEIGHT) {
        eprintln!("CFVigil Windows compact restore failed: {error}");
    }
}

pub fn begin_drag(app: &AppHandle) {
    let state = app.state::<NativeCard>();
    *state.last_written.lock().unwrap() = None;
    state.dragging.store(true, Ordering::Release);
}

fn persist_anchor(app: &AppHandle, anchor: Anchor) {
    // No credentials, only screen coordinates. Atomic private JSON storage.
    if write_json(app, "windows-card-position.json",
        &SavedPosition {anchor: Some(anchor)}).is_err() {
        eprintln!("CFVigil Windows card position could not be saved");
    }
}

pub fn end_drag(app: &AppHandle) {
    let state = app.state::<NativeCard>();
    if !state.dragging.swap(false, Ordering::AcqRel) { return; }
    if let Some(window) = app.get_webview_window("main") {
        if let Some(anchor) = current_anchor(&window) {
            *state.anchor.lock().unwrap() = Some(anchor);
            persist_anchor(app, anchor);
        }
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::Moved(position) = event else { return };
    let app = window.app_handle();
    let state = app.state::<NativeCard>();
    if !state.dragging.load(Ordering::Acquire) { return; }
    if let Ok(size) = window.outer_size() {
        let bottom = i64::from(position.y) + i64::from(size.height);
        if let Ok(bottom) = i32::try_from(bottom) {
            let anchor = Anchor {x: position.x, bottom};
            *state.anchor.lock().unwrap() = Some(anchor);
            // A Windows system move loop can swallow WebView pointerup;
            // saving the latest observed real move guarantees restart
            // persistence even in that case. Programmatic moves are excluded.
            // A native Windows move loop can swallow the WebView pointerup.
            // Persist a bounded checkpoint while dragging, then persist the
            // exact final position from stop_card_drag when delivered.
            let now = Instant::now();
            let should_write = {
                let mut last = state.last_written.lock().unwrap();
                if last.is_none_or(|time| now.duration_since(time) >= Duration::from_millis(300)) {
                    *last = Some(now);
                    true
                } else {
                    false
                }
            };
            if should_write { persist_anchor(app, anchor); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_anchor_is_independent_of_each_view_height() {
        let a = Anchor {x: 100, bottom: 900};
        let top = |height: i32| a.bottom - height;
        assert_eq!(top(510) + 510, top(92) + 92);
        assert_eq!(top(510), 390);
        assert_eq!(top(92), 808);
    }

    #[test]
    fn saved_position_keeps_bottom_not_intermediate_top() {
        let value = SavedPosition {anchor: Some(Anchor {x: -40, bottom: 700})};
        let json = serde_json::to_string(&value).unwrap();
        let restored: SavedPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.anchor.unwrap().bottom, 700);
    }
}
