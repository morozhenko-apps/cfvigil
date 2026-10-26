use crate::native_messages as msg;
use reqwest::{redirect::Policy, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::{HashMap, HashSet}, time::Duration};
use crate::api_budget::ApiBudget;

const API: &str = "https://api.cloudflare.com/client/v4";
const PAGE_LIMIT: usize = 50;
// Cloudflare Pages can reject query list options with error 8000024.
// Discover the first page using the plain API path, then follow result_info.

fn cloudflare_failure(status: reqwest::StatusCode, body: &[u8]) -> String {
    // Never copy arbitrary API error text: messages can contain resource URLs
    // and account information. A bounded numeric code is safe and actionable.
    let code = serde_json::from_slice::<Value>(body).ok()
        .and_then(|json| json.get("errors")?.as_array()?.iter()
            .find_map(|error| error.get("code").and_then(Value::as_u64)));
    if let Some(code) = code {
        format!("Cloudflare HTTP {} (code {code})", status.as_u16())
    } else {
        format!("Cloudflare HTTP {}", status.as_u16())
    }
}

fn pages_page_path(path: &str, page: usize) -> String {
    if page == 1 { return path.to_owned(); }
    let delimiter = if path.contains('?') { '&' } else { '?' };
    format!("{path}{delimiter}page={page}")
}

/// Cloudflare timestamps are RFC3339; compare parsed instants, not the
/// textual timezone offset or assumed response order.
fn latest_pages_deployment(result: &Value) -> Option<&Value> {
    result.as_array()?.iter().max_by_key(|deployment| {
        chrono::DateTime::parse_from_rfc3339(str_field(deployment, "created_on")).ok()
    })
}

fn scoped_provider_warning(provider: &str, error: &str, scope: &str) -> String {
    // Authentication/scope errors can be identified by HTTP 403. HTTP 400
    // is a bad request and must never blame a token scope without evidence.
    if error.contains("HTTP 403") {
        format!("{provider}: {error} (check {scope})")
    } else {
        format!("{provider}: {error}")
    }
}
const MAX_RESOURCES_PER_TYPE: usize = 256;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub key: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub changed_at: String,
    pub attempt_id: String,
    pub url: String,
    pub detail: String,
}

fn valid_name(input: &str) -> bool {
    !input.is_empty()
        && input.len() <= 128
        && input
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36 && value.bytes().enumerate().all(|(i, byte)| {
        if matches!(i, 8 | 13 | 18 | 23) { byte == b'-' }
        else { byte.is_ascii_hexdigit() }
    })
}

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn first_time<'a>(value: &'a Value, fields: &[&str]) -> &'a str {
    fields.iter().map(|field| str_field(value, field))
        .find(|timestamp| !timestamp.is_empty()).unwrap_or("")
}

fn worker_event_time(build: &Value) -> &str {
    match str_field(build, "status") {
        "stopped" => first_time(build, &["stopped_on", "modified_on", "created_on"]),
        "running" => first_time(build, &["running_on", "modified_on", "created_on"]),
        "initializing" => first_time(build, &["initializing_on", "modified_on", "created_on"]),
        _ => first_time(build, &["created_on", "modified_on"]),
    }
}

fn pages_event_time(deployment: &Value) -> &str {
    let stage = &deployment["latest_stage"];
    let latest = match str_field(stage, "status") {
        "active" | "idle" => first_time(stage, &["started_on", "ended_on"]),
        _ => first_time(stage, &["ended_on", "started_on"]),
    };
    if !latest.is_empty() { latest }
    else { first_time(deployment, &["modified_on", "created_on"]) }
}

fn parse_worker_scripts(result: &Value) -> Result<Vec<Value>, String> {
    result.as_array()
        .cloned()
        .ok_or_else(|| msg::CF_WORKER_LIST.into())
}

// A new in-progress attempt must turn a previously failed resource yellow.
pub fn worker_state(build: &Value) -> &'static str {
    match str_field(build, "status") {
        "queued" | "initializing" | "running" => "deploying",
        "stopped" => match str_field(build, "build_outcome") {
            "success" => "healthy",
            "fail" | "cancelled" | "terminated" => "failed",
            _ => "unknown",
        },
        _ => "unknown",
    }
}

