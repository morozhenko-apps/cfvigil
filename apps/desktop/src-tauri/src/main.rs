#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod cloudflare;
mod language;
mod native_messages;
use native_messages as msg;
mod api_budget;
mod analytics;
mod incident_policy;
mod incident_store;
mod polling_policy;
mod security;
mod storage;
mod resource_cache;
#[cfg(test)]
mod test_http;
#[cfg(target_os = "linux")]
mod linux_surface;
#[cfg(target_os = "linux")]
mod linux_settings;
#[cfg(target_os = "linux")]
mod linux_tray;
#[cfg(target_os = "windows")]
mod windows_card;

use cloudflare::{Cloudflare, Resource};
use language::LanguagePreference;
use api_budget::ApiBudget;
use incident_store::{IncidentJournal, NotificationWindow};
use polling_policy::poll_delay;
use resource_cache::ResourceCatalogCache;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    collections::HashSet,
    path::PathBuf,
    time::Duration,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
#[cfg(not(target_os = "linux"))]
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

const KEYRING_SERVICE: &str = "com.morozhenko.cfvigil.cloudflare";
const KEYRING_USER: &str = "cloudflare-api-token-v1";
const CARD_WIDTH: f64 = 390.0;
const COMPACT_HEIGHT: f64 = 92.0;
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
        msg::EVENT_BUILD_FAILURE
        | msg::EVENT_RUNTIME_SPIKE
        | msg::EVENT_WAF_SPIKE => "incident",
        msg::EVENT_RUNTIME_RECOVERY
        | msg::EVENT_WAF_RECOVERY => "recovery",
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
        .ok_or(msg::ERROR_HISTORY_NOT_FOUND)?;
    if !trusted_dashboard_url(&event.url) {
        return Err(msg::ERROR_HISTORY_DESTINATION.into());
    }
    Ok(&event.url)
}

#[derive(Clone, Serialize)]
struct IncidentNotice {
    key: String,
    message: String,
}

#[derive(Clone, Serialize)]
struct Snapshot {
    ready: bool,
    pending: bool,
    configured: bool,
    resources: Vec<Resource>,
    pending_incident: Option<IncidentNotice>,
    /// Nonsecret Cloudflare endpoint diagnostics, refreshed each poll.
    provider_warnings: Vec<String>,
    storage_warning: bool,
}

struct AppState {
    config: Mutex<Config>,
    language: Mutex<LanguagePreference>,
    resources: Mutex<Vec<Resource>>,
    history: Mutex<Vec<HistoryEvent>>,
    last_notified: Mutex<NotificationWindow>,
    journal: Mutex<IncidentJournal>,
    pending_incident: Mutex<Option<IncidentNotice>>,
    provider_warnings: Mutex<Vec<String>>,
    ready: AtomicBool,
    pending: AtomicBool,
    refresh_lock: tokio::sync::Mutex<()>,
    config_revision: AtomicU64,
    poll_refresh: tokio::sync::watch::Sender<u64>,
    storage_warning: bool,
}

fn data_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|_| msg::ERROR_CONFIG_PATH)?;
    fs::create_dir_all(&dir).map_err(|_| msg::ERROR_CONFIG_DIRECTORY)?;
    Ok(dir.join(filename))
}

fn read_json<T: for<'de> Deserialize<'de> + Default>(app: &AppHandle, name: &str) -> Result<storage::Loaded<T>, String> {
    storage::read_or_default(&data_path(app, name)?, storage::MAX_JSON_BYTES)
}

fn write_json<T: Serialize>(app: &AppHandle, name: &str, data: &T) -> Result<(), String> {
    storage::write_atomic(&data_path(app, name)?, data, storage::MAX_JSON_BYTES)
}

fn token_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|_| msg::ERROR_CREDENTIAL_STORE.into())
}

fn load_token() -> Result<String, String> {
    token_entry()?
        .get_password()
        .map_err(|_| msg::ERROR_TOKEN_LOCKED.into())
}

/// Check keyring presence without exposing a token string to the WebView.
/// Keep a locked or inaccessible vault distinct from an empty vault.
#[tauri::command]
async fn get_token_status() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let entry = token_entry()?;
        match entry.get_password() {
            Ok(value) => Ok(!value.is_empty()),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(_) => Err(msg::ERROR_TOKEN_LOCKED.to_owned()),
        }
    }).await.map_err(|_| msg::ERROR_CREDENTIAL_SERVICE) ?
}


#[tauri::command]
fn get_language(state: State<'_, AppState>) -> String {
    state.language.lock().unwrap().locale.clone()
}

// Locale preferences must never reset provider readiness, notification IDs
// or the one and only polling loop. They are stored separately from credentials.
fn commit_language(app: &AppHandle, state: &AppState, language: String) -> Result<(), String> {
    if !language::supported(&language) {
        return Err("Unsupported interface language".into());
    }
    let mut current = state.language.lock().unwrap();
    if current.locale == language { return Ok(()) }
    let next = LanguagePreference { locale: language.clone() };
    write_json(app, "language.json", &next)?;
    *current = next;
    drop(current);
    update_native_language(app, &language);
    let _ = app.emit("cfvigil-language-changed", language);
    Ok(())
}

#[tauri::command]
fn set_language(app: AppHandle, state: State<'_, AppState>, language: String) -> Result<(), String> {
    commit_language(&app, &state, language)
}

