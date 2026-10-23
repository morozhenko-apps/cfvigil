# Implementation and release gates

Initial source: `dev`. Release branch: `main`. No change may be fast-forwarded or tagged on the strength of unverified implementation alone.

## Current verification

- Implemented provider rules for Workers Builds and Pages latest attempts in Rust.
- Included deterministic Node.js status tests and Rust unit tests, including negative tests for error-rate thresholds, script attribution, spike transitions and GraphQL outages. Node tests were executed in a local copied source snapshot (6 passed); the Rust tests have not been run.
- GitHub checks workflow runs on self-hosted Linux runners for code changes, excluding documentation-only changes.
- Native KDE GTK Layer Shell and StatusNotifierItem support has been added in source, but Rust compilation, live Cloudflare API acceptance, native KDE tray/Wayland behavior and Windows runtime are **not yet verified**.

## Required before announcing v0.1.0

1. Source CI passes, plus cargo test/check on a suitable Rust/Tauri build host.
2. Confirm token storage under KWallet (Linux), Credential Manager (Windows); credentials never appear in UI payloads/logs.
3. Run Cloudflare integration tests for successful, failed, running, missing-privilege, no-build, and manually deployed resources. Verify that per-project/build deep links open the correct Cloudflare screen.
4. Confirm pagination/rate limiting/partial failures and relative event times. Verify account-scoped Zone Read and sampled WAF GraphQL behaviour on Free, including permissions and error thresholds.
5. Verify upward drawer anchored to tray on KDE Wayland and X11, auto-focus on incidents, right-click menu, settings and history.
6. Verify install/upgrade/removal, signed artifacts, release tag-to-main SHA and immutable distribution checksums if a package is shipped.

## Follow-up scope before 1.0

- Worker invocation error-spike detection and **zone-scoped WAF block-spike detection** exist in source, with conservative thresholds and no Worker-level misattribution. Both require real GraphQL API validation and rate-limit/false-positive testing; WAF block spikes are not labelled confirmed attacks.
- Deduplicated alert transitions, local history retention, notifications and reconciliation after resume/offline.
- Same Apache-2.0 + NOTICE model and tagged GitHub Releases / R2 / signed APT distribution as RepoTicker.
- Complete Mode B inventory, positive/negative matrix, branch coverage, stable/flaky separation and physical OS acceptance at release-candidate stage.
