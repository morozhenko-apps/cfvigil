//! Read-only, zone-scoped WAF block anomaly detection.
//! WAF events belong to Cloudflare zones; never attribute one to a Worker
//! unless Cloudflare provides a trustworthy resource-level mapping.
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::{redirect::Policy, Client};
use serde_json::{json, Value};
use std::{collections::HashSet, time::Duration};
use crate::api_budget::ApiBudget;

use crate::cloudflare::Resource;

const BASE: &str = "https://api.cloudflare.com/client/v4";
const MAX_ZONES: usize = 30;
const MAX_GRAPHQL_BYTES: usize = 256 * 1024;

// A grouped query without dimensions returns one aggregate count per window.
// Raw WAF event logs can be sampled and truncated, so never count raw rows.
const QUERY: &str = r#"
query CFVigilWaf(
  $zoneTag: string,
  $recentStart: string,
  $recentEnd: string,
  $baselineStart: string,
  $baselineEnd: string
) {
  viewer {
    zones(filter: {zoneTag: $zoneTag}) {
      recent: firewallEventsAdaptiveGroups(
        filter: {action: "block", datetime_geq: $recentStart, datetime_lt: $recentEnd}
        limit: 1
      ) { count }
      baseline: firewallEventsAdaptiveGroups(
        filter: {action: "block", datetime_geq: $baselineStart, datetime_lt: $baselineEnd}
        limit: 1
      ) { count }
    }
  }
}
"#;

#[derive(Clone, Debug)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

pub struct WafScan {
    pub incidents: Vec<Resource>,
    // Successfully checked zone keys, including zones without an incident.
    // Only verified zones can be used as evidence that an old spike cleared.
    pub verified_zones: HashSet<String>,
}

fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_zone_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 253
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-'
        })
        && !name.starts_with('.')
        && !name.contains("..")
}

fn count_group(data: &Value, key: &str) -> Result<f64, String> {
    let groups = data.get(key).and_then(Value::as_array)
        .ok_or_else(|| "WAF aggregate is missing".to_string())?;
    if groups.len() > 1 {
        return Err("WAF query returned multiple aggregate groups".into());
    }
    let count = match groups.first() {
        None => 0.0,
        Some(group) => group.get("count").and_then(Value::as_f64)
            .ok_or("WAF aggregate contains an invalid count")?,
    };
    if !count.is_finite() || count < 0.0 {
        return Err("Invalid WAF aggregate count".into());
    }
    Ok(count)
}

// Recent: 10 minutes. Baseline: preceding 30 minutes.
// Avoid claiming an "attack" from a single hit or normal ongoing filtering.
pub fn waf_spike(recent: f64, baseline_30m: f64) -> bool {
    if !recent.is_finite() || !baseline_30m.is_finite() {
        return false;
    }
    if recent < 100.0 || baseline_30m < 0.0 {
        return false;
    }
    let expected_10m = baseline_30m / 3.0;
    recent >= expected_10m * 5.0 && recent >= expected_10m + 80.0
}

pub fn parse_waf_counts(body: &Value) -> Result<(f64, f64), String> {
    if body.get("errors").and_then(Value::as_array)
        .is_some_and(|errors| !errors.is_empty()) {
        return Err("Cloudflare WAF query rejected".into());
    }
    let zone = body.pointer("/data/viewer/zones/0")
        .ok_or("Cloudflare WAF dataset unavailable")?;
    let recent = count_group(zone, "recent")?;
    let baseline = count_group(zone, "baseline")?;
    Ok((recent, baseline))
}

fn parse_zones(body: &Value, account: &str) -> Result<Vec<Zone>, String> {
    if body.get("success") != Some(&Value::Bool(true)) {
        return Err("Cloudflare zones query failed".into());
    }
    let input = body.get("result").and_then(Value::as_array)
        .ok_or("Invalid Cloudflare zones response")?;
    let mut output = Vec::new();
    for entry in input {
        // Defensive scope check: a user-scoped token may include multiple accounts.
        if entry.pointer("/account/id").and_then(Value::as_str) != Some(account) {
            continue;
        }
        let id = entry.get("id").and_then(Value::as_str).unwrap_or("");
        let name = entry.get("name").and_then(Value::as_str).unwrap_or("");
        if valid_id(id) && valid_zone_name(name) {
            output.push(Zone { id: id.to_owned(), name: name.to_owned() });
        }
    }
    Ok(output)
}

fn http_client() -> Result<Client, String> {
    Client::builder().redirect(Policy::none())
        .timeout(Duration::from_secs(20))
        .user_agent("CFVigil/0.1.0")
        .build().map_err(|_| "Could not start Cloudflare WAF client".into())
}

