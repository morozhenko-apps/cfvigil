//! Windows card geometry is a single bottom-anchored native state machine.
//! Persist the user's physical (x,bottom) only after an actual drag, never
//! the intermediate top-left produced by expanding/contracting a WebView.
use crate::{read_json, write_json, CARD_WIDTH, COMPACT_HEIGHT};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write as _;
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
#[serde(try_from = "serde_json::Map<String, serde_json::Value>")]
struct SavedPosition {
    anchor: Option<Anchor>,
}
// Serialized positions always include "anchor" (null before first drag).
// Missing/sequence-shaped data is corrupt, not a successfully saved
// empty anchor. Unknown object fields remain compatible with old files.
impl TryFrom<serde_json::Map<String, serde_json::Value>> for SavedPosition {
    type Error = String;
    fn try_from(mut fields: serde_json::Map<String, serde_json::Value>) -> Result<Self, Self::Error> {
        let anchor = match fields.remove("anchor") {
            Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::Object(positions)) => {
                let anchor = serde_json::from_value(serde_json::Value::Object(positions))
                    .map_err(|_| "Invalid saved Windows card anchor".to_owned())?;
                Some(anchor)
            }
            _ => return Err("Missing or invalid Windows card anchor".into()),
        };
        Ok(Self { anchor })
    }
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

// Native physical height, not requested logical content height, owns the anchor.
// Windows frame metrics and fractional DPI can change that height by a few pixels.
fn anchored_position(anchor: Anchor, measured_outer_height: u32) -> Result<PhysicalPosition<i32>, String> {
    let top = i64::from(anchor.bottom) - i64::from(measured_outer_height);
    let top = i32::try_from(top).map_err(|_| "Card position out of bounds")?;
    Ok(PhysicalPosition::new(anchor.x, top))
}

fn move_to_anchor(window: &WebviewWindow, anchor: Anchor) -> Result<(), String> {
    let measured_height = window.outer_size()
        .map_err(|_| "Unable to read Windows card size")?.height;
    let next = anchored_position(anchor, measured_height)?;
    // A redundant native move after a drag must not shift the card on first reopen.
    if window.outer_position().ok() == Some(next) { return Ok(()); }
    window.set_position(next).map_err(|_| "Unable to position Windows card".to_owned())
}

fn resize_if_needed(window: &WebviewWindow, height: f64) -> Result<(), String> {
    let scale = window.scale_factor().map_err(|_| "Unable to read Windows DPI scale")?;
    let expected = tauri::PhysicalSize::new(
        (CARD_WIDTH * scale).round() as u32,
        physical_height(window, height) as u32,
    );
    if window.inner_size().ok() == Some(expected) { return Ok(()); }
    window.set_size(tauri::LogicalSize::new(CARD_WIDTH, height))
        .map_err(|_| "Unable to resize Windows card".to_owned())
}

