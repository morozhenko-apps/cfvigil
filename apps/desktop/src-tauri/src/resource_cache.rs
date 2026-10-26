//! Account-scoped last-known Worker/Pages roster for instant grey startup.
//! A cached entry proves only that this resource previously existed. It NEVER
//! proves current deployment health, endpoint uptime, or a recovered incident.
use crate::cloudflare::Resource;
use crate::native_messages as msg;
use serde::{Deserialize, Serialize};

const MAX_CACHED: usize = 512;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceCatalogCache {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub resources: Vec<Resource>,
}

fn safe_name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 &&
        value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn safe_account(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn accepted(row: &Resource) -> bool {
    matches!(row.kind.as_str(), "worker" | "pages")
        && safe_name(&row.name)
        && row.key == format!("{}:{}", row.kind, row.name)
}

fn stale_row(row: &Resource, account: &str) -> Resource {
    // Reconstruct a trusted dashboard URL: never follow a persisted URL from
    // an untrusted or manually edited JSON file.
    let route = if row.kind == "worker" {
        format!("workers/services/view/{}/production", row.name)
    } else {
        format!("pages/view/{}", row.name)
    };
    Resource {
        key: row.key.clone(),
        name: row.name.clone(),
        kind: row.kind.clone(),
        status: "unknown".into(),
        changed_at: row.changed_at.clone(),
        attempt_id: String::new(),
        url: format!("https://dash.cloudflare.com/{account}/{route}"),
        detail: msg::DATA_STALE.into(),
    }
}

impl ResourceCatalogCache {
    /// Previous account metadata must never become visible in another account.
    pub fn show_stale(&self, account: &str) -> Vec<Resource> {
        if !safe_account(account) || self.account_id != account || self.resources.len() > MAX_CACHED {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        self.resources.iter().filter(|r| accepted(r) && seen.insert(r.key.clone()))
            .map(|r| stale_row(r, account)).collect()
    }

    /// Save only a bounded catalog. Never record WAF incidents, token or
    /// provider success/failed status as a verified startup cache.
    pub fn from_discovery(account: &str, rows: &[Resource]) -> Option<Self> {
        if !safe_account(account) { return None; }
        let eligible: Vec<Resource> = rows.iter().filter(|r| accepted(r)).cloned().collect();
        if eligible.len() > MAX_CACHED { return None; }
        let mut seen = std::collections::HashSet::new();
        Some(Self {
            account_id: account.to_owned(),
            resources: eligible.iter().filter(|r| seen.insert(r.key.clone()))
                .map(|r| stale_row(r, account)).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make(kind: &str, name: &str, status: &str) -> Resource {
        Resource {
            key: format!("{kind}:{name}"), name: name.into(), kind: kind.into(),
            status: status.into(), changed_at: "2026-10-26T12:00:00Z".into(),
            attempt_id: "old-attempt".into(),
            url: "https://evil.example/steal?token=secret".into(),
            detail: "previous live status".into()
        }
    }

    #[test]
    fn cached_success_failure_and_work_in_progress_always_restart_unverified() {
        let account = "a".repeat(32);
        let source = [make("worker", "alpha", "healthy"), make("pages","beta","failed"),
            make("worker","gamma","deploying")];
        let cache = ResourceCatalogCache::from_discovery(&account, &source).unwrap();
        let rows = cache.show_stale(&account);
        assert_eq!(rows.len(), 3);
        for row in &rows {
            assert_eq!(row.status, "unknown");
            assert_eq!(row.attempt_id, "");
            assert_eq!(row.detail, msg::DATA_STALE);
            assert!(row.url.starts_with(&format!("https://dash.cloudflare.com/{account}/")));
            assert!(!row.url.contains("evil") && !row.url.contains("secret"));
        }
    }

    #[test]
    fn account_switch_and_corrupt_cached_names_are_fail_closed() {
        let account = "a".repeat(32);
        let mut cache = ResourceCatalogCache::from_discovery(
            &account, &[make("worker","alpha","healthy")]).unwrap();
        assert!(cache.show_stale(&"b".repeat(32)).is_empty());
        assert!(cache.show_stale("invalid").is_empty());
        cache.resources.push(make("worker","../evil","healthy"));
        cache.resources.push(make("zone","alpha","failed"));
        cache.resources.push(make("worker","alpha","failed"));
        assert_eq!(cache.show_stale(&account).len(), 1);
        cache.resources[0].key = "worker:forged".into();
        assert!(cache.show_stale(&account).is_empty());
    }

    #[test]
    fn oversized_roster_is_not_replayed_or_stored() {
        let account = "a".repeat(32);
        let source: Vec<_> = (0..513).map(|i| make("worker", &format!("worker{i}"), "healthy")).collect();
        assert!(ResourceCatalogCache::from_discovery(&account, &source).is_none());
        let cache = ResourceCatalogCache { account_id:account.clone(), resources:source };
        assert!(cache.show_stale(&account).is_empty());
    }
}
