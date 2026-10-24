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
}
