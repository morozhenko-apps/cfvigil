//! Localhost-only REST integration contracts. These run the real reqwest
//! client, request quota, pagination, discovery and latest-attempt assembly.
//! No Cloudflare token, external hostname, sleep or production request.
use super::*;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

struct Loopback {
    address: SocketAddr,
    requests: Arc<Mutex<Vec<String>>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Loopback {
    fn start(respond: impl Fn(&str) -> (u16, String) + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::<String>::new()));
        let captured = Arc::clone(&requests);
        let worker = thread::spawn(move || {
            for incoming in listener.incoming() {
                let mut stream = incoming.unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut headers = Vec::new();
                let mut buf = [0u8; 2048];
                while headers.len() < 16 * 1024 {
                    let bytes = stream.read(&mut buf).unwrap_or(0);
                    if bytes == 0 { break; }
                    headers.extend_from_slice(&buf[..bytes]);
                    if headers.windows(4).any(|w| w == b"\r\n\r\n") { break; }
                }
                let request = String::from_utf8_lossy(&headers);
                let first = request.lines().next().unwrap_or("");
                let (method, path) = {
                    let mut parts = first.split_whitespace();
                    (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
                };
                if path == "/__close__" { break; }
                // Only an actual GET is accepted, proving the provider
                // cannot accidentally introduce a mutating request.
                assert_eq!(method, "GET");
                captured.lock().unwrap().push(path.to_owned());
                let (status, body) = respond(path);
                let reason = match status {
                    200 => "OK", 400 => "Bad Request", 401 => "Unauthorized",
                    403 => "Forbidden", 429 => "Too Many Requests",
                    500 => "Internal Server Error", _ => "Unexpected",
                };
                let retry = if status == 429 { "Retry-After: 75\r\n" } else { "" };
                write!(stream,
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n{retry}\r\n{}",
                    body.len(), body).unwrap();
                stream.flush().unwrap();
            }
        });
        Self { address, requests, worker: Some(worker) }
    }
    fn client(&self) -> Cloudflare {
        Cloudflare::fixture(format!("http://{}", self.address))
    }
    fn paths(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for Loopback {
    fn drop(&mut self) {
        if let Ok(mut stream) = TcpStream::connect(self.address) {
            let _ = stream.write_all(b"GET /__close__ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        }
        if let Some(thread) = self.worker.take() { thread.join().unwrap(); }
    }
}

fn okay(result: Value) -> (u16, String) {
    (200, json!({"success":true,"result":result}).to_string())
}
fn pages(result: Value, total: usize) -> (u16, String) {
    (200, json!({"success":true,"result":result,
        "result_info":{"total_pages":total}}).to_string())
}
fn denied(status: u16, numeric_code: u64) -> (u16, String) {
    (status,json!({"success":false,"errors":[{
        "code":numeric_code,"message":"never repeat private account or token"}]}).to_string())
}

#[tokio::test]
async fn given_two_pages_and_new_worker_build_when_discovering_then_true_http_latest_attempt_wins() {
    let account = "a".repeat(32);
    let tag = "b".repeat(32);
    let build_id = "182bd5e5-6e1a-4fe4-a799-aa6d9a6ab26e";
    let worker_deploy_id = "023e105f-2a42-4f8b-a1c1-73f6a2a30c0f";
    let pages_id = "1ad7f12c-cfb2-4976-b341-acf800e0ba80";
    let account_for_response = account.clone();
    let tag_for_response = tag.clone();
    let server = Loopback::start(move |path| {
        let base = format!("/accounts/{account_for_response}");
        match path {
            p if p == format!("{base}/workers/scripts") =>
                okay(json!([{"id":"worker-one","tag":tag_for_response}])),
            p if p == format!("{base}/pages/projects") =>
                pages(json!([{"name":"front-a"}]),2),
            p if p == format!("{base}/pages/projects?page=2") =>
                pages(json!([{"name":"front-b"}]),2),
            p if p.starts_with(&format!("{base}/builds/builds/latest?")) => {
                okay(json!({"builds": {
                    (tag_for_response.clone()): {
                        "status":"queued", "build_outcome":"fail",
                        "build_uuid":"182bd5e5-6e1a-4fe4-a799-aa6d9a6ab26e",
                        "created_on":"2026-10-26T12:01:00Z"
                    }
                }}))
            }
            p if p == format!("{base}/workers/scripts/worker-one/deployments?page=1&per_page=1") =>
                okay(json!({"deployments":[{
                    "id":"023e105f-2a42-4f8b-a1c1-73f6a2a30c0f",
                    "created_on":"2026-10-26T12:00:00Z"
                }]})),
            p if p == format!("{base}/pages/projects/front-a/deployments") =>
                okay(json!([
                    {"id":"1ad7f12c-cfb2-4976-b341-acf800e0ba80",
                     "created_on":"2026-10-26T12:02:00Z",
                     "latest_stage":{"status":"success","ended_on":"2026-10-26T12:04:00Z"}},
                    {"id":"older","created_on":"2026-10-26T11:59:00Z",
                     "latest_stage":{"status":"failure"}}
                ])),
            p if p == format!("{base}/pages/projects/front-b/deployments") =>
                okay(json!([{"id":"failed",
                    "created_on":"2026-10-26T12:05:00Z",
                    "latest_stage":{"status":"failure"}}])),
            _ => panic!("unexpected local mock endpoint: {path}"),
        }
    });
    let budget=ApiBudget::new();
    let discovered=server.client().discover(&account,"test-only-token",&budget).await.unwrap();
    assert!(discovered.complete);
    assert!(discovered.workers_complete);
    assert!(discovered.pages_complete);
    assert!(discovered.warnings.is_empty());
    assert_eq!(discovered.resources.len(),3);
    let worker=discovered.resources.iter().find(|r|r.key=="worker:worker-one").unwrap();
    assert_eq!(worker.status,"deploying","new pending attempt must override older active deployment");
    assert_eq!(worker.attempt_id,build_id);
    assert!(worker.url.ends_with(&format!("/builds/{build_id}")));
    let front_a=discovered.resources.iter().find(|r|r.key=="pages:front-a").unwrap();
    assert_eq!(front_a.status,"healthy");
    assert_eq!(front_a.attempt_id,pages_id);
    assert_eq!(front_a.changed_at,"2026-10-26T12:04:00Z");
    let front_b=discovered.resources.iter().find(|r|r.key=="pages:front-b").unwrap();
    assert_eq!(front_b.status,"failed");
    let calls=server.paths();
    assert_eq!(calls.len(),7);
    assert!(calls.iter().any(|p|p.ends_with("/pages/projects")));
    assert!(calls.iter().any(|p|p.ends_with("/pages/projects?page=2")));
    assert!(!calls.iter().any(|p|p.contains("pages/projects?page=1")));
    assert_eq!(budget.retry_after_seconds(),0);
    assert!(!budget.exhausted());
    assert!(!discovered.resources.iter().any(|r|r.url.contains("test-only-token")));
    assert!(!discovered.warnings.join(" ").contains("test-only-token"));
    assert!(!server.paths().iter().any(|p|p.contains("test-only-token")));
    let _ = worker_deploy_id; // Fixture proves older active deployment remains secondary.
}

#[tokio::test]
async fn given_worker_scope_denied_when_pages_respond_then_other_provider_is_preserved() {
    let account="a".repeat(32);
    let account_for_response=account.clone();
    let server=Loopback::start(move |path| {
        let base=format!("/accounts/{account_for_response}");
        match path {
            p if p == format!("{base}/workers/scripts") => denied(403,10000),
            p if p == format!("{base}/pages/projects") =>
                pages(json!([{"name":"site"}]),1),
            p if p == format!("{base}/pages/projects/site/deployments") =>
                okay(json!([{"id":"success",
                    "created_on":"2026-10-26T12:00:00Z",
                    "latest_stage":{"status":"success"}}])),
            _ => panic!("unexpected endpoint {path}"),
        }
    });
    let budget=ApiBudget::new();
    let result=server.client().discover(&account,"test-only-token",&budget).await.unwrap();
    assert!(!result.complete);
    assert!(!result.workers_complete);
    assert!(result.pages_complete);
    assert_eq!(result.resources.len(),1);
    assert_eq!(result.resources[0].status,"healthy");
    assert!(result.warnings.iter().any(|s|s=="Workers: Cloudflare HTTP 403 (code 10000)"));
    assert!(!result.warnings.join(" ").contains("private"));
    assert_eq!(server.paths().len(),3);
}

#[tokio::test]
async fn given_both_catalogs_unavailable_when_discovering_then_fail_closed_without_fake_empty_success() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { denied(401,10001) }
        else if path.ends_with("/pages/projects") { denied(403,10000) }
        else { panic!("unexpected endpoint {path}"); }
    });
    let account="a".repeat(32);
    let result=server.client().discover(&account,"test-only-token",&ApiBudget::new()).await;
    assert_eq!(result.err().as_deref(),Some(msg::CF_DISCOVERY_UNAVAILABLE));
    assert_eq!(server.paths().len(),2);
}

#[tokio::test]
async fn given_paginated_pages_second_page_empty_when_discovering_then_catalog_is_not_marked_complete() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { okay(json!([])) }
        else if path.ends_with("/pages/projects") { pages(json!([{"name":"first"}]),2) }
        else if path.ends_with("/pages/projects?page=2") { pages(json!([]),2) }
        else { panic!("unexpected endpoint {path}"); }
    });
    let result=server.client().discover(&"a".repeat(32),"test-only-token",&ApiBudget::new())
        .await.unwrap();
    assert!(!result.complete);
    assert!(result.workers_complete);
    assert!(!result.pages_complete);
    assert!(result.resources.is_empty(),"partial Pages catalog cannot be misrepresented");
    assert!(result.warnings.iter().any(|s|s.starts_with("Pages: ")));
    assert_eq!(server.paths().len(),3);
}