async fn list_zones(http: &Client, account: &str, token: &str, budget: &ApiBudget) -> Result<Vec<Zone>, String> {
    let mut zones = Vec::new();
    for page in 1..=MAX_ZONES.div_ceil(50).max(1) {
        let url = format!("{BASE}/zones?account.id={account}&page={page}&per_page=50");
        budget.check()?;
        let response = http.get(url).bearer_auth(token).send().await
            .map_err(|_| "Cloudflare zone list unavailable")?;
        budget.observe(&response);
        if !response.status().is_success() {
            return Err(format!("Cloudflare zones HTTP {}", response.status().as_u16()));
        }
        let bytes = response.bytes().await.map_err(|_| "Could not read Cloudflare zones")?;
        if bytes.len() > MAX_GRAPHQL_BYTES {
            return Err("Cloudflare zone list too large".into());
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|_| "Malformed Cloudflare zones JSON")?;
        zones.extend(parse_zones(&body, account)?);
        let pages = body.pointer("/result_info/total_pages")
            .and_then(Value::as_u64).unwrap_or(1);
        if pages <= page as u64 {
            break;
        }
        // Reject rather than omit zones silently.
        if page == 1 {
            return Err("Too many Cloudflare zones to scan safely".into());
        }
    }
    if zones.len() > MAX_ZONES {
        return Err("Too many Cloudflare zones to scan safely".into());
    }
    Ok(zones)
}

async fn get_zone_counts(
    http: &Client,
    token: &str,
    zone: &Zone,
    budget: &ApiBudget,
) -> Result<(f64, f64), String> {
    let now = Utc::now();
    let recent_end = now - ChronoDuration::minutes(5);
    let recent_start = recent_end - ChronoDuration::minutes(10);
    let baseline_start = recent_start - ChronoDuration::minutes(30);
    let body = json!({
        "query": QUERY,
        "variables": {
            "zoneTag": zone.id,
            "recentStart": recent_start.to_rfc3339(),
            "recentEnd": recent_end.to_rfc3339(),
            "baselineStart": baseline_start.to_rfc3339(),
            "baselineEnd": recent_start.to_rfc3339()
        }
    });
    budget.check()?;
    let response = http.post(format!("{BASE}/graphql"))
        .bearer_auth(token).json(&body).send().await
        .map_err(|_| "Cloudflare WAF GraphQL request failed")?;
    budget.observe(&response);
    if !response.status().is_success() {
        return Err(format!("Cloudflare WAF HTTP {}", response.status().as_u16()));
    }
    let bytes = response.bytes().await.map_err(|_| "Could not read WAF response")?;
    if bytes.len() > MAX_GRAPHQL_BYTES {
        return Err("Cloudflare WAF response too large".into());
    }
    let data: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "Malformed Cloudflare WAF JSON")?;
    parse_waf_counts(&data)
}

/// Returns only zones with confirmed WAF-block spikes, not every DNS zone.
/// An unavailable zone is not silently assumed healthy; the caller can retain
/// its last incident as unknown until successful analytics are available.
pub async fn detect(
    account: &str,
    token: &str,
    budget: &ApiBudget,
) -> Result<WafScan, String> {
    if !valid_id(account) {
        return Err("Invalid Cloudflare account ID".into());
    }
    let http = http_client()?;
    let zones = list_zones(&http, account, token, budget).await?;
    let mut scan = WafScan { incidents: Vec::new(), verified_zones: HashSet::new() };
    for zone in zones {
        match get_zone_counts(&http, token, &zone, budget).await {
            Ok((recent, baseline)) => {
                scan.verified_zones.insert(format!("zone:{}", zone.id));
                if !waf_spike(recent, baseline) { continue; }
                scan.incidents.push(Resource {
                    key: format!("zone:{}", zone.id),
                    name: zone.name.clone(),
                    kind: "zone".into(),
                    status: "failed".into(),
                    changed_at: Utc::now().to_rfc3339(),
                    attempt_id: "waf-block-spike".into(),
                    url: format!("https://dash.cloudflare.com/{account}/{}/security/analytics/events?action=block", zone.name),
                    detail: format!("WAF blocks spiking: {:.0} in 10m; previous 30m {:.0}", recent, baseline),
                });
            }
            // A failed zone must not conceal the successful results for
            // unrelated zones. It remains unverified in the caller.
            Err(_) => {}
        }
    }
    Ok(scan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_noise_and_sustained_waf_volume() {
        assert!(!waf_spike(10.0, 0.0));
        assert!(!waf_spike(120.0, 360.0));
        assert!(!waf_spike(120.0, 100.0));
        assert!(waf_spike(150.0, 20.0));
        assert!(!waf_spike(f64::NAN, 0.0));
    }

    #[test]
    fn zone_validation_blocks_redirects_and_traversal() {
        assert!(valid_zone_name("example.net"));
        assert!(!valid_zone_name("../external"));
        assert!(!valid_zone_name("example.net/path"));
        assert!(!valid_zone_name("bad..zone"));
    }

    #[test]
    fn analytics_must_be_structured_and_error_free() {
        let okay = json!({"data":{"viewer":{"zones":[{
            "recent":[{"count":155}], "baseline":[{"count":20}]
        }]}}});
        assert_eq!(parse_waf_counts(&okay).unwrap(), (155.0,20.0));
        assert!(parse_waf_counts(&json!({"errors":[{"message":"Forbidden"}]})).is_err());
        assert!(parse_waf_counts(&json!({})).is_err());
    }

    #[test]
    fn zone_discovery_must_remain_account_scoped() {
        let body = json!({"success":true,"result":[
            {"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "name":"our.net",
              "account":{"id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}},
            {"id":"cccccccccccccccccccccccccccccccc", "name":"other.net",
              "account":{"id":"dddddddddddddddddddddddddddddddd"}}
        ]});
        let zones = parse_zones(&body, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].name, "our.net");
    }
}