fn update_native_language(app: &AppHandle, locale: &str) {
    #[cfg(target_os = "linux")]
    linux_tray::set_language(app, locale);
    for label in ["settings", "history", "about"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.set_title(language::window_title(locale, label));
        }
    }
    #[cfg(not(target_os = "linux"))]
    if let Some(tray) = app.tray_by_id("cfvigil") {
        // Preserve the tray event handler; only replace its localized menu.
        if let Ok(menu) = translated_tray_menu(app, locale) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn translated_tray_menu(app: &AppHandle, locale: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let open = MenuItem::with_id(app, "open", language::menu(locale, "open"), true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", language::menu(locale, "history"), true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", language::menu(locale, "settings"), true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", language::menu(locale, "about"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", language::menu(locale, "quit"), true, None::<&str>)?;
    Menu::with_items(app, &[&open, &history, &settings, &about, &quit])
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

fn current_snapshot(state: &AppState) -> Snapshot {
    Snapshot {
        ready: state.ready.load(Ordering::Acquire),
        pending: state.pending.load(Ordering::Acquire),
        configured: !state.config.lock().unwrap().account_id.is_empty(),
        resources: state.resources.lock().unwrap().clone(),
        pending_incident: state.pending_incident.lock().unwrap().clone(),
        provider_warnings: state.provider_warnings.lock().unwrap().clone(),
        storage_warning: state.storage_warning,
    }
}

fn publish_snapshot(app: &AppHandle, state: &AppState) {
    let _ = app.emit("cfvigil-resources", current_snapshot(state));
}

fn invalidate_resources(state: &AppState, reason: &str) {
    state.ready.store(false, Ordering::Release);
    for resource in state.resources.lock().unwrap().iter_mut() {
        resource.status = "unknown".into();
        resource.detail = reason.to_owned();
    }
}

fn report_sync_failure(app: &AppHandle, state: &AppState, message: &str) {
    state.pending.store(false, Ordering::Release);
    invalidate_resources(state, msg::DATA_STALE);
    *state.provider_warnings.lock().unwrap() = vec![message.to_owned()];
    publish_snapshot(app, state);
    let _ = app.emit("cfvigil-sync-error", message);
}

fn needs_refresh(old: &Config, next: &Config, token_replaced: bool) -> bool {
    token_replaced
        || old.account_id != next.account_id
        || old.refresh_seconds != next.refresh_seconds
}

fn superseded_revision(revision: &AtomicU64, captured: u64) -> bool {
    revision.load(Ordering::Acquire) != captured
}

#[derive(PartialEq, Eq)]
enum CredentialAction {
    None,
    VerifyExisting,
    Replace(String),
}

fn credential_action(previous: &Config, account_id: &str, token: Option<String>)
    -> CredentialAction {
    match token {
        Some(value) => CredentialAction::Replace(value),
        None if !account_id.is_empty() && previous.account_id != account_id =>
            CredentialAction::VerifyExisting,
        None => CredentialAction::None,
    }
}

// Native operation is injected only at its I/O boundary. On an error the
// caller aborts BEFORE writing account JSON, changing language, modifying
// autostart or waking Cloudflare, and no token is returned to the WebView.
fn perform_credential_action(
    action: CredentialAction,
    verify: impl FnOnce() -> Result<(), String>,
    write: impl FnOnce(&str) -> Result<(), String>,
) -> Result<(), String> {
    match action {
        CredentialAction::None => Ok(()),
        CredentialAction::VerifyExisting => verify(),
        CredentialAction::Replace(secret) => write(&secret),
    }
}

// No start-at-login entry is installed unless the user explicitly opts in.
// Read the real platform registration, not a potentially stale JSON flag.
#[tauri::command]
fn get_autostart(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled()
        .map_err(|_| msg::ERROR_AUTOSTART_CHECK.into())
}

// This controls *both* real OS autostart behavior and deterministic
// failure injection. An unchanged state must never invoke native APIs.
fn apply_autostart(
    requested: Option<bool>,
    read: impl FnOnce() -> Result<bool, String>,
    change: impl FnOnce(bool) -> Result<(), String>,
) -> Result<(), String> {
    let Some(enabled) = requested else { return Ok(()) };
    let current = read()?;
    if current == enabled { return Ok(()) }
    change(enabled)
}

fn update_autostart(app: &AppHandle, requested: Option<bool>) -> Result<(), String> {
    apply_autostart(
        requested,
        || app.autolaunch().is_enabled().map_err(|_| msg::ERROR_AUTOSTART_READ.into()),
        |enabled| {
            let manager = app.autolaunch();
            let result = if enabled { manager.enable() } else { manager.disable() };
            result.map_err(|_| msg::ERROR_AUTOSTART_UPDATE.into())
        },
    )
}

#[tauri::command]
fn get_snapshot(state: State<'_, AppState>) -> Snapshot {
    current_snapshot(&state)
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
fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn open_about_website() -> Result<(), String> {
    open::that("https://morozhenko.com/").map_err(|_| msg::ERROR_BROWSER.into())
}

#[tauri::command]
fn open_about_source() -> Result<(), String> {
    open::that("https://github.com/morozhenko-apps/cfvigil")
        .map_err(|_| msg::ERROR_BROWSER.into())
}


#[tauri::command]
async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    account_id: String,
    token: Option<String>,
    refresh_seconds: u64,
    autostart: Option<bool>,
    language: String,
) -> Result<(), String> {
    if !language::supported(&language) { return Err("Unsupported interface language".into()); }
    // No Cloudflare account is necessary to save locale, interval or autostart.
    // Credentials become mandatory only when an account is actually configured.
    if (!account_id.is_empty() && (account_id.len() != 32 ||
        !account_id.bytes().all(|ch| ch.is_ascii_hexdigit())))
        || (account_id.is_empty() && token.is_some()) {
        return Err(msg::ERROR_ACCOUNT_ID.into());
    }
    if ![60, 180, 300, 600].contains(&refresh_seconds) {
        return Err(msg::ERROR_INTERVAL.into());
    }
    if token.as_ref().is_some_and(|value| {
        value.len() < 20 || value.len() > 4096 || value.chars().any(char::is_whitespace)
    }) {
        return Err(msg::ERROR_TOKEN_FORMAT.into());
    }
    // Serialize credentials, settings and network reads: no request may use
    // an old account ID with a newly replaced OS-keyring token.
    let _refresh_guard = state.refresh_lock.lock().await;
    let token_replaced = token.is_some();
    let previous = state.config.lock().unwrap().clone();
    // Fail credentials before writing JSON, refreshing the provider, or
    // committing a locale. A pure locale edit never touches OS keyring.
    let credential_op = credential_action(&previous, &account_id, token);
    if !matches!(credential_op, CredentialAction::None) {
        tauri::async_runtime::spawn_blocking(move || {
            perform_credential_action(
                credential_op,
                || load_token().map(|_| ()),
                |secret| token_entry()?.set_password(secret)
                    .map_err(|_| msg::ERROR_TOKEN_WRITE.to_string()),
            )
        }).await.map_err(|_| msg::ERROR_CREDENTIAL_SERVICE)??;
    }
    let next = Config { account_id, refresh_seconds };
    let should_refresh = needs_refresh(&previous, &next, token_replaced);
    if previous.account_id != next.account_id ||
        previous.refresh_seconds != next.refresh_seconds {
        write_json(&app, "settings.json", &next)?;
        *state.config.lock().unwrap() = next.clone();
    }
    if should_refresh {
        if previous.account_id != next.account_id {
            // A previous account's resource names must not survive a switch.
            let empty_catalog = ResourceCatalogCache {
                account_id: next.account_id.clone(), resources: Vec::new(),
            };
            if let Err(error) = write_json(&app, "resource-catalog.json", &empty_catalog) {
                eprintln!("CFVigil: could not reset resource catalog: {error}");
            }
        }
        // Invalidate any in-flight Cloudflare response before it can publish
        // rows from a previous account or previous secret after this save.
        state.config_revision.fetch_add(1, Ordering::AcqRel);
        state.ready.store(false, Ordering::Release);
        state.pending.store(!next.account_id.is_empty(), Ordering::Release);
        state.provider_warnings.lock().unwrap().clear();
        if previous.account_id != next.account_id {
            // Never reuse incidents or identities from another account.
            state.resources.lock().unwrap().clear();
            state.last_notified.lock().unwrap().clear();
            let journal = IncidentJournal::empty(&next.account_id);
            *state.journal.lock().unwrap() = journal.clone();
            if write_json(&app, "incident-state.json", &journal).is_err() {
                eprintln!("CFVigil: could not reset the incident journal");
            }
        } else {
            invalidate_resources(&state, msg::DATA_PENDING);
        }
        *state.pending_incident.lock().unwrap() = None;
        publish_snapshot(&app, &state);
        // One refresh per substantive credentials/polling change.
        state.poll_refresh.send_modify(|revision| {
            *revision = revision.wrapping_add(1);
        });
    }
    // Settings may have saved even when the OS prevents autostart updates.
    // Report the partial outcome instead of silently claiming success.
    update_autostart(&app, autostart)
        .map_err(|message| format!("Connection settings saved; {message}"))?;
    // Language is committed only after the primary settings have passed
    // validation and persistence. Never apply drafts on select.change.
    commit_language(&app, &state, language)?;
    Ok(())
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if let Some(icon) = app.default_window_icon().cloned() {
            let _ = window.set_icon(icon);
        }
        #[cfg(target_os = "linux")]
        if let Ok(native) = window.gtk_window() {
            use gtk::prelude::*;
            native.set_icon_name(Some("cfvigil"));
        }
        let _ = window.show();
        let _ = window.set_focus();
        if label == "settings" {
            let _ = app.emit_to("settings", "cfvigil-settings-opened", ());
        }
    }
}

fn show_card(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window("main") {
        // Already open: do not reset WebView state while leaving its large
        // expanded native rectangle on screen (blank drawer and huge gap).
        if window.is_visible().unwrap_or(false) {
            let _ = window.set_focus();
            return;
        }
        windows_card::restore_for_show(app, &window);
        let _ = app.emit("cfvigil-card-reset", ());
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("cfvigil-card-shown", ());
        return;
    }
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        let _ = linux_surface::request_resize(app, COMPACT_HEIGHT as i32);
        let _ = app.emit("cfvigil-card-reset", ());
        let _ = linux_surface::show(app);
        let _ = app.emit("cfvigil-card-shown", ());
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("cfvigil-card-reset", ());
        let _ = app.emit("cfvigil-card-shown", ());
    }
}

