//! Pure incident transition policy; independent from OS windows and APIs.
use crate::native_messages as msg;
use crate::cloudflare::Resource;

fn waf_spike(resource: &Resource) -> bool {
    resource.kind == "zone"
        && resource.attempt_id == "waf-block-spike"
        && resource.status == "failed"
}

fn runtime_spike(resource: &Resource) -> bool {
    resource.detail.starts_with("Workers errors spiking:")
        || resource.attempt_id == "runtime-error-spike"
}

pub fn describe_transition(old: &Resource, current: &Resource) -> Option<&'static str> {
    // An analytics outage or app restart can turn a known WAF alert grey.
    // Its return to red is continuation, not a fresh notification.
    if old.kind == "zone" && old.attempt_id == "waf-block-spike" && waf_spike(current) {
        return None;
    }
    if old.kind == "zone" && current.kind == "zone" && current.status == "unknown" {
        return None;
    }
    if current.kind == "zone" {
        return waf_spike(current).then_some(msg::EVENT_WAF_SPIKE);
    }
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
        return Some(msg::EVENT_RUNTIME_SPIKE);
    }
    if had_spike {
        return Some(msg::EVENT_RUNTIME_RECOVERY);
    }
    match current.status.as_str() {
        "failed" => Some(msg::EVENT_BUILD_FAILURE),
        "deploying" => Some(msg::EVENT_BUILD_STARTED),
        "healthy" => Some(msg::EVENT_BUILD_SUCCESS),
        _ => None,
    }
}

pub fn should_notify(current: &Resource, message: &str) -> bool {
    current.status == "failed"
        && (message == msg::EVENT_BUILD_FAILURE
            || message == msg::EVENT_RUNTIME_SPIKE
            || message == msg::EVENT_WAF_SPIKE)
}

pub fn initial_security_event(current: &Resource) -> Option<&'static str> {
    waf_spike(current).then_some(msg::EVENT_WAF_SPIKE)
}

