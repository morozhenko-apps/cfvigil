# Implementation and release gates

Initial source: `dev`. Release branch: `main`. No change may be fast-forwarded or tagged on the strength of unverified implementation alone.

## Current verification

- Implemented provider rules for Workers Builds and Pages latest attempts in Rust.
- Included deterministic Node.js status tests and Rust unit tests, including negative tests for error-rate thresholds, script attribution, spike transitions and GraphQL outages. Node tests were executed in a local copied source snapshot (6 passed); the Rust tests have not been run.
- GitHub checks workflow runs on self-hosted Linux runners for code changes, excluding documentation-only changes.
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

## CI repair: container-required Alcotester runners (2026-10-23)

**Primary task mode:** B — Testing / verification infrastructure.

**Root cause from existing runner configuration:** both `Alcotester` and `Alcotester-2` are organization-level self-hosted runners using rootless Docker (`DOCKER_HOST=unix:///run/user/1001/docker.sock`). Their configuration requires job containers via `ACTIONS_RUNNER_REQUIRE_JOB_CONTAINER=true`. The existing CFVigil `checks.yml` targets `[self-hosted, Linux, X64]` but has **no `container:` declaration**, so it is incompatible with that requirement before source checkout or compilation.

**Implementation plan (single CI-only stage):**
1. Add an explicit `ubuntu:24.04` job container, retaining self-hosted runner labels. Do **not** remove/relax container restrictions or change runner service configuration.
2. Install the Tauri 2 Linux development packages in the ephemeral job container (WebKitGTK 4.1, GTK3, GTK Layer Shell, appindicator, OpenSSL, Wayland, base build tools).
3. Provision Node.js 24 and a stable Rust toolchain inside that container so CI does not assume software is installed on the host or runner user.
4. Execute JavaScript syntax checks and unit tests, then `cargo check` and `cargo test` in one stable suite. No repeated suite executions during implementation.
5. Publish a native GitHub Commit Status named `cfvigil/stable` on completion (whether passed or failed). This does not change release gating; it makes the actual outcome visible through GitHub's standard commit-status API when the connected client cannot list push-workflow jobs. Failure remains a failure and is never suppressed.
6. Preserve existing docs-only path exclusions, no new `main` commit, no tags, no publishing.

**Validation evidence required:** GitHub status context `cfvigil/stable` for the exact `dev` SHA, job logs for any failed native step, and eventually physical KDE acceptance. Publishing an action or creating a commit is **not proof** that the workflow ran, compiled, or passed.

**Risks:** first container pull and Rust/Tauri compilation may require significant network/cache time; the rootless daemon must be available to the runner service. The new workflow needs `statuses: write` permission scoped to the same repository. If the job cannot even start, its reporting step cannot run; the absence of a status must remain `unknown`, not success.

**Rollback:** revert only the CI workflow commit. No changes to production code, persistent storage, security credentials or runner host configuration.