#[tauri::command]
fn hide_card(app: AppHandle) {
    *app.state::<AppState>().pending_incident.lock().unwrap() = None;
    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window("main") {
        // A hidden window must always be compact when next shown; preserve
        // bottom anchor instead of cumulatively shifting Y on each resize.
        let _ = window.hide();
        windows_card::collapse_hidden(&app, &window);
        let _ = app.emit("cfvigil-card-hidden", ());
        return;
    }
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        let _ = linux_surface::hide(&app);
        let _ = linux_surface::request_resize(&app, COMPACT_HEIGHT as i32);
        let _ = app.emit("cfvigil-card-hidden", ());
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let _ = app.emit("cfvigil-card-hidden", ());
}

#[tauri::command]
async fn start_card_drag(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    windows_card::begin_drag(&app);
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() { return linux_surface::start_drag(&app).await; }
    let window = app.get_webview_window("main").ok_or(msg::ERROR_CARD_MISSING)?;
    let result = window.start_dragging().map_err(|_| "Unable to drag card".to_owned());
    #[cfg(target_os = "windows")]
    if result.is_err() { windows_card::end_drag(&app); }
    result
}

#[tauri::command]
async fn stop_card_drag(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    windows_card::end_drag(&app);
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() { return linux_surface::stop_drag(&app).await; }
    let _ = app;
    Ok(())
}

/// Browser content sizes its localized form; only native owns OS window size.
#[tauri::command]
fn start_settings_drag(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("settings")
        .ok_or("Settings window unavailable")?
        .start_dragging()
        .map_err(|err| format!("Unable to move Settings window: {err}"))
}

#[tauri::command]
fn start_about_drag(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("about")
        .ok_or("About window unavailable")?
        .start_dragging()
        .map_err(|_| "Unable to move About window".to_owned())
}

#[tauri::command]
fn fit_settings(app: AppHandle, height: f64) -> Result<(), String> {
    if !height.is_finite() { return Err("Invalid settings height".into()); }
    #[cfg(target_os = "linux")]
    if linux_settings::enabled() {
        return linux_settings::fit(&app, height.ceil() as i32);
    }
    let window = app.get_webview_window("settings").ok_or("Settings window unavailable")?;
    let height = height.clamp(370.0, 740.0);
    let scale = window.scale_factor().unwrap_or(1.0);
    let target_width = (460.0 * scale).round() as i64;
    let target_height = (height * scale).round() as i64;
    if let Ok(actual) = window.inner_size() {
        // Skip an identical OS resize. A no-op must not alter placement on KDE.
        if (i64::from(actual.width) - target_width).abs() <= 1 &&
            (i64::from(actual.height) - target_height).abs() <= 1 {
            return Ok(());
        }
        if std::env::var_os("CFVIGIL_UI_DEBUG").is_some() {
            eprintln!(
                "CFVigil Settings fit: current={}x{} target={}x{} scale={scale}",
                actual.width, actual.height, target_width, target_height
            );
        }
    }
    window.set_size(tauri::LogicalSize::new(460.0, height))
        .map_err(|_| "Unable to size settings window".into())
}

#[tauri::command]
async fn set_card_expanded(app: AppHandle, expanded: bool, height: Option<f64>) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or(msg::ERROR_CARD_MISSING)?;
    // The DOM measures the content, but native code owns both the limits
    // and the bottom-anchored resize. Never accept NaN or unbounded height.
    let height = if expanded {
        let requested = height.unwrap_or(EXPANDED_HEIGHT);
        if !requested.is_finite() { return Err(msg::ERROR_RESIZE.into()); }
        requested.clamp(COMPACT_HEIGHT + 50.0, EXPANDED_HEIGHT)
    } else { COMPACT_HEIGHT };
    #[cfg(target_os = "linux")]
    if linux_surface::enabled() {
        return linux_surface::resize(app, height as i32).await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows_card::resize_anchored(&app, &window, height);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let old_position = window.outer_position().ok();
        let old_size = window.outer_size().ok();
        window.set_size(tauri::LogicalSize::new(CARD_WIDTH, height))
            .map_err(|_| msg::ERROR_RESIZE)?;
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
        .ok_or(msg::ERROR_RESOURCE_MISSING)?;
    if !trusted_dashboard_url(&url) {
        return Err(msg::ERROR_EXTERNAL_DESTINATION.into());
    }
    open::that(&url).map_err(|_| msg::ERROR_BROWSER.into())
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
    open::that(url).map_err(|_| msg::ERROR_BROWSER.into())
}

