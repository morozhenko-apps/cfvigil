mod cloudflare;
mod analytics;
mod incident_policy;
mod security;
#[cfg(target_os = "linux")]
mod linux_surface;
#[cfg(target_os = "linux")]
mod linux_tray;

use cloudflare::{Cloudflare, Resource};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    collections::HashSet,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
#[cfg(not(target_os = "linux"))]
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
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
    #[serde(default)]
    id: String,
    #[serde(default)]
    category: String,
    at: String,
    key: String,
    name: String,
    message: String,
    #[serde(default)]
    url: String,
}

// Preserve old events.json records, which lacked event IDs and categories.
// IDs remain stable when new items are prepended or the list is filtered.
fn classify_history(message: &str) -> &'static str {
    match message {
        "Latest Cloudflare build/deployment failed"
        | "Workers runtime error spike detected"
        | "WAF block spike detected" => "incident",
        "Workers runtime error spike cleared"
        | "WAF block spike cleared" => "recovery",
        _ => "update",
    }
}

fn normalize_history(events: &mut [HistoryEvent]) {
    for (index, event) in events.iter_mut().enumerate() {
        if event.id.is_empty() {
            event.id = format!("legacy:{}:{}", index, event.at);
        }
        if event.category.is_empty() {
            event.category = classify_history(&event.message).into();
        }
    }
}

fn new_history_event(current: &Resource, message: &str, index: usize) -> HistoryEvent {
    let at = chrono::Utc::now().to_rfc3339();
    HistoryEvent {
        id: format!("event:{}:{}", at, index),
        category: classify_history(message).into(),
        at,
        key: current.key.clone(),
        name: current.name.clone(),
        message: message.into(),
        url: current.url.clone(),
    }
}

fn history_url<'a>(events: &'a [HistoryEvent], id: &str) -> Result<&'a str, String> {
    let event = events.iter()
        .find(|event| !id.is_empty() && event.id == id)
        .ok_or("History event not found")?;
    if !trusted_dashboard_url(&event.url) {
        return Err("Invalid Cloudflare history destination".into());
    }
    Ok(&event.url)
}

#[derive(Clone, Serialize)]
struct IncidentNotice {
    key: String,
    message: String,
}

#[derive(Serialize)]
struct Snapshot {
    ready: bool,
    resources: Vec<Resource>,
    pending_incident: Option<IncidentNotice>,
}

struct AppState {
    config: Mutex<Config>,
    resources: Mutex<Vec<Resource>>,
    history: Mutex<Vec<HistoryEvent>>,
    last_notified: Mutex<HashSet<String>>,
    pending_incident: Mutex<Option<IncidentNotice>>,
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
    fs::rename(&tmp, path).map_err(|_| "Cannot persist settings".to_owned())
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
        pending_incident: state.pending_incident.lock().unwrap().clone(),
    }
}

