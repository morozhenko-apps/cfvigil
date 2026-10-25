# CFVigil

Free, local-first desktop monitoring for Cloudflare Workers and Pages.
Apache-2.0: [LICENSE](LICENSE), [NOTICE](NOTICE), [contribution policy](CONTRIBUTING.md).

## Development status

`dev` is the active source branch; `main` remains the initial repository baseline.
Current source includes monitoring, native tray/Settings, notifications,
private local history, optional launch-at-sign-in and 20 draft locale catalogs.
This is an unsigned, unreleased development candidate, not stable v1.0.

| Evidence boundary | Recorded state | Owner |
| --- | --- | --- |
| KDE geometry/Settings/position | Scoped v5 owner PASS; earlier failures retained | [Native-machine checkpoint](docs/NATIVE_MACHINE_GATE.md) |
| Connected Cloudflare | Worker rows/search/green lamps observed; partial result, missing Pages and corrective source await installed retest | [Connected smoke](docs/NATIVE_CLOUDFLARE_SMOKE.md) |
| Languages | All 20 source dictionaries exist; editorial/context and physical acceptance pending | [Per-locale ledger](docs/i18n/CONTEXT_AUDIT.md) |
| Whole machine gate, Windows and public release | Not accepted; broad coverage/Mode B follows owner machine acceptance | [Roadmap](docs/ROADMAP_TO_1_0.md), [delivery](docs/DELIVERY.md) |

## What the current source implements

- Compact tray card and upward resource drawer with search, Workers/Pages rows,
  resource-specific links, native drag and persistent position/settings.
- Read-only REST discovery and build/deployment status, serialized bounded
  polling, shared API budget/429 handling and partial-provider warnings.
- Worker-error and zone-scoped WAF aggregate detection in source; native
  notifications, account-scoped durable deduplication and filtered local history.
- Latest attempts determine the lamp: running takes precedence over earlier
  failure; unknown/permission/stale outcomes must not be presented as success.
  Counters count resources in a state, not lifetime builds or proven app uptime.

Read [product invariants](docs/PRODUCT.md) and the
[current checkpoint](docs/DELIVERY.md#current-checkpoint-2026-10-25) before
interpreting an older source/test count. A new successful build may leave the
healthy count unchanged; a brief yellow interval can finish between polls.

## Localization readiness

All 20 target dictionaries are draft source. Key/interpolation/CLDR/RTL checks
do not establish editorial correctness, native language fit or full acceptance.
Use [i18n architecture](docs/I18N_ARCHITECTURE.md) and the complete
[locale ledger](docs/i18n/CONTEXT_AUDIT.md); no locale or context audit is waived.

## Development

Use Rust, Node.js 24 (matching Preflight), Git and Tauri 2 Linux/Windows platform
dependencies. Cargo.lock stays committed. Existing stable source checks:

```sh
node scripts/check-i18n.mjs
node --test tests/*.test.mjs
python3 -m unittest discover -s tests -p 'test_release_*.py'
cargo check --locked -p cfvigil-desktop
cargo test --locked -p cfvigil-desktop
```

JavaScript syntax checks are also defined by `.github/workflows/checks.yml`.
To explicitly launch a development app: `cargo run -p cfvigil-desktop`.
Housekeeping does not launch/install it, open a wallet, inspect a token or query
a live account. Native/Windows acceptance requires its own recorded session.

## Principles and publication boundaries

- Read-only Cloudflare integration; native Rust owns API tokens and network
  requests. No GitHub deployment monitoring, health-URL probes or API mutations.
- Missing Pages permission is distinct from a verified empty catalog. Preserve
  verified Worker results while labeling failed coverage unknown; never invent
  permissions, free-plan entitlements or successful incident detection.
- No hosted product backend or paid tier. No broad test/Mode B expansion before
  native-machine acceptance. Source/candidate CI does not grant release approval.
- Normal dev code/docs/ignore commits do not request CI. Feature verification
  uses the explicit marker once; cleanup leaves every CI/candidate marker intact.
- Candidate packages are unsigned/unreleased. Stable binary distribution belongs
  only in CFVigil's isolated R2 prefix after protected approval and public hash
  readback; never permanent GitHub binary attachments. No live publication is
  claimed by the source-only [distribution contract](docs/DISTRIBUTION.md).

## Project and contribution policy

| Document | Responsibility |
| --- | --- |
| [AGENTS](AGENTS.md) / [CONTRIBUTING](CONTRIBUTING.md) | Working branch, efficient gates, bug reports, Apache forks and upstream policy |
| [PRODUCT](docs/PRODUCT.md) / [ROADMAP](docs/ROADMAP_TO_1_0.md) | Existing contracts, feature/native/i18n/Mode B/release sequence |
| [DELIVERY](docs/DELIVERY.md) | Dated implementation, CI evidence, gaps and release requirements |
| [NATIVE MACHINE](docs/NATIVE_MACHINE_GATE.md) / [CONNECTED SMOKE](docs/NATIVE_CLOUDFLARE_SMOKE.md) | Actual owner/platform/provider evidence and pending retests |
| [I18N ARCHITECTURE](docs/I18N_ARCHITECTURE.md) / [CONTEXT AUDIT](docs/i18n/CONTEXT_AUDIT.md) | Semantic messages and all 20 independent language audits |
| [DISTRIBUTION](docs/DISTRIBUTION.md) | Separate bucket/credentials, exact-main/tag/protected publication and cleanup gates |

## Historical root links

The complete [previous root](docs/DELIVERY.md#imported-root-readme-before-october-25-housekeeping)
is retained as dated provenance. Existing section bookmarks remain compatible:

<a id="known-limitations-of-this-initial-implementation"></a>

- [Known limitations of this initial implementation](docs/DELIVERY.md#known-limitations-of-this-initial-implementation)

<a id="principles"></a>

- [Principles](docs/DELIVERY.md#principles)
