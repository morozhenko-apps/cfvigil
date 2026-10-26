//! External GraphQL requests are replaced with isolated 127.0.0.1 fixtures;
//! query_spikes_at still uses its actual reqwest and per-batch state logic.
use super::*;
use crate::test_http::{FixtureHttp, Observed};
use serde_json::{json, Map, Value};

fn aggregate_batch(size: usize, spike: bool) -> Value {
    let mut account=Map::new();
    for index in 0..size {
        let errors = if index == 0 && spike {60} else {0};
        account.insert(format!("r{index}"),json!([
            {"sum":{"requests":200,"errors":errors}}
        ]));
        account.insert(format!("b{index}"),json!([
            {"sum":{"requests":1000,"errors":10}}
        ]));
    }
    json!({"data":{"viewer":{"accounts":[Value::Object(account)]}}})
}
fn workers(count: usize) -> HashSet<String> {
    (0..count).map(|i|format!("worker{i:02}")).collect()
}
fn assert_query(request: &Observed) {
    assert_eq!(request.method,"POST");
    assert_eq!(request.path,"/graphql");
    let body=request.body.get("query").and_then(Value::as_str).unwrap();
    assert!(body.contains("workersInvocationsAdaptive(limit: 1"));
    assert!(body.contains("scriptName:"));
    assert!(!body.contains("test-only-token"));
}

#[tokio::test]
async fn given_nine_workers_and_second_batch_denied_when_scanning_then_verified_first_batch_survives() {
    let server=FixtureHttp::start(|request| {
        assert_query(request);
        let query=request.body["query"].as_str().unwrap();
        if query.contains("worker08") {
            (403,json!({"errors":[{"message":"private token scope missing"}]}))
        } else {
            (200,aggregate_batch(8,true))
        }
    });
    let budget=ApiBudget::new();
    let result=query_spikes_at(&"a".repeat(32),"test-only-token",&workers(9),
        &budget,&server.url("/graphql")).await.unwrap();
    assert_eq!(result.spikes.len(),1);
    let first=result.spikes.get("worker00").unwrap();
    assert_eq!(first.requests,200);
    assert_eq!(first.errors,60);
    assert!((first.ratio-0.30).abs()<1e-9);
    assert_eq!(result.unverified,HashSet::from(["worker08".to_string()]));
    let requests=server.requests();
    assert_eq!(requests.len(),2);
    for req in &requests {assert_query(req);}
    assert_eq!(budget.retry_after_seconds(),0);
    assert!(!budget.exhausted());
}

#[tokio::test]
async fn given_partial_graphql_response_when_scanning_then_every_worker_in_batch_remains_unverified() {
    let server=FixtureHttp::start(|request| {
        assert_query(request);
        (200,json!({"data":{"viewer":{"accounts":[{
            "r0":[{"sum":{"requests":200,"errors":60}}]
        }]}}}))
    });
    let allowed=workers(3);
    let result=query_spikes_at(&"a".repeat(32),"test-only-token",&allowed,
        &ApiBudget::new(),&server.url("/graphql")).await.unwrap();
    assert!(result.spikes.is_empty(),"a partial response cannot manufacture success");
    assert_eq!(result.unverified,allowed);
    assert_eq!(server.requests().len(),1);
}

#[tokio::test]
async fn given_graphql_rate_limit_when_scanning_then_all_following_batches_stop() {
    let server=FixtureHttp::start(|request| {
        assert_query(request);
        (429,json!({"errors":[{"code":1015,"message":"private token"}]}))
    });
    let budget=ApiBudget::new();
    let allowed=workers(9);
    let result=query_spikes_at(&"a".repeat(32),"test-only-token",&allowed,
        &budget,&server.url("/graphql")).await.unwrap();
    assert!(result.spikes.is_empty());
    assert_eq!(result.unverified,allowed);
    assert_eq!(budget.retry_after_seconds(),75);
    assert!(budget.exhausted());
    assert_eq!(server.requests().len(),1,"no retry storm or second batch request");
}

#[tokio::test]
async fn given_zero_or_excess_workers_when_scanning_then_no_http_request_is_attempted() {
    let server=FixtureHttp::start(|request| {
        panic!("invalid catalog must never make request: {}",request.path)
    });
    let budget=ApiBudget::new();
    let empty=query_spikes_at(&"a".repeat(32),"test-only-token",&workers(0),
        &budget,&server.url("/graphql")).await.unwrap();
    assert!(empty.spikes.is_empty() && empty.unverified.is_empty());
    let large=query_spikes_at(&"a".repeat(32),"test-only-token",&workers(257),
        &budget,&server.url("/graphql")).await;
    assert_eq!(large.err().as_deref(),Some(msg::ANALYTICS_TOO_MANY));
    assert!(server.requests().is_empty());
}