// A temporarily unavailable catalog cannot prove resource removal.
fn preserve_unverified_catalogs(
    current: &mut Vec<Resource>,
    previous: &[Resource],
    workers_complete: bool,
    pages_complete: bool,
) {
    for old in previous {
        let missing_catalog = (old.kind == "worker" && !workers_complete)
            || (old.kind == "pages" && !pages_complete);
        if missing_catalog && !current.iter().any(|item| item.key == old.key) {
            let mut stale = old.clone();
            stale.status = "unknown".into();
            stale.detail = msg::DATA_CATALOG_UNAVAILABLE.into();
            current.push(stale);
        }
    }
}

// Only a successfully queried zone may resolve a previous WAF incident.
fn preserved_waf(previous: &[Resource], verified: &HashSet<String>) -> Vec<Resource> {
    previous.iter().filter(|old| old.kind == "zone" && !verified.contains(&old.key))
        .map(|old| {
            let mut stale = old.clone();
            stale.status = "unknown".into();
            stale.detail = msg::DATA_WAF_UNAVAILABLE.into();
            stale
        }).collect()
}

async fn refresh_once(app: AppHandle, budget: &ApiBudget) -> bool {
    let state = app.state::<AppState>();
    // Hold the credential lock ONLY to take a consistent account/token
    // snapshot. Cloudflare discovery + metrics may take many seconds and must
    // never block a settings Apply. A revision check before publication
    // discards data fetched using outdated credentials.
    let (config, token, revision) = {
        let _credential_guard = state.refresh_lock.lock().await;
        let config = state.config.lock().unwrap().clone();
        if config.account_id.is_empty() {
            state.pending.store(false, Ordering::Release);
            return true;
        }
        let token = match tauri::async_runtime::spawn_blocking(load_token).await {
            Ok(Ok(token)) => token,
            _ => {
                report_sync_failure(&app, &state, msg::DATA_CREDENTIALS_UNAVAILABLE);
                return false;
            }
        };
        (config, token, state.config_revision.load(Ordering::Acquire))
    };
    let result = match Cloudflare::new() {
        Ok(api) => api.discover(&config.account_id, &token, budget).await,
        Err(message) => Err(message),
    };
    let discovery = match result {
        Ok(discovery) => discovery,
        Err(message) => {
            let _guard = state.refresh_lock.lock().await;
            if superseded_revision(&state.config_revision, revision) {
                return true; // The new settings already scheduled a fresh poll.
            }
            report_sync_failure(&app, &state, &message);
            return false;
        }
    };
    let core_complete = discovery.complete;
    // A permanently denied Pages scope must not back off verified Workers
    // polling to 30 minutes. Backoff is reserved for total discovery outage
    // and API-budget exhaustion; partial provider failures remain visible.
    let catalog_available = discovery.workers_complete || discovery.pages_complete;
    let warnings = discovery.warnings;
    let previous = state.resources.lock().unwrap().clone();
    let worker_names: HashSet<String> = discovery.resources.iter()
        .filter(|resource| resource.kind == "worker")
        .map(|resource| resource.name.clone()).collect();
    let mut resources = discovery.resources;
    preserve_unverified_catalogs(&mut resources, &previous,
        discovery.workers_complete, discovery.pages_complete);
    // Metrics never probe the deployed application. Unavailable analytics
    // leave deployment status unchanged rather than inventing an incident.
    match analytics::query_spikes(&config.account_id, &token, &worker_names, budget).await {
        Ok(scan) => {
            for resource in resources.iter_mut().filter(|resource| resource.kind == "worker") {
                if let Some(spike) = scan.spikes.get(&resource.name) {
                    let old_alert = previous.iter().find(|old| old.key == resource.key)
                        .filter(|old| old.attempt_id == "runtime-error-spike");
                    resource.changed_at = old_alert
                        .map(|old| old.changed_at.clone())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
                    resource.status = "failed".into();
                    resource.detail = msg::worker_spike_detail(spike.errors, spike.requests, spike.ratio);
                    resource.attempt_id = "runtime-error-spike".into();
                } else if scan.unverified.contains(&resource.name) {
                    if let Some(old) = previous.iter().find(|old| old.key == resource.key)
                        .filter(|old| old.attempt_id == "runtime-error-spike")
                    {
                        resource.status = "unknown".into();
                        resource.detail = msg::DATA_WORKERS_ANALYTICS_UNAVAILABLE.into();
                        resource.changed_at = old.changed_at.clone();
                        resource.attempt_id = old.attempt_id.clone();
                    }
                }
            }
        }
        Err(_) => {
            // Total analytics failure also cannot confirm a recovery.
            for resource in resources.iter_mut().filter(|resource| resource.kind == "worker") {
                if let Some(old) = previous.iter().find(|old| old.key == resource.key)
                    .filter(|old| old.attempt_id == "runtime-error-spike")
                {
                    resource.status = "unknown".into();
                    resource.detail = msg::DATA_WORKERS_ANALYTICS_UNAVAILABLE.into();
                    resource.changed_at = old.changed_at.clone();
                    resource.attempt_id = old.attempt_id.clone();
                }
            }
        }
    }
    let verified_zones = match security::detect(&config.account_id, &token, budget).await {
        Ok(mut report) => {
            for current in &mut report.incidents {
                if let Some(old) = previous.iter().find(|old| {
                    old.key == current.key && old.attempt_id == "waf-block-spike"
                }) {
                    current.changed_at = old.changed_at.clone();
                }
            }
            resources.extend(report.incidents);
            // Do not grey out or resolve the zones that did answer; only
            // unverified previous incidents remain grey.
            resources.extend(preserved_waf(&previous, &report.verified_zones));
            report.verified_zones
        }
        Err(_) => {
            resources.extend(preserved_waf(&previous, &HashSet::new()));
            HashSet::new()
        }
    };
    resources.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    let _credential_guard = state.refresh_lock.lock().await;
    if superseded_revision(&state.config_revision, revision) {
        // A concurrent Apply won: never publish old-account rows, incidents,
        // notifications or journal snapshots.
        return true;
    }
    if catalog_available {
        if let Some(catalog) = ResourceCatalogCache::from_discovery(&config.account_id, &resources) {
            if let Err(error) = write_json(&app, "resource-catalog.json", &catalog) {
                eprintln!("CFVigil: could not persist resource catalog: {error}");
            }
        }
    }
    *state.provider_warnings.lock().unwrap() = warnings;
    state.ready.store(core_complete, Ordering::Release);
    state.pending.store(false, Ordering::Release);
    *state.resources.lock().unwrap() = resources.clone();
    publish_snapshot(&app, &state);
    if !core_complete {
        let _ = app.emit("cfvigil-sync-error", msg::DATA_PARTIAL);
    }
    let mut events = Vec::new();
    for current in &resources {
        let old = previous.iter().find(|entry| entry.key == current.key);
        // Only zone-level WAF incidents alert on first observation. Pre-existing
        // Workers/Pages deployment attempts do not spam at initial launch.
        let message = match old {
            // A stale startup roster exists for fast display, not for
            // manufacturing notifications about old provider attempts.
            Some(old) if old.detail == msg::DATA_STALE && old.attempt_id.is_empty() => None,
            Some(old) => incident_policy::describe_transition(old, current),
            None => incident_policy::initial_security_event(current),
        };
        let Some(message) = message else { continue; };
        if message == msg::EVENT_RUNTIME_RECOVERY {
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
    for old in previous.iter().filter(|r| r.kind == "zone" && r.attempt_id == "waf-block-spike") {
        if verified_zones.contains(&old.key) && !resources.iter().any(|r| r.key == old.key) {
                state.last_notified.lock().unwrap().remove(&format!("{}:waf-spike", old.key));
                events.push(new_history_event(old, msg::EVENT_WAF_RECOVERY, events.len()));
        }
    }
    if !events.is_empty() {
        let mut history = state.history.lock().unwrap();
        history.splice(0..0, events);
        history.truncate(500);
        if write_json(&app, "events.json", &*history).is_err() {
            eprintln!("CFVigil: could not persist event history");
        }
        let _ = app.emit("cfvigil-history-changed", ());
    }
    // Persist only active security/runtime incidents and a bounded set of
    // notification identities. Normal green resources are never rehydrated
    // from disk as verified after restart.
    let next_journal = {
        let notified = state.last_notified.lock().unwrap();
        IncidentJournal::snapshot(&config.account_id, &resources, &notified)
    };
    let mut saved = state.journal.lock().unwrap();
    if *saved != next_journal {
        if write_json(&app, "incident-state.json", &next_journal).is_ok() {
            *saved = next_journal;
        } else {
            eprintln!("CFVigil: could not persist the incident journal");
        }
    }
    catalog_available && !budget.exhausted()
}

fn run_polling(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut changes = app.state::<AppState>().poll_refresh.subscribe();
        let mut failures = 0u32;
        loop {
            // An update while fetching remains unconsumed and triggers
            // an immediate next cycle after the previous request finishes.
            let _ = *changes.borrow_and_update();
            let budget = ApiBudget::new();
            if refresh_once(app.clone(), &budget).await {
                failures = 0;
            } else {
                failures = failures.saturating_add(1);
            }
            let interval = app.state::<AppState>().config.lock().unwrap().refresh_seconds;
            let delay = poll_delay(interval, failures)
                .max(Duration::from_secs(budget.retry_after_seconds()));
            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                update = changes.changed() => {
                    if update.is_err() { break; }
                }
            }
        }
    });
}

