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