#[tokio::test]
async fn given_pages_detail_429_when_discovering_then_budget_stops_new_requests_and_resource_is_unknown() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { okay(json!([])) }
        else if path.ends_with("/pages/projects") { pages(json!([{"name":"site"}]),1) }
        else if path.ends_with("/pages/projects/site/deployments") { denied(429,1015) }
        else { panic!("unexpected endpoint {path}"); }
    });
    let budget=ApiBudget::new();
    let result=server.client().discover(&"a".repeat(32),"test-only-token",&budget).await.unwrap();
    assert!(!result.complete);
    assert_eq!(result.resources.len(),1);
    assert_eq!(result.resources[0].status,"unknown");
    assert!(result.warnings.iter().any(|s|s.contains("HTTP 429 (code 1015)")));
    assert_eq!(budget.retry_after_seconds(),75);
    assert!(budget.exhausted());
    assert!(budget.check().is_err());
    assert_eq!(server.paths().len(),3);
}


#[tokio::test]
async fn given_build_permission_denied_but_worker_deployed_when_discovering_then_not_falsely_green() {
    let account="a".repeat(32);
    let tag="b".repeat(32);
    let account_copy=account.clone();
    let tag_copy=tag.clone();
    let server=Loopback::start(move |path| {
        let base=format!("/accounts/{account_copy}");
        if path==format!("{base}/workers/scripts") {
            okay(json!([{"id":"worker-a","tag":tag_copy}]))
        } else if path==format!("{base}/pages/projects") {
            pages(json!([]),1)
        } else if path.starts_with(&format!("{base}/builds/builds/latest?")) {
            denied(403,10000)
        } else if path==format!("{base}/workers/scripts/worker-a/deployments?page=1&per_page=1") {
            okay(json!({"deployments":[{"created_on":"2026-10-26T12:00:00Z"}]}))
        } else { panic!("unexpected endpoint: {path}"); }
    });
    let result=server.client().discover(&account,"test-only-token",&ApiBudget::new())
        .await.unwrap();
    assert!(!result.complete);
    assert!(result.workers_complete && result.pages_complete,
        "resource catalog permissions are independent of Builds scope");
    assert_eq!(result.resources.len(),1);
    assert_eq!(result.resources[0].status,"unknown",
        "existing active deployment is not proof of latest build when Builds denied");
    assert!(result.warnings.iter().any(|w|w.contains("Workers CI Read")));
    assert_eq!(server.paths().len(),4);
}

