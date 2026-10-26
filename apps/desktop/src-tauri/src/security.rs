//! Read-only, zone-scoped WAF block anomaly detection.
//! WAF events belong to Cloudflare zones; never attribute one to a Worker
//! unless Cloudflare provides a trustworthy resource-level mapping.
use crate::native_messages as msg;
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
        .ok_or_else(|| msg::WAF_GROUP_MISSING.to_string())?;
    if groups.len() > 1 {
        return Err(msg::WAF_AMBIGUOUS.into());
    }
    let count = match groups.first() {
        None => 0.0,
        Some(group) => group.get("count").and_then(Value::as_f64)
            .ok_or(msg::WAF_INVALID_COUNT)?,
    };
    if !count.is_finite() || count < 0.0 {
        return Err(msg::WAF_BAD_COUNT.into());
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
        return Err(msg::WAF_REJECTED.into());
    }
    let zone = body.pointer("/data/viewer/zones/0")
        .ok_or(msg::WAF_DATA_UNAVAILABLE)?;
    let recent = count_group(zone, "recent")?;
    let baseline = count_group(zone, "baseline")?;
    Ok((recent, baseline))
}

fn parse_zones(body: &Value, account: &str) -> Result<Vec<Zone>, String> {
    if body.get("success") != Some(&Value::Bool(true)) {
        return Err(msg::WAF_ZONES_ERROR.into());
    }
    let input = body.get("result").and_then(Value::as_array)
        .ok_or(msg::WAF_ZONES_INVALID)?;
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
    let client = Client::builder().redirect(Policy::none())
        .timeout(Duration::from_secs(20))
        .user_agent("CFVigil/0.1.0");
    #[cfg(test)]
    let client = client.no_proxy(); // Loopback fixtures never use machine proxy.
    client.build().map_err(|_| msg::WAF_CLIENT_INIT.into())
}

async fn list_zones(
    http: &Client, account: &str, token: &str, budget: &ApiBudget, base: &str,
) -> Result<Vec<Zone>, String> {
    let mut zones = Vec::new();
    for page in 1..=MAX_ZONES.div_ceil(50).max(1) {
        let url = format!("{base}/zones?account.id={account}&page={page}&per_page=50");
        budget.check()?;
        let response = http.get(url).bearer_auth(token).send().await
            .map_err(|_| msg::WAF_ZONES_UNAVAILABLE)?;
        budget.observe(&response);
        if !response.status().is_success() {
            return Err(format!("Cloudflare zones HTTP {}", response.status().as_u16()));
        }
        let bytes = response.bytes().await.map_err(|_| msg::WAF_ZONES_READ)?;
        if bytes.len() > MAX_GRAPHQL_BYTES {
            return Err(msg::WAF_ZONES_BIG.into());
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|_| msg::WAF_ZONES_JSON)?;
        zones.extend(parse_zones(&body, account)?);
        let pages = body.pointer("/result_info/total_pages")
            .and_then(Value::as_u64).unwrap_or(1);
        if pages <= page as u64 {
            break;
        }
        // Reject rather than omit zones silently.
        if page == 1 {
            return Err(msg::WAF_ZONES_TOO_MANY.into());
        }
    }
    if zones.len() > MAX_ZONES {
        return Err(msg::WAF_ZONES_TOO_MANY.into());
    }
    Ok(zones)
}

async fn get_zone_counts(
    http: &Client,
    token: &str,
    zone: &Zone,
    budget: &ApiBudget,
    base: &str,
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
    let response = http.post(format!("{base}/graphql"))
        .bearer_auth(token).json(&body).send().await
        .map_err(|_| msg::WAF_REQUEST_FAILED)?;
    budget.observe(&response);
    if !response.status().is_success() {
        return Err(format!("Cloudflare WAF HTTP {}", response.status().as_u16()));
    }
    let bytes = response.bytes().await.map_err(|_| msg::WAF_READ_FAILED)?;
    if bytes.len() > MAX_GRAPHQL_BYTES {
        return Err(msg::WAF_RESPONSE_BIG.into());
    }
    let data: Value = serde_json::from_slice(&bytes)
        .map_err(|_| msg::WAF_RESPONSE_JSON)?;
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
    detect_at(account, token, budget, BASE).await
}