#[cfg(target_os = "linux")]
fn configure_gdk_for_native_wayland() {
    // RepoTicker root-cause fix: an inherited GDK_BACKEND=x11 inside KDE
    // Wayland creates an XWayland card (minimum ~200px, recentering and no
    // layer-shell position persistence). Choose before Tauri initializes GTK.
    // An actual X11 session must keep its original backend.
    let session_is_wayland = std::env::var("XDG_SESSION_TYPE")
        .is_ok_and(|value| value.eq_ignore_ascii_case("wayland"));
    let has_wayland_display = std::env::var("WAYLAND_DISPLAY")
        .is_ok_and(|value| !value.trim().is_empty());
    if session_is_wayland && has_wayland_display {
        std::env::set_var("GDK_BACKEND", "wayland");
    }
}

/// Use one branded Tauri icon for ALL system-managed windows. KDE's
/// titlebar otherwise falls back to the generic WebKit/Tauri application icon.
fn install_window_icons(app: &tauri::App) {
    // Explicitly apply the *same* committed icon to the GTK window:
    // the WebView favicon and Tauri tray image do not establish the
    // window-manager icon for History and About on every KDE backend.
    #[cfg(target_os = "linux")]
    {
        use gtk::gdk_pixbuf::prelude::*;
        use gtk::prelude::*;
        let loader = gtk::gdk_pixbuf::PixbufLoader::new();
        if loader.write(include_bytes!("../icons/icon.png")).is_ok()
            && loader.close().is_ok() {
            if let Some(pixbuf) = loader.pixbuf() {
                for label in ["main", "settings", "history", "about"] {
                    if let Some(window) = app.get_webview_window(label) {
                        if let Ok(native) = window.gtk_window() {
                            native.set_icon(Some(&pixbuf));
                        }
                    }
                }
            }
        }
    }
    if let Some(icon) = app.default_window_icon().cloned() {
        for label in ["main", "settings", "history", "about"] {
            if let Some(window) = app.get_webview_window(label) {
                if let Err(error) = window.set_icon(icon.clone()) {
                    eprintln!("CFVigil could not set {label} window icon: {error}");
                }
            }
        }
    } else {
        eprintln!("CFVigil application icon is missing from bundled resources");
    }
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        // KDE may resolve a desktop-entry Icon= name instead of the pixbuf,
        // particularly for Wayland's app-id window/task-switcher resolution.
        for label in ["main", "settings", "history", "about"] {
            if let Some(window) = app.get_webview_window(label) {
                if let Ok(native) = window.gtk_window() {
                    native.set_icon_name(Some("cfvigil"));
                }
            }
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    configure_gdk_for_native_wayland();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|app| {
            // Explicitly brand History, Settings, About and the main window.
            install_window_icons(app);
            let handle = app.handle().clone();
            let locale = read_json::<LanguagePreference>(&handle, "language.json")
                .map_err(std::io::Error::other)?;
            let settings = read_json::<Config>(&handle, "settings.json")
                .map_err(std::io::Error::other)?;
            let stored = read_json::<IncidentJournal>(&handle, "incident-state.json")
                .map_err(std::io::Error::other)?;
            let history = read_json::<Vec<HistoryEvent>>(&handle, "events.json")
                .map_err(std::io::Error::other)?;
            let catalog = read_json::<ResourceCatalogCache>(&handle, "resource-catalog.json")
                .map_err(std::io::Error::other)?;
            let storage_warning = settings.quarantined || stored.quarantined
                || history.quarantined || locale.quarantined || catalog.quarantined;
            let language = language::sanitize(locale.value);
            let journal = stored.value.restore(&settings.value.account_id);
            let pending_initial_sync = !settings.value.account_id.is_empty();
            // Render last known resource names before the first network call.
            // Every cached resource is unverified; active incidents in the
            // journal take precedence and preserve their dedup identities.
            let mut initial_resources = journal.active.clone();
            for stale in catalog.value.show_stale(&settings.value.account_id) {
                if !initial_resources.iter().any(|existing| existing.key == stale.key) {
                    initial_resources.push(stale);
                }
            }
            initial_resources.sort_by(|a,b| a.name.cmp(&b.name));
            let (poll_refresh, _receiver) = tokio::sync::watch::channel(0u64);
            app.manage(AppState {
                provider_warnings: Mutex::new(Vec::new()),
                config: Mutex::new(settings.value),
                language: Mutex::new(language.clone()),
                resources: Mutex::new(initial_resources),
                history: Mutex::new({
                    let mut events = history.value;
                    events.truncate(500);
                    normalize_history(&mut events);
                    events
                }),
                last_notified: Mutex::new(NotificationWindow::from_ids(journal.notified.clone())),
                journal: Mutex::new(journal),
                pending_incident: Mutex::new(None),
                ready: AtomicBool::new(false),
                pending: AtomicBool::new(pending_initial_sync),
                refresh_lock: tokio::sync::Mutex::new(()),
                config_revision: AtomicU64::new(0),
                poll_refresh,
                storage_warning,
            });
            #[cfg(target_os = "linux")]
            {
                linux_surface::install(app)?;
                linux_settings::install(app)?;
                // Always log the backend, including fallback. No secrets.
                eprintln!(
                    "CFVigil surface mode={} session={} wayland_display={} gdk_backend={}",
                    if linux_surface::enabled() { "gtk-layer-shell" } else { "tauri-fallback" },
                    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
                    std::env::var_os("WAYLAND_DISPLAY").is_some(),
                    std::env::var("GDK_BACKEND").unwrap_or_else(|_| "default".into())
                );
                linux_tray::install(app)?;
            }
            #[cfg(target_os = "windows")]
            windows_card::install(app)?;
            #[cfg(not(target_os = "linux"))]
            {
            let menu = translated_tray_menu(app.handle(), &language.locale)?;
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
                        #[cfg(target_os = "windows")]
                        if tray.app_handle().get_webview_window("main")
                            .is_some_and(|window| window.is_visible().unwrap_or(false)) {
                            hide_card(tray.app_handle().clone());
                        } else {
                            show_card(tray.app_handle());
                        }
                    }
                })
                .build(app)?;
            }
            update_native_language(&handle, &language.locale);
            run_polling(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(target_os = "windows")]
            if window.label() == "main" {
                windows_card::on_window_event(window, event);
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if window.label() == "main" {
                    hide_card(window.app_handle().clone());
                } else {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            get_token_status,
            get_language,
            set_language,
            get_autostart,
            save_settings,
            get_snapshot,
            acknowledge_incident,
            get_history,
            get_app_version,
            open_about_website,
            open_about_source,
            hide_card,
            set_card_expanded,
            start_card_drag,
            stop_card_drag,
            fit_settings,
            start_settings_drag,
            start_about_drag,
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
    fn provider_and_keyring_failures_invalidate_verified_status() {
        use super::*;
        let (poll_refresh, _rx) = tokio::sync::watch::channel(0u64);
        let previously_healthy = cloudflare::Resource {
            key: "worker:api".into(), name: "api".into(), kind: "worker".into(),
            status: "healthy".into(), changed_at: "2026-10-23T12:00:00Z".into(),
            attempt_id: "build-123".into(), url: "https://dash.cloudflare.com/x".into(),
            detail: "Latest Cloudflare Workers build".into(),
        };
        let state = AppState {
            provider_warnings: Mutex::new(Vec::new()),
            config: Mutex::new(Config {account_id:"a".repeat(32),refresh_seconds:60}),
            language: Mutex::new(LanguagePreference::default()),
            resources: Mutex::new(vec![previously_healthy]),
            history: Mutex::new(Vec::new()),
            last_notified: Mutex::new(NotificationWindow::default()),
            journal: Mutex::new(IncidentJournal::empty(&"a".repeat(32))),
            pending_incident: Mutex::new(None),
            ready: AtomicBool::new(true),
            pending: AtomicBool::new(false),
            refresh_lock: tokio::sync::Mutex::new(()),
            config_revision: AtomicU64::new(0),
            poll_refresh,
            storage_warning: false,
        };
        invalidate_resources(&state, "Credentials unavailable");
        let snapshot = current_snapshot(&state);
        assert!(!snapshot.ready);
        assert!(snapshot.configured);
        assert_eq!(snapshot.resources.len(), 1);
        assert_eq!(snapshot.resources[0].status, "unknown");
        assert_eq!(snapshot.resources[0].detail, "Credentials unavailable");
        assert_eq!(snapshot.resources[0].attempt_id, "build-123");
    }

    #[test]
    fn an_unavailable_worker_catalog_keeps_only_its_old_rows_grey() {
        use super::*;
        let old_worker = cloudflare::Resource {
            key: "worker:alpha".into(), name: "alpha".into(), kind: "worker".into(),
            status: "failed".into(), changed_at: "2026-10-23T12:00:00Z".into(),
            attempt_id: "build-3".into(), url: "https://dash.cloudflare.com/example".into(),
            detail: "Latest build failed".into(),
        };
        let mut old_pages = old_worker.clone();
        old_pages.key = "pages:site".into();
        old_pages.kind = "pages".into();
        let mut actual = vec![old_pages.clone()];
        preserve_unverified_catalogs(&mut actual, &[old_worker.clone(),old_pages], false, true);
        assert_eq!(actual.len(),2);
        assert_eq!(actual[0].status,"failed");
        assert_eq!(actual[1].key,"worker:alpha");
        assert_eq!(actual[1].status,"unknown");
        assert_eq!(actual[1].attempt_id,"build-3");
        preserve_unverified_catalogs(&mut actual, &[old_worker], false, true);
        assert_eq!(actual.len(),2);
    }

    #[test]
    fn only_verified_waf_zones_can_recover() {
        use super::*;
        let make = |key: &str| cloudflare::Resource {
            key: key.into(), name: key.into(), kind: "zone".into(),
            status: "failed".into(), changed_at: "2026-10-23T12:00:00Z".into(),
            attempt_id: "waf-block-spike".into(),
            url: "https://dash.cloudflare.com/example".into(),
            detail: "WAF blocks spiking".into(),
        };
        let old = vec![make("zone:ok"), make("zone:offline")];
        let verified = HashSet::from(["zone:ok".to_owned()]);
        let retained = preserved_waf(&old, &verified);
        assert_eq!(retained.len(),1);
        assert_eq!(retained[0].key,"zone:offline");
        assert_eq!(retained[0].status,"unknown");
        assert_eq!(retained[0].attempt_id,"waf-block-spike");
        assert_eq!(preserved_waf(&old, &HashSet::new()).len(),2);
    }

    #[test]
    fn startup_only_changes_never_repoll_cloudflare() {
        use super::*;
        let original = Config { account_id: "a".repeat(32), refresh_seconds: 180 };
        let current = original.clone();
        assert!(!needs_refresh(&original, &current, false));
        assert!(needs_refresh(&original, &current, true));
        let changed = Config { account_id: "b".repeat(32), refresh_seconds: 180 };
        assert!(needs_refresh(&original, &changed, false));
        let shorter = Config { account_id: "a".repeat(32), refresh_seconds: 60 };
        assert!(needs_refresh(&original, &shorter, false));
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


#[cfg(test)]
mod settings_effect_tests {
    use super::*;
    use std::cell::Cell;

    fn prior() -> Config {
        Config { account_id: "a".repeat(32), refresh_seconds: 180 }
    }

    #[test]
    fn locale_only_and_interval_only_changes_do_not_touch_os_keyring() {
        let existing = prior();
        for requested in [
            existing.account_id.clone(),
            String::new()
        ] {
            if requested.is_empty() { continue; }
            assert!(matches!(
                credential_action(&existing, &requested, None),
                CredentialAction::None
            ));
        }
        assert!(!needs_refresh(&existing, &existing, false));
        assert!(needs_refresh(&existing, &Config {
            account_id: existing.account_id.clone(), refresh_seconds: 60
        }, false),"interval change refreshes without keyring read");
    }

    #[test]
    fn changing_account_without_new_token_requires_existing_keyring_check_first() {
        let old = prior();
        assert!(matches!(
            credential_action(&old, &"b".repeat(32), None),
            CredentialAction::VerifyExisting
        ));
        assert!(matches!(
            credential_action(&old, "", None),
            CredentialAction::None
        ));
        assert!(needs_refresh(&old, &Config {
            account_id: "b".repeat(32), refresh_seconds: old.refresh_seconds
        }, false));
    }

    #[test]
    fn explicit_token_replacement_is_selected_exactly_once_even_when_account_unchanged() {
        let old=prior();
        for account in [old.account_id.clone(), "b".repeat(32)] {
            match credential_action(&old, &account, Some("secret-not-logged".into())) {
                CredentialAction::Replace(value) => {
                    assert_eq!(value,"secret-not-logged");
                }
                _ => panic!("explicit replacement cannot be omitted"),
            }
        }
        assert!(needs_refresh(&old, &old, true));
        assert!(matches!(credential_action(&old, &old.account_id, None),
            CredentialAction::None));
    }

    #[test]
    fn no_autostart_change_does_not_even_query_os_registration() {
        let reads=Cell::new(0u32);
        let writes=Cell::new(0u32);
        let result=apply_autostart(None,
            || { reads.set(reads.get()+1); Err("read unavailable".into()) },
            |_| { writes.set(writes.get()+1); Err("unexpected write".into()) });
        assert!(result.is_ok());
        assert_eq!(reads.get(),0);
        assert_eq!(writes.get(),0);
    }

    #[test]
    fn unchanged_autostart_flag_is_idempotent_and_never_reapplied() {
        for desired in [true,false] {
            let changes=Cell::new(0u32);
            let result=apply_autostart(Some(desired),
                || Ok(desired),
                |_| { changes.set(changes.get()+1); Ok(()) });
            assert_eq!(result,Ok(()));
            assert_eq!(changes.get(),0);
        }
    }

    #[test]
    fn enabled_and_disabled_registration_are_applied_only_when_needed() {
        for (existing,desired) in [(false,true),(true,false)] {
            let changes=Cell::new(0u32);
            let changed_to=Cell::new(None);
            let result=apply_autostart(Some(desired),
                || Ok(existing),
                |value| {
                    changes.set(changes.get()+1);
                    changed_to.set(Some(value));
                    Ok(())
                });
            assert!(result.is_ok());
            assert_eq!(changes.get(),1);
            assert_eq!(changed_to.get(),Some(desired));
        }
    }

    #[test]
    fn autostart_read_denial_prevents_os_mutation_and_preserves_error() {
        let mutations=Cell::new(0u32);
        let error="OS autostart read denied";
        let result=apply_autostart(Some(true),
            || Err(error.into()),
            |_| { mutations.set(mutations.get()+1); Ok(()) });
        assert_eq!(result,Err(error.into()));
        assert_eq!(mutations.get(),0);
    }

    #[test]
    fn autostart_write_denial_is_reported_without_second_write_attempt() {
        let writes=Cell::new(0u32);
        let error="OS autostart update denied";
        let result=apply_autostart(Some(false),
            || Ok(true),
            |_| { writes.set(writes.get()+1); Err(error.into()) });
        assert_eq!(result,Err(error.into()));
        assert_eq!(writes.get(),1);
    }

    #[test]
    fn settings_locale_changes_do_not_trigger_cloudflare_polling() {
        let old=prior();
        let locale_only=old.clone();
        assert!(!needs_refresh(&old,&locale_only,false));
        assert!(matches!(
            credential_action(&old,&locale_only.account_id,None),
            CredentialAction::None
        ));
        let token_replaced=needs_refresh(&old,&locale_only,true);
        assert!(token_replaced);
    }
    #[test]
    fn keyring_none_requires_no_native_read_or_write_even_if_vault_locked() {
        let reads=Cell::new(0u32);
        let writes=Cell::new(0u32);
        let result=perform_credential_action(
            CredentialAction::None,
            || { reads.set(reads.get()+1); Err("locked".into()) },
            |_| { writes.set(writes.get()+1); Err("denied".into()) }
        );
        assert!(result.is_ok());
        assert_eq!(reads.get(),0);
        assert_eq!(writes.get(),0);
    }

    #[test]
    fn foreign_account_verification_denied_aborts_before_any_secret_replacement() {
        let reads=Cell::new(0u32);
        let writes=Cell::new(0u32);
        let result=perform_credential_action(
            credential_action(&prior(),&"b".repeat(32),None),
            || {
                reads.set(reads.get()+1);
                Err(msg::ERROR_TOKEN_LOCKED.into())
            },
            |_| { writes.set(writes.get()+1); Ok(()) }
        );
        assert_eq!(result,Err(msg::ERROR_TOKEN_LOCKED.into()));
        assert_eq!(reads.get(),1);
        assert_eq!(writes.get(),0);
    }

    #[test]
    fn new_token_write_denied_aborts_without_falling_back_to_old_credentials() {
        let reads=Cell::new(0u32);
        let writes=Cell::new(0u32);
        let expected="explicit-secret-for-keyring";
        let result=perform_credential_action(
            credential_action(&prior(),&prior().account_id,Some(expected.into())),
            || { reads.set(reads.get()+1); Ok(()) },
            |secret| {
                writes.set(writes.get()+1);
                assert_eq!(secret,expected);
                Err(msg::ERROR_TOKEN_WRITE.into())
            }
        );
        assert_eq!(result,Err(msg::ERROR_TOKEN_WRITE.into()));
        assert_eq!(reads.get(),0);
        assert_eq!(writes.get(),1);
    }

    #[test]
    fn successful_explicit_token_write_uses_only_the_provided_secret() {
        let reads=Cell::new(0u32);
        let writes=Cell::new(0u32);
        let result=perform_credential_action(
            CredentialAction::Replace("verified-test-secret".into()),
            || { reads.set(reads.get()+1); Err("unexpected read".into()) },
            |secret| {
                writes.set(writes.get()+1);
                assert_eq!(secret,"verified-test-secret");
                Ok(())
            }
        );
        assert_eq!(result,Ok(()));
        assert_eq!(reads.get(),0);
        assert_eq!(writes.get(),1);
    }

}


#[cfg(test)]
mod mode_b_partial_state_tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn resource(kind: &str, name: &str, state: &str, attempt: &str) -> Resource {
        Resource {
            key: format!("{kind}:{name}"), name: name.into(), kind: kind.into(),
            status: state.into(), changed_at: "2026-10-26T12:00:00Z".into(),
            attempt_id: attempt.into(),
            detail: "previous verified attempt".into(),
            url: "https://dash.cloudflare.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()
        }
    }

    #[test]
    fn independent_workers_pages_catalog_flags_preserve_only_unverified_old_rows() {
        let previous=[
            resource("worker","w","failed","b1"),
            resource("pages","p","healthy","d1"),
        ];
        for (workers_ok,pages_ok,expected_old) in [
            (true,true,Vec::<&str>::new()),
            (false,true,vec!["worker:w"]),
            (true,false,vec!["pages:p"]),
            (false,false,vec!["worker:w","pages:p"])
        ] {
            let mut now=Vec::new();
            preserve_unverified_catalogs(&mut now,&previous,workers_ok,pages_ok);
            let keys=now.iter().map(|r|r.key.as_str()).collect::<Vec<_>>();
            assert_eq!(keys,expected_old,"workers={workers_ok} pages={pages_ok}");
            for r in &now {
                assert_eq!(r.status,"unknown");
                assert_eq!(r.detail,msg::DATA_CATALOG_UNAVAILABLE);
            }
            assert_eq!(now.len(),expected_old.len());
        }
    }

    #[test]
    fn verified_pages_and_worker_attempts_are_never_overwritten_by_old_catalog_data() {
        let old=[
            resource("worker","w","failed","old-b1"),
            resource("pages","p","failed","old-d1")
        ];
        let verified=[
            resource("worker","w","deploying","new-b2"),
            resource("pages","p","healthy","new-d2")
        ];
        for workers_ok in [false,true] {
            for pages_ok in [false,true] {
                let mut now=verified.to_vec();
                preserve_unverified_catalogs(&mut now,&old,workers_ok,pages_ok);
                assert_eq!(now,verified,"do not duplicate or revert current rows");
            }
        }
    }

    #[test]
    fn partial_waf_response_never_mislabels_unverified_zone_as_recovered() {
        let old=[
            resource("zone","alpha","failed","waf-block-spike"),
            resource("zone","beta","failed","waf-block-spike"),
            resource("zone","gamma","failed","waf-block-spike"),
            resource("worker","other","failed","runtime-error-spike")
        ];
        let verified=HashSet::from(["zone:alpha".to_string(),"zone:gamma".to_string()]);
        let retained=preserved_waf(&old,&verified);
        assert_eq!(retained.len(),1);
        assert_eq!(retained[0].key,"zone:beta");
        assert_eq!(retained[0].status,"unknown");
        assert_eq!(retained[0].attempt_id,"waf-block-spike");
        assert_eq!(retained[0].detail,msg::DATA_WAF_UNAVAILABLE);
        assert_eq!(preserved_waf(&old,&HashSet::new()).len(),3);
        assert!(preserved_waf(&old,&HashSet::from([
            "zone:alpha".into(),"zone:beta".into(),"zone:gamma".into()
        ])).is_empty());
    }

    #[test]
    fn all_stale_provider_replies_are_discarded_after_concurrent_settings_revision() {
        let revision=Arc::new(AtomicU64::new(6));
        let acquired=revision.load(Ordering::Acquire);
        let barrier=Arc::new(Barrier::new(2));
        let writer=Arc::clone(&revision);
        let ready=Arc::clone(&barrier);
        let join=thread::spawn(move || {
            ready.wait();
            writer.fetch_add(1,Ordering::AcqRel);
            writer.load(Ordering::Acquire)
        });
        barrier.wait();
        let next=join.join().unwrap();
        assert_eq!(next,7);
        assert!(superseded_revision(&revision,acquired),
            "old-account Cloudflare response cannot be published");
        assert!(!superseded_revision(&revision,7),
            "the subsequent current-account response remains eligible");
        revision.fetch_add(1,Ordering::AcqRel);
        assert!(superseded_revision(&revision,7));
    }

    #[test]
    fn locale_only_change_and_noop_never_increment_provider_revision() {
        let config=Config {account_id:"a".repeat(32),refresh_seconds:180};
        let revision=AtomicU64::new(100);
        let before=revision.load(Ordering::Acquire);
        for replaced in [false] {
            if needs_refresh(&config,&config,replaced) {
                revision.fetch_add(1,Ordering::AcqRel);
            }
        }
        assert_eq!(revision.load(Ordering::Acquire),before);
        assert!(!superseded_revision(&revision,before));
        let newer=Config {account_id:config.account_id.clone(),refresh_seconds:300};
        if needs_refresh(&config,&newer,false) {
            revision.fetch_add(1,Ordering::AcqRel);
        }
        assert!(superseded_revision(&revision,before));
    }
}