#[tauri::command]
fn acknowledge_incident(state: State<'_, AppState>) {
    *state.pending_incident.lock().unwrap() = None;
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
    // Prevent a completed fetch for the old account from overwriting new settings.
    let _refresh_guard = state.refresh_lock.lock().await;
    let next = Config { account_id, refresh_seconds };
    write_json(&app, "settings.json", &next)?;
    *state.config.lock().unwrap() = next;
    state.ready.store(false, Ordering::Release);
    state.resources.lock().unwrap().clear();
    state.last_notified.lock().unwrap().clear();
    *state.pending_incident.lock().unwrap() = None;
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
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        let _ = linux_surface::show(app);
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn hide_card(app: AppHandle) {
    *app.state::<AppState>().pending_incident.lock().unwrap() = None;
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        let _ = linux_surface::hide(&app);
        let _ = linux_surface::request_resize(&app, 122);
        let _ = app.emit("cfvigil-card-hidden", ());
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let _ = app.emit("cfvigil-card-hidden", ());
}

#[tauri::command]
async fn set_card_expanded(app: AppHandle, expanded: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Card unavailable")?;
    let height = if expanded { EXPANDED_HEIGHT } else { COMPACT_HEIGHT };
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        return linux_surface::resize(app, height as i32).await;
    }
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

fn trusted_dashboard_url(candidate: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(candidate) else { return false };
    // URL parsers canonicalize :443 away, so insist on the literal
    // native Dashboard authority as well as validating the parsed origin.
    candidate.starts_with("https://dash.cloudflare.com/")
        && parsed.scheme() == "https"
        && parsed.host_str() == Some("dash.cloudflare.com")
        && parsed.port().is_none()
        && parsed.username().is_empty()
        && parsed.password().is_none()
}

fn open_key(state: &AppState, key: &str) -> Result<(), String> {
    let url = state.resources.lock().unwrap().iter()
        .find(|resource| resource.key == key)
        .map(|resource| resource.url.clone())
        .ok_or("Resource not found")?;
    if !trusted_dashboard_url(&url) {
        return Err("Unexpected external destination".into());
    }
    open::that(&url).map_err(|_| "Could not open browser".into())
}

#[tauri::command]
fn open_resource(state: State<'_, AppState>, key: String) -> Result<(), String> {
    open_key(&state, &key)
}

#[tauri::command]
fn open_history_event(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // The URL belongs to the selected historical event, not the current
    // resource (which may now point at a different deployment).
    let url = {
        let history = state.history.lock().unwrap();
        history_url(&history, &id)?.to_owned()
    };
    open::that(url).map_err(|_| "Could not open browser".into())
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
    let mut resources = match result {
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
    let worker_names: HashSet<String> = resources.iter()
        .filter(|resource| resource.kind == "worker")
        .map(|resource| resource.name.clone()).collect();
    // Metrics never probe the deployed application. Unavailable analytics
    // leave deployment status unchanged rather than inventing an incident.
    match analytics::query_spikes(&config.account_id, &token, &worker_names).await {
        Ok(spikes) => {
            for resource in resources.iter_mut().filter(|resource| resource.kind == "worker") {
                if let Some(spike) = spikes.get(&resource.name) {
                    let old_alert = previous.iter().find(|old| old.key == resource.key)
                        .filter(|old| old.detail.starts_with("Workers errors spiking:"));
                    resource.changed_at = old_alert
                        .map(|old| old.changed_at.clone())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
                    resource.status = "failed".into();
                    resource.detail = format!(
                        "Workers errors spiking: {} errors / {} requests ({:.1}%)",
                        spike.errors, spike.requests, spike.ratio * 100.0
                    );
                    resource.attempt_id = "runtime-error-spike".into();
                }
            }
        }
        Err(_) => {
            // A missing GraphQL response is NOT proof that an existing spike
            // recovered. Keep the affected resource grey until checked.
            for resource in resources.iter_mut().filter(|resource| resource.kind == "worker") {
                if let Some(old) = previous.iter().find(|old| old.key == resource.key)
                    .filter(|old| old.detail.starts_with("Workers errors spiking:"))
                {
                    resource.status = "unknown".into();
                    resource.detail = "Workers analytics unavailable; previous spike unverified".into();
                    resource.changed_at = old.changed_at.clone();
                    resource.attempt_id = old.attempt_id.clone();
                }
            }
        }
    }
    let security_complete = match security::detect(&config.account_id, &token).await {
        Ok(mut waf_issues) => {
            for current in &mut waf_issues {
                // Show when the ongoing spike was first detected, not the
                // latest polling time. Preserve this across API outages.
                if let Some(old) = previous.iter().find(|old| {
                    old.key == current.key && old.attempt_id == "waf-block-spike"
                }) {
                    current.changed_at = old.changed_at.clone();
                }
            }
            resources.extend(waf_issues);
            true
        }
        Err(_) => {
            // WAF analytics outage cannot resolve a previously active alert.
            for old in previous.iter().filter(|r| r.kind == "zone") {
                let mut stale = old.clone();
                stale.status = "unknown".into();
                stale.detail = "WAF analytics unavailable; previous alert unverified".into();
                resources.push(stale);
            }
            false
        }
    };
    resources.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    state.ready.store(true, Ordering::Release);
    *state.resources.lock().unwrap() = resources.clone();
    let _ = app.emit("cfvigil-resources", &resources);
    let mut events = Vec::new();
    for current in &resources {
        let old = previous.iter().find(|entry| entry.key == current.key);
        // Only zone-level WAF incidents alert on first observation. Pre-existing
        // Workers/Pages deployment attempts do not spam at initial launch.
        let message = match old {
            Some(old) => incident_policy::describe_transition(old, current),
            None => incident_policy::initial_security_event(current),
        };
        let Some(message) = message else { continue; };
        if message == "Workers runtime error spike cleared" {
            state.last_notified.lock().unwrap().remove(
                &format!("{}:runtime-spike", current.key)
            );
        }
        let notification = incident_policy::notification_identity(current, message);
        if let Some(identity) = notification.as_ref() {
            // Reconnects and transient API errors must not re-alert an
            // already acknowledged attempt as if it were a new failure.
            if !state.last_notified.lock().unwrap().insert(identity.clone()) {
                continue;
            }
        }
        events.push(new_history_event(current, message, events.len()));
        if notification.is_some() {
            let notice = IncidentNotice {
                key: current.key.clone(),
                message: format!("{}: {}", current.name, message),
            };
            *state.pending_incident.lock().unwrap() = Some(notice.clone());
            show_card(&app);
            let _ = app.emit("cfvigil-incident", &notice);
        }
    }
    if security_complete {
        for old in previous.iter().filter(|r| r.kind == "zone" && r.attempt_id == "waf-block-spike") {
            if !resources.iter().any(|r| r.key == old.key) {
                state.last_notified.lock().unwrap().remove(&format!("{}:waf-spike", old.key));
                events.push(new_history_event(old, "WAF block spike cleared", events.len()));
            }
        }
    }
    if !events.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.splice(0..0, events);
        history.truncate(500);
        let _ = write_json(&app, "events.json", &*history);
        let _ = app.emit("cfvigil-history-changed", ());
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
                history: Mutex::new({
                    let mut events: Vec<HistoryEvent> = read_json(&handle, "events.json");
                    normalize_history(&mut events);
                    events
                }),
                last_notified: Mutex::new(HashSet::new()),
                pending_incident: Mutex::new(None),
                ready: AtomicBool::new(false),
                refresh_lock: tokio::sync::Mutex::new(()),
            });
            #[cfg(target_os = "linux")]
            {
                linux_surface::install(app)?;
                linux_tray::install(app)?;
            }
            #[cfg(not(target_os = "linux"))]
            {
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
            }
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
            acknowledge_incident,
            get_history,
            hide_card,
            set_card_expanded,
            open_resource,
            open_history_event,
        ])
        .run(tauri::generate_context!())
        .expect("CFVigil startup failed");
}