pub fn pages_state(deployment: &Value) -> &'static str {
    if deployment.get("is_skipped").and_then(Value::as_bool) == Some(true) {
        return "unknown";
    }
    let status = deployment
        .get("latest_stage")
        .and_then(|stage| stage.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("");
    match status {
        "active" | "idle" => "deploying",
        "success" => "healthy",
        "failure" => "failed",
        _ => "unknown",
    }
}

fn deployed_after_build(deployment: &Value, build: &Value) -> bool {
    let deployed = chrono::DateTime::parse_from_rfc3339(str_field(deployment, "created_on"));
    let built = chrono::DateTime::parse_from_rfc3339(str_field(build, "created_on"));
    matches!((deployed, built), (Ok(deployed), Ok(built)) if deployed > built)
}

#[derive(Debug, PartialEq, Eq)]
struct WorkerAttempt {
    status: &'static str,
    attempt_id: String,
    changed_at: String,
    detail: &'static str,
}

// The latest attempt wins. The active Worker deployment endpoint returns
// versions SERVING traffic, not the state of a more recent pending build.
fn select_worker_attempt(
    build: Option<&Value>,
    active: Option<&Value>,
    builds_available: bool,
) -> WorkerAttempt {
    let (status, id, time, detail) = if !builds_available {
        ("unknown", "", "", msg::DETAIL_BUILDS_UNAVAILABLE)
    } else if let (Some(deployed), Some(build)) = (active, build) {
        if deployed_after_build(deployed, build) {
            ("healthy", str_field(deployed, "id"), str_field(deployed, "created_on"), msg::DETAIL_WORKER_DEPLOYMENT)
        } else {
            (worker_state(build), str_field(build, "build_uuid"), worker_event_time(build), msg::DETAIL_WORKERS_BUILD)
        }
    } else if let Some(build) = build {
        (worker_state(build), str_field(build, "build_uuid"), worker_event_time(build), msg::DETAIL_WORKERS_BUILD)
    } else if let Some(deployed) = active {
        ("healthy", str_field(deployed, "id"), str_field(deployed, "created_on"), msg::DETAIL_WORKER_DEPLOYMENT)
    } else {
        ("unknown", "", "", msg::DETAIL_NO_WORKER_DEPLOYMENT)
    };
    WorkerAttempt {
        status,
        attempt_id: id.to_owned(),
        changed_at: time.to_owned(),
        detail,
    }
}

pub struct Cloudflare {
    http: Client,
}

pub struct Discovery {
    pub resources: Vec<Resource>,
    pub complete: bool,
    pub workers_complete: bool,
    pub pages_complete: bool,
    /// Sanitized endpoint labels + HTTP status only, never request URLs,
    /// resource names, account IDs, response bodies or credentials.
    pub warnings: Vec<String>,
}

