//! Account-isolated WAF integration against read-only local Cloudflare
//! fixtures. Real reqwest/REST/GraphQL/budget and detection code executes.
use super::*;
use crate::test_http::{FixtureHttp, Observed};
use serde_json::{json, Value};

fn zones(account: &str, count: usize) -> Value {
    let result = (0..count).map(|i| {
        let hex = format!("{i:032x}");
        json!({"id":hex,"name":format!("zone{i}.example"),
            "account":{"id":account}})
    }).collect::<Vec<_>>();
    json!({"success":true,"result":result,"result_info":{"total_pages":1}})
}
fn analytics(recent: u64, baseline: u64) -> Value {
    json!({"data":{"viewer":{"zones":[{
        "recent":[{"count":recent}],"baseline":[{"count":baseline}]
    }]}}})
}
fn assert_query(request: &Observed) {
    assert_eq!(request.method,"POST");
    assert_eq!(request.path,"/graphql");
    assert!(request.body["query"].as_str().unwrap()
        .contains("firewallEventsAdaptiveGroups"));
    assert_eq!(request.body["variables"]["baselineEnd"],
        request.body["variables"]["recentStart"]);
}

#[tokio::test]
async fn given_one_waf_spike_one_normal_and_one_denied_when_detecting_then_verified_zones_are_isolated() {
    let account="a".repeat(32);
    let own=account.clone();
    let server=FixtureHttp::start(move |req| {
        if req.method=="GET" && req.path.starts_with("/zones?") {
            (200,zones(&own,3))
        } else if req.path=="/graphql" {
            assert_query(req);
            match req.body["variables"]["zoneTag"].as_str().unwrap() {
                "00000000000000000000000000000000" => (200,analytics(150,20)),
                "00000000000000000000000000000001" => (500,json!({
                    "errors":[{"message":"private diagnostic"}]
                })),
                "00000000000000000000000000000002" => (200,analytics(10,200)),
                id=>panic!("unexpected zone {id}")
            }
        } else { panic!("unexpected WAF request: {}",req.path) }
    });
    let budget=ApiBudget::new();
    let scan=detect_at(&account,"test-only-token",&budget,
        &server.url("")).await.unwrap();
    assert_eq!(scan.incidents.len(),1);
    assert_eq!(scan.incidents[0].kind,"zone");
    assert_eq!(scan.incidents[0].name,"zone0.example");
    assert_eq!(scan.incidents[0].status,"failed");
    assert_eq!(scan.incidents[0].attempt_id,"waf-block-spike");
    assert!(scan.incidents[0].url.contains("/zone0.example/security/analytics/events?action=block"));
    assert_eq!(scan.verified_zones.len(),2);
    assert!(scan.verified_zones.contains("zone:00000000000000000000000000000000"));
    assert!(scan.verified_zones.contains("zone:00000000000000000000000000000002"));
    assert!(!scan.verified_zones.contains("zone:00000000000000000000000000000001"));
    assert_eq!(server.requests().len(),4,"one zone GET and three GraphQL reads");
}

#[tokio::test]
async fn given_foreign_account_and_path_injections_when_listing_zones_then_no_foreign_graphql_is_issued() {
    let account="a".repeat(32);
    let own=account.clone();
    let server=FixtureHttp::start(move |req| {
        if req.method=="GET" && req.path.starts_with("/zones?") {
            (200,json!({"success":true,"result":[
                {"id":"00000000000000000000000000000001","name":"mine.example",
                    "account":{"id":own}},
                {"id":"00000000000000000000000000000002","name":"foreign.example",
                    "account":{"id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}},
                {"id":"00000000000000000000000000000003","name":"bad/../url",
                    "account":{"id":own}}
            ]}))
        } else if req.path=="/graphql" {
            assert_query(req);
            assert_eq!(req.body["variables"]["zoneTag"],
                "00000000000000000000000000000001");
            (200,analytics(10,0))
        } else { panic!("unexpected WAF request: {}",req.path) }
    });
    let scan=detect_at(&account,"test-only-token",&ApiBudget::new(),
        &server.url("")).await.unwrap();
    assert!(scan.incidents.is_empty());
    assert_eq!(scan.verified_zones,
        HashSet::from(["zone:00000000000000000000000000000001".into()]));
    assert_eq!(server.requests().len(),2);
}

#[tokio::test]
async fn given_waf_graphql_rate_limit_when_detecting_then_remaining_zones_cannot_be_verified() {
    let account="a".repeat(32);
    let own=account.clone();
    let server=FixtureHttp::start(move |req| {
        if req.method=="GET" && req.path.starts_with("/zones?") {
            (200,zones(&own,3))
        } else if req.path=="/graphql" {
            assert_query(req);
            (429,json!({"errors":[{"code":1015,"message":"secret"}]}))
        } else { panic!("unexpected request {}",req.path) }
    });
    let budget=ApiBudget::new();
    let scan=detect_at(&account,"test-only-token",&budget,
        &server.url("")).await.unwrap();
    assert!(scan.incidents.is_empty());
    assert!(scan.verified_zones.is_empty(),"HTTP failure is not zone recovery");
    assert_eq!(budget.retry_after_seconds(),75);
    assert!(budget.exhausted());
    assert_eq!(server.requests().len(),2,"single GraphQL attempt, no retry storm");
}

#[tokio::test]
async fn given_invalid_account_or_oversized_zone_catalog_when_detecting_then_no_unsafe_traversal() {
    let account="a".repeat(32);
    let own=account.clone();
    let server=FixtureHttp::start(move |req| {
        if req.method=="GET" && req.path.starts_with("/zones?") {
            (200,zones(&own,31))
        } else { panic!("zone count must be rejected before GraphQL: {}",req.path) }
    });
    let budget=ApiBudget::new();
    assert_eq!(detect_at("not-32-hex","test-only-token",&budget,
        &server.url("")).await.err().as_deref(),Some(msg::WAF_BAD_ACCOUNT));
    assert!(server.requests().is_empty());
    assert_eq!(detect_at(&account,"test-only-token",&budget,
        &server.url("")).await.err().as_deref(),Some(msg::WAF_ZONES_TOO_MANY));
    assert_eq!(server.requests().len(),1);
}
