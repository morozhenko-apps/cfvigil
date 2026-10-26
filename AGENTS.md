# CFVigil working contract for coding agents

This file applies to all coding agents working in this repository.

## Branch ownership and cleanup

- **Work directly in `dev` by default. `dev` is the primary working branch.**
- Check the current remote `dev` head and active work before beginning.
  If another agent is actively changing `dev`, use a short-lived feature branch
  and PR solely to avoid simultaneous writes. Do not create routine branches.
- Finish each feature or fix on `dev` after validation. When a temporary branch
  or PR was necessary, integrate through the agreed review path, **close the PR
  and delete the short-lived branch** once merged. No abandoned branches or PRs.
  Do not delete another agent's unmerged work.
- Never fast-forward `dev` to `main`, tag or publish a release without the
  owner's explicit instruction and passing release acceptance.
- **Distribution storage invariant:** GitHub stores source, CI and optional
  release metadata only; no permanent DEB, AppImage, NSIS, checksum, or
  signed APT binary attachments on GitHub Releases. Upload release assets
  only into CFVigil's isolated Cloudflare R2 prefix, verify the public GET
  body size and SHA-256, and ONLY THEN delete exact matching temporary GitHub
  Actions archives. A failed upload/readback MUST NOT trigger deletion.
  Check deletion success; fail the release gate if cleanup fails.
  Candidate artifacts have a minimum one-day expiry because R2 test
  publication does not yet exist. Cargo.lock is tracked in source, never
  separately uploaded again for review.
- Separate source/config changes from documentation-only commits.
- **Never trigger CI for every source commit.** CFVigil dev's Preflight uses
  an explicit verification marker: update `ci/run-preflight.txt` exactly once
  after finishing a feature/fix batch. Normal source commits and documentation
  commits MUST NOT launch CI. A temporary branch PR runs code checks on PR changes.
- When the feature gate reports failure, fix the root cause first, then update
  the marker once after corrections. Do not spam CI by polling or committing
  multiple changes while another verification run is active.
- `workflow_dispatch` works only after the workflow exists on the repository
  default branch; on `dev`, the marker is the reliable explicit trigger.

## Native machine gate before broad test coverage / Mode B

After finishing 0.8, build dev-only GitHub-hosted Linux/Windows candidate
artifacts for the exact same SHA and have them inspected on the owner's
real KDE/Windows machines. Fix observed app problems before expanding broad
regression coverage or Mode B. Stable foundational unit/contract tests
continue to gate source changes. A successful CI or hosted package build
does not count as actual installation/tray/UI evidence. Do not modify or
use the owner's self-hosted CI runners for this. See
`docs/NATIVE_MACHINE_GATE.md`.

## Engineering and validation

- Follow `docs/PRODUCT.md` and `docs/ROADMAP_TO_1_0.md` as feature scope.
- Read existing RepoTicker conventions before modifying tray, packaging or CI.
- Release publication is enabled only by owner-approved version tag matching exact `main` HEAD AND a protected `cfvigil-release` environment with a matching `CFVIGIL_RELEASE_APPROVED_TAG`. Never create or populate release credentials, advance `main`, add tags, deploy the Worker, or publish R2 objects without explicit owner approval. Use a separate `cfvigil-dist` bucket and dedicated secrets, never the RepoTicker bucket token. See `docs/DISTRIBUTION.md`.
  Public GitHub Actions use **standard GitHub-hosted runners** (ubuntu-24.04,
  and windows-2025 only when Windows CI is introduced). Do not consume or
  change the user's Alcotester self-hosted runners for this open-source app.
- Correct root causes rather than introducing workarounds. Avoid duplicate
  polling, speculative success statuses, unbounded API queries and invented
  permissions. Never log API tokens.
- Keep read-only Cloudflare integration. No GitHub deployment monitoring,
  no HTTP health checking and no Cloudflare mutations.
- Validate changed units efficiently. Run comprehensive stable/flaky suites
  at the final acceptance stage rather than redundantly after every small edit.
  Distinguish tests executed from tests merely committed. Record evidence.
- Never claim Rust or native KDE smoke tests passed without actual run logs
  and observed exit status.
- Build all remaining MVP features **before** the 20-language localization
  milestone. Do a contextual audit for each supported locale and then perform
  final Mode B quality/testing and release acceptance.
- Bug reports are accepted through the one GitHub Issue form. No external PR
  contribution process, external maintainers or feature request intake.
  Forks are welcome under Apache License 2.0.

## Repository housekeeping (2026-10-25)

Mode A. Scope: existing documentation and local ignore rules only. The owner
requested this next repository cleanup and normal working-branch publication.
Selected baseline `a22b481`, clean synchronized dev, 114 tracked files and
11 Markdown files. A reviewed fast-forward preserved 19 upstream commits after
local `20847fc`; no uncommitted work, stash, reset or pull. No open PR or tracked
disposable IDE/assistant preset, generated report or empty workflow was found.

