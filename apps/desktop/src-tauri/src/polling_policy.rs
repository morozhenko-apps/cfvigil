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

    #[tokio::test]
    async fn settings_revisions_are_coalesced_and_not_lost() {
        let (sender, mut receiver) = tokio::sync::watch::channel(0u64);
        let _ = *receiver.borrow_and_update();
        for _ in 0..3 {
            sender.send_modify(|revision| *revision += 1);
        }
        assert!(receiver.has_changed().unwrap());
        receiver.changed().await.unwrap();
        assert_eq!(*receiver.borrow_and_update(), 3);
        assert!(!receiver.has_changed().unwrap());
        sender.send_modify(|revision| *revision += 1);
        receiver.changed().await.unwrap();
        assert_eq!(*receiver.borrow_and_update(), 4);
    }

    #[test]
    fn failures_back_off_without_busy_retries() {
        assert_eq!(poll_delay(60, 1), Duration::from_secs(120));
        assert_eq!(poll_delay(60, 2), Duration::from_secs(240));
        assert_eq!(poll_delay(180, 3), Duration::from_secs(1440));
        assert_eq!(poll_delay(600, 15), Duration::from_secs(1800));
        assert_eq!(poll_delay(600, u32::MAX), Duration::from_secs(1800));
    }
    #[test]
    fn given_every_retry_backoff_boundary_when_calculating_delay_then_no_busy_poll_or_overflow_occurs() {
        let cases = [
            (0,0,60),(59,0,60),(60,0,60),(61,0,61),
            (180,0,180),(599,0,599),(600,0,600),(601,0,600),
            (u64::MAX,0,600),
            (60,1,120),(60,2,240),(60,3,480),(60,4,960),
            (60,5,1800),(60,6,1800),(60,u32::MAX,1800),
            (180,1,360),(180,2,720),(180,3,1440),
            (180,4,1800),(180,u32::MAX,1800),
            (600,1,1200),(600,2,1800),(600,u32::MAX,1800),
            (u64::MAX,u32::MAX,1800),
        ];
        for (configured,failures,expected) in cases {
            assert_eq!(poll_delay(configured,failures),
                Duration::from_secs(expected),
                "interval={configured}, consecutive_failures={failures}");
        }
    }

    #[test]
    fn given_settings_revision_burst_when_coalescing_then_latest_revision_wakes_once() {
        let (sender,mut watcher)=tokio::sync::watch::channel(0u64);
        assert_eq!(*watcher.borrow_and_update(),0);
        for revision in 1..=64 {
            sender.send_modify(|counter| *counter = revision);
        }
        assert!(watcher.has_changed().unwrap());
        assert_eq!(*watcher.borrow_and_update(),64);
        assert!(!watcher.has_changed().unwrap());
    }

}
