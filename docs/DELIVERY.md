# Implementation and release gates

Initial source: `dev`. Release branch: `main`. No change may be fast-forwarded or tagged on the strength of unverified implementation alone.

## Current verification

- Implemented provider rules for Workers Builds and Pages latest attempts in Rust.
- Included deterministic Node.js status tests and Rust unit tests, including negative tests for error-rate thresholds, script attribution, spike transitions and GraphQL outages. Node tests were executed in a local copied source snapshot (6 passed); the Rust tests have not been run.
- GitHub Preflight checks now target standard GitHub-hosted Ubuntu 24.04 runners for source changes; documentation-only commits do not trigger push CI.
- Native KDE GTK Layer Shell and StatusNotifierItem support has been added in source; Tauri Settings Cancel has a settings-only `core:window:allow-hide` permission. Rust compilation, live Cloudflare API acceptance, native KDE tray/Wayland behavior and Windows runtime are **not yet verified**.

## Required before announcing v0.1.0

1. Source CI passes, plus cargo test/check on a suitable Rust/Tauri build host.
2. Confirm token storage under KWallet (Linux), Credential Manager (Windows); credentials never appear in UI payloads/logs.
3. Run Cloudflare integration tests for successful, failed, running, missing-privilege, no-build, and manually deployed resources. Verify that per-project/build deep links open the correct Cloudflare screen.
4. Confirm pagination/rate limiting/partial failures and relative event times. Verify per-Worker `limit: 1` GraphQL aggregates (max 8 names per query) against real metrics; verify account-scoped Zone Read and sampled WAF GraphQL behaviour on Free, including permissions and error thresholds.
5. Verify upward drawer anchored to tray on KDE Wayland and X11, auto-focus on incidents including cold-start notification delivery, right-click menu, settings and history.
6. Verify install/upgrade/removal, signed artifacts, release tag-to-main SHA and immutable distribution checksums if a package is shipped.

## Follow-up scope before 1.0

- Worker invocation error-spike detection and **zone-scoped WAF block-spike detection** exist in source, with conservative thresholds and no Worker-level misattribution. Both require real GraphQL API validation and rate-limit/false-positive testing; WAF block spikes are not labelled confirmed attacks.
- Deduplicated alert transitions, local history retention, notifications and reconciliation after resume/offline.
- Same Apache-2.0 + NOTICE model and tagged GitHub Releases / R2 / signed APT distribution as RepoTicker.
- Complete Mode B inventory, positive/negative matrix, branch coverage, stable/flaky separation and physical OS acceptance at release-candidate stage.

## 2026-10-23 correctness audit (Mode A)

Context: The connector can read/write source files but cannot execute a shell on Alcotester or retrieve push-triggered GitHub Actions runs. The temporary local container has Node.js but no Rust toolchain, and network access to GitHub is unavailable. Consequently, `cargo check` and `cargo test` results remain **unverified**.

Evidence and fixes planned (source-level, no UI or product scope expansion):

1. **Workers enumeration contract:** the Cloudflare Worker Scripts List endpoint exposes a scripts array and a `tags` filter, not `page/per_page`. Use an explicitly non-paginated list read for Workers, keep paginated Pages discovery, and test both result shapes and failures. An unsupported query parameter must never be assumed to enable pagination.
2. **Pages latest attempt:** `is_skipped=true` must not be classified as a successful deployment; the stage can also be `canceled` or `skipped`. Return unknown rather than green for skipped/canceled attempts. Preserve the latest-attempt-first rule (a new active attempt is yellow).
3. **Dashboard navigation:** event history is read from local JSON. Accept only a parsed `https` URL with exact host `dash.cloudflare.com`, no user-info and no alternate port. Disallow deceptive prefix matches; add a negative test matrix.
4. **Event timestamp semantics:** the resource row shows the time of the latest *state change*, not the creation time of a build that may have been running for hours. Use Workers `stopped_on` / `running_on` / `initializing_on` and Pages latest-stage `ended_on` / `started_on` when available, falling back to valid API timestamps. Add deterministic helper tests for each transition.
5. **Validation:** final stable suite = Node.js status tests, JavaScript syntax checks and Rust check/tests on the actual Linux runner. Do not claim passing native checks unless logs and exit status are obtained. No flaky suite is declared yet.

