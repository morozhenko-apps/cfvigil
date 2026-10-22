mod cloudflare;

use cloudflare::{Cloudflare, Resource};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

const KEYRING_SERVICE: &str = "com.morozhenko.cfvigil.cloudflare";
const KEYRING_USER: &str = "cloudflare-api-token-v1";
const CARD_WIDTH: f64 = 390.0;
const COMPACT_HEIGHT: f64 = 122.0;
const EXPANDED_HEIGHT: f64 = 510.0;

fn default_refresh() -> u64 {
    180
}

#[derive(Clone, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    account_id: String,
    #[serde(default = "default_refresh")]
    refresh_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { account_id: String::new(), refresh_seconds: default_refresh() }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct HistoryEvent {
    at: String,
    key: String,
    name: String,
    message: String,
}

#[derive(Serialize)]
struct Snapshot {
    ready: bool,
    resources: Vec<Resource>,
}

struct AppState {
    config: Mutex<Config>,
    resources: Mutex<Vec<Resource>>,
    history: Mutex<Vec<HistoryEvent>>,
    ready: AtomicBool,
    refresh_lock: tokio::sync::Mutex<()>,
}

fn data_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|_| "Config path unavailable")?;
    fs::create_dir_all(&dir).map_err(|_| "Cannot create config directory")?;
    Ok(dir.join(filename))
}

fn read_json<T: for<'de> Deserialize<'de> + Default>(app: &AppHandle, name: &str) -> T {
    data_path(app, name)
        .ok()
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn write_json<T: Serialize>(app: &AppHandle, name: &str, data: &T) -> Result<(), String> {
    let path = data_path(app, name)?;
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(data).map_err(|_| "Cannot serialize settings")?;
    fs::write(&tmp, bytes).map_err(|_| "Cannot write settings")?;
    fs::rename(&tmp, path).map_err(|_| "Cannot persist settings")
}

fn token_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|_| "Operating system credential store unavailable".into())
}

