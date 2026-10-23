//! One poller uses the configured period on success and bounded backoff
//! on network/credentials failures. A settings change wakes it immediately.
use std::time::Duration;

pub fn poll_delay(configured_seconds: u64, consecutive_failures: u32) -> Duration {
    let base = configured_seconds.clamp(60, 600);
    if consecutive_failures == 0 {
        return Duration::from_secs(base);
    }
    let multiplier = 1u64 << consecutive_failures.min(5);
    Duration::from_secs(base.saturating_mul(multiplier).min(1800))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_delay_bounds_unsafe_settings() {
        assert_eq!(poll_delay(180, 0), Duration::from_secs(180));
        assert_eq!(poll_delay(0, 0), Duration::from_secs(60));
        assert_eq!(poll_delay(10000, 0), Duration::from_secs(600));
    }

    #[test]
    fn failures_back_off_without_busy_retries() {
        assert_eq!(poll_delay(60, 1), Duration::from_secs(120));
        assert_eq!(poll_delay(60, 2), Duration::from_secs(240));
        assert_eq!(poll_delay(180, 3), Duration::from_secs(1440));
        assert_eq!(poll_delay(600, 15), Duration::from_secs(1800));
        assert_eq!(poll_delay(600, u32::MAX), Duration::from_secs(1800));
    }
}
