//! Shared per-poll budget for Cloudflare REST + GraphQL calls.
//! No immediate retries, no token/URL logging, and no unbounded scans.
use crate::native_messages as msg;
use reqwest::{header::RETRY_AFTER, Response, StatusCode};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

pub const MAX_CALLS_PER_POLL: usize = 120;
const DEFAULT_RATE_LIMIT_DELAY: u64 = 300;
const MAX_RETRY_DELAY: u64 = 1800;

#[derive(Default)]
pub struct ApiBudget {
    calls: AtomicUsize,
    limited: AtomicBool,
    retry_seconds: AtomicU64,
}

impl ApiBudget {
    pub fn new() -> Self { Self::default() }

    pub fn check(&self) -> Result<(), String> {
        if self.limited.load(Ordering::Acquire) {
            return Err(msg::BUDGET_RATE_LIMIT.into());
        }
        let previous = self.calls.fetch_add(1, Ordering::AcqRel);
        if previous >= MAX_CALLS_PER_POLL {
            return Err(msg::BUDGET_EXHAUSTED.into());
        }
        Ok(())
    }

    pub fn observe(&self, response: &Response) {
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let seconds = response.headers().get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(parse_retry_after)
                .unwrap_or(DEFAULT_RATE_LIMIT_DELAY);
            self.retry_seconds.fetch_max(seconds, Ordering::AcqRel);
            self.limited.store(true, Ordering::Release);
        }
    }

    pub fn retry_after_seconds(&self) -> u64 {
        self.retry_seconds.load(Ordering::Acquire)
    }

    pub fn exhausted(&self) -> bool {
        self.limited.load(Ordering::Acquire)
            || self.calls.load(Ordering::Acquire) >= MAX_CALLS_PER_POLL
    }

    #[cfg(test)]
    pub fn observe_retry_after_for_test(&self, value: Option<&str>) {
        let delay = value.and_then(parse_retry_after).unwrap_or(DEFAULT_RATE_LIMIT_DELAY);
        self.retry_seconds.fetch_max(delay, Ordering::AcqRel);
        self.limited.store(true, Ordering::Release);
    }
}