// GUI releases do not have a console. Geometry-only diagnostics are opt-in.
// Set CFVIGIL_UI_DEBUG=1 to trace drag and tray reopen on an actual Windows PC.
fn trace_geometry(app: &AppHandle, window: &WebviewWindow, phase: &str, anchor: Anchor) {
    if std::env::var_os("CFVIGIL_UI_DEBUG").is_none() { return; }
    let (Ok(pos), Ok(outer), Ok(inner), Ok(scale)) = (
        window.outer_position(), window.outer_size(),
        window.inner_size(), window.scale_factor(),
    ) else { return };
    let Ok(path) = crate::data_path(app, "windows-card-geometry.log") else { return };
    let reset = path.metadata().is_ok_and(|info| info.len() >= 128 * 1024);
    let Ok(mut file) = OpenOptions::new().create(true).write(true)
        .append(!reset).truncate(reset).open(path) else { return };
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    let _ = writeln!(file,
        "{timestamp} {phase} anchor=({}, {}) outer=({}, {}) {}x{} inner={}x{} dpi={scale}",
        anchor.x, anchor.bottom, pos.x, pos.y,
        outer.width, outer.height, inner.width, inner.height);
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
    trace_geometry(app, window, "show.before", anchor);
    if let Err(error) = resize_if_needed(window, COMPACT_HEIGHT)
        .and_then(|_| move_to_anchor(window, anchor)) {
        eprintln!("CFVigil Windows card restore failed: {error}");
    }
    trace_geometry(app, window, "show.after", anchor);
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
    trace_geometry(app, window, "resize.before", anchor);
    resize_if_needed(window, height)?;
    move_to_anchor(window, anchor)?;
    trace_geometry(app, window, "resize.after", anchor);
    Ok(())
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
            trace_geometry(app, &window, "drag.final", anchor);
            persist_anchor(app, anchor);
        }
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let app = window.app_handle();
    let state = app.state::<NativeCard>();
    if let WindowEvent::Resized(_) = event {
        // Late Win32 resize notifications must preserve the real bottom edge.
        if !state.dragging.load(Ordering::Acquire) {
            let anchor = *state.anchor.lock().unwrap();
            if let (Some(anchor), Some(card)) = (anchor, app.get_webview_window("main")) {
                if let Err(error) = move_to_anchor(&card, anchor) {
                    eprintln!("CFVigil Windows card resize anchor failed: {error}");
                }
                trace_geometry(app, &card, "resize.event", anchor);
            }
        }
        return;
    }
    let WindowEvent::Moved(position) = event else { return };
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
            if should_write {
                if let Some(card) = app.get_webview_window("main") {
                    trace_geometry(app, &card, "drag.moved", anchor);
                }
                persist_anchor(app, anchor);
            }
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
    fn native_outer_height_prevents_first_reopen_shift_after_drag() {
        let saved = Anchor {x: -100, bottom: 900};
        let native = anchored_position(saved, 100).unwrap();
        assert_eq!((native.x, native.y), (-100, 800));
        assert_eq!(native.y + 100, saved.bottom);
        assert_ne!(anchored_position(saved, 92).unwrap().y, native.y);
        assert_eq!(anchored_position(saved, 100).unwrap(), native);
    }

    #[test]
    fn saved_position_keeps_bottom_not_intermediate_top() {
        let value = SavedPosition {anchor: Some(Anchor {x: -40, bottom: 700})};
        let json = serde_json::to_string(&value).unwrap();
        let restored: SavedPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.anchor.unwrap().bottom, 700);
    }

    #[test]
    fn given_compact_and_expanded_heights_when_positioning_then_physical_bottom_remains_fixed() {
        for anchor in [
            Anchor {x: 80,bottom: 900},
            Anchor {x: -3000,bottom: -400},
            Anchor {x: 0,bottom: 0},
            Anchor {x: 1920,bottom: 2160},
        ] {
            let compact=anchored_position(anchor,92).unwrap();
            let expanded=anchored_position(anchor,510).unwrap();
            assert_eq!(compact.x,anchor.x);
            assert_eq!(expanded.x,anchor.x);
            assert_eq!(i64::from(compact.y)+92,i64::from(anchor.bottom));
            assert_eq!(i64::from(expanded.y)+510,i64::from(anchor.bottom));
            assert_eq!(i64::from(compact.y)-i64::from(expanded.y),418);
        }
    }

    #[test]
    fn given_exact_physical_outer_heights_when_restoring_then_no_logical_height_is_assumed() {
        let anchor=Anchor {x:-40,bottom:1000};
        for (height,top) in [
            (0,1000),(1,999),(92,908),(100,900),(510,490),(511,489),(1000,0)
        ] {
            let physical=anchored_position(anchor,height).unwrap();
            assert_eq!(physical.x,-40);
            assert_eq!(physical.y,top,"physical outer height {height}");
            assert_eq!(i64::from(physical.y)+i64::from(height),1000);
        }
        let reopened=anchored_position(anchor,100).unwrap();
        assert_eq!(reopened.y,900,
            "window is anchored by measured 100px height, never 92px CSS height");
        assert_ne!(reopened,anchored_position(anchor,92).unwrap());
    }

    #[test]
    fn given_i32_screen_edges_when_calculating_top_then_overflow_fails_closed() {
        let cases=[
            (i32::MIN,0,true),
            (i32::MIN,1,false),
            (i32::MAX,0,true),
            (i32::MAX,1,true),
            (i32::MAX,u32::MAX,true),
            (0,u32::MAX,false),
            (-1,u32::MAX,false),
            (i32::MAX,u32::MAX-1,true),
        ];
        for (bottom,height,valid) in cases {
            let result=anchored_position(Anchor{x:i32::MIN,bottom},height);
            assert_eq!(result.is_ok(),valid,
                "bottom {bottom} and height {height} must be range-checked");
            if let Ok(pos)=result {
                assert_eq!(pos.x,i32::MIN);
                assert_eq!(i64::from(pos.y)+i64::from(height),i64::from(bottom));
            }
        }
    }

    #[test]
    fn given_saved_negative_coordinates_when_restarting_then_anchor_roundtrips_without_drift() {
        for anchor in [
            Anchor {x:0,bottom:0},
            Anchor {x:-500,bottom:-200},
            Anchor {x:i32::MIN,bottom:i32::MAX},
            Anchor {x:i32::MAX,bottom:i32::MIN},
        ] {
            let json=serde_json::to_string(&SavedPosition{anchor:Some(anchor)}).unwrap();
            let restored:SavedPosition=serde_json::from_str(&json).unwrap();
            let result=restored.anchor.unwrap();
            assert_eq!((result.x,result.bottom),(anchor.x,anchor.bottom));
            let again=serde_json::to_string(
                &SavedPosition{anchor:Some(result)}).unwrap();
            assert_eq!(again,json,"restarts must not introduce coordinate drift");
        }
    }

    #[test]
    fn given_corrupt_or_legacy_saved_position_when_restoring_then_no_forged_anchor_is_accepted() {
        let none:SavedPosition=serde_json::from_str(
            r#"{"anchor":null,"legacy_top":900}"#).unwrap();
        assert!(none.anchor.is_none());
        let empty=SavedPosition::default();
        assert!(empty.anchor.is_none());
        assert!(serde_json::from_str::<SavedPosition>(r#"{}"#).is_err(),
            "missing required anchor is rejected by current persistence contract");
        for input in [
            "[]",r#"{"anchor":[]}"#,
            r#"{"anchor":{}}"#,
            r#"{"anchor":{"x":50}}"#,
            r#"{"anchor":{"bottom":900}}"#,
            r#"{"anchor":{"x":"50","bottom":900}}"#,
            r#"{"anchor":{"x":1.5,"bottom":900}}"#,
            r#"{"anchor":{"x":2147483648,"bottom":900}}"#,
            r#"{"anchor":{"x":50,"bottom":-2147483649}}"#,
            r#"{"anchor":[]}"#,
            r#"{"anchor":"50,900"}"#,
            "not-json",
        ] {
            assert!(serde_json::from_str::<SavedPosition>(input).is_err(),
                "invalid Windows persisted coordinates: {input}");
        }
    }

    #[test]
    fn given_separate_native_card_sessions_when_created_then_no_stale_drag_or_checkpoint_leaks() {
        let first=NativeCard::default();
        assert!(first.anchor.lock().unwrap().is_none());
        assert!(!first.dragging.load(Ordering::Acquire));
        assert!(first.last_written.lock().unwrap().is_none());

        *first.anchor.lock().unwrap()=Some(Anchor{x:10,bottom:900});
        first.dragging.store(true,Ordering::Release);
        *first.last_written.lock().unwrap()=Some(Instant::now());

        let restarted=NativeCard::default();
        assert!(restarted.anchor.lock().unwrap().is_none());
        assert!(!restarted.dragging.load(Ordering::Acquire));
        assert!(restarted.last_written.lock().unwrap().is_none());
        assert!(first.anchor.lock().unwrap().is_some(),
            "the first session remains independently owned");
    }

}
