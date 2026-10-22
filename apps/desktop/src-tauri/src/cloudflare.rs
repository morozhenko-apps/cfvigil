use reqwest::{redirect::Policy, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

const API: &str = "https://api.cloudflare.com/client/v4";
const PAGE_LIMIT: usize = 50;

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

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
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

pub struct Cloudflare {
    http: Client,
}

impl Cloudflare {
    pub fn new() -> Result<Self, String> {
        let http = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(20))
            .user_agent("CFVigil/0.1.0")
            .build()
            .map_err(|_| "Cannot initialize HTTPS client".to_string())?;
        Ok(Self { http })
    }

    async fn get(&self, token: &str, path: &str) -> Result<(Value, usize), String> {
        let response = self
            .http
            .get(format!("{API}{path}"))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| "Cloudflare network request failed".to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("Cloudflare HTTP {}", status.as_u16()));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| "Cloudflare response read failed".to_string())?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err("Cloudflare response exceeds safety limit".into());
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|_| "Cloudflare returned invalid JSON".to_string())?;
        if body.get("success") != Some(&Value::Bool(true)) {
            return Err("Cloudflare API returned an unsuccessful response".into());
        }
        let pages = body
            .pointer("/result_info/total_pages")
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        Ok((body["result"].clone(), pages))
    }

    async fn list_paginated(
        &self,
        token: &str,
        path: &str,
    ) -> Result<Vec<Value>, String> {
        let mut output = Vec::new();
        for page in 1..=PAGE_LIMIT {
            let delimiter = if path.contains('?') { '&' } else { '?' };
            let url = format!("{path}{delimiter}page={page}&per_page=100");
            let (result, total_pages) = self.get(token, &url).await?;
            let items = result
                .as_array()
                .ok_or_else(|| "Cloudflare returned an unexpected list format".to_string())?;
            output.extend(items.iter().cloned());
            if page >= total_pages || items.is_empty() {
                return Ok(output);
            }
        }
        Err("Too many pages in Cloudflare API response".into())
    }

    pub async fn discover(&self, account: &str, token: &str) -> Result<Vec<Resource>, String> {
        let base = format!("/accounts/{account}");
        // Independent failures are explicit. Never turn an unobservable build into green.
        let workers = self.list_paginated(token, &format!("{base}/workers/scripts")).await;
        let pages = self.list_paginated(token, &format!("{base}/pages/projects")).await;
        if workers.is_err() && pages.is_err() {
            return Err("Cloudflare denied access to Workers and Pages. Check token scopes.".into());
        }
        let mut resources = Vec::new();

        if let Ok(workers) = workers {
            let mut keyed = HashMap::<String, Value>::new();
            let mut script_list = Vec::new();
            for script in workers {
                let name = str_field(&script, "id");
                if !valid_name(name) { continue; }
                let tag = str_field(&script, "tag").to_owned();
                if !tag.is_empty() {
                    keyed.insert(tag.clone(), script.clone());
                }
                script_list.push((name.to_owned(), tag, script));
            }
            let tags = keyed.keys().cloned().collect::<Vec<_>>();
            let mut latest = HashMap::<String, Value>::new();
            let mut builds_available = true;
            for chunk in tags.chunks(20) {
                let path = format!(
                    "{base}/builds/builds/latest?external_script_ids={}",
                    chunk.join(",")
                );
                match self.get(token, &path).await {
                    Ok((result, _)) => {
                        if let Some(map) = result.get("builds").and_then(Value::as_object) {
                            for (key, value) in map {
                                latest.insert(key.to_owned(), value.clone());
                            }
                        }
                    }
                    Err(_) => {
                        builds_available = false;
                        break;
                    }
                }
            }
            for (name, tag, _) in script_list {
                let build = latest.get(&tag).or_else(|| latest.get(&name));
                let active = self.get(
                    token,
                    &format!("{base}/workers/scripts/{name}/deployments?page=1&per_page=1"),
                ).await.ok().and_then(|(result, _)| {
                    result.get("deployments")?.as_array()?.first().cloned()
                });
                let deployed_after_build = if let (Some(deployment), Some(build)) = (&active, build) {
                    let deployed = chrono::DateTime::parse_from_rfc3339(str_field(deployment, "created_on"));
                    let built = chrono::DateTime::parse_from_rfc3339(str_field(build, "created_on"));
                    matches!((deployed, built), (Ok(deployed), Ok(built)) if deployed > built)
                } else { false };
                let (state, attempt, timestamp, detail) = if builds_available && deployed_after_build {
                    let deployed = active.as_ref().expect("checked above");
                    ("healthy", str_field(deployed, "id"), str_field(deployed, "created_on"), "Latest Worker deployment")
                } else if let Some(build) = build.filter(|_| builds_available) {
                    (worker_state(build), str_field(build, "build_uuid"), str_field(build, "created_on"), "Latest Cloudflare Workers build")
                } else if builds_available {
                    match active.as_ref() {
                        Some(deployed) => ("healthy", str_field(deployed, "id"), str_field(deployed, "created_on"), "Latest Worker deployment"),
                        None => ("unknown", "", "", "No verified Worker deployment"),
                    }
                } else {
                    ("unknown", "", "", "Workers Builds unavailable")
                };
                resources.push(Resource {
                    key: format!("worker:{name}"),
                    name: name.clone(),
                    kind: "worker".into(),
                    status: state.into(),
                    changed_at: timestamp.into(),
                    attempt_id: attempt.into(),
                    url: format!("https://dash.cloudflare.com/{account}/workers/services/view/{name}/production"),
                    detail: detail.into(),
                });
            }
        }

        if let Ok(projects) = pages {
            for project in projects {
                let name = str_field(&project, "name");
                if !valid_name(name) { continue; }
                let name = name.to_owned();
                let result = self
                    .get(token, &format!("{base}/pages/projects/{name}/deployments?per_page=1&page=1"))
                    .await;
                let (state, attempt, timestamp, detail) = match result {
                    Ok((result, _)) => {
                        if let Some(deployment) = result.as_array().and_then(|arr| arr.first()) {
                            (
                                pages_state(deployment),
                                str_field(deployment, "id"),
                                str_field(deployment, "created_on"),
                                "Latest Cloudflare Pages deployment",
                            )
                        } else {
                            ("unknown", "", "", "No Pages deployments")
                        }
                    }
                    Err(_) => ("unknown", "", "", "Pages deployment status unavailable"),
                };
                resources.push(Resource {
                    key: format!("pages:{name}"),
                    name: name.clone(),
                    kind: "pages".into(),
                    status: state.into(),
                    changed_at: timestamp.into(),
                    attempt_id: attempt.into(),
                    url: format!("https://dash.cloudflare.com/{account}/pages/view/{name}"),
                    detail: detail.into(),
                });
            }
        }
        resources.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.kind.cmp(&b.kind)));
        Ok(resources)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
