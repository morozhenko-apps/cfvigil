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
- Separate source/config changes from documentation-only commits so docs edits
  do not trigger expensive CI runs.

## Engineering and validation

- Follow `docs/PRODUCT.md` and `docs/ROADMAP_TO_1_0.md` as feature scope.
- Read existing RepoTicker conventions before modifying tray, packaging or CI.
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