#[tokio::test]
async fn given_one_pages_detail_bad_request_when_others_succeed_then_only_one_is_unverified() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { okay(json!([])) }
        else if path.ends_with("/pages/projects") {
            pages(json!([{"name":"bad-options"},{"name":"healthy-site"}]),1)
        } else if path.ends_with("/pages/projects/bad-options/deployments") {
            denied(400,8000024)
        } else if path.ends_with("/pages/projects/healthy-site/deployments") {
            okay(json!([{"id":"current","created_on":"2026-10-26T12:00:00Z",
                "latest_stage":{"status":"success"}}]))
        } else { panic!("unexpected endpoint {path}"); }
    });
    let result=server.client().discover(&"a".repeat(32),"test-only-token",&ApiBudget::new())
        .await.unwrap();
    assert!(!result.complete);
    assert!(result.workers_complete && result.pages_complete);
    assert_eq!(result.resources.len(),2);
    assert_eq!(result.resources.iter().find(|r|r.name=="healthy-site").unwrap().status,
        "healthy");
    assert_eq!(result.resources.iter().find(|r|r.name=="bad-options").unwrap().status,
        "unknown");
    assert!(result.warnings.iter().any(|w|w.contains("HTTP 400 (code 8000024)")));
    assert!(!result.warnings.join(" ").contains("(check Cloudflare Pages Read)"),
        "HTTP 400 does not imply invalid token scopes");
    assert_eq!(server.paths().len(),4);
}

