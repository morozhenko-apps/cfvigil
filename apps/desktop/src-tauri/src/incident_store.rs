//! Durable, account-scoped incident continuity. No Cloudflare credentials
//! are saved here: only the same resource identifiers/URLs already held in
//! local event history and bounded notification identities.
use crate::cloudflare::Resource;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

const MAX_NOTIFICATIONS: usize = 512;
const MAX_ACTIVE: usize = 320;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentJournal {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub active: Vec<Resource>,
    #[serde(default)]
    pub notified: Vec<String>,
}

fn active_spike(resource: &Resource) -> bool {
    (resource.kind == "zone" && resource.attempt_id == "waf-block-spike")
        || (resource.kind == "worker" && resource.attempt_id == "runtime-error-spike")
}

fn stale_spike(resource: &Resource) -> Resource {
    let mut item = resource.clone();
    item.status = "unknown".into();
    item.detail = if item.kind == "zone" {
        "WAF analytics unavailable; previous alert unverified".into()
    } else {
        "Workers errors spiking: previous incident unverified".into()
    };
    item
}

impl IncidentJournal {
    pub fn empty(account_id: &str) -> Self {
        Self { account_id: account_id.to_owned(), ..Self::default() }
    }

    /// Never load incidents belonging to another Cloudflare account.
    /// Bound malformed/oversized persisted JSON arrays defensively.
    pub fn restore(self, account_id: &str) -> Self {
        if self.account_id != account_id || account_id.is_empty() {
            return Self::empty(account_id);
        }
        let mut active = Vec::new();
        let mut seen = HashSet::new();
        for item in self.active {
            if active.len() >= MAX_ACTIVE { break; }
            if active_spike(&item) && seen.insert(item.key.clone()) {
                active.push(stale_spike(&item));
            }
        }
        let notified = NotificationWindow::from_ids(self.notified).ids();
        Self { account_id: account_id.to_owned(), active, notified }
    }

    pub fn snapshot(account_id: &str, resources: &[Resource], notified: &NotificationWindow) -> Self {
        Self {
            account_id: account_id.to_owned(),
            active: resources.iter().filter(|item| active_spike(item))
                .take(MAX_ACTIVE).map(stale_spike).collect(),
            notified: notified.ids(),
        }
    }
}

/// Deduplicate across polls/restarts without retaining a permanent,
/// unbounded set of old deployment attempt IDs.
#[derive(Default)]
pub struct NotificationWindow {
    order: VecDeque<String>,
    seen: HashSet<String>,
}

impl NotificationWindow {
    pub fn from_ids(ids: Vec<String>) -> Self {
        let mut window = Self::default();
        for id in ids { window.insert(id); }
        window
    }

    pub fn insert(&mut self, id: String) -> bool {
        if id.is_empty() || !self.seen.insert(id.clone()) {
            return false;
        }
        self.order.push_back(id);
        if self.order.len() > MAX_NOTIFICATIONS {
            if let Some(expired) = self.order.pop_front() {
                self.seen.remove(&expired);
            }
        }
        true
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if !self.seen.remove(id) { return false; }
        self.order.retain(|value| value != id);
        true
    }

    pub fn clear(&mut self) {
        self.order.clear();
        self.seen.clear();
    }