Rollback: each production correction is a separate `fix:` commit on `dev`; documentation changes remain separate `docs:` commits. `main` and release tags remain untouched.


### Audit implementation and evidence (2026-10-23)

- **Completed (source):** Worker Scripts discovery now uses the documented non-paginated REST list; Pages keeps its own paginated REST list. A shape-validation unit test was added.
- **Completed (source):** skipped and canceled Pages deployments never appear healthy; Rust and JavaScript policy tests cover this explicitly. A newly active attempt stays yellow regardless of a previous failure.
- **Completed (source):** browser navigation validates the parsed HTTPS Cloudflare Dashboard origin, literal authority (to reject explicit default ports), absence of user-info and alternate ports. Native history navigation does not fall back after an otherwise valid current-resource browser launch error. Negative tests added.
- **Completed (source):** Workers and Pages row age now comes from the latest build/stage transition timestamp when provided, not just the deployment creation time. Deterministic tests added.
- **Node final gate:** the exact `status.mjs` and `tests/status.test.mjs` content was reconstructed from GitHub and executed with Node.js v22.16.0 in the accessible local container; **6/6 passed**, and `node --check status.mjs` passed. This does not verify other WebView JS files.
- **Native gate: BLOCKED / UNVERIFIED** — neither Rust compiler nor Cargo is installed in the available local container, network access is restricted, there is no connected remote shell on Alcotester, and the installed GitHub connector cannot list push-triggered Actions logs. Therefore no Rust compile/test pass, live Cloudflare contract pass, installer or KDE smoke pass is claimed.
- **Branch evidence:** GitHub comparison `main...dev` returned `ahead=54`, `behind=0` at this audit point; only `dev` has the source changes.

Required next execution on Alcotester (once the source is checked out and system Tauri dependencies are available):

```sh
git switch dev
git pull --ff-only
node --test tests/*.test.mjs
cargo check -p cfvigil-desktop
cargo test -p cfvigil-desktop
```

Inspect Rust compiler diagnostics, correct root causes and rerun only affected checks plus the stable gate before any fast-forward to `main`. Do not ship release artifacts until platform smoke acceptance.

## Current CI strategy — public GitHub-hosted runners

The old container-required Alcotester/self-hosted workflow was superseded.
CFVigil is public; standard GitHub-hosted runner usage is free for public
repositories. Its CI should follow RepoTicker's structure rather than changing
the existing user's self-hosted runner configuration.

- `.github/workflows/checks.yml`: **Preflight** on pushes to `dev`/`main`,
  PRs against those branches (only for temporary owner/agent conflict PRs),
  and manual dispatch.
- `ui` on `ubuntu-24.04`: Node.js 24, JavaScript syntax checks and stable
  deterministic tests.
- `desktop` on `ubuntu-24.04`: stable Rust + Linux Tauri dependencies,
  `cargo check -p cfvigil-desktop` and `cargo test -p cfvigil-desktop`.
- `preflight` requires both jobs to succeed; a skipped or failed job does
  not count as success.
- Docs-only commits are excluded from push-triggered CI; code and documentation
  commits stay separate. No custom commit-status POST or `statuses: write`
  permissions are necessary.
- Standard hosted runners only: **no Alcotester, Alcotester-2 or rootless
  Docker configuration changes**. Do not run the public-repo CI on the user's
  private machines.

### What has been checked versus what remains pending

The workflow has been committed and confirmed present in `dev`; this is not
equivalent to successful execution. Rust compilation, native smoke tests,
Cloudflare API contracts and package installers remain unverified until actual
workflow logs and physical system evidence are available.

