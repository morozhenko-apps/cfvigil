//! Account-level read-only Workers invocation anomaly detection.
//! Query Cloudflare Analytics once per refresh, never one request per Worker.
use std::collections::{HashMap, HashSet};
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::{redirect::Policy, Client};
use serde_json::{json, Value};

const QUERY: &str = r#"
query CFVigilErrors(
  $account: string,
  $recentStart: string,
  $recentEnd: string,
  $baselineStart: string,
  $baselineEnd: string
) {
  viewer {
    accounts(filter: {accountTag: $account}) {
      recent: workersInvocationsAdaptive(
        limit: 1000,
        filter: {datetime_geq: $recentStart, datetime_leq: $recentEnd}
      ) {
        dimensions { scriptName }
        sum { requests errors }
      }
      baseline: workersInvocationsAdaptive(
        limit: 1000,
        filter: {datetime_geq: $baselineStart, datetime_leq: $baselineEnd}
      ) {
        dimensions { scriptName }
        sum { requests errors }
      }
    }
  }
}
"#;

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

fn number(value: &Value) -> Option<f64> {
    value.as_f64().filter(|value| value.is_finite() && *value >= 0.0)
}

fn summarize(value: &Value, name: &str) -> Result<HashMap<String, Metrics>, String> {
    let records = value.get(name).and_then(Value::as_array)
        .ok_or_else(|| "Missing Cloudflare Workers Analytics records".to_owned())?;
    // The query has an explicit limit; do not trust a silently truncated result.
    if records.len() >= 1000 {
        return Err("Cloudflare Analytics results may be truncated".into());
    }
    let mut totals = HashMap::new();
    for row in records {
        let worker = row.pointer("/dimensions/scriptName")
            .and_then(Value::as_str).ok_or("Invalid scriptName in analytics response")?;
        let requests = row.pointer("/sum/requests").and_then(number)
            .ok_or("Invalid requests in analytics response")?;
        let errors = row.pointer("/sum/errors").and_then(number)
            .ok_or("Invalid errors in analytics response")?;
        if errors > requests {
            return Err("Analytics errors exceed requests".into());
        }
        let metrics = totals.entry(worker.to_owned()).or_insert(Metrics::default());
        metrics.requests += requests;
        metrics.errors += errors;
    }
    Ok(totals)
}

pub fn is_spike(recent: Metrics, baseline: Metrics) -> bool {
    if recent.requests < 100.0 || recent.errors < 10.0 {
        return false;
    }
    let ratio = recent.errors / recent.requests;
    if ratio < 0.10 {
        return false;
    }
    if baseline.requests < 100.0 {
        // There is no statistically useful history. Demand stronger evidence.
        return recent.errors >= 20.0 && ratio >= 0.20;
    }
    let old_ratio = baseline.errors / baseline.requests;
    // Both a substantial absolute change and a relative jump are required.
    ratio >= old_ratio * 3.0 && ratio >= old_ratio + 0.05
}

pub fn extract_spikes(
    body: &Value,
    allowed_workers: &HashSet<String>,
) -> Result<HashMap<String, Spike>, String> {
    if body.get("errors").and_then(Value::as_array)
        .is_some_and(|errors| !errors.is_empty()) {
        return Err("Cloudflare GraphQL returned errors".into());
    }
    let account = body.pointer("/data/viewer/accounts/0")
        .ok_or("Workers Analytics is unavailable for this account")?;
    let recent = summarize(account, "recent")?;
    let baseline = summarize(account, "baseline")?;
    let mut spikes = HashMap::new();
    for (worker, sample) in recent {
        if !allowed_workers.contains(&worker) {
            continue;
        }
        let historical = baseline.get(&worker).copied().unwrap_or_default();
        if is_spike(sample, historical) {
            spikes.insert(worker, Spike {
                requests: sample.requests.round() as u64,
                errors: sample.errors.round() as u64,
                ratio: sample.errors / sample.requests,
            });
        }
    }
    Ok(spikes)
}

