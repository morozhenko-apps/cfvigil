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
        // The forged item is rejected; the independently valid duplicate
        // remains safe and is still shown only as grey.
        let remaining = cache.show_stale(&account);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].key, "worker:alpha");
        assert_eq!(remaining[0].status, "unknown");
    }

    #[test]
    fn oversized_roster_is_not_replayed_or_stored() {
        let account = "a".repeat(32);
        let source: Vec<_> = (0..513).map(|i| make("worker", &format!("worker{i}"), "healthy")).collect();
        assert!(ResourceCatalogCache::from_discovery(&account, &source).is_none());
        let cache = ResourceCatalogCache { account_id:account.clone(), resources:source };
        assert!(cache.show_stale(&account).is_empty());
    }

    #[test]
    fn given_account_id_when_validating_then_exact_ascii_hex_bounds_are_enforced() {
        for account in [
            "a".repeat(32), "A".repeat(32), "0123456789abcdef0123456789ABCDEF".into()
        ] {
            assert!(safe_account(&account), "valid hex: {account}");
        }
        for account in [
            "".to_owned(), "a".repeat(31), "a".repeat(33),
            "g".repeat(32), "a".repeat(31) + " ", "é".repeat(16),
        ] {
            assert!(!safe_account(&account), "invalid account: {account}");
        }
    }

    #[test]
    fn given_resource_name_when_validating_then_injection_and_128_character_boundary_are_enforced() {
        let exact_limit = "a".repeat(128);
        let over_limit = "a".repeat(129);
        for name in ["x", "worker_name", "pages-service", "A0123", exact_limit.as_str()] {
            assert!(safe_name(name), "valid name: {name}");
        }
        for name in ["", ".", "..", "../escape", "worker/path", "name?secret",
            "name#fragment", "<script>", "tab\tname", "newline\nname",
            "é", "bad%2fescape", over_limit.as_str()] {
            assert!(!safe_name(name), "invalid name: {name}");
        }
    }

    #[test]
    fn given_0_1_512_513_entries_when_cached_then_the_limit_is_strict() {
        let account = "a".repeat(32);
        for (count, allowed) in [(0,true),(1,true),(511,true),(512,true),(513,false)] {
            let input: Vec<Resource> = (0..count)
                .map(|i| make("worker", &format!("worker{i}"), "healthy")).collect();
            let cache = ResourceCatalogCache::from_discovery(&account, &input);
            assert_eq!(cache.is_some(), allowed, "catalog length {count}");
            if let Some(cache) = cache {
                let rows = cache.show_stale(&account);
                assert_eq!(rows.len(), count);
                assert!(rows.iter().all(|r| r.status == "unknown"
                    && r.attempt_id.is_empty()));
            }
        }
    }

    #[test]
    fn given_invalid_kinds_and_keys_when_building_cache_then_only_trusted_worker_pages_survive() {
        let account = "a".repeat(32);
        let mut forged_key = make("worker", "real", "healthy");
        forged_key.key = "worker:someone-else".into();
        let mut corrupted_kind = make("worker", "other", "failed");
        corrupted_kind.kind = "zone".into();
        let rows = [
            make("worker", "real", "healthy"),
            make("pages", "site-one", "deploying"),
            make("zone", "zone-one", "failed"),
            make("worker", "bad/path", "failed"),
            forged_key, corrupted_kind
        ];
        let cache = ResourceCatalogCache::from_discovery(&account, &rows).unwrap();
        let expected: Vec<_> = cache.show_stale(&account)
            .into_iter().map(|r| r.key).collect();
        assert_eq!(expected, ["worker:real", "pages:site-one"]);
    }

    #[test]
    fn given_duplicate_row_keys_when_caching_then_first_row_wins_and_output_does_not_duplicate() {
        let account = "a".repeat(32);
        let source = [
            make("worker", "alpha", "failed"),
            make("worker", "alpha", "healthy"),
            make("pages", "alpha", "deploying")
        ];
        let cache = ResourceCatalogCache::from_discovery(&account, &source).unwrap();
        assert_eq!(cache.resources.len(), 2);
        assert_eq!(cache.show_stale(&account).len(), 2);
        assert_eq!(cache.resources[0].key, "worker:alpha");
        assert_eq!(cache.resources[1].key, "pages:alpha");
    }

    #[test]
    fn given_account_switch_and_cached_history_when_restarting_then_no_foreign_row_or_verified_state_leaks() {
        let a = "a".repeat(32);
        let b = "b".repeat(32);
        let from_a = ResourceCatalogCache::from_discovery(
            &a, &[make("worker", "alpha", "failed"),
                make("pages", "beta", "healthy")]).unwrap();
        assert!(from_a.show_stale(&b).is_empty());
        assert!(from_a.show_stale(&a.to_uppercase()).is_empty());
        assert_eq!(from_a.show_stale(&a).len(), 2);
        let from_b = ResourceCatalogCache::from_discovery(
            &b, &[make("worker", "beta", "healthy")]).unwrap();
        assert_eq!(from_b.show_stale(&b).len(), 1);
        assert!(from_b.show_stale(&a).is_empty());
    }

    #[test]
    fn given_legacy_or_corrupt_serialized_catalog_when_restoring_then_missing_identity_is_fail_closed() {
        let parsed: ResourceCatalogCache =
            serde_json::from_str(r#"{"resources":[]}"#).unwrap();
        assert!(parsed.show_stale(&"a".repeat(32)).is_empty());
        assert!(serde_json::from_str::<ResourceCatalogCache>(
            r#"{"account_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","resources":"invalid"}"#)
            .is_err());
        let default: ResourceCatalogCache = serde_json::from_str("{}").unwrap();
        assert!(default.show_stale(&"a".repeat(32)).is_empty());
    }

    #[test]
    fn given_hostile_saved_urls_when_serialized_then_cache_contains_only_canonical_dashboard_urls() {
        let account = "a".repeat(32);
        let cache = ResourceCatalogCache::from_discovery(&account, &[
            make("worker", "script", "healthy"),
            make("pages", "frontend", "failed")
        ]).unwrap();
        let json = serde_json::to_string(&cache).unwrap();
        assert!(!json.contains("evil.example"));
        assert!(!json.contains("token=secret"));
        assert!(!json.contains("old-attempt"));
        assert!(!json.contains("previous live status"));
        assert_eq!(cache.show_stale(&account)[0].url,
            format!("https://dash.cloudflare.com/{account}/workers/services/view/script/production"));
        assert_eq!(cache.show_stale(&account)[1].url,
            format!("https://dash.cloudflare.com/{account}/pages/view/frontend"));
    }

    struct TempCatalogDir(std::path::PathBuf);
    impl TempCatalogDir {
        fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            let id=SEQUENCE.fetch_add(1,Ordering::Relaxed);
            let root=std::env::temp_dir().join(format!(
                "cfvigil-roster-{}-{id}",std::process::id()));
            std::fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn file(&self)->std::path::PathBuf { self.0.join("resource-catalog.json") }
    }
    impl Drop for TempCatalogDir {
        fn drop(&mut self) { let _=std::fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn given_successful_real_storage_roundtrip_when_restarting_then_cached_rows_are_unverified() {
        use crate::storage::{read_or_default, write_atomic, MAX_JSON_BYTES};
        let account="a".repeat(32);
        let other="b".repeat(32);
        let source=ResourceCatalogCache::from_discovery(&account,&[
            make("worker","backend","healthy"),
            make("pages","frontend","failed")
        ]).unwrap();
        let dir=TempCatalogDir::new();
        write_atomic(&dir.file(),&source,MAX_JSON_BYTES).unwrap();
        let loaded=read_or_default::<ResourceCatalogCache>(&dir.file(),MAX_JSON_BYTES).unwrap();
        assert!(!loaded.quarantined);
        let rows=loaded.value.show_stale(&account);
        assert_eq!(rows.len(),2);
        assert!(rows.iter().all(|r|r.status=="unknown"&&r.attempt_id.is_empty()));
        assert!(loaded.value.show_stale(&other).is_empty());
        assert!(!std::fs::read_to_string(dir.file()).unwrap().contains("evil.example"));
    }

    #[test]
    fn given_corrupt_cached_catalog_when_restarting_then_storage_quarantines_without_drawing_old_health() {
        use crate::storage::{read_or_default, MAX_JSON_BYTES};
        let dir=TempCatalogDir::new();
        let broken=b"{\"account_id\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"resources\":[";
        std::fs::write(dir.file(),broken).unwrap();
        let loaded=read_or_default::<ResourceCatalogCache>(&dir.file(),MAX_JSON_BYTES)
            .unwrap();
        assert!(loaded.quarantined);
        assert!(loaded.value.show_stale(&"a".repeat(32)).is_empty());
        assert!(!dir.file().exists());
        let backups=std::fs::read_dir(&dir.0).unwrap()
            .map(|entry|entry.unwrap().path()).collect::<Vec<_>>();
        assert_eq!(backups.len(),1);
        assert!(backups[0].file_name().unwrap().to_string_lossy()
            .contains("resource-catalog.json.corrupt."));
        assert_eq!(std::fs::read(&backups[0]).unwrap(),broken);
    }

}