    pub fn ids(&self) -> Vec<String> { self.order.iter().cloned().collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(kind: &str, key: &str, attempt: &str, status: &str) -> Resource {
        Resource {
            key: key.into(), kind: kind.into(), name: "example".into(),
            status: status.into(), changed_at: "2026-10-23T12:00:00Z".into(),
            attempt_id: attempt.into(),
            url: "https://dash.cloudflare.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            detail: "Workers errors spiking: 30/120".into(),
        }
    }

    #[test]
    fn restart_never_recreates_a_green_or_red_verified_status() {
        let mut ids = NotificationWindow::default();
        ids.insert("worker:api:runtime-spike".into());
        ids.insert("zone:site:waf-spike".into());
        let current = vec![
            resource("worker", "worker:api", "runtime-error-spike", "failed"),
            resource("zone", "zone:site", "waf-block-spike", "failed"),
            resource("worker", "worker:normal", "new-build", "healthy"),
        ];
        let journal = IncidentJournal::snapshot("a", &current, &ids);
        assert_eq!(journal.active.len(), 2);
        let bytes = serde_json::to_vec(&journal).unwrap();
        let saved: IncidentJournal = serde_json::from_slice(&bytes).unwrap();
        let restored = saved.restore("a");
        assert_eq!(restored.active.len(), 2);
        assert!(restored.active.iter().all(|r| r.status == "unknown"));
        assert_eq!(restored.active[0].changed_at, current[0].changed_at);
        assert_eq!(restored.notified.len(), 2);
        assert!(!NotificationWindow::from_ids(restored.notified)
            .insert("zone:site:waf-spike".into()));
    }

    #[test]
    fn account_switch_discards_all_old_incidents_and_identities() {
        let saved = IncidentJournal {
            account_id: "old".into(),
            active: vec![resource("zone", "zone:old", "waf-block-spike", "failed")],
            notified: vec!["zone:old:waf-spike".into()],
        };
        let restored = saved.restore("new");
        assert_eq!(restored.account_id, "new");
        assert!(restored.active.is_empty());
        assert!(restored.notified.is_empty());
    }

    #[test]
    fn notification_window_is_bounded_and_recovery_can_rearm() {
        let mut window = NotificationWindow::default();
        assert!(window.insert("zone:site:waf-spike".into()));
        assert!(!window.insert("zone:site:waf-spike".into()));
        assert!(window.remove("zone:site:waf-spike"));
        assert!(window.insert("zone:site:waf-spike".into()));
        for index in 0..600 { window.insert(format!("attempt:{index}")); }
        assert_eq!(window.ids().len(), MAX_NOTIFICATIONS);
        assert!(window.insert("attempt:0".into()));
        assert_eq!(window.ids().len(), MAX_NOTIFICATIONS);
        assert!(!window.insert("attempt:599".into()));
    }

    #[test]
    fn malformed_and_duplicate_saved_active_incidents_are_sanitized() {
        let good = resource("worker", "worker:api", "runtime-error-spike", "failed");
        let saved = IncidentJournal {
            account_id: "a".into(),
            active: vec![good.clone(), good, resource("pages", "pages:foo", "build1", "healthy")],
            notified: vec!["same".into(), "same".into(), "".into()],
        };
        let checked = saved.restore("a");
        assert_eq!(checked.active.len(), 1);
        assert_eq!(checked.notified, vec!["same"]);
    }
    #[test]
    fn given_account_missing_changed_or_matching_when_restoring_then_incident_identity_is_scoped() {
        let original=IncidentJournal {
            account_id:"a".repeat(32),
            active:vec![resource("worker","worker:alpha","runtime-error-spike","failed")],
            notified:vec!["worker:alpha:runtime-spike".into()],
        };
        for wrong in ["".to_owned(),"b".repeat(32),"a".repeat(31)] {
            let restored=original.clone().restore(&wrong);
            assert_eq!(restored.account_id,wrong);
            assert!(restored.active.is_empty());
            assert!(restored.notified.is_empty());
        }
        let same=original.clone().restore(&original.account_id);
        assert_eq!(same.active.len(),1);
        assert_eq!(same.active[0].status,"unknown");
        assert_eq!(same.notified,original.notified);
    }

    #[test]
    fn given_0_1_320_321_incidents_when_snapshotting_then_journal_is_strictly_bounded() {
        for size in [0,1,319,320,321] {
            let resources=(0..size).map(|index|resource(
                "worker",&format!("worker:job{index}"),"runtime-error-spike","failed"
            )).collect::<Vec<_>>();
            let journal=IncidentJournal::snapshot("a",&resources,&NotificationWindow::default());
            assert_eq!(journal.active.len(),size.min(MAX_ACTIVE));
            assert!(journal.active.iter().all(|item|item.status=="unknown"
                && item.attempt_id=="runtime-error-spike"));
            let restored=journal.restore("a");
            assert_eq!(restored.active.len(),size.min(MAX_ACTIVE));
            assert_eq!(restored.active.first().map(|r|r.key.as_str()),
                resources.first().map(|r|r.key.as_str()));
        }
    }

    #[test]
    fn given_each_notification_window_boundary_when_evicted_then_old_ids_can_rearm() {
        let mut window=NotificationWindow::default();
        assert!(!window.insert(String::new()));
        for index in 0..MAX_NOTIFICATIONS {
            assert!(window.insert(format!("alert:{index}")));
        }
        assert_eq!(window.ids().len(),MAX_NOTIFICATIONS);
        assert!(!window.insert("alert:0".into()));
        assert!(!window.insert("alert:511".into()));
        assert!(window.insert("alert:new".into()));
        assert_eq!(window.ids().len(),MAX_NOTIFICATIONS);
        assert!(!window.ids().contains(&"alert:0".to_string()));
        assert!(window.insert("alert:0".into()));
        assert!(window.remove("alert:new"));
        assert!(!window.remove("alert:new"));
        assert!(window.insert("alert:new".into()));
        window.clear();
        assert!(window.ids().is_empty());
        assert!(window.insert("alert:511".into()));
    }

    #[test]
    fn given_corrupt_and_legacy_journal_shapes_when_parsing_then_default_fields_or_fail_closed() {
        let legacy: IncidentJournal=serde_json::from_str(
            r#"{"account_id":"a"}"#).unwrap();
        assert_eq!(legacy.account_id,"a");
        assert!(legacy.active.is_empty() && legacy.notified.is_empty());
        for json in [
            r#"{"account_id":null,"active":[]}"#,
            r#"{"account_id":"a","active":"unexpected"}"#,
            r#"{"account_id":"a","notified":[7]}"#,
            r#"{not-json"#,
        ] {
            assert!(serde_json::from_str::<IncidentJournal>(json).is_err(),
                "invalid JSON: {json}");
        }
    }

}
