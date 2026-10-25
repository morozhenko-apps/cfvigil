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
2. [ ] Reconcile root status and compact navigation to all existing owners.
   Retain the complete previous root in the existing delivery chronology with
   only link/heading/Markdown whitespace normalization; preserve root anchors.
   Add reading-boundary prefaces to delivery and both native checkpoint owners,
   keeping all original bodies, earlier FAILs, partial states, cases and limits.
   Do not create another SSOT or rename/delete the compact documentation set.
3. [ ] Add missing local editor/assistant/provider-state ignores; preserve all
   original rules, safe examples and neutral agent instructions. Keep source,
   CI/packaging markers, Cargo.lock, all 20 dictionaries and per-locale audit,
   licensing/security/distribution contracts and ignored runtime data unchanged.
4. [ ] Final acceptance: existing JavaScript syntax/i18n/Node tests, offline
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