/// Internal transport seam: the public detector always uses Cloudflare's
/// fixed endpoint; only cfg(test) fixtures can address loopback.
async fn detect_at(
    account: &str,
    token: &str,
    budget: &ApiBudget,
    base: &str,
) -> Result<WafScan, String> {
    if !valid_id(account) {
        return Err(msg::WAF_BAD_ACCOUNT.into());
    }
    let http = http_client()?;
    let zones = list_zones(&http, account, token, budget, base).await?;
    let mut scan = WafScan { incidents: Vec::new(), verified_zones: HashSet::new() };
    for zone in zones {
        match get_zone_counts(&http, token, &zone, budget, base).await {
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
                    detail: msg::waf_spike_detail(recent, baseline),
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
    fn mode_b_waf_aggregation_rejects_partial_duplicate_and_impossible_counts() {
        let missing_baseline = json!({"data":{"viewer":{"zones":[{
            "recent":[{"count":150}]
        }]}}});
        assert!(parse_waf_counts(&missing_baseline).is_err());
        let duplicate = json!({"data":{"viewer":{"zones":[{
            "recent":[{"count":150},{"count":180}],
            "baseline":[{"count":20}]
        }]}}});
        assert!(parse_waf_counts(&duplicate).is_err());
        let negative = json!({"data":{"viewer":{"zones":[{
            "recent":[{"count":-4.0}],
            "baseline":[{"count":20}]
        }]}}});
        assert!(parse_waf_counts(&negative).is_err());
        let empty = json!({"data":{"viewer":{"zones":[{
            "recent":[], "baseline":[]
        }]}}});
        assert_eq!(parse_waf_counts(&empty).unwrap(), (0.0,0.0));
        assert!(!waf_spike(100.0, 1000.0));
        assert!(waf_spike(150.0, 20.0));
    }

    #[test]
    fn mode_b_account_isolation_rejects_foreign_and_invalid_zones() {
        let own = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let raw = json!({"success":true,"result":[
            {"id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "name":"safe.example",
             "account":{"id":own}},
            {"id":"cccccccccccccccccccccccccccccccc", "name":"foreign.example",
             "account":{"id":"dddddddddddddddddddddddddddddddd"}},
            {"id":"bad/redirect", "name":"safe.example", "account":{"id":own}},
            {"id":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "name":"bad/path", "account":{"id":own}}
        ]});
        let zones = parse_zones(&raw, own).unwrap();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].name, "safe.example");
        assert!(parse_zones(&json!({"success":false,"result":[]}), own).is_err());
    }

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

    #[test]
    fn given_waf_volume_threshold_edges_when_evaluating_then_noise_and_sustained_blocks_are_not_spikes() {
        for (recent,baseline,expected) in [
            (99.0,0.0,false),
            (100.0,0.0,true),
            (99.0,60.0,false),
            (100.0,60.0,true),
            (166.0,100.0,false),
            (167.0,100.0,true),
            (999.0,600.0,false),
            (1000.0,600.0,true),
            (300.0,900.0,false),
            (f64::NAN,0.0,false),
            (f64::INFINITY,0.0,false),
            (100.0,-1.0,false),
        ] {
            assert_eq!(waf_spike(recent,baseline),expected,
                "recent={recent}, baseline={baseline}");
        }
    }

    #[test]
    fn given_zone_names_and_account_ids_when_validating_then_no_malicious_url_path_can_escape() {
        let max="a".repeat(253);
        let over="a".repeat(254);
        for name in ["example.net","a","sub-domain.example",max.as_str()] {
            assert!(valid_zone_name(name), "{name}");
        }
        for name in ["",".leading","double..dot","bad/path","fragment#",
            "query?", "space name", "<script>","é",over.as_str()] {
            assert!(!valid_zone_name(name),"{name}");
        }
        for id in ["a".repeat(32),"A".repeat(32)] {
            assert!(valid_id(&id));
        }
        for id in ["".to_owned(),"a".repeat(31),"a".repeat(33),"z".repeat(32)] {
            assert!(!valid_id(&id));
        }
    }

    #[test]
    fn given_waf_aggregation_response_when_partial_or_ambiguous_then_do_not_clear_prior_alerts() {
        let accepted=serde_json::json!({"data":{"viewer":{"zones":[{
            "recent":[{"count":100}],"baseline":[{"count":60}]
        }]}}});
        assert_eq!(parse_waf_counts(&accepted).unwrap(),(100.0,60.0));
        let empty=serde_json::json!({"data":{"viewer":{"zones":[{
            "recent":[],"baseline":[]
        }]}}});
        assert_eq!(parse_waf_counts(&empty).unwrap(),(0.0,0.0));
        for bad in [
            serde_json::json!({}),
            serde_json::json!({"errors":[{"message":"private"}]}),
            serde_json::json!({"data":{"viewer":{"zones":[]}}}),
            serde_json::json!({"data":{"viewer":{"zones":[{
                "recent":[{"count":100}],"baseline":null
            }]}}}),
            serde_json::json!({"data":{"viewer":{"zones":[{
                "recent":[{"count":"100"}],"baseline":[]
            }]}}}),
            serde_json::json!({"data":{"viewer":{"zones":[{
                "recent":[{"count":-1}],"baseline":[]
            }]}}}),
            serde_json::json!({"data":{"viewer":{"zones":[{
                "recent":[{"count":100},{"count":100}],"baseline":[]
            }]}}}),
        ] {
            assert!(parse_waf_counts(&bad).is_err(),"invalid {bad}");
        }
    }

    #[test]
    fn given_foreign_scoped_zones_when_parsing_then_only_matching_account_and_safe_names_pass() {
        let own="a".repeat(32);
        let entries=serde_json::json!({"success":true,"result":[
            {"id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","name":"mine.example",
             "account":{"id":own}},
            {"id":"cccccccccccccccccccccccccccccccc","name":"foreign.example",
             "account":{"id":"dddddddddddddddddddddddddddddddd"}},
            {"id":"bad/path","name":"mine.example","account":{"id":own}},
            {"id":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","name":"bad/path",
             "account":{"id":own}}
        ]});
        let zones=parse_zones(&entries,&own).unwrap();
        assert_eq!(zones.len(),1);
        assert_eq!(zones[0].name,"mine.example");
        assert_eq!(zones[0].id,"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        assert!(parse_zones(&serde_json::json!({"success":false,"result":[]}),&own).is_err());
        assert!(parse_zones(&serde_json::json!({"success":true,"result":"not array"}),&own).is_err());
    }

}

#[cfg(test)]
#[path = "security_http_tests.rs"]
mod http_tests;