Plan and milestones:

1. [x] Read the working contract, contribution/product/roadmap, delivery,
   distribution, i18n and native/live-smoke owners. Root initial-source and
   "not executed" introductions contradict later dated evidence. KDE geometry
   has scoped owner PASS; connected Cloudflare is partial, corrected source
   awaits installed retest, Windows/editorial/overall machine gates remain open.
2. [x] Reconcile root status and compact navigation to all existing owners.
   Retain the complete previous root in the existing delivery chronology with
   only link/heading/Markdown whitespace normalization; preserve root anchors.
   Add reading-boundary prefaces to delivery and both native checkpoint owners,
   keeping all original bodies, earlier FAILs, partial states, cases and limits.
   Do not create another SSOT or rename/delete the compact documentation set.
3. [x] Add missing local editor/assistant/provider-state ignores; preserve all
   original rules, safe examples and neutral agent instructions. Keep source,
   CI/packaging markers, Cargo.lock, all 20 dictionaries and per-locale audit,
   licensing/security/distribution contracts and ignored runtime data unchanged.
4. [x] Final acceptance: existing JavaScript syntax/i18n/Node tests, offline
   Python release contracts, locked Rust check/tests once; local Markdown
   links/anchors, complete owner catalog, body/hash preservation, isolated-ignore
   and whitespace checks. No new coverage expansion, Mode B, physical/native
   install, live account/API, token/keyring, release or provider operation.
   Do not change CI markers or dispatch CI/package jobs for housekeeping.
   Commit docs/config separately; check concurrent dev before publishing and
   use the short-lived-branch rule only if concurrent writes are observed.
   Confirm remote SHA and export patch, preservation map and exact results.

Risks: claiming full machine acceptance from geometry/source, confusing draft
20-locale catalogs with editorial acceptance, hiding Pages permission failures,
or treating a repeated green build as an incrementing lifetime counter. Preserve
owner scope and unresolved cases; no product/API/architecture/security change.
Rollback: scoped reverse docs/ignore commits; no data, credentials, package,
archive format, provider or source restoration. Ignored private state is outside
scope and must not be opened or removed.

### Validation-generated build output classification

Existing local Rust check/tests completed successfully, but Tauri generated
untracked JSON permission schemas under `apps/desktop/src-tauri/gen/schemas/`.
They did not exist as tracked/untracked source at the clean baseline and are
regenerable compiler/IDE output, not user runtime or credential data. Preserve
committed capabilities/config and all original files; ignore this exact generated
schema directory and remove only the task-created JSON schemas after the compiler
exits. Retain any unrelated data. Also cover nested local IDE state. No source or
test change is required; rerun only preservation/ignore/whitespace checks.

### Housekeeping validation and recovery receipt (2026-10-25)

Local cleanup acceptance is complete. Root status/navigation separates source,
scoped KDE geometry PASS, real partial connected smoke, pending installed retest,
20 draft catalogs and still-open editorial/Windows/overall machine acceptance.
The complete old root remains in DELIVERY after only link/heading/whitespace
normalization; all former anchors and complete original bodies of three prefaced
owners remain. All 11 original documents and 114 tracked paths remain, with no
new repository index or deleted original. The original agent contract remains.
All 108 protected original files retain physical SHA-256, including executable
source/tests, Cargo.lock, 20 locales and every CI/packaging marker.

Executed STABLE groups once, all exit 0:

- Workflow-selected `node --check`: 17 JavaScript files parsed.
- `node scripts/check-i18n.mjs`: source message/dictionary checks PASS.
- `node --test tests/*.test.mjs`: 73 passed, 0 failed/skipped.
- `python3 -m unittest discover -s tests -p 'test_release_*.py' -v`:
  7 offline release contracts passed, no skips or live provider operations.
- `cargo check --locked -p cfvigil-desktop`: Linux compile PASS.
- `cargo test --locked -p cfvigil-desktop`: 51 passed, 0 failed/ignored.

Rust check/test each reports five pre-existing unused native tray-message
constants (`dead_code`) in unchanged source. They are warnings, not a clean
warning-free result; full stderr is retained externally. No assertion, source,
CI gate or flaky classification was weakened. No separate known-flaky suite was
identified, no broad coverage/Mode B expansion and no repeated stability claim.

Final external cleanup validator: exit 0; 315 static/hash/body/catalog/anchor/
isolated-ignore checks, 49 local Markdown links, zero broken. Baseline
`git diff --check a22b481`: exit 0. Initial static validation exposed the four
Tauri-generated untracked schema JSON files. They were classified above, ignored
and removed only after compiler exit; all original tracked files/source remained.
The separate task-owned compiler directory was also removed. No user build,
runtime, credential, token/keyring or private data was opened or removed.