pub async fn query_spikes(
    account: &str,
    token: &str,
    allowed_workers: &HashSet<String>,
) -> Result<HashMap<String, Spike>, String> {
    if allowed_workers.is_empty() {
        return Ok(HashMap::new());
    }
    let now = Utc::now();
    // Leave a five-minute ingestion buffer to reduce false positives from
    // partially aggregated Cloudflare Metrics.
    let end = now - ChronoDuration::minutes(5);
    let recent_start = end - ChronoDuration::minutes(10);
    let baseline_start = recent_start - ChronoDuration::minutes(30);
    let request = json!({
        "query": QUERY,
        "variables": {
            "account": account,
            "recentStart": recent_start.to_rfc3339(),
            "recentEnd": end.to_rfc3339(),
            "baselineStart": baseline_start.to_rfc3339(),
            "baselineEnd": recent_start.to_rfc3339()
        }
    });
    let http = Client::builder()
        .redirect(Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("CFVigil/0.1.0")
        .build().map_err(|_| "Cannot configure Analytics HTTP client")?;
    let response = http.post("https://api.cloudflare.com/client/v4/graphql")
        .bearer_auth(token)
        .json(&request)
        .send().await
        .map_err(|_| "Cloudflare Analytics request failed")?;
    if !response.status().is_success() {
        return Err(format!("Workers Analytics HTTP {}", response.status().as_u16()));
    }
    let bytes = response.bytes().await.map_err(|_| "Workers Analytics read failed")?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Workers Analytics response too large".into());
    }
    let body: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "Invalid Workers Analytics JSON")?;
    extract_spikes(&body, allowed_workers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_normal_and_low_volume_error_rates() {
        assert!(!is_spike(Metrics {requests: 50.0,errors: 20.0}, Metrics::default()));
        assert!(!is_spike(Metrics {requests: 1000.0,errors: 5.0}, Metrics::default()));
        assert!(!is_spike(Metrics {requests: 1000.0,errors: 15.0}, Metrics::default()));
    }

    #[test]
    fn requires_a_real_increase_over_baseline() {
        let old = Metrics {requests: 1000.0,errors: 10.0};
        assert!(is_spike(Metrics {requests: 200.0,errors: 30.0}, old));
        assert!(!is_spike(Metrics {requests: 200.0,errors: 18.0}, old));
        let high_baseline = Metrics {requests: 1000.0,errors: 170.0};
        assert!(!is_spike(Metrics {requests: 200.0,errors: 30.0}, high_baseline));
    }

    #[test]
    fn stronger_threshold_when_no_baseline() {
        assert!(!is_spike(Metrics {requests: 100.0,errors: 11.0}, Metrics::default()));
        assert!(is_spike(Metrics {requests: 100.0,errors: 22.0}, Metrics::default()));
    }

    #[test]
    fn parsing_does_not_attribute_another_script_to_monitored_resource() {
        let data = json!({"data":{"viewer":{"accounts":[{
            "recent":[
                {"dimensions":{"scriptName":"ours"},"sum":{"requests":120,"errors":30}},
                {"dimensions":{"scriptName":"other"},"sum":{"requests":120,"errors":30}}
            ],
            "baseline":[{"dimensions":{"scriptName":"ours"},"sum":{"requests":800,"errors":3}}]
        }]}}});
        let allowed = HashSet::from(["ours".to_string()]);
        let result = extract_spikes(&data, &allowed).unwrap();
        assert!(result.contains_key("ours"));
        assert!(!result.contains_key("other"));
    }

    #[test]
    fn missing_or_truncated_analytics_is_not_an_incident() {
        let allowed = HashSet::from(["ours".to_string()]);
        assert!(extract_spikes(&json!({"errors":[{"message":"denied"}]}), &allowed).is_err());
        assert!(extract_spikes(&json!({}), &allowed).is_err());
        let invalid = json!({"data":{"viewer":{"accounts":[{
            "recent":[{"dimensions":{"scriptName":"ours"},"sum":{"requests":9,"errors":15}}],
            "baseline":[]
        }]}}});
        assert!(extract_spikes(&invalid, &allowed).is_err());
    }
}