The repo's **Apache License 2.0** matches RepoTicker byte for byte; `NOTICE`
provides copyright attribution. Public upstream accepts only structured bug
reports, not arbitrary Issues, feature requests, external code contributions
or additional maintainers. Security vulnerabilities should use private
GitHub advisories; independent forks are allowed under the license.

### Localization gate before the final test campaign

See [ROADMAP_TO_1_0.md](ROADMAP_TO_1_0.md). Implement all other features and
distribution foundations first, then finish **all 20 standard UI locales** and
document their individual contextual, typography, RTL, grammar and meaning
audits. Only **after that milestone** run the exhaustive Mode B QA campaign.
Development-time unit checks and CI remain mandatory during earlier features;
the ordering rule governs final full release acceptance.

### GitHub default-branch activation boundary

GitHub loads repository Issue Forms from the **default branch**. Currently
`main` remains the default branch, while the bug-report template and
`blank_issues_enabled: false` have only been written to `dev`. Thus the
public Bug report chooser is **configured in source but not yet active**.
It becomes active after those policy files reach the default branch, or if
the repository owner explicitly changes its GitHub default branch to `dev`.
Do not silently change `main` or GitHub repository settings to activate it.

Likewise `workflow_dispatch` requires the workflow file on the default
branch; push-triggered Preflight on `dev` is the available CI path until
the owner chooses a default-branch policy or approves integrating the workflow.
Nothing here authorizes a premature app release.

The working branch is `dev`. Create a short-lived branch and PR only if
another agent is already writing to `dev`. Merge, close the temporary PR
and delete its merged branch after the feature. `main` and release tags
require explicit owner approval and passing release gates.

## 2026-10-23 GitHub-hosted Preflight compiler diagnosis

