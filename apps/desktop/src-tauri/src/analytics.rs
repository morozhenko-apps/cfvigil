//! Read-only Workers invocation error-rate anomaly detector.
//! Per-worker aggregations use Cloudflare's documented GraphQL query pattern
//! with limit: 1 and scriptName filters. Summing a first page of sampled raw
//! adaptive rows would silently omit traffic, so it is explicitly forbidden.
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::{redirect::Policy, Client};
use serde_json::Value;
use std::{collections::{HashMap, HashSet}, time::Duration};

const GRAPHQL: &str = "https://api.cloudflare.com/client/v4/graphql";
const MAX_BATCH: usize = 8;
const MAX_WORKERS: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Metrics {
    pub requests: f64,
    pub errors: f64,
}

#[derive(Clone, Debug)]
pub struct Spike {
    pub requests: u64,
    pub errors: u64,
    pub ratio: f64,
}

fn worker_name_safe(name: &str) -> bool {
    !name.is_empty() && name.len() <= 128 &&
        name.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
}

fn parse_number(value: &Value) -> Option<f64> {
    value.as_f64().filter(|num| num.is_finite() && *num >= 0.0)
}

fn read_aggregate(account: &Value, alias: &str) -> Result<Metrics, String> {
    let rows = account.get(alias).and_then(Value::as_array)
        .ok_or("Missing Workers invocation aggregation")?;
    if rows.len() > 1 {
        // A limit-one filter must never produce ambiguous partial results.
        return Err("Workers invocation aggregation was not unique".into());
    }
    let Some(row) = rows.first() else { return Ok(Metrics::default()) };
    let requests = row.pointer("/sum/requests").and_then(parse_number)
        .ok_or("Workers request total is missing")?;
    let errors = row.pointer("/sum/errors").and_then(parse_number)
        .ok_or("Workers error total is missing")?;
    if errors > requests { return Err("Workers errors exceed requests".into()); }
    Ok(Metrics { requests, errors })
}

pub fn is_spike(recent: Metrics, baseline: Metrics) -> bool {
    if recent.requests < 100.0 || recent.errors < 10.0 { return false; }
    let ratio = recent.errors / recent.requests;
    if ratio < 0.10 { return false; }
    if baseline.requests < 100.0 {
        // Unreliable baseline: demand stronger absolute evidence.
        return recent.errors >= 20.0 && ratio >= 0.20;
    }
    let old_ratio = baseline.errors / baseline.requests;
    ratio >= old_ratio * 3.0 && ratio >= old_ratio + 0.05
}

fn create_query(
    account: &str, names: &[String], recent_start: &str,
    recent_end: &str, baseline_start: &str,
) -> Result<String, String> {
    if account.len() != 32 || !account.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid account for analytics request".into());
    }
    if names.is_empty() || names.len() > MAX_BATCH || names.iter().any(|n| !worker_name_safe(n)) {
        return Err("Invalid Worker batch".into());
    }
    // The only dynamic values are validated names/IDs and RFC3339 timestamps.
    // serde_json quoting is valid for a GraphQL string literal.
    let mut query = format!("query CFVigilMetrics {{ viewer {{ accounts(filter: {{accountTag: \"{account}\"}}) {{ ");
    for (index, name) in names.iter().enumerate() {
        let script = serde_json::to_string(name).map_err(|_| "Could not encode Worker name")?;
        query.push_str(&format!(
            "r{index}: workersInvocationsAdaptive(limit: 1, filter: {{scriptName: {script}, datetime_geq: \"{recent_start}\", datetime_lt: \"{recent_end}\"}}) {{sum {{requests errors}}}} "
        ));
        query.push_str(&format!(
            "b{index}: workersInvocationsAdaptive(limit: 1, filter: {{scriptName: {script}, datetime_geq: \"{baseline_start}\", datetime_lt: \"{recent_start}\"}}) {{sum {{requests errors}}}} "
        ));
    }
    query.push_str("} } }");
    Ok(query)
}

fn parse_batch(
    body: &Value, names: &[String],
) -> Result<HashMap<String, Spike>, String> {
    if body.get("errors").and_then(Value::as_array)
        .is_some_and(|errors| !errors.is_empty()) {
        return Err("Cloudflare Workers Analytics rejected the query".into());
    }
    let account = body.pointer("/data/viewer/accounts/0")
        .ok_or("Cloudflare Workers Analytics account unavailable")?;
    let mut result = HashMap::new();
    for (index, name) in names.iter().enumerate() {
        let recent = read_aggregate(account, &format!("r{index}"))?;
        let baseline = read_aggregate(account, &format!("b{index}"))?;
        if is_spike(recent, baseline) {
            result.insert(name.clone(), Spike {
                requests: recent.requests.round() as u64,
                errors: recent.errors.round() as u64,
                ratio: recent.errors / recent.requests,
            });
        }
    }
    Ok(result)
}