#[cfg(test)]
mod security_tests {
    use super::trusted_dashboard_url;

    #[test]
    fn migrates_legacy_json_history_and_opens_exact_attempt() {
        let json = r#"[{"at":"2026-10-23T10:00:00Z","key":"worker:alpha",
            "name":"alpha","message":"Latest Cloudflare build/deployment failed",
            "url":"https://dash.cloudflare.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/workers/services/view/alpha/production/builds/11111111-1111-4111-8111-111111111111"},
            {"at":"2026-10-23T10:01:00Z","key":"worker:alpha",
            "name":"alpha","message":"Latest Cloudflare deployment succeeded",
            "url":"https://dash.cloudflare.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/workers/services/view/alpha/production/builds/22222222-2222-4222-8222-222222222222"}]"#;
        let mut history: Vec<super::HistoryEvent> = serde_json::from_str(json).unwrap();
        super::normalize_history(&mut history);
        assert_eq!(history.len(), 2);
        assert_ne!(history[0].id, history[1].id);
        assert_eq!(history[0].category, "incident");
        assert_eq!(history[1].category, "update");
        let first = super::history_url(&history, &history[0].id).unwrap();
        assert!(first.ends_with("/11111111-1111-4111-8111-111111111111"));
        let second = super::history_url(&history, &history[1].id).unwrap();
        assert!(second.ends_with("/22222222-2222-4222-8222-222222222222"));
        assert!(super::history_url(&history, "missing").is_err());
        assert_eq!(serde_json::to_value(&history).unwrap()[0]["id"], history[0].id);
    }

    #[test]
    fn historical_link_rejects_attacker_host_and_stores_recovery_kind() {
        let event = super::HistoryEvent {
            id: "event:1".into(), category: "incident".into(),
            at: "2026-10-23T10:00:00Z".into(), key: "worker:alpha".into(),
            name: "alpha".into(), message: "Failed".into(),
            url: "https://dash.cloudflare.com.evil.example/a".into(),
        };
        assert!(super::history_url(&[event], "event:1").is_err());
        let resource = super::cloudflare::Resource {
            key: "zone:example".into(), name: "example.net".into(), kind: "zone".into(),
            status: "failed".into(), changed_at: "".into(), attempt_id: "waf-block-spike".into(),
            url: "https://dash.cloudflare.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/example.net/security/analytics/events".into(),
            detail: "".into(),
        };
        let recovery = super::new_history_event(&resource, "WAF block spike cleared", 0);
        assert_eq!(recovery.category, "recovery");
        assert!(!recovery.id.is_empty());
    }

    #[test]
    fn only_exact_cloudflare_dashboard_host_is_accepted() {
        assert!(trusted_dashboard_url("https://dash.cloudflare.com/0123/workers"));
        assert!(trusted_dashboard_url("https://dash.cloudflare.com/0123/example.com/security/analytics/events?action=block"));
        assert!(!trusted_dashboard_url("http://dash.cloudflare.com/workers"));
        assert!(!trusted_dashboard_url("https://dash.cloudflare.com.evil.example/path"));
        assert!(!trusted_dashboard_url("https://dash.cloudflare.com@evil.example/path"));
        assert!(!trusted_dashboard_url("https://evil@dash.cloudflare.com/path"));
        assert!(!trusted_dashboard_url("https://dash.cloudflare.com:8443/path"));
        assert!(!trusted_dashboard_url("https://dash.cloudflare.com:443/path"));
        assert!(!trusted_dashboard_url("https://example.com/dash.cloudflare.com"));
        assert!(!trusted_dashboard_url("javascript:alert(1)"));
        assert!(!trusted_dashboard_url(""));
    }
}
