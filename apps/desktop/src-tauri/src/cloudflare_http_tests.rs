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
        Self::with_retry_header(move |path| {
            let (status, body) = respond(path);
            let retry = (status == 429).then(|| "75".to_owned());
            (status, body, retry)
        })
    }
    // Specific malformed/missing Retry-After variants are tested by changing
    // the external HTTP response, not by mocking ApiBudget::observe.
    fn with_retry_header(
        respond: impl Fn(&str) -> (u16, String, Option<String>) + Send + 'static
    ) -> Self {
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
                let (status, body, retry_seconds) = respond(path);
                let reason = match status {
                    200 => "OK", 400 => "Bad Request", 401 => "Unauthorized",
                    403 => "Forbidden", 404 => "Not Found", 429 => "Too Many Requests",
                    500 => "Internal Server Error", 503 => "Service Unavailable",
                    _ => "Unexpected",
                };
                let retry = retry_seconds.map(|value|
                    format!("Retry-After: {value}\r\n")).unwrap_or_default();
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


#[tokio::test]
async fn given_rest_success_envelopes_when_read_then_only_true_boolean_success_is_accepted() {
    for (body, expected) in [
        (r#"{"success":true,"result":[{"name":"safe"}]}"#,None),
        (r#"{"success":false,"result":[]}"#,Some(msg::CF_UNSUCCESSFUL)),
        (r#"{"success":"true","result":[]}"#,Some(msg::CF_UNSUCCESSFUL)),
        (r#"{"result":[]}"#,Some(msg::CF_UNSUCCESSFUL)),
        (r#"[{"success":true}]"#,Some(msg::CF_UNSUCCESSFUL)),
        (r#"{"success":true,"result":null}"#,None),
        ("not JSON",Some(msg::CF_BAD_JSON)),
        ("",Some(msg::CF_BAD_JSON)),
        (r#"{"success":true,"result":"#,Some(msg::CF_BAD_JSON)),
    ] {
        let payload=body.to_owned();
        let server=Loopback::start(move |_| (200,payload.clone()));
        let budget=ApiBudget::new();
        let result=server.client().get("test-only-token","/fixture",&budget).await;
        match expected {
            Some(reason) => assert_eq!(result.err().as_deref(),Some(reason),
                "untrusted REST envelope {body:?}"),
            None => {
                let (value,pages)=result.unwrap();
                assert_eq!(pages,1);
                assert_eq!(value.is_null(),body.contains(r#""result":null"#));
            }
        }
        assert_eq!(server.paths(),["/fixture"]);
        assert_eq!(budget.retry_after_seconds(),0);
    }
}

#[tokio::test]
async fn given_rest_payload_at_two_megabytes_boundary_when_read_then_max_is_inclusive() {
    const MAX:usize=2*1024*1024;
    let prefix=r#"{"success":true,"result":[]}"#;
    for (size,accepted) in [(MAX-1,true),(MAX,true),(MAX+1,false)] {
        let mut body=String::with_capacity(size);
        body.push_str(prefix);
        body.push_str(&" ".repeat(size-prefix.len()));
        assert_eq!(body.len(),size);
        let server=Loopback::start(move |_| (200,body.clone()));
        let result=server.client().get("test-only-token","/large",&ApiBudget::new()).await;
        if accepted {
            let (value,_) = result.expect("exact bounded JSON should be accepted");
            assert_eq!(value,json!([]));
        } else {
            assert_eq!(result.err().as_deref(),Some(msg::CF_TOO_LARGE));
        }
        assert_eq!(server.paths(),["/large"]);
    }
}

#[tokio::test]
async fn given_untrusted_http_errors_when_read_then_only_status_and_numeric_code_escape() {
    const PRIVATE:&str="secret-token-and-account-abcdef";
    for (status,body,expected) in [
        (400,format!(r#"{{"errors":[{{"code":8000024,"message":"{PRIVATE}"}}]}}"#),
            "Cloudflare HTTP 400 (code 8000024)".to_owned()),
        (401,format!(r#"{{"errors":[{{"code":"10000","message":"{PRIVATE}"}}]}}"#),
            "Cloudflare HTTP 401".to_owned()),
        (403,format!(r#"{{"errors":[{{"code":10000,"message":"{PRIVATE}"}}]}}"#),
            "Cloudflare HTTP 403 (code 10000)".to_owned()),
        (404,format!(r#"{{"message":"{PRIVATE}"}}"#),
            "Cloudflare HTTP 404".to_owned()),
        (500,format!(r#"{{"errors":[{{"code":null,"message":"{PRIVATE}"}}]}}"#),
            "Cloudflare HTTP 500".to_owned()),
        (503,"not-json".to_owned(),"Cloudflare HTTP 503".to_owned()),
        (403,format!(r#"{{"errors":[{{"code":123,"message":"{}"}}]}}"#,
            "x".repeat(9000)),
            "Cloudflare HTTP 403".to_owned())
    ] {
        let server=Loopback::start(move |_|(status,body.clone()));
        let result=server.client().get(PRIVATE,"/safe",&ApiBudget::new()).await;
        let err=result.err().expect("HTTP error must not return success");
        assert_eq!(err,expected,"HTTP {status}");
        assert!(!err.contains(PRIVATE));
        assert!(!err.contains("message"));
        assert_eq!(server.paths(),["/safe"]);
    }
}

#[tokio::test]
async fn given_429_without_usable_retry_after_when_read_then_default_delay_stops_next_request() {
    for retry in [None,Some("not-seconds"),Some("18446744073709551616")] {
        let retry=retry.map(str::to_owned);
        let server=Loopback::with_retry_header(move |_| {
            (429,json!({"errors":[{"code":1015,
                "message":"do not leak token"}]}).to_string(),retry.clone())
        });
        let budget=ApiBudget::new();
        let client=server.client();
        assert_eq!(client.get("secret","/limit",&budget).await.err().as_deref(),
            Some("Cloudflare HTTP 429 (code 1015)"));
        assert_eq!(budget.retry_after_seconds(),300);
        assert!(budget.exhausted());
        assert_eq!(client.get("secret","/forbidden-after",&budget).await.err().as_deref(),
            Some(msg::BUDGET_RATE_LIMIT));
        assert_eq!(server.paths(),["/limit"]);
    }
}

#[tokio::test]
async fn given_0_1_256_257_workers_when_read_then_strict_limit_has_no_unsupported_query() {
    for (count,expected) in [(0,true),(1,true),(256,true),(257,false)] {
        let rows=(0..count).map(|i|json!({"id":format!("worker{i}")}))
            .collect::<Vec<_>>();
        let server=Loopback::start(move |_|okay(json!(rows)));
        let result=server.client().list_workers("token","/accounts/valid",
            &ApiBudget::new()).await;
        if expected {
            assert_eq!(result.unwrap().len(),count);
        } else {
            assert_eq!(result.err().as_deref(),Some(msg::CF_WORKERS_TOO_MANY));
        }
        assert_eq!(server.paths(),["/accounts/valid/workers/scripts"],
            "Worker Scripts List must not invent unsupported page/per_page");
    }
}

#[tokio::test]
async fn given_worker_list_as_nonarray_when_read_then_schema_is_rejected() {
    for shape in [json!(null),json!({}),json!("worker"),json!(false)] {
        let server=Loopback::start(move |_|okay(shape.clone()));
        assert_eq!(server.client().list_workers("token","/accounts/valid",
            &ApiBudget::new()).await.err().as_deref(),Some(msg::CF_WORKER_LIST));
        assert_eq!(server.paths().len(),1);
    }
}

#[tokio::test]
async fn given_pages_catalog_0_1_50_51_pages_when_read_then_upper_bound_rejects_before_request_storm() {
    for total in [0,1,50,51] {
        let server=Loopback::start(move |path| {
            if path=="/pages" {
                pages(json!([{"name":"first"}]),total)
            } else if let Some(page)=path.strip_prefix("/pages?page=") {
                let page=page.parse::<usize>().unwrap();
                assert!(page>1 && page<=50);
                pages(json!([{"name":format!("page{page}")}]),total)
            } else { panic!("unexpected endpoint {path}"); }
        });
        let result=server.client().list_paginated("token","/pages",&ApiBudget::new()).await;
        if total==51 {
            assert_eq!(result.err().as_deref(),Some(msg::CF_PAGES_PAGINATION));
            assert_eq!(server.paths().len(),1);
        } else {
            assert_eq!(result.unwrap().len(),total.max(1));
            assert_eq!(server.paths().len(),total.max(1));
        }
    }
}

#[tokio::test]
async fn given_cumulative_pages_count_at_256_257_when_read_then_over_limit_is_rejected() {
    for (total,expected) in [(256,true),(257,false)] {
        let first=(0..255).map(|i|json!({"name":format!("page{i}")}))
            .collect::<Vec<_>>();
        let second=(255..total).map(|i|json!({"name":format!("page{i}")}))
            .collect::<Vec<_>>();
        let server=Loopback::start(move |path| {
            match path {
                "/pages" => pages(json!(first),2),
                "/pages?page=2" => pages(json!(second),2),
                _ => panic!("unexpected endpoint {path}"),
            }
        });
        let result=server.client().list_paginated("token","/pages",
            &ApiBudget::new()).await;
        if expected {
            assert_eq!(result.unwrap().len(),256);
        } else {
            assert_eq!(result.err().as_deref(),Some(msg::CF_PAGES_TOO_MANY));
        }
        assert_eq!(server.paths(),["/pages","/pages?page=2"]);
    }
}

#[tokio::test]
async fn given_malformed_later_pages_when_read_then_partial_catalog_never_returns_success() {
    for body in [
        r#"{"success":true,"result":null}"#,
        r#"{"success":true,"result":{}}"#,
        r#"{"success":false,"result":[]}"#,
        "malformed JSON"
    ] {
        let broken=body.to_owned();
        let server=Loopback::start(move |path| {
            if path=="/pages" {
                pages(json!([{"name":"first"}]),2)
            } else {
                assert_eq!(path,"/pages?page=2");
                (200,broken.clone())
            }
        });
        assert!(server.client().list_paginated("token","/pages",
            &ApiBudget::new()).await.is_err());
        assert_eq!(server.paths(),["/pages","/pages?page=2"]);
    }
}

#[tokio::test]
async fn given_existing_query_in_pages_path_when_appending_page_then_use_ampersand_once() {
    let server=Loopback::start(|path| {
        match path {
            "/pages?filter=active" => pages(json!([{"name":"a"}]),2),
            "/pages?filter=active&page=2" => pages(json!([{"name":"b"}]),2),
            _ => panic!("unexpected query {path}")
        }
    });
    let result=server.client().list_paginated("token","/pages?filter=active",
        &ApiBudget::new()).await.unwrap();
    assert_eq!(result.len(),2);
    assert_eq!(server.paths(),[
        "/pages?filter=active","/pages?filter=active&page=2"
    ]);
}

#[tokio::test]
async fn given_pages_catalog_http_error_matrix_when_workers_are_available_then_scope_and_readiness_are_exact() {
    let account = "a".repeat(32);
    for status in [400u16, 401, 403, 404, 500, 503] {
        let code = if status == 400 { 8000024 } else { 10000 };
        let server = Loopback::start(move |path| {
            if path.ends_with("/workers/scripts") {
                okay(json!([]))
            } else if path.ends_with("/pages/projects") {
                denied(status, code)
            } else {
                panic!("a failed Pages catalog must not make detail requests: {path}")
            }
        });
        let budget = ApiBudget::new();
        let result = server.client().discover(&account, "test-only-token", &budget)
            .await.unwrap();
        assert!(!result.complete, "HTTP {status} cannot prove full discovery");
        assert!(result.workers_complete, "unaffected Workers catalog remains verified");
        assert!(!result.pages_complete, "HTTP {status} Pages catalog is unverified");
        assert!(result.resources.is_empty(), "no invented healthy deployment");
        assert_eq!(result.warnings.len(), 1, "HTTP {status}");
        assert!(result.warnings[0].contains(&format!("HTTP {status}")), "HTTP {status}");
        assert!(result.warnings[0].contains(&format!("code {code}")), "HTTP {status}");
        assert_eq!(result.warnings[0].contains("check Cloudflare Pages Read"),
            status == 403, "only an authorization denial suggests an explicit scope");
        assert!(!result.warnings[0].contains("never repeat private account or token"));
        assert!(!result.warnings[0].contains("test-only-token"));
        assert_eq!(server.paths().len(), 2, "HTTP {status} must not cause retries");
        assert_eq!(budget.retry_after_seconds(), 0, "only 429 sets HTTP backoff");
    }
}

#[tokio::test]
async fn given_failed_pages_discovery_then_recovered_provider_when_repolled_then_success_is_fresh_only() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let catalog_reads = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&catalog_reads);
    let server = Loopback::start(move |path| {
        if path.ends_with("/workers/scripts") {
            okay(json!([]))
        } else if path.ends_with("/pages/projects") {
            if seen.fetch_add(1, Ordering::SeqCst) == 0 {
                denied(503, 10001)
            } else {
                pages(json!([{"name":"recovered-page"}]), 1)
            }
        } else if path.ends_with("/pages/projects/recovered-page/deployments") {
            okay(json!([{
                "id":"attempt-2",
                "created_on":"2026-10-27T12:00:00Z",
                "latest_stage":{"status":"success","ended_on":"2026-10-27T12:03:00Z"}
            }]))
        } else {
            panic!("unexpected recovery fixture request: {path}")
        }
    });
    let first_budget = ApiBudget::new();
    let first = server.client().discover(&"a".repeat(32),"test-only-token",&first_budget)
        .await.unwrap();
    assert!(!first.complete);
    assert!(first.resources.is_empty());
    assert!(first.workers_complete && !first.pages_complete);
    assert_eq!(first.warnings.len(),1);
    assert!(first.warnings[0].contains("HTTP 503"));
    assert_eq!(first_budget.retry_after_seconds(),0);

    let second_budget = ApiBudget::new();
    let second = server.client().discover(&"a".repeat(32),"test-only-token",&second_budget)
        .await.unwrap();
    assert!(second.complete);
    assert!(second.workers_complete && second.pages_complete);
    assert!(second.warnings.is_empty());
    assert_eq!(second.resources.len(),1);
    assert_eq!(second.resources[0].key,"pages:recovered-page");
    assert_eq!(second.resources[0].status,"healthy");
    assert_eq!(second.resources[0].attempt_id,"attempt-2");
    assert_eq!(second.resources[0].changed_at,"2026-10-27T12:03:00Z");
    assert_eq!(catalog_reads.load(Ordering::SeqCst),2);
    assert_eq!(server.paths().len(),5,"two catalogs per poll plus verified detail");
    assert!(!server.paths().join(" ").contains("test-only-token"));
}
