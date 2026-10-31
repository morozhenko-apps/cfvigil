# CFVigil QA acceptance contract

This file defines testing obligations, not a lifetime test counter or a transcript of individual CI runs. The code and hosted test results provide executable evidence.

## Critical invariants

| Area | Essential checks |
| --- | --- |
| Deployments | Latest production attempt wins; Pages Preview cannot override Production; running after failure is yellow; unverified remains grey |
| Provider isolation | Workers, Pages, error analytics and WAF failures remain independent |
| Security | Read-only Cloudflare access; no token leakage; untrusted names/URLs remain opaque |
| Networking | One bounded poller; offline and rate-limit handling; no duplicate requests |
| Local state | Corruption recovery, account isolation, restart continuity and history deduplication |
| UI | Pending first-sync feedback after Apply, About native drag, tray anchoring, search, keyboard, accessibility, scaling and native window behavior |
| Languages | Placeholder parity, plural forms, directional isolation and event/status consistency |
| Distribution | Identity/version parity, signed flat APT, immutable checksums and fail-closed publication |

## Validation

Ordinary CI keeps one stable pass and separates independent JavaScript,
Python distribution contracts and Rust desktop verification. The existing
Stable UI and Rust verification status remains an always-reported strict
aggregate. Prose-only PRs may skip workers only after complete merge-diff
classification; deleted/renamed packaged inputs and unknown paths require
checks. Failed, canceled, missing or inconsistent scope/results never pass.

Bound scope/result jobs to 5 minutes, JS/Python to 10 and Rust to 20. Preserve
explicit marker dispatch, read-only tokens, nonpersistent checkouts and
trusted-dev-only dependency cache writes. Policy/result/scope regressions are
part of the existing contract coverage. Platform packaging, protected release
and approved recovery admission remain separate and unchanged.

Use stable Node, Python and Rust tests for deterministic source behavior. Run package-level checks for both supported platforms and real native-device checks for tray, credential-store and uninstall behavior.

Treat unavailable external accounts, permissions, native environments or signed publishing credentials as unverified rather than passed. Synthetic fixtures cannot establish public R2 accessibility or physical installation. Security-sensitive negative scenarios must fail closed.

Do not turn this page into a list of job IDs, terminal output, owner device names or dated pass/fail narratives. Actual outcome and provenance remain in the associated CI run or issue.

Keep both dev and main as permanent branches. Develop on dev;
promote reviewed changes to the protected release branch. PR gates cover both
branches; explicit verification and private candidate markers run on dev. Only
trusted non-PR dev runs may save dependency caches. Signing, exact-source/tag
admission and owner publication approval remain independent requirements.
