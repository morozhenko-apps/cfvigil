// Stable native English presentation and legacy incident messages.
// Do not mutate incident wording before a versioned event-code migration.
// Native menu/notification copy is centralized here; user-visible WebView
// copy lives in apps/desktop/ui/locales/en.mjs.
pub const TRAY_SHOW_CARD: &str = "Show Card";
pub const TRAY_EVENT_HISTORY: &str = "Event History";
pub const TRAY_SETTINGS: &str = "Settings";
pub const TRAY_ABOUT: &str = "About";
pub const TRAY_EXIT: &str = "Exit";
pub const EVENT_BUILD_FAILURE: &str = "Latest Cloudflare build/deployment failed";
pub const EVENT_RUNTIME_SPIKE: &str = "Workers runtime error spike detected";
pub const EVENT_WAF_SPIKE: &str = "WAF block spike detected";
pub const EVENT_RUNTIME_RECOVERY: &str = "Workers runtime error spike cleared";
pub const EVENT_WAF_RECOVERY: &str = "WAF block spike cleared";
pub const EVENT_BUILD_STARTED: &str = "New Cloudflare build/deployment started";
pub const EVENT_BUILD_SUCCESS: &str = "Latest Cloudflare deployment succeeded";
pub const DETAIL_WORKERS_BUILD: &str = "Latest Cloudflare Workers build";
pub const DETAIL_WORKER_DEPLOYMENT: &str = "Latest Worker deployment";
pub const DETAIL_BUILDS_UNAVAILABLE: &str = "Workers Builds unavailable";
pub const DETAIL_NO_WORKER_DEPLOYMENT: &str = "No verified Worker deployment";
pub const DETAIL_PAGES_DEPLOYMENT: &str = "Latest Cloudflare Pages deployment";
pub const DETAIL_NO_PAGES_DEPLOYMENTS: &str = "No Pages deployments";
pub const DETAIL_PAGES_MALFORMED: &str = "Malformed Pages deployment response";
pub const DETAIL_PAGES_UNAVAILABLE: &str = "Pages deployment status unavailable";
pub const DATA_STALE: &str = "Cloudflare data is stale or unavailable";
pub const DATA_PENDING: &str = "Cloudflare data pending refresh";
pub const DATA_CATALOG_UNAVAILABLE: &str = "Resource catalog unavailable; previous state unverified";
pub const DATA_WAF_UNAVAILABLE: &str = "WAF analytics unavailable; previous alert unverified";
pub const DATA_WORKERS_ANALYTICS_UNAVAILABLE: &str = "Workers analytics unavailable; previous spike unverified";
pub const DATA_CREDENTIALS_UNAVAILABLE: &str = "Cloudflare credentials unavailable";
pub const DATA_PARTIAL: &str = "Some Cloudflare resources are unverified";
pub const ERROR_HISTORY_NOT_FOUND: &str = "History event not found";
pub const ERROR_HISTORY_DESTINATION: &str = "Invalid Cloudflare history destination";
pub const ERROR_CONFIG_PATH: &str = "Config path unavailable";
pub const ERROR_CONFIG_DIRECTORY: &str = "Cannot create config directory";
pub const ERROR_CREDENTIAL_STORE: &str = "Operating system credential store unavailable";
pub const ERROR_TOKEN_LOCKED: &str = "Cloudflare API token not set or keychain locked";
pub const ERROR_AUTOSTART_CHECK: &str = "Could not check launch-at-login registration";
pub const ERROR_AUTOSTART_READ: &str = "Could not read launch-at-login registration";
pub const ERROR_AUTOSTART_UPDATE: &str = "Could not update launch-at-login registration";
pub const ERROR_ACCOUNT_ID: &str = "Account ID must be 32 hexadecimal characters";
pub const ERROR_INTERVAL: &str = "Unsupported refresh interval";
pub const ERROR_TOKEN_FORMAT: &str = "Invalid API token format";
pub const ERROR_TOKEN_WRITE: &str = "Could not store token in OS keychain";
pub const ERROR_CREDENTIAL_SERVICE: &str = "Credential service unavailable";
pub const ERROR_CARD_MISSING: &str = "Card unavailable";
pub const ERROR_RESIZE: &str = "Cannot resize card";
pub const ERROR_RESOURCE_MISSING: &str = "Resource not found";
pub const ERROR_EXTERNAL_DESTINATION: &str = "Unexpected external destination";
pub const ERROR_BROWSER: &str = "Could not open browser";
pub const CF_WORKER_LIST: &str = "Cloudflare Worker Scripts response must be an array";
pub const CF_BUILDER_INIT: &str = "Cannot initialize HTTPS client";
pub const CF_NETWORK: &str = "Cloudflare network request failed";
pub const CF_READ: &str = "Cloudflare response read failed";
pub const CF_TOO_LARGE: &str = "Cloudflare response exceeds safety limit";
pub const CF_BAD_JSON: &str = "Cloudflare returned invalid JSON";
pub const CF_UNSUCCESSFUL: &str = "Cloudflare API returned an unsuccessful response";
pub const CF_WORKERS_TOO_MANY: &str = "Cloudflare Workers count exceeds safe per-poll limit";
pub const CF_LIST_FORMAT: &str = "Cloudflare returned an unexpected list format";
pub const CF_PAGES_TOO_MANY: &str = "Cloudflare Pages count exceeds safe per-poll limit";
pub const CF_PAGES_PAGINATION: &str = "Too many pages in Cloudflare API response";
pub const CF_DISCOVERY_UNAVAILABLE: &str = "Workers and Pages discovery unavailable. Check token scopes or API limits.";
pub const ANALYTICS_MISSING_AGGREGATE: &str = "Missing Workers invocation aggregation";
pub const ANALYTICS_AMBIGUOUS: &str = "Workers invocation aggregation was not unique";
pub const ANALYTICS_REQUESTS_MISSING: &str = "Workers request total is missing";
pub const ANALYTICS_ERRORS_MISSING: &str = "Workers error total is missing";
pub const ANALYTICS_ERRORS_EXCEED: &str = "Workers errors exceed requests";
pub const ANALYTICS_BAD_ACCOUNT: &str = "Invalid account for analytics request";
pub const ANALYTICS_BAD_BATCH: &str = "Invalid Worker batch";
pub const ANALYTICS_NAME_ENCODING: &str = "Could not encode Worker name";
pub const ANALYTICS_REJECTED: &str = "Cloudflare Workers Analytics rejected the query";
pub const ANALYTICS_ACCOUNT_UNAVAILABLE: &str = "Cloudflare Workers Analytics account unavailable";
pub const ANALYTICS_TOO_MANY: &str = "Too many Workers for bounded analytics scan";
pub const ANALYTICS_CLIENT_INIT: &str = "Cannot initialize Analytics HTTPS client";
pub const ANALYTICS_REQUEST_FAILED: &str = "Workers Analytics request failed";
pub const ANALYTICS_RESPONSE_FAILED: &str = "Workers Analytics response failed";
pub const ANALYTICS_RESPONSE_BIG: &str = "Workers Analytics response too large";
pub const ANALYTICS_JSON: &str = "Invalid Workers Analytics JSON";
pub const WAF_GROUP_MISSING: &str = "WAF aggregate is missing";
pub const WAF_AMBIGUOUS: &str = "WAF query returned multiple aggregate groups";
pub const WAF_INVALID_COUNT: &str = "WAF aggregate contains an invalid count";
pub const WAF_BAD_COUNT: &str = "Invalid WAF aggregate count";
pub const WAF_REJECTED: &str = "Cloudflare WAF query rejected";
pub const WAF_DATA_UNAVAILABLE: &str = "Cloudflare WAF dataset unavailable";
pub const WAF_ZONES_ERROR: &str = "Cloudflare zones query failed";
pub const WAF_ZONES_INVALID: &str = "Invalid Cloudflare zones response";
pub const WAF_CLIENT_INIT: &str = "Could not start Cloudflare WAF client";
pub const WAF_ZONES_UNAVAILABLE: &str = "Cloudflare zone list unavailable";
pub const WAF_ZONES_READ: &str = "Could not read Cloudflare zones";
pub const WAF_ZONES_BIG: &str = "Cloudflare zone list too large";
pub const WAF_ZONES_JSON: &str = "Malformed Cloudflare zones JSON";
pub const WAF_ZONES_TOO_MANY: &str = "Too many Cloudflare zones to scan safely";
pub const WAF_REQUEST_FAILED: &str = "Cloudflare WAF GraphQL request failed";
pub const WAF_READ_FAILED: &str = "Could not read WAF response";
pub const WAF_RESPONSE_BIG: &str = "Cloudflare WAF response too large";
pub const WAF_RESPONSE_JSON: &str = "Malformed Cloudflare WAF JSON";
pub const WAF_BAD_ACCOUNT: &str = "Invalid Cloudflare account ID";
pub const BUDGET_RATE_LIMIT: &str = "Cloudflare rate limit active; waiting before next poll";
pub const BUDGET_EXHAUSTED: &str = "Cloudflare per-poll request budget exhausted";
pub const STORAGE_NOT_FILE: &str = "Persistent state path is not a regular file";
pub const STORAGE_INSPECT: &str = "Cannot inspect persisted state";
pub const STORAGE_PROTECT: &str = "Cannot protect damaged state";
pub const STORAGE_FILENAME: &str = "Unexpected state filename";
pub const STORAGE_BACKUP_EXISTS: &str = "Damaged state backup already exists";
pub const STORAGE_BACKUP_FAILED: &str = "Cannot preserve damaged state backup";
pub const STORAGE_OPEN: &str = "Cannot open persisted state";
pub const STORAGE_PERMISSIONS: &str = "Cannot secure persisted state permissions";
pub const STORAGE_READ: &str = "Cannot read persisted state";
pub const STORAGE_SERIALIZE: &str = "Cannot serialize persisted state";
pub const STORAGE_BIG: &str = "Persisted state exceeds safety limit";
pub const STORAGE_PERSIST: &str = "Cannot durably persist state";

pub fn worker_spike_detail(errors: u64, requests: u64, rate: f64) -> String {
    format!("Workers errors spiking: {} errors / {} requests ({:.1}%)", errors, requests, rate * 100.0)
}
pub fn waf_spike_detail(recent: f64, baseline: f64) -> String {
    format!("WAF blocks spiking: {:.0} in 10m; previous 30m {:.0}", recent, baseline)
}