pub fn notification_identity(current: &Resource, message: &str) -> Option<String> {
    if !should_notify(current, message) {
        return None;
    }
    if waf_spike(current) {
        Some(format!("{}:waf-spike", current.key))
    } else if runtime_spike(current) {
        Some(format!("{}:runtime-spike", current.key))
    } else {
        Some(format!("{}:failed:{}", current.key, current.attempt_id))
    }
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
    fn incident_keys_deduplicate_reconnects_but_distinguish_new_failed_builds() {
        let old = make("failed", "first", "Cloudflare build");
        let newer = make("failed", "second", "Cloudflare build");
        let msg = "Latest Cloudflare build/deployment failed";
        assert_eq!(notification_identity(&old, msg), notification_identity(&old, msg));
        assert_ne!(notification_identity(&old, msg), notification_identity(&newer, msg));
        let spike = make("failed", "runtime-error-spike", "Workers errors spiking: 20 errors / 100 requests");
        assert_eq!(notification_identity(&spike, "Workers runtime error spike detected"),
            Some("worker:example:runtime-spike".to_string()));
        assert_eq!(notification_identity(&spike, "Workers runtime error spike cleared"), None);
    }

    #[test]
    fn waf_zone_alerts_are_new_only_and_remain_attributed_to_zone() {
        let mut waf = make("failed", "waf-block-spike", "WAF blocks spiking: 200 in 10m");
        waf.kind = "zone".into();
        waf.key = "zone:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into();
        assert_eq!(initial_security_event(&waf), Some("WAF block spike detected"));
        assert_eq!(notification_identity(&waf,"WAF block spike detected"),
            Some("zone:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee:waf-spike".into()));
        assert_eq!(describe_transition(&waf, &waf), None);
        let mut unknown = waf.clone();
        unknown.status = "unknown".into();
        assert_eq!(describe_transition(&waf,&unknown), None);
        assert_eq!(describe_transition(&unknown, &waf), None);
        let mut new_zone = waf.clone();
        new_zone.key = "zone:new".into();
        assert_eq!(initial_security_event(&new_zone), Some("WAF block spike detected"));
    }

    #[test]
    fn unknown_to_unknown_and_unknown_status_are_silent() {
        let a = make("unknown", "", "No build data");
        let b = make("unknown", "new-id", "No build data");
        assert_eq!(describe_transition(&a, &b), None);
    }
    #[test]
    fn given_each_latest_attempt_transition_when_classifying_then_only_verified_events_are_emitted() {
        let scenarios = [
            ("healthy", "old", "deploying", "new", Some(msg::EVENT_BUILD_STARTED)),
            ("failed", "old", "deploying", "new", Some(msg::EVENT_BUILD_STARTED)),
            ("deploying", "old", "failed", "old", Some(msg::EVENT_BUILD_FAILURE)),
            ("deploying", "old", "healthy", "old", Some(msg::EVENT_BUILD_SUCCESS)),
            ("failed", "old", "failed", "new", Some(msg::EVENT_BUILD_FAILURE)),
            ("healthy", "old", "healthy", "new", Some(msg::EVENT_BUILD_SUCCESS)),
            ("healthy", "old", "unknown", "new", None),
            ("failed", "old", "unknown", "new", None),
            ("unknown", "", "unknown", "new", None),
            ("deploying", "one", "deploying", "one", None),
            ("failed", "one", "failed", "one", None),
        ];
        for (before, first_id, after, next_id, expected) in scenarios {
            let old = make(before, first_id, "Latest verified attempt");
            let next = make(after, next_id, "Latest verified attempt");
            assert_eq!(describe_transition(&old,&next),expected,
                "{before}:{first_id} -> {after}:{next_id}");
        }
    }

    #[test]
    fn given_all_event_messages_when_deciding_notification_then_only_new_failed_alerts_are_notifiable() {
        for status in ["healthy","deploying","failed","unknown"] {
            let row = make(status,"build-1","Cloudflare build");
            for (message, category_allowed) in [
                (msg::EVENT_BUILD_FAILURE,true),
                (msg::EVENT_RUNTIME_SPIKE,true),
                (msg::EVENT_WAF_SPIKE,true),
                (msg::EVENT_BUILD_STARTED,false),
                (msg::EVENT_BUILD_SUCCESS,false),
                (msg::EVENT_RUNTIME_RECOVERY,false),
                (msg::EVENT_WAF_RECOVERY,false),
                ("unexpected message",false),
            ] {
                let expected = status == "failed" && category_allowed;
                assert_eq!(should_notify(&row,message),expected,
                    "{status} / {message}");
            }
        }
    }

    #[test]
    fn given_repeated_runtime_spikes_and_unknown_analytics_when_traversing_states_then_no_false_recovery() {
        let stable = make("healthy","build-1","Build verified");
        let spike = make("failed","runtime-error-spike","Workers errors spiking: 20");
        let stale = make("unknown","runtime-error-spike","Workers analytics unavailable");
        let retry = make("failed","runtime-error-spike","Workers errors spiking: 40");
        let recovered = make("healthy","build-2","Build verified");
        assert_eq!(describe_transition(&stable,&spike),Some(msg::EVENT_RUNTIME_SPIKE));
        assert_eq!(describe_transition(&spike,&stale),None);
        assert_eq!(describe_transition(&stale,&retry),None);
        assert_eq!(describe_transition(&retry,&recovered),Some(msg::EVENT_RUNTIME_RECOVERY));
        assert_eq!(describe_transition(&stale,&recovered),Some(msg::EVENT_RUNTIME_RECOVERY));
        assert!(notification_identity(&stale,msg::EVENT_RUNTIME_SPIKE).is_none());
    }

    #[test]
    fn given_zone_level_waf_alert_when_old_zone_unverified_then_it_is_not_a_second_new_incident() {
        let mut initial = make("unknown","","No WAF confirmation");
        initial.kind = "zone".into();
        initial.key = "zone:aabbcc".into();
        let mut waf = initial.clone();
        waf.status = "failed".into();
        waf.attempt_id = "waf-block-spike".into();
        waf.detail = "WAF blocks spiking: 10".into();
        assert_eq!(describe_transition(&initial,&waf),Some(msg::EVENT_WAF_SPIKE));
        assert_eq!(initial_security_event(&waf),Some(msg::EVENT_WAF_SPIKE));
        let mut unverified = waf.clone();
        unverified.status = "unknown".into();
        assert_eq!(describe_transition(&waf,&unverified),None);
        assert_eq!(describe_transition(&unverified,&waf),None);
        assert_eq!(notification_identity(&waf,msg::EVENT_WAF_SPIKE),
            Some("zone:aabbcc:waf-spike".to_owned()));
    }

    #[test]
    fn given_different_failed_attempt_ids_when_identifying_notifications_then_each_new_attempt_is_distinct() {
        let old = make("failed","uuid-1","Worker build failed");
        let newer = make("failed","uuid-2","Worker build failed");
        assert_ne!(
            notification_identity(&old,msg::EVENT_BUILD_FAILURE),
            notification_identity(&newer,msg::EVENT_BUILD_FAILURE)
        );
        assert_eq!(
            notification_identity(&old,msg::EVENT_BUILD_FAILURE),
            Some("worker:example:failed:uuid-1".into())
        );
        let unchanged = make("failed","uuid-1","Worker build failed");
        assert_eq!(
            notification_identity(&old,msg::EVENT_BUILD_FAILURE),
            notification_identity(&unchanged,msg::EVENT_BUILD_FAILURE)
        );
    }

}