#[tokio::test]
async fn given_pages_catalog_declares_over_fifty_pages_then_bound_rejects_without_extra_http() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { okay(json!([])) }
        else if path.ends_with("/pages/projects") { pages(json!([{"name":"first"}]),51) }
        else { panic!("unexpected endpoint: {path}"); }
    });
    let result=server.client().discover(&"a".repeat(32),"test-only-token",&ApiBudget::new())
        .await.unwrap();
    assert!(!result.complete);
    assert!(result.workers_complete);
    assert!(!result.pages_complete);
    assert!(result.resources.is_empty(),"unverified catalog must not be returned as complete");
    assert_eq!(server.paths().len(),2,"no unbounded page crawl");
}

#[tokio::test]
async fn given_rate_limit_in_first_pages_batch_then_next_batch_never_issues_http() {
    let server=Loopback::start(|path| {
        if path.ends_with("/workers/scripts") { okay(json!([])) }
        else if path.ends_with("/pages/projects") {
            pages(json!([
                {"name":"first"},{"name":"second"},{"name":"third"},
                {"name":"fourth"},{"name":"fifth"}
            ]),1)
        } else if path.ends_with("/pages/projects/first/deployments") {
            denied(429,1015)
        } else if path.contains("/pages/projects/") && path.ends_with("/deployments") {
            okay(json!([{"created_on":"2026-10-26T12:00:00Z",
                "latest_stage":{"status":"success"}}]))
        } else { panic!("unexpected endpoint: {path}"); }
    });
    let budget=ApiBudget::new();
    let result=server.client().discover(&"a".repeat(32),"test-only-token",&budget)
        .await.unwrap();
    assert!(!result.complete);
    assert_eq!(result.resources.len(),5);
    assert_eq!(result.resources.iter().find(|r|r.name=="first").unwrap().status,"unknown");
    assert_eq!(result.resources.iter().find(|r|r.name=="fifth").unwrap().status,"unknown");
    assert_eq!(budget.retry_after_seconds(),75);
    assert!(budget.exhausted());
    let paths=server.paths();
    assert_eq!(paths.len(),6,"two catalogs and at most first four detail requests");
    assert!(!paths.iter().any(|p|p.contains("/pages/projects/fifth/deployments")));
}