No CI marker update, manual CI/candidate dispatch, package/GUI launch/install,
live-account/API probe, Cloudflare/Worker/R2 write, main/tag/release promotion or
other repository change. Existing read-only and machine-before-Mode-B boundaries
remain; this receipt does not close a product or physical acceptance gate.

Continue from product/roadmap and current native/live checkpoint owners.
Before normal dev publication recheck concurrent work; confirm remote branch SHA.
The audit workspace exports final commits/push, full command and warning output,
original-body/hash map, document catalog and deterministic rollback patch as
`outputs/CFVIGIL_CLEANUP*`, `outputs/CFVIGIL_PRESERVATION_MAP.json` and
`outputs/CFVIGIL_DOCUMENT_CATALOG.json`. Rollback uses scoped reverse docs/config
reverts; no source, user data, credentials, package or provider restoration.
All owned local commands completed; no app, watcher or server was launched.

### Concurrent Pages correction integration (2026-10-25)

The final fetch found six new upstream commits after `a22b481`, through
`f2b0e46`: provider Rust, two verification/package markers and appended native/
delivery evidence. Pages Read is owner-verified; HTTP 400 pagination is distinct
from missing permissions. Retain those facts and pending installed retest.
Only the six owned, unpublished housekeeping commits were replayed onto that
base. One DELIVERY append conflict was resolved by retaining the entire new
upstream body plus the complete old-root appendix; no source was edited.
Initial baseline/commands/maps and a recovery bundle remain externally saved.

Active dev requires an exceptional short-lived docs branch and PR, not direct
concurrent writes. Publish only docs/ignore cleanup, verify its diff and merge
normally, then delete the temporary branch. CI marker changes belong solely to
upstream and are preserved byte-for-byte relative to the integrated base.

The existing upstream Preflight at exact `33a53d3` (run 40181633690) has SUCCESS.
Confirm executable/test/config input identity before reusing it for the final
integrated source; do not repeat a compiler campaign, update a marker or dispatch
another CI for documentation housekeeping. Initial local counts apply to the
initial base; latest source/CI and all warning/outcome receipts are recorded
separately in audit evidence. Whole native/Pages/editorial/Windows gates remain
open despite source success. Final publication and rollback use the exact merged
PR diff, preserving every concurrent source and marker change.

## Python bytecode housekeeping (2026-10-26)

Mode A. The owner requested inspection and publication of useful pending files.
Local Ubuntu shell and Git access are available. Baseline `02673be` on `dev`
matches freshly fetched `origin/dev`; no tracked modifications are present.
The only pending files are `tests/__pycache__/test_icon_import.cpython-314.pyc`
and `tools/__pycache__/import_icon.cpython-314.pyc`. Their Python sources are
already tracked. These are regenerable interpreter caches, not source changes.
The existing ignore policy covers other generated output but omits Python caches.

Plan and milestones:

1. [x] Inspect agent/contribution guides, icon pipeline, tracked source,
   pending paths and remote working-branch state.
2. [x] Add `__pycache__/` and `*.pyc` ignore rules. Retain existing cache files
   locally; do not commit generated bytecode or change executable source.
3. [x] At the final gate, verify the actual caches are ignored, synthetic
   nested cache/bytecode paths are ignored, Python sources remain visible to
   Git, and the diff has no whitespace errors. No runtime test suite is needed
   for ignore-only changes; do not update CI markers or dispatch workflows.
4. [x] Commit documentation separately from ignore configuration and check
   remote concurrency. A concurrent marker-only commit requires publication
   through a short-lived branch and PR instead of direct writes to `dev`.

No product, architecture, public API, persistence or security contract changes.
Risk: an overly broad ignore might hide Python source; validate representative
source paths explicitly. Rollback: revert the scoped ignore commit and the
separate documentation commits. No local cache deletion is required.

Validation receipt: the final Python-driven Git checks exited 0: four cache
ignore checks, three source-visibility checks, two tracked-source checks and
two original-cache-preservation checks passed. `git diff --check` exited 0;
the executable/source diff is empty and configuration changes only `.gitignore`.
Runtime STABLE/FLAKY suites, builds and native checks were not run because no
executable source changed. No CI marker was modified or workflow dispatched.

Publication recovery: the first direct push was rejected because `dev` advanced
concurrently from `02673be` to `549d292` (only Linux/Windows candidate markers).
No force push was attempted. The two owned commits were replayed without
conflicts onto `549d292` on `chore/python-bytecode-ignore-20261026`. Publish this
branch, merge its PR normally after required checks, delete the temporary branch
and fast-forward local `dev`; verify the remote SHA and clean status. The PR and
Git history provide the subsequent publication outcome; this pre-publication
receipt alone does not claim push or merge success.
