# Implementation and release gates

## Current checkpoint (2026-10-25)

This owner preserves implementation/CI chronology and release requirements.
Earlier "Rust not run", "native not verified" and initial verification counts
below are dated checkpoints, not a current absence of source or evidence.
The [native-machine owner](NATIVE_MACHINE_GATE.md) records scoped v5 KDE
geometry/Settings PASS and an overall gate still IN PROGRESS; the
[connected Cloudflare owner](NATIVE_CLOUDFLARE_SMOKE.md) records a real partial
Worker connection and corrected Pages-diagnostic/polling source awaiting
installed retest. Do not convert that scope into full machine/live acceptance.
Pages Read is now owner-verified; the latest observed HTTP 400 is a request/
pagination defect, not a confirmed missing-scope denial. The bounded 20-item
query and sanitized numeric-code correction still await native retest.

All 20 UI dictionaries exist as drafts; [per-locale editorial/context and
physical acceptance](i18n/CONTEXT_AUDIT.md) remain pending. Follow the
[roadmap](ROADMAP_TO_1_0.md): native-machine acceptance precedes broad coverage
expansion/Mode B. CI/packaging markers and exact-SHA runs are separate from
physical acceptance and owner-authorized publication. No new gate is closed
by housekeeping; distribution remains source-only per its owning contract.
The complete [former root README](#imported-root-readme-before-october-25-housekeeping)
is retained at the end as dated provenance, not another active status page.


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

## 2026-10-24 — Native card keyboard/focus and flyout usability

Implemented in `dev`:
- Added `resource-navigation.mjs`, a deterministic, DOM-free keyboard
  navigation policy for Up/Down/Home/End against the *visible* resource list.
  Native Tab, Shift+Tab, Enter and Space behavior is preserved.
- Resource action flyout is hosted by the top-level card instead of the
  overflow-clipped scroll region, with bounded placement and proper
  menu/menuitem semantics. Opening moves focus to the action, Escape returns
  focus to its trigger, and outside clicks dismiss the flyout.
- Resource options expose `aria-haspopup`, `aria-expanded`, resource
  rows are listitems, and lamp statuses are exposed through accessible
  descriptions. Drawer expansion uses `aria-expanded` and `aria-controls`.
- On resource updates, focused actions are restored by stable resource key.
  With a pending incident, opening the drawer focuses the incident action;
  otherwise it focuses search. Closing the drawer returns to Resources.
  Concurrent resize commands are suppressed while the native resize runs.
- Kept the compact card at three metrics + two actions and unchanged
  390×122/390×510 target sizes. All incident text remains non-overlay.
- Added tests for focus/navigation contract and keyboard boundary cases
  alongside existing card/layout contract tests.

The last completed preflight for the *previous* launch-at-login block,
`40085602559`, passed. The new keyboard block requires its own single
explicit Preflight and real-device KDE Wayland/X11 + Windows confirmation.
This source work does not imply native interactive acceptance or a release.

## 2026-10-24 — Private crash-safe local state and damage recovery

Completed in source on `dev`:
- New `storage.rs` implements bounded (4 MiB) typed reads, atomic
  file replacement via unique same-directory temporary files, file flush,
  Unix directory sync and `0600` permissions on new and previously existing
  JSON files. Cloudflare token handling is unchanged and remains exclusively
  in the OS keychain.
- Malformed or oversized state is preserved unmodified as a unique
  `<filename>.corrupt.<pid>.<timestamp>` sibling before defaults are
  returned; files that cannot be read or quarantined, and symlinks, cause
  explicit failure rather than silent empty-state replacement.
- Startup restoration of `settings.json`, `incident-state.json` and
  `events.json` all uses that same checked storage layer. History is
  truncated to 500 entries at load, before legacy event IDs are normalized.
  Persistence errors of event history now emit a redacted diagnostic.
- A `storage_warning` flag on native snapshots identifies recovery at
  startup. The compact card exposes the warning through the Resources
  tooltip and screen-reader status region; Settings describes where to
  find the preserved backup. There is no additional compact-card overlay.
- Rust tests cover exact-byte quarantine retention, valid/missing files,
  oversized reads and writes, second atomic replacement, restrictive Unix
  permissions, directory misuse and symlink rejection. Node contract tests
  cover integration into all three records and the UI warning path.
- Source-level CI is not a substitute for physical interrupted-write testing
  or Windows filesystem checks. Recovery does not imply automatic repair of
  lost data; the user must inspect the preserved backup if needed.
- Prior keyboard accessibility Preflight
  [#40086493690](https://github.com/morozhenko-apps/cfvigil/actions/runs/40086493690)
  finished green before this milestone. A new single marker-driven Preflight
  is still required for the storage work. No main merge or release.

## 2026-10-24 — 0.6: deliberately gated Linux distribution candidate

- Previous storage durability Preflight
  [#40087462852](https://github.com/morozhenko-apps/cfvigil/actions/runs/40087462852)
  finished successfully.
- Added `.github/workflows/package-candidate.yml`, entirely separate from
  `checks.yml`. Only an explicit changed `ci/run-package-candidate.txt`
  on `dev` starts one Linux GitHub-hosted build. It uses a single
  job and `permissions: contents: read`. There is **no** release/tag,
  R2/APT uploader, signing key, or GitHub Release creation path in it.
- `scripts/candidate-version.mjs` rejects mismatched desktop Rust/Tauri
  versions, non-semver identifiers, bundle identity or license drift, and
  missing source attribution/icons. In-tree deterministic Node tests exercise
  positive and negative cases.
- Linux bundling now explicitly requires `libgtk-layer-shell0`, adapted
  from RepoTicker's checked desktop packaging convention. The candidate
  installs the Rust Tauri 2 CLI on `ubuntu-24.04`, checks Rust/JS suites
  and attempts both amd64 DEB and AppImage packages.
- `scripts/candidate-assets.mjs` inspects the actual bundle output, fails
  on missing/duplicate files, wrong Debian package/version/architecture,
  missing GTK Layer Shell runtime dependency, invalid headers/oversize
  artifacts, and computes streaming SHA-256 digests plus manifest. Output
  is uploaded to a 7-day GitHub Actions artifact, not a public release.
  `Cargo.lock` is preserved as a separate review artifact when generated.
- **Not yet accepted:** the first source trigger occurred before its
  artifact validator was committed; that run is not candidate evidence.
  The final candidate marker must trigger verification at the complete
  SHA and supersede/cancel earlier in-progress attempts. No source
  success should be claimed until the resulting run passes.
- **Production blocker:** CFVigil `Cargo.lock` is not tracked in Git
  yet. A successfully generated runner artifact is a review input, not
  an automatic change to the branch. Signed APT, Windows x64 NSIS,
  isolated R2 upload, git tag/immutable main admission, physical
  install/upgrade/removal and live Cloudflare acceptance remain distinct
  delivery gates. Never reuse RepoTicker's signing credentials.

The release distribution code deliberately follows RepoTicker's
nonpublishing build-before-publish separation, without silently importing
RepoTicker's publishing infrastructure. No main merge or tag is authorized.

## 2026-10-24 — Linux candidate validation: Tauri staging symlinks

The complete-source candidate [#40091322624](https://github.com/morozhenko-apps/cfvigil/actions/runs/40091322624)
finished with a failure only in `Check Debian identity, dependencies and
immutable checksums`. Source tests, Rust build and Tauri CLI installation
passed, and the bundler produced:
- `CFVigil_0.1.0_amd64.deb` (6.63 MiB).
- `CFVigil_0.1.0_amd64.AppImage` (81.43 MiB).

The actual error was `Unsafe symlink in bundle output`: the candidate
validator descended into AppImage's *intermediate* staging tree, which
contains legitimate symlinks, despite only needing to inspect final bundles.
This was a validation scope bug, not evidence of an unsafe final AppImage.

Fixed the root cause: inspect **only immediate final files** inside the
Tauri `bundle/deb` and `bundle/appimage` directories. Require the
exact versioned Tauri filenames and reject duplicate/wrong-version bundles,
symlinked output directories and symlinked/nonregular **final packages**.
Never recursively inspect unrelated AppDir staging files. Added deterministic
tests for a legitimate staging symlink and a malicious final-package symlink.
The successful SHA-256/DEB metadata/artifact upload is still unverified
until the next **single marker-driven** candidate build completes.

The prior incomplete candidate [#40090926586](https://github.com/morozhenko-apps/cfvigil/actions/runs/40090926586)
was cancelled by workflow concurrency. No package was published, no
`main` fast-forward or tag was created.

## 2026-10-24 — Candidate #40092833835, Debian package name mismatch

GitHub-hosted [Linux candidate #40092833835](https://github.com/morozhenko-apps/cfvigil/actions/runs/40092833835)
at `1a6275fc30bb` completed with failure after Tauri successfully
built the application, 6.63 MiB DEB and 81.43 MiB AppImage. Node
and Rust tests passed. The failing step was final Debian artifact
verification, with `Unexpected Debian package identity/version/architecture`.
Checksums and artifact uploads did not execute.

Root-cause hypothesis verified against Tauri bundler source: its Debian
control `Package` field uses `heck::AsKebabCase(settings.product_name())`,
so the brand `CFVigil` corresponds to Debian package identity
`cf-vigil`, not `cfvigil` (and not the Rust binary
`cfvigil-desktop`). The validator was incorrectly enforcing
`cfvigil`. Updated the exact expected metadata and added explicit
sanitized field values to future mismatch errors; added a regression
test rejecting the legacy spelling. The actual DEB control values
were not printed by that failed run; the next candidate run must
confirm them before claiming the mismatch fixed.

There were no published packages, release tags, or changes to
`main`. The workflow must run once more only after code and docs
are complete; physical installation still requires separate acceptance.

## 2026-10-24 — Locked Rust deps and unsigned Windows candidate gate

- Linux [candidate #40095935553](https://github.com/morozhenko-apps/cfvigil/actions/runs/40095935553)
  **passed** JavaScript/Rust tests, built both DEB and AppImage, validated
  Debian identity and SHA-256 checksums, and uploaded unsigned artifacts.
  This proves Linux CI packaging, not physical installed runtime.
- Imported Cargo-generated `Cargo.lock` from the **same successful**
  Linux job's review artifact (138,030 bytes; Git blob
  `6c3b4457b4f6ee7c5fe01ef9733f579f7243d3be`), with exact
  byte-level blob-SHA comparison against the uploaded original. It is now
  tracked in `dev`, not automatically regenerated during release.
  Linux Preflight, Linux packaging and Windows packaging use `--locked`
  Rust checks. Generation and git integration are distinct from docs.
- Added `windows-candidate.yml`, an isolated `windows-2025` workflow
  activated only by changing `ci/run-windows-candidate.txt` on dev.
  It runs JavaScript/Rust tests, creates native ICO from committed PNG,
  builds unsigned x64 NSIS (`installMode=currentUser`), validates the
  installer PE structure, filename and SHA-256, and uploads a short-lived
  review artifact. GitHub Actions has only contents:read permissions and
  no publication credentials or release step.
- Tauri-build requires a Windows ICO already present during cargo check;
  thus native ICO generation intentionally precedes Windows Rust tests.
  Platform-generated icons stay untracked, and there is a Windows-only
  Tauri configuration overlay. This does not modify the existing Linux
  package definition.
- First Windows candidate
  [#40097647029](https://github.com/morozhenko-apps/cfvigil/actions/runs/40097647029)
  **failed before compilation** because existing `tests/ci-trigger.test.mjs`
  assumed LF YAML newlines, while Windows Git checkout provided CRLF.
  Root cause addressed by newline normalization and updating test
  expectations for Cargo `--locked`. The first run is NOT evidence of
  Windows app compilation or installer success.
- The next explicit final-source Windows candidate run is required to
  confirm the ICO, Windows build, NSIS packaging, checksum and artifact
  upload steps. It is not automatically a signed or physically tested
  release. `main` and release tags remain untouched.

## 2026-10-24 — Windows candidate #40098231423: CRLF-sensitive Cargo.lock guard

[Windows candidate #40098231423](https://github.com/morozhenko-apps/cfvigil/actions/runs/40098231423)
at `e59efc9db60c` failed in **Validate source and stable tests** with
`Invalid Cargo.lock header`. The Windows runner successfully checked out
the repository and installed Node.js/Rust; no Tauri CLI installation, icon
generation, Windows Rust compilation, NSIS packaging, checksum verification
or artifact upload ran.

The committed `Cargo.lock` starts with the correct Cargo-generated
header and `version = 4`, but Git checkout on Windows may use CRLF
newlines. The source validator previously required an LF-only prefix.
Corrected the guard to match both LF and CRLF while still requiring
Cargo's exact two-line generated-file header and version 4. The guard does
not modify the lockfile. Added explicit positive and negative Node tests
for LF/CRLF and tampered headers.

A fresh Windows candidate is required before claiming an NSIS build
or Windows Rust test pass. This is strictly a nonpublishing candidate;
there is no Windows installer acceptance, release tag or `main` change.

## 2026-10-24 — Binary retention policy: Cloudflare R2 only

Owner decision: after published files are uploaded and verified in
Cloudflare R2, no generated installers, checksums or APT archives may
remain as GitHub-hosted binary assets. GitHub continues hosting the
source repository, CI logs/run history, source tags and optional text-only
GitHub Releases; it is *not* a second download mirror.

Changes now in `dev`:
- Linux and Windows **candidate** workflow upload retention shortened
  from seven days to **one day** (GitHub's minimum). This is not
  retroactive to artifacts of previous workflow runs. Candidate archives
  are intentionally retained briefly for physical testing because there
  is not yet a CFVigil R2 publisher.
- Removed the redundant dedicated `Cargo.lock` review upload; the exact
  lockfile is already committed and reviewed through source control.
- Added `scripts/cleanup-published-artifacts.mjs`. Its fail-closed
  release-only API deletion helper requires a publisher-generated
  receipt containing two exact CFVigil release artifact names, source
  run IDs, commit SHA, scoped public R2 URLs, and matching original vs
  public readback SHA-256 and sizes. Deletes only those source-run
  artifacts using GitHub Actions REST DELETE (Actions:write); re-lists
  to confirm deletion. Safe to retry if already cleaned. Cannot delete
  unrelated GitHub run artifacts or RepoTicker packages.
- Added deterministic tests of invalid/missing publication receipts,
  unrelated artifacts, mismatched source SHA, successful deletion and
  one-day retention of current development candidates.

**Not integrated into the production pipeline yet:** no CFVigil R2
publication or verified receipt generator exists, so the cleanup helper
is never automatically invoked. The eventual tag/main/release publisher
MUST wire it as the final gated action after R2 public GET hash checks,
with `actions:write` scoped to cleanup and a separate last verification
that no release artifacts remain on GitHub. It must not attach binary
assets to a GitHub Release. No signing key, R2 token, release or deletion
of previous test artifacts was performed in this development change.

Reference: https://docs.github.com/en/rest/actions/artifacts

## 2026-10-24 — Source implementation: separate signed CFVigil R2 release

The previous R2 artifact retention [Preflight #40103001805](https://github.com/morozhenko-apps/cfvigil/actions/runs/40103001805) completed with success, including stable JS and Rust checks. Both Linux and Windows unsigned candidates had previously passed standalone CI (Linux #40095935553, Windows #40099983469).

Implemented on `dev` without live publication:
- `infra/download-worker`: CFVigil-specific read-only GET/HEAD download Worker, strictly whitelisted release/APT keys, no mutation endpoints, separate hostname path `/dist/CFVigil/*`, binding to dedicated bucket `cfvigil-dist`. RepoTicker Worker and bucket are untouched.
- `scripts/apt_repository.py`: offline signed CFVigil APT metadata (Package `cf-vigil`, pool `pool/main/c/cf-vigil/`, Packages, Packages.gz, SHA256 by-hash, InRelease and independent gpgv verification). `verify_signing_key.py` rejects full certification primary material and enforces a dedicated signing subkey.
- `scripts/r2_release.py`: validates exact Linux/Windows candidate manifest names, checksums, source commit and version, and signed APT pool; uploads immutable objects only under `dist/CFVigil/`; verifies R2 bytes and public GET size/SHA256; updates mutable APT indexes then InRelease last. Only emits a verified receipt after every required readback. No R2 credentials are present in source.
- `.github/workflows/protected-release.yml`: tag-only version gate requiring exact `main` SHA and committed Cargo.lock; Linux/Windows hosted native builds, independent signed APT, `cfvigil-release` protected environment and per-tag `CFVIGIL_RELEASE_APPROVED_TAG` owner variable. A separate `actions:write` job deletes only matching temporary GitHub release binaries after receipt, then a text-only GitHub Release publishes R2 links (no attachments). Candidates remain separate marker-triggered, read-only one-day builds.
- Automated Node tests check public Worker whitelists/methods and protected workflow policies. Offline Python unit tests cover release source provenance, tampering, signed APT pool, symlink injection, and public readback without any Cloudflare request.

**No deployment or approval claimed.** The dedicated bucket, Worker route, signing subkey, protected environment, scoped S3 token and WAF exception still require manual provision. The new release workflow lives only on `dev`; source validation CI does NOT exercise real R2, key import, signing or Windows installer runtime. Real deployment and tag/main integration remain blocked until post-localization acceptance and explicit owner authorization. Setup and precise limitations: `docs/DISTRIBUTION.md`.

### Source Preflight failure and corrections

[Preflight #40105010077](https://github.com/morozhenko-apps/cfvigil/actions/runs/40105010077)
failed at the Node test gate (46 passed, one failed). The Worker originally
accepted an incorrectly versioned package filename under another release
directory, e.g. `releases/v1.2.3/linux/cfvigil_1.2.4_amd64.deb`.
Corrected this at the route-mapping root by extracting the path version
once and comparing it against exact allowed filenames for that version.
Regression tests cover mismatched DEB, AppImage and Windows checksums.

Source review also exposed malformed single-brace GitHub Actions expressions
in the new, unexecuted protected-release YAML. Converted all expressions to
the required `${{ context.value }}` syntax and added a Node guard rejecting
single-brace variants. These fixes were committed as source/test changes
before requesting the next single comprehensive Preflight. No release
jobs, Cloudflare network calls, secrets or user environment were changed.

## 2026-10-24: 0.7 English catalog and feature-freeze i18n guard

Source changes on `dev`: localized desktop HTML and dynamic UI messages,
English baseline dictionary, native Rust error/event constants, context-safe
relative-time/date/plural helpers, RTL-name isolation, key parity and
interpolation diagnostics, and regression tests. Existing event strings,
classification, resource severity and deduplication remain unchanged.
Native OS menu/window title localization is not enabled while only English
is approved. **Source commit exists; verification is pending an explicit
single Preflight run.** Cloudflare live API and KDE/Windows physical smoke
are not claimed. Next feature stage remains 20 locale translations and
editorial/RTL/context QA.

### 0.7 initial Preflight #40112322932: failed, one test harness defect

The explicit marker run compiled JavaScript, passed `node
scripts/check-i18n.mjs` and completed 55 of 56 JavaScript tests. The single
failure was `tests/storage-contract.test.mjs`: a new catalog check called
an undefined `read(...)` helper instead of the existing `source(...)`.
Correction committed separately in the test file. Python and Rust steps
were skipped by fail-fast, **not failed**. One follow-up Preflight is required;
do not label Rust compilation or source acceptance successful yet.

## 2026-10-24: 0.8 partial locale wiring

The initial RU and pt-PT draft dictionaries, independent native locale
persistence, translated native tray/menu/window labels and cross-WebView
updates have been integrated on `dev`. The first 0.8 marker failed safely in
`check-i18n` because the pt-PT plural rule includes `many`, which was
initially missing from the four plural messages. The catalog and tests have
been corrected; follow-up CI validation is not yet recorded here.

Do not label the feature milestone complete: 17 non-English dictionaries,
full native error presentation localization, 20 contextual audit records,
physical KDE/Windows verification and final Mode B testing remain.

## 2026-10-24: 0.8 second translation batch, source scope

Added complete Spanish, German and French locale drafts, native tray/menu labels
and RTL-safe localized ARIA status words; corrected the async locale bootstrap
race and translated history search. The 20-locale milestone remains partial:
6 source-ready language catalogs including English, 14 missing dictionaries.
New stable JavaScript source and negative race tests have been committed;
Preflight verification status will be recorded after the explicit final gate.
No change to `main`, R2 deployment, tagging or the read-only Cloudflare API.

## 2026-10-24: 0.8 third draft locale batch

Added Italian, Polish, Dutch, and Swedish locale catalogs and their native
menu/title counterparts on `dev`. All 91 keys per locale were authored;
source checks enforce placeholder and locale-specific plural parity.
New tests also assert that JS active languages match Rust supported codes.
An independently detected test-regex syntax defect and Polish plural wording
were corrected before the acceptance CI marker. Stable CI acceptance is
still pending after this batch. No release, tags or `main` changes.

## 2026-10-24: 0.8 source catalog completeness checkpoint

The remaining ten full-key dictionaries have been committed, and all 20
locales were activated in the UI and native tray/window mapping. A direct
locale-by-locale structural audit over GitHub source found 91 keys per locale,
matching interpolation placeholders and each locale's CLDR plural categories.
This is **source-level evidence only**. Native-backend diagnostic and
installation/help wording still need localization review; individual
editorial/context and RTL/font acceptance is pending.

The next actions are one stable source Preflight; then exact-SHA unapproved
Linux and Windows candidate builds, then real Ubuntu KDE and Windows
installation checks with recorded evidence **before** adding a broad test
matrix or entering Mode B. See `docs/NATIVE_MACHINE_GATE.md`.

## 2026-10-24: KDE compact geometry and Settings visual smoke accepted

The owner confirmed PASS for the compact card gap fix, contrasting Settings
panel and retained window position after installing candidate
#40137438889 on Alcotester. Native KDE geometry is considered stable for
these three scenarios; prior failing observations remain in the machine
gate's history. Do not mark the entire native-machine gate or Mode B
accepted: Windows and Cloudflare/locale/security scenarios are outstanding.

## 2026-10-25: Partial Cloudflare scope and polling correction

After the first live connection, verified Workers appeared but Pages were
absent without a visible reason. Source corrections expose redacted
per-provider HTTP errors, prevent missing Pages permission from hiding
verified Worker counters, and prevent exponential polling backoff on
one-provider partial failures. Static UI text selection is disabled so
native card/Settings dragging works; actual editable inputs remain editable.
No provider writes or release scope changes. Full CI/candidate verification
and installed KDE retest pending. Mode B not started.

## 2026-10-25: Pages catalog HTTP 400 diagnostic correction

The user verified `Cloudflare Pages — Read` was already granted but the
installed app reported `Pages: Cloudflare HTTP 400`. CFVigil's Pages
pagination used `per_page=100` while the official API example uses 20.
The provider client now requests 20 per page and retains safe pagination
bounds. Provider warnings no longer conflate HTTP 400 malformed requests
with HTTP 403 token-scope denials; Cloudflare numeric error codes are
exposed via bounded/sanitized responses. Await CI and real KDE installed
candidate retest before marking Pages discovery PASS. Mode B blocked.

---

## Imported root README (before October 25 housekeeping)

Complete former root text, retained as historical source/evidence provenance.
Only relative links, heading depth and Markdown whitespace were normalized.
Initial pending/unimplemented statements are dated; use the current checkpoint
above and scoped native/live owners before continuing work.

## CFVigil

Free, local-first desktop monitoring for Cloudflare Workers and Pages.

**Development status:** initial v0.1.0 source implementation on `dev`, not a published or physically accepted installer. The `main` branch is the initial repository baseline.

Workspace `Cargo.lock` is committed for reproducible Rust dependency
resolution. Separate opt-in workflows build unsigned Linux DEB/AppImage and
Windows x64 NSIS **candidates only**; neither publishes a release.

### What the current source implements

- Minimal system tray card inspired by RepoTicker: no product title, three counters (Healthy, Deploying, Issues), then **Resources** and **Hide** buttons.
- One combined, scrollable list of Workers and Pages. It grows upward; search appears between the list and the compact card.
- Row layout: resource name, relative event time, state lamp, and `⋯` menu for opening the Cloudflare Dashboard.
- Green: latest Cloudflare build/deployment succeeded; yellow: a newer attempt is running; red: latest attempt failed. Grey means unknown. A new running attempt takes precedence over a failed previous attempt.
- Automatic resource discovery via Cloudflare REST APIs and a configurable polling interval.
- Native API-token handling with OS keychain, **account-scoped durable incident deduplication across restarts**, **searchable/filterable local event history with exact event-specific links**, deployment/Worker-error/WAF-block notifications, and direct browser links to specific builds, Pages deployment details or zone Security Events. Pending incident focus is retained by Rust until the user opens the resource drawer.

#### Known limitations of this initial implementation

- Worker execution error-spike detection is implemented through **read-only Cloudflare GraphQL Analytics**, with per-script aggregate queries (Cloudflare's documented `limit: 1`, `scriptName` filter), batched up to 8 Workers per GraphQL request. It compares a recent 10-minute window against the preceding 30-minute baseline with a 5-minute ingestion buffer, and requires at least 100 recent requests, 10 errors, and a substantial increase in error rate. It has not yet been validated against a live account.
- **Zone-scoped WAF block-spike detection is implemented in source.** It queries Cloudflare Security Events aggregate counts, compares the latest 10-minute interval to a 30-minute baseline, and adds temporary zone alerts to the unified resource list. This feature has not been validated against a live Cloudflare account. A zone's WAF alert is never misattributed to a Worker.
- A previous failed build followed by a newer running build is yellow. A currently serving deployment is not used as proof that an application is healthy.
- Linux KDE Wayland anchoring now has a native GTK Layer Shell implementation and KSNI tray integration adapted from RepoTicker, but has **not** been physically tested on KDE; fallback window repositioning can be rejected by unsupported Wayland compositors.
- No signed APT, DEB/AppImage/Windows release workflow, production icons, autostart configuration or localized interface has been accepted for CFVigil.
- Rust application compilation, Cloudflare live-account API responses and installers have not yet been verified.
- An account ID and a **user-scoped** API Token with Workers Scripts Read, Workers CI Read, Pages Read, Account Analytics Read, Zone Read and Zone Analytics Read are required. Availability of APIs can vary by token and account.
- The source currently supports one configured Cloudflare account at a time.

### Localization readiness

The desktop development branch contains **all 20 target-language UI dictionaries** as drafts. No language is considered editorially or physically accepted yet. Its user interface uses semantic
keys in `apps/desktop/ui/locales/en.mjs` with native Rust diagnostic strings
centralized in `native_messages.rs`. Validation for keys, interpolation,
plural rules, date formatting and RTL direction is available via:

```sh
node scripts/check-i18n.mjs
```

The 20-language product milestone and full per-language context audit are
planned **after feature freeze**; the presence of locale codes is not a
claim that the additional dictionaries are implemented.

### Development

Prerequisites: Rust toolchain, Tauri 2 OS dependencies and Node.js 22+.

```sh
node --test tests/*.test.mjs
cargo test -p cfvigil-desktop
cargo run -p cfvigil-desktop
```

For Ubuntu KDE development, install Tauri's WebKitGTK and GTK dependencies before building. No published installers are available yet.

### Principles

- No GitHub integration; Cloudflare is the authoritative provider for Cloudflare builds and deployments.
- Never attempt to infer app uptime by requesting the deployed application URL.
- Missing API permissions, stale data and unknown outcomes must not be shown as green.
- No destructive Cloudflare API requests. Native code owns the API token, network requests and browser destination validation.
- No hosted backend or paid tier.

### Project and contribution policy

[Product contract](../docs/PRODUCT.md) · [Roadmap to 1.0.0](../docs/ROADMAP_TO_1_0.md) ·
[Release/testing gates](../docs/DELIVERY.md) · [Working rules](../AGENTS.md)

- Open source, free, Apache License 2.0 (identical to RepoTicker), with [NOTICE](../NOTICE).
- **Working branch: `dev`.** `main` is for owner-approved stable integrations/releases.
- One serialized background poller wakes on settings changes, marks stale/locked-account data unknown, and backs off after core Cloudflare API failures.
- Optional OS-native launch at sign-in is available through Settings, off by default; it does not trigger Cloudflare polling when only that setting is changed.
- Keyboard-accessible resource drawer and menus, stable focus during polling, and resource-specific incident focus.
- Local settings, incident journal and event history use private atomic JSON writes; corrupted files are backed up rather than silently overwritten.
- Explicit, nonpublishing Linux amd64 DEB/AppImage candidate workflow with source/version checks, Debian metadata validation, SHA-256 sidecars and short-lived GitHub Actions artifacts; no release or R2 upload.
- Release distribution policy: **binary packages and checksums live only in CFVigil's isolated Cloudflare R2 namespace**. GitHub Actions artifacts expire after one day in candidate jobs and must be deleted immediately after a later release's verified R2 public readback. GitHub Releases, if used, provide text/links only, never binary attachments.
- Partial Workers/Pages/Analytics/WAF outages preserve verified results and show unavailable data as unknown; one shared 120-call budget per polling cycle respects REST 429 Retry-After. No overlapping per-save refreshes.
- CI uses one **GitHub-hosted Ubuntu 24.04** job. Normal `dev` code/docs commits **do not launch GitHub Actions**. After a completed feature or fix, update [`ci/run-preflight.txt`](../ci/run-preflight.txt) once to request JavaScript and Rust checks on the resulting head; temporary PRs are checked automatically for code changes. This avoids repeated queued jobs. CI is not a CFVigil product integration.
- Dedicated `cfvigil-dist` R2 bucket, read-only CFVigil download Worker and owner-protected main-tag publication workflow are implemented **in source only**. No live bucket, signing credentials, route, R2 upload or GitHub Release was created. See [Distribution setup and safety gates](../docs/DISTRIBUTION.md).
- **20 UI languages are planned** as the last functionality milestone, after all other MVP features and before the final end-to-end QA campaign; the required locale set and per-language contextual audit are in the roadmap.
- Upstream is designed for **structured bug reports only**; feature requests and third-party PRs are not accepted, while independent forks are permitted. The [Bug report form](../.github/ISSUE_TEMPLATE/bug_report.yml) is prepared on `dev` but only becomes visible in GitHub's Issues chooser once it reaches the repository's default branch (`main` at present). See [CONTRIBUTING.md](../CONTRIBUTING.md).

© 2026 Stanislav Moseyko (morozhenko).

## 2026-10-25: live Pages error 8000024, list options correction

The owner provided real evidence `HTTP 400 (code 8000024)` after the
reduced-page-size candidate. The code maps to rejected list options.
The new Pages client first fetches the project list without any pagination
query, follows advertised additional pages with `page` only, and fetches
deployment lists without optional list arguments. It selects the latest
deployment from parsed timestamps. No further read scopes requested,
no Cloudflare mutations, no claimed native PASS until retest.

## 2026-10-25: real KDE Pages catalog smoke — PASS

The owner installed Linux candidate #40185446460 and confirmed Pages
projects appeared in the resource list without Cloudflare error 8000024.
This resolves the observed missing Pages catalog failure and validates
the unpaged first catalog request against the real Cloudflare account.
The screenshot shows 26 healthy resources, one failed and no actively
deploying entries; those are state totals, not build counts. Follow-up
scenarios (build transitions/latest timestamp, deep-links, history,
persistence, locales, Windows) remain incomplete. No Mode B acceptance.

## 2026-10-25: Pages failed-deployment navigation — PASS

The installed candidate's red `bannermatch` row opens the latest failed
deployment, as explicitly confirmed by the owner on Alcotester. This
validates correct resource-specific failed-attempt navigation for this
project. It is not evidence of a newly observed state transition or
incident persistence, which remain to be checked before Mode B.

## 2026-10-25: Evidence update — History smoke PASS, icon candidate pending

KDE screenshot and explicit owner feedback confirm History filtering and
event presentation (4 of 51 entries on "bann"). Full restart persistence
and dynamic incident transitions remain unverified. The icon normalization
assets were regenerated in commit 6706990 but not yet installed: a new
unsigned DEB and physical app-menu verification are required. Stable
Cloudflare provider behavior is unchanged, Mode B remains blocked.

## 2026-10-25: latest KDE smoke acceptance and Windows handoff

The owner confirmed post-install normal-sized native tray shield and
successful persistence of Cloudflare connection, list and History
through exit/relaunch on Alcotester. The old KDE application-launcher
icon remains an OPEN issue, not a source-verified failure of image import.
Linux candidate and Preflight both SUCCESS, Windows NSIS candidate
#40224188456 SUCCESS on GitHub-hosted Windows 2025. Windows native
installation and manual smoke still pending; the package is unsigned.
See [Windows native smoke](WINDOWS_NATIVE_SMOKE.md). No broad Mode B.

## 2026-10-25: first Windows runtime findings and corrective source

Owner screenshots after a successful unsigned NSIS install: Windows
card opens and reads Cloudflare counts (26/0/1), tray/application icons
PASS. The current Windows native gate is FAIL due to downward geometry
drift, lost restart position, broken tray hide/expand state, slow unlabeled
Settings Apply, token-editor saved-state ambiguity, and release console.
A platform-isolated Windows card controller and credential-save revision
synchronization replace the problematic flows in dev source. None of
these remediations may be called native PASS until installed retest.