pub async fn query_spikes(
    account: &str, token: &str, allowed_workers: &HashSet<String>,
) -> Result<HashMap<String, Spike>, String> {
    if allowed_workers.is_empty() { return Ok(HashMap::new()); }
    if allowed_workers.len() > MAX_WORKERS {
        return Err("Too many Workers for bounded analytics scan".into());
    }
    let mut names: Vec<String> = allowed_workers.iter().cloned().collect();
    names.sort();
    let now = Utc::now();
    // Leave ingestion room for edge telemetry.
    let recent_end = now - ChronoDuration::minutes(5);
    let recent_start = recent_end - ChronoDuration::minutes(10);
    let baseline_start = recent_start - ChronoDuration::minutes(30);
    let http = Client::builder().redirect(Policy::none())
        .timeout(Duration::from_secs(20)).user_agent("CFVigil/0.1.0")
        .build().map_err(|_| "Cannot initialize Analytics HTTPS client")?;
    let mut spikes = HashMap::new();
    for batch in names.chunks(MAX_BATCH) {
        let query = create_query(account, batch, &recent_start.to_rfc3339(),
            &recent_end.to_rfc3339(), &baseline_start.to_rfc3339())?;
        let response = http.post(GRAPHQL).bearer_auth(token)
            .json(&serde_json::json!({"query":query}))
            .send().await.map_err(|_| "Workers Analytics request failed")?;
        if !response.status().is_success() {
            return Err(format!("Workers Analytics HTTP {}", response.status().as_u16()));
        }
        let bytes = response.bytes().await.map_err(|_| "Workers Analytics response failed")?;
        if bytes.len() > 1024 * 1024 {
            return Err("Workers Analytics response too large".into());
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid Workers Analytics JSON")?;
        spikes.extend(parse_batch(&body, batch)?);
    }
    Ok(spikes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_low_volume_and_normal_error_rates() {
        assert!(!is_spike(Metrics {requests: 50.0, errors: 20.0}, Metrics::default()));
        assert!(!is_spike(Metrics {requests: 1000.0, errors: 5.0}, Metrics::default()));
        assert!(!is_spike(Metrics {requests: 1000.0, errors: 15.0}, Metrics::default()));
        assert!(is_spike(Metrics {requests: 200.0, errors: 30.0},
            Metrics {requests: 1000.0, errors: 10.0}));
    }

    #[test]
    fn baseline_must_be_significantly_lower() {
        assert!(!is_spike(Metrics {requests: 200.0, errors: 18.0},
            Metrics {requests: 1000.0, errors: 10.0}));
        assert!(!is_spike(Metrics {requests: 200.0, errors: 30.0},
            Metrics {requests: 1000.0, errors: 170.0}));
        assert!(is_spike(Metrics {requests: 100.0, errors: 22.0}, Metrics::default()));
    }

    #[test]
    fn graph_uses_script_filtered_limit_one_aggregates() {
        let names = vec!["growth-api".to_string(), "dailygouda".to_string()];
        let query = create_query(&"a".repeat(32), &names,
            "2026-10-23T10:00:00Z", "2026-10-23T10:10:00Z",
            "2026-10-23T09:30:00Z").unwrap();
        assert!(query.contains("r0: workersInvocationsAdaptive(limit: 1"));
        assert!(query.contains("b1: workersInvocationsAdaptive(limit: 1"));
        assert!(query.contains("scriptName: \"dailygouda\""));
        assert!(!query.contains("limit: 1000"));
        assert!(create_query(&"a".repeat(32), &["../../../evil".to_string()],
            "2026-10-23T10:00:00Z", "2026-10-23T10:10:00Z",
            "2026-10-23T09:30:00Z").is_err());
    }

    #[test]
    fn parse_aggregations_are_independently_attributed() {
        let response = json!({"data":{"viewer":{"accounts":[{
            "r0":[{"sum":{"requests":200,"errors":70}}],
            "b0":[{"sum":{"requests":1000,"errors":10}}],
            "r1":[{"sum":{"requests":200,"errors":0}}],
            "b1":[{"sum":{"requests":1000,"errors":10}}]
        }]}}});
        let result = parse_batch(&response, &["growth-api".into(), "dailygouda".into()]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("growth-api"));
    }

    #[test]
    fn refuses_unavailable_or_partial_analytics() {
        let workers = ["growth-api".into()];
        assert!(parse_batch(&json!({"errors":[{"message":"denied"}]}), &workers).is_err());
        assert!(parse_batch(&json!({"data":{"viewer":{"accounts":[{"r0":[]}]}}}), &workers).is_err());
        let malformed = json!({"data":{"viewer":{"accounts":[{
            "r0":[{"sum":{"requests":10,"errors":11}}],"b0":[]
        }]}}});
        assert!(parse_batch(&malformed, &workers).is_err());
    }
}
