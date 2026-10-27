# CFVigil QA acceptance contract

This file defines testing obligations, not a lifetime test counter or a transcript of individual CI runs. The code and hosted test results provide executable evidence.

## Critical invariants

| Area | Essential checks |
| --- | --- |
| Deployments | Latest attempt wins; running after failure is yellow; unverified remains grey |
| Provider isolation | Workers, Pages, error analytics and WAF failures remain independent |
| Security | Read-only Cloudflare access; no token leakage; untrusted names/URLs remain opaque |
| Networking | One bounded poller; offline and rate-limit handling; no duplicate requests |
| Local state | Corruption recovery, account isolation, restart continuity and history deduplication |
| UI | Tray anchoring, search, keyboard, accessibility, scaling and native window behavior |
| Languages | Placeholder parity, plural forms, directional isolation and event/status consistency |
| Distribution | Identity/version parity, signed flat APT, immutable checksums and fail-closed publication |

## Validation

Use stable Node, Python and Rust tests for deterministic source behavior. Run package-level checks for both supported platforms and real native-device checks for tray, credential-store and uninstall behavior.

Treat unavailable external accounts, permissions, native environments or signed publishing credentials as unverified rather than passed. Synthetic fixtures cannot establish public R2 accessibility or physical installation. Security-sensitive negative scenarios must fail closed.

Do not turn this page into a list of job IDs, terminal output, owner device names or dated pass/fail narratives. Actual outcome and provenance remain in the associated CI run or issue.