fn load_token() -> Result<String, String> {
    token_entry()?
        .get_password()
        .map_err(|_| "Cloudflare API token not set or keychain locked".into())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn get_snapshot(state: State<'_, AppState>) -> Snapshot {
    Snapshot {
        ready: state.ready.load(Ordering::Acquire),
        resources: state.resources.lock().unwrap().clone(),
    }
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Vec<HistoryEvent> {
    state.history.lock().unwrap().clone()
}

#[tauri::command]
async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    account_id: String,
    token: Option<String>,
    refresh_seconds: u64,
) -> Result<(), String> {
    if account_id.len() != 32 || !account_id.bytes().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("Account ID must be 32 hexadecimal characters".into());
    }
    if ![60, 180, 300, 600].contains(&refresh_seconds) {
        return Err("Unsupported refresh interval".into());
    }
    if let Some(token) = token {
        if token.len() < 20 || token.len() > 4096 || token.chars().any(char::is_whitespace) {
            return Err("Invalid API token format".into());
        }
        tauri::async_runtime::spawn_blocking(move || {
            token_entry()?.set_password(&token).map_err(|_| "Could not store token in OS keychain".to_string())
        }).await.map_err(|_| "Credential service unavailable")??;
    } else {
        tauri::async_runtime::spawn_blocking(load_token)
            .await.map_err(|_| "Credential service unavailable")??;
    }
    let next = Config { account_id, refresh_seconds };
    write_json(&app, "settings.json", &next)?;
    *state.config.lock().unwrap() = next;
    state.ready.store(false, Ordering::Release);
    state.resources.lock().unwrap().clear();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move { refresh_once(handle).await; });
    Ok(())
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn show_card(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn hide_card(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn set_card_expanded(app: AppHandle, expanded: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Card unavailable")?;
    let height = if expanded { EXPANDED_HEIGHT } else { COMPACT_HEIGHT };
    let old_position = window.outer_position().ok();
    let old_size = window.outer_size().ok();
    window
        .set_size(tauri::LogicalSize::new(CARD_WIDTH, height))
        .map_err(|_| "Cannot resize card")?;
    // Preserve the bottom edge when the window manager permits repositioning.
    if let (Some(position), Some(size)) = (old_position, old_size) {
        let scale = window.scale_factor().unwrap_or(1.0);
        let bottom = i64::from(position.y) + i64::from(size.height);
        let next_top = bottom - (height * scale).round() as i64;
        if let Ok(top) = i32::try_from(next_top) {
            let _ = window.set_position(tauri::PhysicalPosition::new(position.x, top));
        }
    }
    Ok(())
}

fn open_key(state: &AppState, key: &str) -> Result<(), String> {
    let url = state.resources.lock().unwrap().iter()
        .find(|resource| resource.key == key)
        .map(|resource| resource.url.clone())
        .ok_or("Resource not found")?;
    if !url.starts_with("https://dash.cloudflare.com/") {
        return Err("Unexpected external destination".into());
    }
    open::that(&url).map_err(|_| "Could not open browser".into())
}

#[tauri::command]
fn open_resource(state: State<'_, AppState>, key: String) -> Result<(), String> {
    open_key(&state, &key)
}

#[tauri::command]
fn open_history_resource(state: State<'_, AppState>, key: String) -> Result<(), String> {
    open_key(&state, &key)
}

async fn refresh_once(app: AppHandle) {
    let state = app.state::<AppState>();
    let _refresh_guard = state.refresh_lock.lock().await;
    let config = state.config.lock().unwrap().clone();
    if config.account_id.is_empty() {
        return;
    }
    let token = match tauri::async_runtime::spawn_blocking(load_token).await {
        Ok(Ok(token)) => token,
        _ => {
            let _ = app.emit("cfvigil-sync-error", "Cloudflare credentials unavailable");
            return;
        }
    };
    let result = match Cloudflare::new() {
        Ok(api) => api.discover(&config.account_id, &token).await,
        Err(message) => Err(message),
    };
    let resources = match result {
        Ok(resources) => resources,
        Err(message) => {
            // Stale resources must not be presented as verified green.
            let mut stale = state.resources.lock().unwrap();
            for item in stale.iter_mut() {
                item.status = "unknown".into();
                item.detail = "Cloudflare data is stale".into();
            }
            let _ = app.emit("cfvigil-resources", stale.clone());
            let _ = app.emit("cfvigil-sync-error", message);
            return;
        }
    };

    let previous = state.resources.lock().unwrap().clone();
    let initialized = state.ready.swap(true, Ordering::AcqRel);
    *state.resources.lock().unwrap() = resources.clone();
    let _ = app.emit("cfvigil-resources", &resources);
    if !initialized {
        return;
    }

    let mut events = Vec::new();
    for current in &resources {
        let old = previous.iter().find(|entry| entry.key == current.key);
        // A resource identity alone is not an event. A new build ID or state transition is.
        if let Some(old) = old {
            if old.attempt_id == current.attempt_id && old.status == current.status {
                continue;
            }
        } else {
            continue;
        }
        let message = match current.status.as_str() {
            "failed" => "Latest Cloudflare build/deployment failed",
            "deploying" => "New Cloudflare build/deployment started",
            "healthy" => "Latest Cloudflare deployment succeeded",
            _ => continue,
        };
        events.push(HistoryEvent {
            at: chrono::Utc::now().to_rfc3339(),
            key: current.key.clone(),
            name: current.name.clone(),
            message: message.into(),
        });
        if current.status == "failed" {
            show_card(&app);
            let _ = app.emit("cfvigil-incident", serde_json::json!({
                "key":current.key,
                "message":format!("{}: latest deployment failed", current.name),
            }));
        }
    }
    if !events.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.splice(0..0, events);
        history.truncate(500);
        let _ = write_json(&app, "events.json", &*history);
    }
}

fn run_polling(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            refresh_once(app.clone()).await;
            let interval = app.state::<AppState>().config.lock().unwrap().refresh_seconds;
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            app.manage(AppState {
                config: Mutex::new(read_json(&handle, "settings.json")),
                resources: Mutex::new(Vec::new()),
                history: Mutex::new(read_json(&handle, "events.json")),
                ready: AtomicBool::new(false),
                refresh_lock: tokio::sync::Mutex::new(()),
            });
            let open = MenuItem::with_id(app, "open", "Show Card", true, None::<&str>)?;
            let history = MenuItem::with_id(app, "history", "Event History", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &history, &settings, &about, &quit])?;
            let icon = app.default_window_icon().cloned()
                .unwrap_or_else(|| Image::new_owned(vec![50,50,50,255],1,1));
            TrayIconBuilder::with_id("cfvigil")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => show_card(app),
                    "history" => show_window(app, "history"),
                    "settings" => show_window(app, "settings"),
                    "about" => show_window(app, "about"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        show_card(tray.app_handle());
                    }
                })
                .build(app)?;
            run_polling(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_snapshot,
            get_history,
            hide_card,
            set_card_expanded,
            open_resource,
            open_history_resource,
        ])
        .run(tauri::generate_context!())
        .expect("CFVigil startup failed");
}
