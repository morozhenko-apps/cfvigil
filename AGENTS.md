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
