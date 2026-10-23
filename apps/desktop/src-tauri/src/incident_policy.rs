//! Pure incident transition policy; independent from OS windows and APIs.
use crate::cloudflare::Resource;

fn runtime_spike(resource: &Resource) -> bool {
    resource.detail.starts_with("Workers errors spiking:")
        || resource.attempt_id == "runtime-error-spike"
}

pub fn describe_transition(old: &Resource, current: &Resource) -> Option<&'static str> {
    let had_spike = runtime_spike(old);
    let has_spike = runtime_spike(current);
    if had_spike && has_spike {
        return None;
    }
    // Analytics outage is not a verified recovery.
    if had_spike && current.status == "unknown" {
        return None;
    }
    if old.status == current.status && old.attempt_id == current.attempt_id {
        return None;
    }
    if has_spike {
        return Some("Workers runtime error spike detected");
    }
    if had_spike {
        return Some("Workers runtime error spike cleared");
    }
    match current.status.as_str() {
        "failed" => Some("Latest Cloudflare build/deployment failed"),
        "deploying" => Some("New Cloudflare build/deployment started"),
        "healthy" => Some("Latest Cloudflare deployment succeeded"),
        _ => None,
    }
}

pub fn should_notify(current: &Resource, message: &str) -> bool {
    current.status == "failed"
        && (message == "Latest Cloudflare build/deployment failed"
            || message == "Workers runtime error spike detected")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(status: &str, id: &str, detail: &str) -> Resource {
        Resource {
            key: "worker:example".into(),
            name: "example".into(),
            kind: "worker".into(),
            status: status.into(),
            changed_at: "2026-10-22T12:00:00Z".into(),
            attempt_id: id.into(),
            detail: detail.into(),
            url: "https://dash.cloudflare.com/example".into(),
        }
    }

    #[test]
    fn new_running_build_supersedes_old_failure() {
        let old = make("failed", "build-1", "Cloudflare build");
        let new = make("deploying", "build-2", "Cloudflare build");
        assert_eq!(describe_transition(&old, &new), Some("New Cloudflare build/deployment started"));
        assert!(!should_notify(&new, "New Cloudflare build/deployment started"));
    }

    #[test]
    fn repeated_failed_build_does_not_notify() {
        let failed = make("failed", "build-1", "Cloudflare build");
        assert_eq!(describe_transition(&failed, &failed), None);
    }

    #[test]
    fn runtime_spike_triggers_once_and_is_not_cleared_by_new_build() {
        let baseline = make("healthy", "build-1", "Cloudflare build");
        let spike = make("failed", "runtime-error-spike", "Workers errors spiking: 30 errors / 120 requests");
        let ongoing = make("failed", "runtime-error-spike", "Workers errors spiking: 40 errors / 140 requests");
        let message = describe_transition(&baseline, &spike).unwrap();
        assert_eq!(message, "Workers runtime error spike detected");
        assert!(should_notify(&spike, message));
        assert_eq!(describe_transition(&spike, &ongoing), None);
    }

    #[test]
    fn resolving_spike_does_not_claim_a_deployment_happened() {
        let spike = make("failed", "runtime-error-spike", "Workers errors spiking: 24 errors / 100 requests");
        let recovered = make("healthy", "build-1", "Cloudflare build");
        let message = describe_transition(&spike, &recovered).unwrap();
        assert_eq!(message, "Workers runtime error spike cleared");
        assert!(!should_notify(&recovered, message));
    }

    #[test]
    fn analytics_outage_keeps_spike_unresolved_without_repeat_notification() {
        let incident = make("failed", "runtime-error-spike", "Workers errors spiking: 30 errors / 120 requests");
        let offline = make("unknown", "runtime-error-spike", "Workers analytics unavailable");
        let resuming = make("failed", "runtime-error-spike", "Workers errors spiking: 31 errors / 123 requests");
        let resolved = make("healthy", "build-12", "Cloudflare build");
        assert_eq!(describe_transition(&incident, &offline), None);
        assert_eq!(describe_transition(&offline, &resuming), None);
        assert_eq!(describe_transition(&offline, &resolved), Some("Workers runtime error spike cleared"));
    }

    #[test]
    fn unknown_to_unknown_and_unknown_status_are_silent() {
        let a = make("unknown", "", "No build data");
        let b = make("unknown", "new-id", "No build data");
        assert_eq!(describe_transition(&a, &b), None);
    }
}