impl Cloudflare {
    pub fn new() -> Result<Self, String> {
        let http = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(20))
            .user_agent("CFVigil/0.1.0")
            .build()
            .map_err(|_| msg::CF_BUILDER_INIT.to_string())?;
        Ok(Self { http })
    }

    async fn get(&self, token: &str, path: &str, budget: &ApiBudget) -> Result<(Value, usize), String> {
        budget.check()?;
        let mut response = self
            .http
            .get(format!("{API}{path}"))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| msg::CF_NETWORK.to_string())?;
        budget.observe(&response);
        let status = response.status();
        if !status.is_success() {
            // Read only a small bounded Cloudflare error body; use its numeric
            // code, never echo potentially sensitive free-form messages.
            let mut failure_body = Vec::new();
            while let Some(chunk) = response.chunk().await
                .map_err(|_| format!("Cloudflare HTTP {}", status.as_u16()))?
            {
                if failure_body.len() + chunk.len() > 8192 { break; }
                failure_body.extend_from_slice(&chunk);
            }
            return Err(cloudflare_failure(status, &failure_body));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| msg::CF_READ.to_string())?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(msg::CF_TOO_LARGE.into());
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|_| msg::CF_BAD_JSON.to_string())?;
        if body.get("success") != Some(&Value::Bool(true)) {
            return Err(msg::CF_UNSUCCESSFUL.into());
        }
        let pages = body
            .pointer("/result_info/total_pages")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        Ok((body["result"].clone(), pages))
    }

    // Worker Scripts List is NOT a paginated endpoint. Do not supply
    // page/per_page parameters that are not in its documented contract.
    async fn list_workers(&self, token: &str, base: &str, budget: &ApiBudget) -> Result<Vec<Value>, String> {
        let (result, _) = self.get(token, &format!("{base}/workers/scripts"), budget).await?;
        let scripts = parse_worker_scripts(&result)?;
        if scripts.len() > MAX_RESOURCES_PER_TYPE {
            return Err(msg::CF_WORKERS_TOO_MANY.into());
        }
        Ok(scripts)
    }

    async fn list_paginated(
        &self,
        token: &str,
        path: &str,
        budget: &ApiBudget,
    ) -> Result<Vec<Value>, String> {
        // Do not add page=1/per_page to the initial request: the live Pages
        // API returned 8000024 (invalid list options) with those parameters.
        let (first, total_pages) = self.get(token, path, budget).await?;
        let items = first.as_array().ok_or_else(|| msg::CF_LIST_FORMAT.to_string())?;
        if items.len() > MAX_RESOURCES_PER_TYPE {
            return Err(msg::CF_PAGES_TOO_MANY.into());
        }
        if total_pages > PAGE_LIMIT {
            return Err(msg::CF_PAGES_PAGINATION.into());
        }
        let mut output = items.clone();
        for page in 2..=total_pages {
            let url = pages_page_path(path, page);
            let (result, _) = self.get(token, &url, budget).await?;
            let rows = result.as_array().ok_or_else(|| msg::CF_LIST_FORMAT.to_string())?;
            if rows.is_empty() {
                // A claimed additional page must exist; do not call this a
                // complete catalog and silently lose Pages resources.
                return Err(msg::CF_PAGES_PAGINATION.into());
            }
            if output.len() + rows.len() > MAX_RESOURCES_PER_TYPE {
                return Err(msg::CF_PAGES_TOO_MANY.into());
            }
            output.extend(rows.iter().cloned());
        }
        Ok(output)
    }


    // Four in-flight reads per provider are enough to remove per-resource
    // serial latency without creating an unbounded Cloudflare request burst.
    // All calls still pass through the shared global ApiBudget.
    async fn worker_deployment(
        &self, token: &str, base: &str, row: Option<&(String, String, Value)>,
        budget: &ApiBudget,
    ) -> Option<Result<(Value, usize), String>> {
        let (name, _, _) = row?;
        Some(self.get(token,
            &format!("{base}/workers/scripts/{name}/deployments?page=1&per_page=1"),
            budget).await)
    }

    async fn pages_deployment(
        &self, token: &str, base: &str, project: Option<&Value>,
        budget: &ApiBudget,
    ) -> Option<Result<(Value, usize), String>> {
        let project = project?;
        let name = str_field(project, "name");
        if !valid_name(name) { return None; }
        Some(self.get(token,
            &format!("{base}/pages/projects/{name}/deployments"),
            budget).await)
    }

    pub async fn discover(&self, account: &str, token: &str, budget: &ApiBudget) -> Result<Discovery, String> {
        let base = format!("/accounts/{account}");
        // The two catalog endpoints are independent: a failing Pages scope
        // must not conceal verified Workers, and vice versa.
        let pages_path = format!("{base}/pages/projects");
        let (workers, pages) = tokio::join!(
            self.list_workers(token, &base, budget),
            self.list_paginated(token, &pages_path, budget)
        );
        let workers_complete = workers.is_ok();
        let pages_complete = pages.is_ok();
        if !workers_complete && !pages_complete {
            return Err(msg::CF_DISCOVERY_UNAVAILABLE.into());
        }
        let mut complete = workers_complete && pages_complete;
        let mut warnings = Vec::<String>::new();
        if let Err(error) = &workers {
            warnings.push(format!("Workers: {error}"));
        }
        if let Err(error) = &pages {
            warnings.push(scoped_provider_warning("Pages", error, "Cloudflare Pages Read"));
        }
        let mut resources = Vec::new();

        if let Ok(workers) = workers {
            let mut tags = HashSet::<String>::new();
            let mut script_list = Vec::new();
            for script in workers {
                let name = str_field(&script, "id");
                if !valid_name(name) { continue; }
                let tag = str_field(&script, "tag").to_owned();
                // Cloudflare's immutable external_script_id is 32 hex chars.
                // Do not interpolate unchecked API values into a URL query.
                if tag.len() == 32 && tag.bytes().all(|ch| ch.is_ascii_hexdigit()) {
                    tags.insert(tag.clone());
                }
                script_list.push((name.to_owned(), tag, script));
            }
            let mut tags = tags.into_iter().collect::<Vec<_>>();
            tags.sort();
            let mut latest = HashMap::<String, Value>::new();
            let mut checked_tags = HashSet::<String>::new();
            for chunk in tags.chunks(20) {
                let path = format!(
                    "{base}/builds/builds/latest?external_script_ids={}",
                    chunk.join(",")
                );
                match self.get(token, &path, budget).await {
                    Ok((result, _)) => {
                        if let Some(map) = result.get("builds").and_then(Value::as_object) {
                            for (key, value) in map {
                                latest.insert(key.to_owned(), value.clone());
                            }
                            checked_tags.extend(chunk.iter().cloned());
                        } else {
                            complete = false;
                            if !warnings.iter().any(|w| w.starts_with("Workers Builds:")) {
                                warnings.push("Workers Builds: invalid Cloudflare response".into());
                            }
                        }
                    }
                    Err(error) => {
                        complete = false;
                        if !warnings.iter().any(|w| w.starts_with("Workers Builds:")) {
                            warnings.push(scoped_provider_warning("Workers Builds", &error, "Workers CI Read"));
                        }
                    }
                }
            }
            for group in script_list.chunks(4) {
                let (a,b,c,d) = tokio::join!(
                    self.worker_deployment(token, &base, group.first(), budget),
                    self.worker_deployment(token, &base, group.get(1), budget),
                    self.worker_deployment(token, &base, group.get(2), budget),
                    self.worker_deployment(token, &base, group.get(3), budget)
                );
                for ((name, tag, _), active_result) in group.iter()
                    .zip([a,b,c,d].into_iter().flatten()) {
                    let build = latest.get(tag).or_else(|| latest.get(name));
                    let active = active_result.as_ref().ok()
                        .and_then(|(result, _)| result.get("deployments"))
                        .and_then(Value::as_array);
                    let deployment_known = active_result.is_ok() && active.is_some();
                    let builds_known = checked_tags.contains(tag);
                    if !deployment_known || !builds_known {
                        complete = false;
                        if !deployment_known && !warnings.iter().any(|w| w.starts_with("Worker deployments:")) {
                            let reason = active_result.as_ref().err().map(String::as_str)
                                .unwrap_or("unexpected Cloudflare response");
                            warnings.push(format!("Worker deployments: {reason}"));
                        }
                    }
                    let choice = if !deployment_known {
                        select_worker_attempt(None, None, false)
                    } else {
                        select_worker_attempt(build, active.and_then(|rows| rows.first()), builds_known)
                    };
                    let mut url = format!("https://dash.cloudflare.com/{account}/workers/services/view/{name}/production");
                    if choice.detail == msg::DETAIL_WORKERS_BUILD && valid_uuid(&choice.attempt_id) {
                        url.push_str(&format!("/builds/{}", choice.attempt_id));
                    }
                    resources.push(Resource {
                        key: format!("worker:{name}"),
                        name: name.clone(),
                        kind: "worker".into(),
                        status: choice.status.into(),
                        changed_at: choice.changed_at,
                        attempt_id: choice.attempt_id,
                        url,
                        detail: choice.detail.into(),
                    });
                }
            }
        }

        if let Ok(projects) = pages {
            for group in projects.chunks(4) {
                let (a,b,c,d) = tokio::join!(
                    self.pages_deployment(token, &base, group.first(), budget),
                    self.pages_deployment(token, &base, group.get(1), budget),
                    self.pages_deployment(token, &base, group.get(2), budget),
                    self.pages_deployment(token, &base, group.get(3), budget)
                );
                for (project, result) in group.iter().zip([a,b,c,d].into_iter()) {
                    let name = str_field(project, "name");
                    if !valid_name(name) { continue; }
                    let name = name.to_owned();
                    let Some(result) = result else { continue; };
                // Own the strings before the REST result is dropped. Holding
                // &str into a JSON Value from this match caused Rust E0597.
                let (state, attempt, timestamp, detail) = match result {
                    Ok((result, _)) => {
                        if let Some(deployment) = latest_pages_deployment(&result) {
                            (
                                pages_state(deployment).to_owned(),
                                str_field(deployment, "id").to_owned(),
                                pages_event_time(deployment).to_owned(),
                                msg::DETAIL_PAGES_DEPLOYMENT.to_owned(),
                            )
                        } else if result.as_array().is_some() {
                            ("unknown".into(), String::new(), String::new(), msg::DETAIL_NO_PAGES_DEPLOYMENTS.into())
                        } else {
                            complete = false;
                            ("unknown".into(), String::new(), String::new(), msg::DETAIL_PAGES_MALFORMED.into())
                        }
                    }
                    Err(error) => {
                        complete = false;
                        if !warnings.iter().any(|w| w.starts_with("Pages deployments:")) {
                            warnings.push(scoped_provider_warning("Pages deployments", &error, "Cloudflare Pages Read"));
                        }
                        ("unknown".into(), String::new(), String::new(), msg::DETAIL_PAGES_UNAVAILABLE.into())
                    },
                };
                let mut url = format!("https://dash.cloudflare.com/{account}/pages/view/{name}");
                if valid_uuid(&attempt) { url.push_str(&format!("/{attempt}")); }
                    let mut url = format!("https://dash.cloudflare.com/{account}/pages/view/{name}");
                    if valid_uuid(&attempt) { url.push_str(&format!("/{attempt}")); }
                    resources.push(Resource {
                        key: format!("pages:{name}"),
                        name: name.clone(),
                        kind: "pages".into(),
                        status: state,
                        changed_at: timestamp,
                        attempt_id: attempt,
                        url,
                        detail,
                    });
                }
            }
        }
        resources.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.kind.cmp(&b.kind)));
        Ok(Discovery { resources, complete, workers_complete, pages_complete, warnings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mode_b_provider_errors_never_confuse_bad_requests_with_scopes_or_leak_text() {
        let unsafe_error = br#"{"errors":[{"message":"private URL and account information","code":1015}]}"#;
        assert_eq!(cloudflare_failure(reqwest::StatusCode::TOO_MANY_REQUESTS, unsafe_error),
            "Cloudflare HTTP 429 (code 1015)");
        assert_eq!(cloudflare_failure(reqwest::StatusCode::BAD_REQUEST,
            br#"{"errors":[{"code":"8000024","message":"secret"}]}"#), "Cloudflare HTTP 400");
        assert_eq!(scoped_provider_warning("Pages", "Cloudflare HTTP 400 (code 8000024)",
            "Cloudflare Pages Read"), "Pages: Cloudflare HTTP 400 (code 8000024)");
        assert_eq!(scoped_provider_warning("Pages", "Cloudflare HTTP 429 (code 1015)",
            "Cloudflare Pages Read"), "Pages: Cloudflare HTTP 429 (code 1015)");
        assert_eq!(scoped_provider_warning("Workers Builds", "Cloudflare HTTP 403",
            "Workers CI Read"), "Workers Builds: Cloudflare HTTP 403 (check Workers CI Read)");
    }

    #[test]
    fn mode_b_latest_attempts_use_absolute_times_and_uncertainty_not_response_order() {
        let unordered = json!([
            {"id":"later-clock-earlier-utc","created_on":"2026-10-26T12:30:00+03:00"},
            {"id":"earlier-clock-later-utc","created_on":"2026-10-26T10:00:00Z"},
            {"id":"invalid","created_on":"not-an-instant"}
        ]);
        let chosen = latest_pages_deployment(&unordered).unwrap();
        assert_eq!(str_field(chosen, "id"), "earlier-clock-later-utc");
        let latest_build = json!({"build_uuid":"a", "created_on":"2026-10-26T10:01:00Z",
            "status":"queued", "build_outcome":"fail"});
        let old_live = json!({"id":"b", "created_on":"2026-10-26T10:00:00Z"});
        assert_eq!(select_worker_attempt(Some(&latest_build), Some(&old_live), true).status,
            "deploying");
        assert_eq!(select_worker_attempt(Some(&latest_build), Some(&old_live), false).status,
            "unknown");
        assert_eq!(select_worker_attempt(None, Some(&old_live), false).status,
            "unknown");
    }

    #[test]
    fn cloudflare_error_messages_expose_only_numeric_codes_not_private_details() {
        let body = br#"{"errors":[{"code":7001,"message":"account 123: secret detail"}]}"#;
        assert_eq!(
            cloudflare_failure(reqwest::StatusCode::BAD_REQUEST, body),
            "Cloudflare HTTP 400 (code 7001)"
        );
        assert_eq!(
            cloudflare_failure(reqwest::StatusCode::FORBIDDEN, b"not json"),
            "Cloudflare HTTP 403"
        );
    }

    #[test]
    fn pages_initial_request_has_no_list_options_and_followup_uses_page_only() {
        assert_eq!(
            pages_page_path("/accounts/abc/pages/projects", 1),
            "/accounts/abc/pages/projects"
        );
        assert_eq!(
            pages_page_path("/accounts/abc/pages/projects", 2),
            "/accounts/abc/pages/projects?page=2"
        );
        assert_eq!(
            pages_page_path("/accounts/abc/pages/projects?order=name", 3),
            "/accounts/abc/pages/projects?order=name&page=3"
        );
        assert!(!pages_page_path("/accounts/abc/pages/projects", 1).contains('?'));
        assert_eq!(
            scoped_provider_warning("Pages", "Cloudflare HTTP 400", "Cloudflare Pages Read"),
            "Pages: Cloudflare HTTP 400"
        );
        assert_eq!(
            scoped_provider_warning("Pages", "Cloudflare HTTP 403", "Cloudflare Pages Read"),
            "Pages: Cloudflare HTTP 403 (check Cloudflare Pages Read)"
        );
    }

    #[test]
    fn pages_latest_deployment_uses_timestamps_not_provider_array_order() {
        let unordered = json!([
            {"id":"old", "created_on":"2026-10-24T15:00:00Z"},
            {"id":"new", "created_on":"2026-10-25T09:00:00Z"},
            {"id":"middle", "created_on":"2026-10-24T23:00:00Z"}
        ]);
        assert_eq!(str_field(latest_pages_deployment(&unordered).unwrap(), "id"), "new");
        assert!(latest_pages_deployment(&json!([])).is_none());
        assert!(latest_pages_deployment(&json!({"result":[]})).is_none());
    }

    #[test]
    fn worker_list_contract_expects_array_without_silent_defaults() {
        let workers = json!([{"id":"a", "tag":"1234"}, {"id":"b"}]);
        assert_eq!(parse_worker_scripts(&workers).unwrap().len(), 2);
        assert!(parse_worker_scripts(&json!({"result":[]})).is_err());
        assert!(parse_worker_scripts(&Value::Null).is_err());
    }

    #[test]
    fn next_worker_attempt_overrides_previous_failure() {
        assert_eq!(worker_state(&json!({"status":"stopped","build_outcome":"fail"})), "failed");
        assert_eq!(worker_state(&json!({"status":"running","build_outcome":null})), "deploying");
        assert_eq!(worker_state(&json!({"status":"queued","build_outcome":"fail"})), "deploying");
    }

    #[test]
    fn worker_terminal_states_are_explicit() {
        assert_eq!(worker_state(&json!({"status":"stopped","build_outcome":"success"})), "healthy");
        assert_eq!(worker_state(&json!({"status":"stopped","build_outcome":"skipped"})), "unknown");
        assert_eq!(worker_state(&json!({"status":"stopped","build_outcome":"terminated"})), "failed");
        assert_eq!(worker_state(&json!({})), "unknown");
    }

    #[test]
    fn pages_latest_deployment_state_is_authoritative() {
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"active"}})), "deploying");
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"failure"}})), "failed");
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"success"}})), "healthy");
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"skipped"}})), "unknown");
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"canceled"}})), "unknown");
        assert_eq!(pages_state(&json!({"is_skipped":true,"latest_stage":{"status":"success"}})), "unknown");
        assert_eq!(pages_state(&json!({"is_skipped":false,"latest_stage":{"status":"success"}})), "healthy");
        assert_eq!(pages_state(&json!({"latest_stage":{"status":"active"}})), "deploying");
    }

    #[test]
    fn cloudflare_response_fixtures_prefer_newer_build_attempts() {
        // Shapes follow documented GET /builds/builds/latest and
        // GET /workers/scripts/{name}/deployments success responses.
        let tag = "dd7160bb9cef458093557736f4b9e75b";
        let response = json!({"success":true, "result":{"builds": {
            "dd7160bb9cef458093557736f4b9e75b": {"build_uuid":"182bd5e5-6e1a-4fe4-a799-aa6d9a6ab26e",
                  "created_on":"2026-10-23T12:00:00Z", "status":"running"}
        }}});
        let build = &response["result"]["builds"][tag];
        let deployment = json!({"result":{"deployments":[{
            "id":"023e105f-2a42-4f8b-a1c1-73f6a2a30c0f",
            "created_on":"2026-10-23T11:59:00Z"
        }]}});
        let active = &deployment["result"]["deployments"][0];
        let chosen = select_worker_attempt(Some(build), Some(active), true);
        assert_eq!(chosen.status, "deploying");
        assert_eq!(chosen.detail, "Latest Cloudflare Workers build");
        assert_eq!(chosen.changed_at, "2026-10-23T12:00:00Z");

        let failed = json!({"created_on":"2026-10-23T12:00:00Z",
            "build_uuid":"182bd5e5-6e1a-4fe4-a799-aa6d9a6ab26e",
            "status":"stopped", "build_outcome":"fail"});
        assert_eq!(select_worker_attempt(Some(&failed), Some(active), true).status, "failed");
        let manual = json!({"id":"023e105f-2a42-4f8b-a1c1-73f6a2a30c0f",
            "created_on":"2026-10-23T12:01:00Z"});
        assert_eq!(select_worker_attempt(Some(&failed), Some(&manual), true).status, "healthy");
    }

    #[test]
    fn unavailable_build_information_cannot_be_inferred_as_healthy() {
        let active = json!({"id":"023e105f-2a42-4f8b-a1c1-73f6a2a30c0f",
            "created_on":"2026-10-23T12:01:00Z"});
        assert_eq!(select_worker_attempt(None, Some(&active), false).status, "unknown");
        assert_eq!(select_worker_attempt(None, Some(&active), true).status, "healthy");
        assert_eq!(select_worker_attempt(None, None, true).status, "unknown");
    }

    #[test]
    fn newer_active_deployment_can_resolve_an_older_failed_build() {
        let build = json!({"created_on":"2026-10-22T12:00:00Z", "status":"stopped", "build_outcome":"fail"});
        let newer = json!({"created_on":"2026-10-22T12:01:00Z"});
        let older = json!({"created_on":"2026-10-22T11:59:00Z"});
        assert!(deployed_after_build(&newer, &build));
        assert!(!deployed_after_build(&older, &build));
        assert!(!deployed_after_build(&json!({}), &build));
    }

    #[test]
    fn status_transition_time_is_not_build_creation_time() {
        let build = json!({
            "status": "stopped", "created_on":"2026-10-23T09:00:00Z",
            "running_on":"2026-10-23T09:01:00Z",
            "stopped_on":"2026-10-23T09:20:00Z"
        });
        assert_eq!(worker_event_time(&build), "2026-10-23T09:20:00Z");
        let running = json!({
            "status":"running", "created_on":"2026-10-23T09:00:00Z",
            "running_on":"2026-10-23T09:05:00Z"
        });
        assert_eq!(worker_event_time(&running), "2026-10-23T09:05:00Z");
        assert_eq!(worker_event_time(&json!({"status":"queued","created_on":"2026-10-23T09:00:00Z"})),
            "2026-10-23T09:00:00Z");
    }

    #[test]
    fn pages_stage_event_time_tracks_completion() {
        let done = json!({
            "created_on":"2026-10-23T09:00:00Z",
            "latest_stage":{"status":"failure","started_on":"2026-10-23T09:01:00Z","ended_on":"2026-10-23T09:03:00Z"}
        });
        assert_eq!(pages_event_time(&done), "2026-10-23T09:03:00Z");
        let active = json!({"latest_stage":{"status":"active","started_on":"2026-10-23T09:02:00Z"}});
        assert_eq!(pages_event_time(&active), "2026-10-23T09:02:00Z");
        let missing = json!({"created_on":"2026-10-23T09:00:00Z","latest_stage":{}});
        assert_eq!(pages_event_time(&missing), "2026-10-23T09:00:00Z");
    }

    #[test]
    fn accepts_only_real_uuid_shapes_for_deep_links() {
        assert!(valid_uuid("d1a48d17-1547-4097-9ac4-26f092366eb4"));
        assert!(!valid_uuid("unknown"));
        assert!(!valid_uuid("a1234567-aaaa-bbbb-cccc-dddddddddddd/redirect"));
        assert!(!valid_uuid(""));
    }

    #[test]
    fn reject_untrusted_names_in_generated_dashboard_links() {
        assert!(valid_name("growth-api"));
        assert!(valid_name("personal_landing"));
        assert!(!valid_name("../redirect?token=x"));
        assert!(!valid_name("name/other"));
        assert!(!valid_name(""));
    }
}