fn parse_retry_after(value: &str) -> Option<u64> {
    // Cloudflare REST publishes integer seconds. Reject malformed and
    // absurd/unbounded values rather than parsing arbitrary server input.
    let seconds = value.trim().parse::<u64>().ok()?;
    Some(seconds.clamp(60, MAX_RETRY_DELAY))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_b_rate_limit_values_are_bounded_and_fail_closed() {
        for (header, expected) in [
            (Some("0"),60), (Some("1"),60), (Some("60"),60),
            (Some("1800"),1800), (Some("1801"),1800),
            (Some("-1"),300), (Some("9999999999999999999999999999"),300),
            (Some("1;token=secret"),300), (None,300)
        ] {
            let budget = ApiBudget::new();
            budget.observe_retry_after_for_test(header);
            assert_eq!(budget.retry_after_seconds(), expected, "{header:?}");
            assert!(budget.check().is_err());
            assert!(budget.exhausted());
        }
    }

    #[test]
    fn finite_global_budget_prevents_unbounded_requests() {
        let budget = ApiBudget::new();
        for _ in 0..MAX_CALLS_PER_POLL { assert!(budget.check().is_ok()); }
        assert!(budget.check().is_err());
        assert!(budget.exhausted());
    }

    #[test]
    fn throttling_uses_retry_after_and_blocks_further_calls() {
        let budget = ApiBudget::new();
        budget.observe_retry_after_for_test(Some(" 440 "));
        assert_eq!(budget.retry_after_seconds(), 440);
        assert!(budget.check().is_err());
        let huge = ApiBudget::new();
        huge.observe_retry_after_for_test(Some("99999999"));
        assert_eq!(huge.retry_after_seconds(), 1800);
        let missing = ApiBudget::new();
        missing.observe_retry_after_for_test(Some("n/a"));
        assert_eq!(missing.retry_after_seconds(), 300);
    }
    #[test]
    fn given_parallel_detail_requests_when_passing_shared_budget_then_at_most_120_are_admitted() {
        use std::sync::Arc;
        let budget = Arc::new(ApiBudget::new());
        let allowed = Arc::new(AtomicUsize::new(0));
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let budget = Arc::clone(&budget);
                let allowed = Arc::clone(&allowed);
                scope.spawn(move || {
                    for _ in 0..16 {
                        if budget.check().is_ok() {
                            allowed.fetch_add(1, Ordering::AcqRel);
                        }
                    }
                });
            }
        });
        assert_eq!(allowed.load(Ordering::Acquire), MAX_CALLS_PER_POLL);
        assert!(budget.exhausted());
        assert!(budget.check().is_err());
    }

    #[test]
    fn given_repeated_and_parallel_429_headers_when_seen_then_longest_bounded_delay_wins() {
        use std::sync::Arc;
        let budget = Arc::new(ApiBudget::new());
        std::thread::scope(|scope| {
            for input in ["60", "440", "1801", "1000", "120", "invalid"] {
                let clone = Arc::clone(&budget);
                scope.spawn(move || clone.observe_retry_after_for_test(Some(input)));
            }
        });
        assert_eq!(budget.retry_after_seconds(),1800);
        assert!(budget.exhausted());
        assert!(budget.check().is_err());
    }

    #[test]
    fn given_retry_after_boundary_variants_when_parsing_then_no_unbounded_or_negative_delay() {
        for (raw, result) in [
            ("",None),("-0",None),("-60",None),("NaN",None),
            ("inf",None),("1.5",None),("300.0",None),
            (" 60 ",Some(60)),("59",Some(60)),("60",Some(60)),
            ("61",Some(61)),("1799",Some(1799)),
            ("1800",Some(1800)),("1801",Some(1800)),
            ("18446744073709551615",Some(1800)),
            ("18446744073709551616",None),
            ("999999999999999999999999999999999999",None),
        ] {
            assert_eq!(parse_retry_after(raw),result,"{raw:?}");
        }
    }

    #[test]
    fn given_no_rate_limit_when_requests_are_under_cap_then_retry_delay_is_zero() {
        let budget = ApiBudget::new();
        for i in 0..MAX_CALLS_PER_POLL {
            assert!(!budget.exhausted(),"before admitted request {i}");
            assert!(budget.check().is_ok(),"request {i}");
            assert_eq!(budget.retry_after_seconds(),0);
        }
        assert!(budget.exhausted());
        assert!(budget.check().is_err());
    }


    #[tokio::test]
    async fn given_non429_http_responses_when_observed_then_budget_remains_available() {
        use crate::test_http::FixtureHttp;
        use serde_json::json;

        let server = FixtureHttp::start(|request| {
            let status = match request.path.as_str() {
                "/success" => 200,
                "/bad-request" => 400,
                "/unauthorized" => 401,
                "/forbidden" => 403,
                "/server-error" => 500,
                "/unavailable" => 503,
                path => panic!("unexpected fixture path: {path}"),
            };
            (status, json!({"errors":[{"message":"fixture-private-secret"}]}))
        });
        let budget = ApiBudget::new();
        let client = reqwest::Client::new();
        for (path, status) in [
            ("/success", 200), ("/bad-request", 400),
            ("/unauthorized", 401), ("/forbidden", 403),
            ("/server-error", 500), ("/unavailable", 503),
        ] {
            assert!(budget.check().is_ok(), "admission before {path}");
            let response = client.get(server.url(path)).send().await.unwrap();
            assert_eq!(response.status().as_u16(), status, "{path}");
            budget.observe(&response);
            assert_eq!(budget.retry_after_seconds(), 0, "{path}");
            assert!(!budget.exhausted(), "{path} must not throttle");
        }
        let requests = server.requests();
        assert_eq!(requests.len(), 6);
        for (request, (path, _)) in requests.iter().zip([
            ("/success", 200), ("/bad-request", 400),
            ("/unauthorized", 401), ("/forbidden", 403),
            ("/server-error", 500), ("/unavailable", 503),
        ]) {
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, path);
        }
        assert!(budget.check().is_ok(), "non-429 HTTP failures are not rate limits");
    }

    #[tokio::test]
    async fn given_real_http_429_when_observed_then_retry_after_stops_followup_admissions() {
        use crate::test_http::FixtureHttp;
        use serde_json::json;

        let server = FixtureHttp::start(|request| {
            assert_eq!(request.path, "/throttled");
            (429, json!({"errors":[{"message":"fixture-private-secret"}]}))
        });
        let budget = ApiBudget::new();
        assert_eq!(budget.retry_after_seconds(), 0);
        assert!(!budget.exhausted());
        budget.check().unwrap();

        let response = reqwest::Client::new().get(server.url("/throttled"))
            .send().await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        budget.observe(&response);
        assert_eq!(budget.retry_after_seconds(), 75,
            "read the Retry-After header from the actual HTTP response");
        assert!(budget.exhausted());
        let error = budget.check().unwrap_err();
        assert_eq!(error, msg::BUDGET_RATE_LIMIT);
        assert!(!error.contains("fixture-private-secret"));
        assert_eq!(server.requests().len(), 1,
            "rejected retry must not issue another HTTP request");
    }

    #[tokio::test]
    async fn given_independent_poll_budgets_when_one_gets_429_then_other_is_not_throttled() {
        use crate::test_http::FixtureHttp;
        use serde_json::json;

        let server = FixtureHttp::start(|request| {
            match request.path.as_str() {
                "/first-poll" => (429, json!({"success":false})),
                "/other-poll" => (200, json!({"success":true})),
                path => panic!("unexpected fixture path: {path}"),
            }
        });
        let throttled = ApiBudget::new();
        let independent = ApiBudget::new();
        let client = reqwest::Client::new();

        throttled.check().unwrap();
        let first = client.get(server.url("/first-poll")).send().await.unwrap();
        throttled.observe(&first);
        assert_eq!(throttled.retry_after_seconds(), 75);
        assert!(throttled.exhausted());
        assert_eq!(throttled.check().unwrap_err(), msg::BUDGET_RATE_LIMIT);

        independent.check().unwrap();
        let second = client.get(server.url("/other-poll")).send().await.unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        independent.observe(&second);
        assert_eq!(independent.retry_after_seconds(), 0);
        assert!(!independent.exhausted());
        assert!(independent.check().is_ok());

        let later_poll = ApiBudget::new();
        assert_eq!(later_poll.retry_after_seconds(), 0);
        assert!(!later_poll.exhausted());
        assert!(later_poll.check().is_ok());
        assert_eq!(server.requests().iter().map(|r| r.path.as_str())
            .collect::<Vec<_>>(), vec!["/first-poll", "/other-poll"]);
    }

}