Actual GitHub Actions evidence, from `Preflight` push run [39996928456](https://github.com/morozhenko-apps/cfvigil/actions/runs/39996928456) for commit `37fb82872b87c7c08072eaf8d1325634a835868e`:

- `JavaScript syntax and stable tests` **PASS**; the job log records 6 tests, 6 passed, 0 failed.
- `Install Tauri 2 Linux build dependencies` **PASS**.
- `cargo check -p cfvigil-desktop` **FAIL**, with two compiler diagnostics:
  - `E0597` in `cloudflare.rs`: Pages deployment ID/time borrowed from a JSON `result` dropped at the end of the `match`. Fixed by owning strings before leaving the result scope in source commit `d406634f2a2059b965505fc8e29af33e1b62d2b4`.
  - `E0308` in `main.rs`: persistence `map_err` returned `Result<(), &str>` where `Result<(), String>` was declared. Fixed by returning an owned String in commit `6fb19d669e2f29f353ddcf20976c51e6088f02a7`. The same commit gates Windows-only tray imports on non-Linux platforms.
- Native Rust tests did not run because `cargo check` failed. **Do not count them as passed**.
- At the time of review, GitHub Actions run [40024544307](https://github.com/morozhenko-apps/cfvigil/actions/runs/40024544307), triggered by the second fix, was **pending**. The predecessor run [40024522708](https://github.com/morozhenko-apps/cfvigil/actions/runs/40024522708) was still in progress. No successful Rust check is yet demonstrated.

The connector's general commit-status endpoint was insufficient for GitHub Actions checks, but the GitHub workflow runs collection and job logs can be fetched directly through the approved GitHub connector. Further compiler work should be driven by actual current workflow job diagnostics, not guesses. The documentation-only evidence commit must not start source CI.

## 2026-10-23 — Overlay removal and provider contract fixtures

Source changes on `dev`:
- Removed the visible incident banner/toast from the five-cell compact card. Incidents still blink the Resources button, clear the search filter, focus the affected list row and keep the message in a screen-reader-only live region and button tooltip. File-level DOM contract tests assert the compact layout and absence of the visual overlay.
- Extracted the latest Worker attempt selection into a pure Rust helper. Documented Cloudflare REST response-shaped fixtures cover a newer running build outranking an older serving deployment, a failed build, a subsequent manual deployment, a missing build and an unavailable Workers Builds API.
- Explicitly validate immutable 32-character hexadecimal Worker script tags before placing them in a query string; sort tags for deterministic batch requests. A malformed or missing `result.builds` mapping is no longer interpreted as successful completed deployments.
- Cross-checked public Cloudflare API documentation for `/workers/scripts`, `/builds/builds/latest`, `/workers/scripts/{script_name}/deployments` and `/pages/projects/{name}/deployments`. That is **contract verification against published documentation**, not real-user-token/live-account verification.
- No Cloudflare credentials are provided to the connected tools. A genuine live-account acceptance test and KDE Wayland/X11 tray/click/expand smoke **remain open**. A headless CI compilation is not equivalent to physically verifying the KDE popup.

The preceding green GitHub-hosted Preflight `40024544307` validated the earlier Rust/JS source only. New provider/UI test commits require their own green run before source-level gates can be considered current. CI runs may be cancelled by superseding pushes because `cancel-in-progress` is enabled. Documentation-only edits must not trigger a new run.

**Acceptance pending:** confirm newest source commit's `Preflight` (JS syntax/tests, Rust check/tests), perform read-only Cloudflare API integration checks on a configured account, and verify the popup on actual KDE Wayland and X11. No `main` fast-forward, release or package publication is authorized by this note.

## 2026-10-23: explicit dev CI gate; queue-storm correction

The earlier RepoTicker-inspired `push: branches: [dev, main]` source paths
still started one 3-job Preflight for **every code commit**, making a storm
of redundant queued/cancelled jobs when the agent created small commits.
`cancel-in-progress` cancels older jobs but cannot prevent them being
registered in the Actions queue. This was not compatible with the owner's
requirement to run full tests **once at the end of feature implementation**.

**Current design** in `.github/workflows/checks.yml`:

- `push` on `dev` runs only if `ci/run-preflight.txt` changes. Normal
  Rust/JS/HTML/CSS/config source commits and documentation-only commits
  do **not** launch CI. Explicitly change the marker only once when
  ready to verify the complete feature/fix batch.
- A temporary internal `pull_request` targeting `dev` or `main` runs
  when it changes source/test/workflow files, not for docs-only updates.
- `workflow_dispatch` is available once the workflow reaches the
  repository's default branch (`main` at present).
- One `ubuntu-24.04` GitHub-hosted job executes Node.js 24 syntax and
  stable UI tests, then Rust check and Rust tests with required Tauri
  dependencies. There are no standalone per-job aggregator runners.
  No Alcotester/self-hosted runner involvement or paid hosted runner.
- `concurrency.cancel-in-progress: true` remains as a safeguard for
  superseded *explicit* requests or PRs, not as the primary dedup mechanism.
- `tests/ci-trigger.test.mjs` guards these invariants and runs as part
  of the same stable UI suite.

**Observed queue evidence:** GitHub's Actions API returned 49 historical
runs for the repository; all had `completed` status at audit time and
there were **zero remaining queued/in-progress runs**. The latest
Preflight before this fix, run `40030521303`, was green. The
new explicit-trigger workflow requires its own verification and must
not be claimed successful merely because previous preflight was green.

## 2026-10-23 — Event History milestone (source implementation)

- Source commit `6be5b403a56d8f48ca0601d3c313cd0a0aa3810e`
  introduces immutable per-event IDs and categories.
- The event-window Cloudflare action now resolves by **event ID** and uses
  the URL saved with that specific event, not a possibly new current resource
  or the first matching resource key. Dashboard URL origin checks still apply.
- Added local search (resource ID/name/message) and independent filters for
  incidents/recoveries/updates. Does not change compact card dimensions.
- Old `events.json` records remain readable. The migration assigns
  deterministic legacy IDs and categories in memory, which are persisted the
  next time event history is written. Missing historical URLs stay unavailable,
  rather than being guessed from the current resource.
- Rust unit tests cover two events for one Worker pointing at different
  deployments, legacy record normalization, malicious Dashboard destinations,
  and WAF recovery category. JavaScript tests cover search/filter semantics,
  invalid timestamps and event-specific native invocation.
- Actual CI results for this commit must be checked after the single explicit
  Preflight. This is not native KDE or live Cloudflare integration acceptance.

## 2026-10-23 — Durable incident state across app restarts

Source implementation:
- `incident_store.rs` models a bounded notification deduplication window
  (512 latest IDs), 320 maximum retained active Worker/WAF incidents and an
  account-scoped JSON journal independent of event history. There is no
  token/secret field in this journal.
- `main.rs` loads only active incidents for the configured Cloudflare account,
  marks all restored statuses **unknown**, persists changed incident snapshots
  and identities, and resets the state on account switch. Same-account settings
  updates do not erase active alerts.
- Analytics failures preserve the previous runtime spike even when the
  previous record was already grey; a continuing WAF spike after a restart
  or metrics outage is no longer counted as a second event.
- Unit tests cover restart after active WAF + runtime spikes, no false green,
  duplicate IDs, bounded eviction, later recovery re-arming, account mismatch,
  malicious/duplicate local data and continued WAF episodes.

**Validation status:** changes were committed directly to `dev`. A single
explicit Preflight is required before declaring stable source checks green.
Native KDE startup, actual keyring interaction and live Cloudflare API
availability are not verified by headless unit tests.

## 2026-10-24 — Single-poller scheduling and provider-failure readiness

Implementation summary:
- Only `run_polling` can call `refresh_once`. Settings changes are serialized
  with the active request using the existing Tokio refresh mutex, update the
  OS-keyring token under that lock and wake the same polling task using a
  bounded `tokio::sync::watch` revision. No second refresh worker is spawned.
- Introduced `polling_policy.rs` with deterministic retry cadence. Successful
  core reads use configured polling delay; credentials or Workers/Pages
  discovery failures retry with exponential backoff, capped at 30 minutes.
  A settings revision bypasses sleep/backoff immediately.
- Initial `get_snapshot` and `cfvigil-resources` events now share the
  same readiness payload: `{ready, configured, resources, pending_incident}`.
  The UI no longer assumes every resource event implies fresh verified
  Cloudflare data.
- Credential/keyring or provider failure marks previous resources grey and
  publishes `ready: false` rather than leaving last green statuses visible.
  Same-account settings updates also mark data stale before waking the poller.
  Configured-but-unavailable is distinguished from not-yet-configured.
- Rust tests exercise bounded backoff, coalesced watch revisions and loss
  of verified health on credential/provider errors. JS tests exercise the
  readiness schema and defend against restoring automatic per-save spawns.
- These are source checks; actual Linux tray runtime, OS keyring unlock,
  Cloudflare rate limits, multi-API partial outages and end-to-end network
  recovery require native/live acceptance.

The previous incident-journal Preflight #40037042894 completed green before
these polling changes. A single separate marker-triggered Preflight will be
needed for the new code. Code/config and documentation remain separate
commits. Do not infer successful Rust tests for new source from the old CI run.

## 2026-10-24 — Preflight diagnosis on single-poller milestone

Actual GitHub-hosted CI run [40037684110](https://github.com/morozhenko-apps/cfvigil/actions/runs/40037684110) for `8e0a3fb1d321`: JavaScript syntax/tests and Linux dependencies passed, but `cargo check -p cfvigil-desktop` failed with only `E0277`, `Snapshot: Clone` not satisfied in `main.rs:192`, because `tauri::Emitter::emit` requires `Serialize + Clone` for its event payload. Rust tests were skipped by the failed compiler gate.

Code commit `b9025ae1e9c956c8982378204e2a13f4f67b9841` derives `Clone` on the native `Snapshot` event payload, keeping the readiness snapshot format unchanged. This is a compiler fix, not verification; a single explicit marker-driven CI rerun is required. No release or `main` fast-forward.

## 2026-10-24 — Independent provider failures, per-poll budget and 429 handling

Source changes on `dev`:
- `api_budget.rs` adds one per-refresh allocation (120 outbound calls
  maximum for Cloudflare REST and GraphQL combined) and a no-retry 429
  stop condition. Parses Cloudflare REST `Retry-After` integer seconds and
  clamps cooldown to 60–1800 seconds; falls back to 300 on invalid/missing
  data. The poller preserves the longer of exponential failure backoff and
  provider-advised cooldown. Budget resets only on the next poll, never
  by making a second immediate request.
- `cloudflare.rs` returns a partial `Discovery` instead of requiring
  both independent Worker and Pages catalog reads to succeed. It preserves
  per-build-batch and per-resource error provenance and uses grey status
  on unreadable Worker/Pages results. Bounded resource scans fail closed
  rather than omitting surplus resources silently.
- `analytics.rs` returns independently verified Workers error-spike
  batches; only failed batches remain unverified. `security.rs` returns
  zone-specific WAF verification results; a failed zone no longer hides
  confirmed spikes in other zones, and only a successfully checked zone
  can prove recovery.
- `main.rs` merges stale resources from unavailable catalogs without
  treating missing responses as deletions; a globally incomplete core
  discovery makes the aggregate counters unverified. Previously confirmed
  incidents remain grey during failed metrics reads. Confirmed recoveries
  still record event history and re-arm future distinct incidents.
- Rust unit tests cover local budget exhaustion, Retry-After normalization,
  greyed catalogs, duplicate elimination and WAF zone-specific recovery.
  Existing Cloudflare REST/GraphQL fixture parsing, incident continuity and
  JS snapshot regressions remain in the stable suite.
- Provider and budget changes have not been verified with a real Cloudflare
  token nor on physical KDE/Windows. **Single marker-triggered Preflight
  is pending**, and results must be read from the actual GitHub Actions run.
  No `main` fast-forward, release tag or publication is approved.

Reference: Cloudflare API rate-limit and header documentation
(https://developers.cloudflare.com/fundamentals/api/reference/limits/),
Workers Builds latest endpoint supports at most 20 script tags per call
(https://developers.cloudflare.com/api/resources/workers_builds/methods/get_latest_builds/).

## 2026-10-24 — Desktop launch-at-login milestone

Source commits `c0d09e2e3d224139664e79b676819b8f190853e8`
and `d072ed800677e1505bf65343b9394f87dd3fc0cd` on `dev`
implement **opt-in**, OS-native autostart:
- Tauri 2 `tauri-plugin-autostart` registered once in Rust, supporting
  the official Linux, Windows and macOS platforms; this product's physical
  acceptance focuses on KDE Linux and Windows x64.
- `get_autostart` reads the actual operating-system registration. The
  existing `save_settings` action accepts optional `autostart`; `None`
  never modifies system startup, including when the OS read fails. Native
  Rust is the only layer permitted to change the entry; no plugin command
  permission is granted to JS.
- Settings adds an opt-in checkbox disabled until the read succeeds. Failed
  operating-system updates report a clear partial-save error.
- A pure Rust decision helper prevents a no-op or autostart-only Save from
  invalidating resource health or waking the Cloudflare poller. Account,
  token and interval changes retain their existing refresh semantics.
- JavaScript contract tests cover the opt-in gate and native-only mutation,
  while Rust tests cover whether a polling refresh is necessary.
- GitHub-hosted CI compiles and tests source but cannot prove registration
  works on a real KDE Wayland/X11 login, Windows sign-in, or installed DEB/
  AppImage paths. These are explicit native acceptance gates.

Official API reference: https://v2.tauri.app/plugin/autostart/
No `main` fast-forward, release, or system login registration is performed
by this development commit.
