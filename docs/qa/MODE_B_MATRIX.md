# CFVigil Mode B: test inventory, branch and interaction matrices

Status: **IN PROGRESS**. Product is a free read-only Cloudflare monitor.
Source authority: `AGENTS.md` and user-supplied `AGENT.md.txt`, Testing Mode
P0–P4 and mandatory Stages 1–12. The prior 88/61-test green Preflight
does **not** prove all branches/categories. Do not chase a numerical
line-coverage percentage or call a partial fixture suite full Mode B.

## Stage 1: repository scan

Tauri 2 Rust native desktop; JS WebView and Node `node --test tests/*.test.mjs`;
Python offline distribution contracts; Rust `cargo check --locked -p cfvigil-desktop`
and `cargo test --locked -p cfvigil-desktop`. Explicit `ci/run-preflight.txt`
marker triggers GitHub-hosted stable CI. UI/native E2E requires actual
owner KDE/Windows; no local desktop control is available to this agent.
No known-flaky group is registered. This does not prove tests never flake.

## Stage 2: risk and invariants

1. Latest attempt wins: new queued/running overrides old failed; green is
   **verified latest success only**; missing/denied/stale is grey.
2. Workers Builds, Pages, Analytics and zone-level WAF failures isolate by
   provider. GraphQL failure is not incident recovery; WAF blocks do not
   prove attacks or Worker attribution.
3. One serialized bounded poller; no duplicate queries; 429 Retry-After
   bounded and secret-free, no API writes or site uptime probes.
4. Identities, recovery, account switch, restart and corrupted files cannot
   fabricate healthy resource state; token remains OS keyring only.
5. All 20 locale dictionaries preserve status identifiers, raw saved event
   bytes, unknown provider strings and Arabic LTR Cloudflare scopes.
6. Distribution may publish only after owner approval, isolated R2 readback
   and SHA-256. GitHub artifacts are temporary, never final release mirrors.

## Stage 3 and Stage 5: inventory and Positive/N1–N12 matrix

N1 invalid inputs/types; N2 bounds; N3 timeout/offline/HTTP failure;
N4 retry/idempotency; N5 concurrency; N6 corruption/legacy records;
N7 authorization and hostile input; N8 locale/time/DST; N9 interruption/UI;
N10 PII/redaction; N11 money/entitlements; N12 storage faults.
✔ = an existing concrete stable regression case for this category,
**not necessarily all branches**. ? = applicable but not proved or missing.
✖ = inapplicable: N11 is not a product feature (no paid plans,
billing or entitlements). Other ? cells must be investigated: if truly
inapplicable, record a specific rationale before changing to ✖.
No empty matrix cells.

| Product artifact and owner | Level | P | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9 | N10 | N11 | N12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Provider REST, pagination, latest attempt (`cloudflare.rs`) | unit/contract | ✔ | ✔ | ✔ | ✔ | ? | ? | ✔ | ✔ | ✔ | ? | ✔ | ✖ | ? |
| Workers Analytics query/parser/spike (`analytics.rs`) | unit/contract | ✔ | ✔ | ✔ | ? | ? | ? | ✔ | ? | ✔ | ? | ? | ✖ | ? |
| WAF zone/parser/spike (`security.rs`) | unit/contract | ✔ | ✔ | ✔ | ? | ? | ? | ✔ | ✔ | ? | ? | ? | ✖ | ? |
| Shared API quota/429/backoff (`api_budget.rs`) | unit | ✔ | ? | ✔ | ✔ | ? | ? | ? | ? | ? | ? | ? | ✖ | ? |
| Poller/revision/wakeup (`polling_policy.rs; main.rs`) | unit/integration | ✔ | ? | ✔ | ? | ? | ✔ | ? | ? | ? | ? | ? | ✖ | ? |
| Incident transition/dedup/notification (`incident_policy.rs`) | unit/integration | ✔ | ? | ? | ? | ✔ | ? | ✔ | ? | ? | ? | ? | ✖ | ? |
| Incident durable IDs/account switch (`incident_store.rs`) | unit/integration | ✔ | ? | ? | ? | ✔ | ✔ | ✔ | ? | ? | ? | ? | ✖ | ? |
| Atomic file storage/quarantine (`storage.rs`) | unit/integration | ✔ | ✔ | ✔ | ? | ? | ? | ✔ | ✔ | ? | ? | ? | ✖ | ✔ |
| Credentials/Settings commands (`main.rs`) | integration/native | ✔ | ✔ | ? | ? | ? | ✔ | ? | ✔ | ? | ? | ✔ | ✖ | ? |
| Historical events/link security (`main.rs; history-model.mjs`) | unit/integration | ✔ | ✔ | ? | ? | ✔ | ? | ✔ | ✔ | ? | ? | ? | ✖ | ? |
| Native locale and tray menu (`language.rs; runtime-language.mjs`) | unit/integration | ✔ | ✔ | ? | ? | ? | ✔ | ? | ? | ✔ | ? | ? | ✖ | ? |
| Dictionary/placeholders/dates (`localization.mjs; locales/*`) | unit/locale | ✔ | ✔ | ? | ? | ? | ? | ✔ | ? | ✔ | ? | ✔ | ✖ | ? |
| WebView card/status/search/focus (`app.mjs; status.mjs; resource-navigation.mjs`) | unit/UI | ✔ | ✔ | ? | ? | ? | ? | ✔ | ? | ? | ✔ | ✔ | ✖ | ? |
| Settings editor Apply/Cancel (`settings.mjs`) | integration/native UI | ✔ | ? | ? | ? | ? | ? | ? | ? | ? | ✔ | ✔ | ✖ | ? |
| History presentation/search (`history.mjs; history-model.mjs`) | unit/UI | ✔ | ✔ | ? | ? | ? | ? | ✔ | ? | ✔ | ? | ? | ✖ | ? |
| Private cached catalog / startup hydration (`resource_cache.rs; main.rs`) | unit/integration | ✔ | ? | ✔ | ? | ? | ? | ✔ | ✔ | ? | ? | ✔ | ✖ | ✔ |
| Changed-only Settings transaction (`settings-transaction.mjs; settings.mjs`) | unit/integration | ✔ | ? | ? | ? | ✔ | ? | ? | ✔ | ✔ | ✔ | ✔ | ✖ | ? |
| Frameless About + native History icon (`about.*; main.rs; tauri.conf.json`) | UI/native | ? | ? | ✖ | ✖ | ? | ? | ? | ✔ | ✔ | ✔ | ? | ✖ | ✖ |
| KDE Wayland/X11 tray/drag (`linux_surface.rs; linux_tray.rs; native-drag.mjs`) | native/E2E | ✔ | ? | ? | ? | ? | ? | ? | ? | ? | ✔ | ? | ✖ | ? |
| Windows window anchor/tray (`windows_card.rs`) | unit/native | ✔ | ? | ? | ? | ? | ? | ? | ? | ? | ✔ | ? | ✖ | ? |
| Autostart opt-in lifecycle (`main.rs; autostart.test.mjs`) | integration/native | ✔ | ? | ? | ? | ? | ? | ? | ? | ? | ✔ | ? | ✖ | ? |
| DEB/AppImage/NSIS artifacts (`scripts/candidate-*.mjs; workflows`) | contract/native | ✔ | ✔ | ? | ? | ? | ? | ✔ | ✔ | ? | ? | ? | ✖ | ? |
| Protected release/receipt/cleanup (`scripts/*release*; protected-release.yml`) | contract/integration | ✔ | ✔ | ✔ | ? | ? | ? | ✔ | ✔ | ? | ? | ✔ | ✖ | ? |
| Read-only download Worker (`infra/download-worker/src/worker.mjs`) | contract | ✔ | ✔ | ✔ | ? | ? | ? | ? | ✔ | ? | ? | ✔ | ✖ | ? |

A credible completion statement requires every **applicable** ? resolved
with branch-specific evidence and the actual test count per category,
not replacing ? with ✔ from the same representative test.

## Stage 4: branch/function map and tests to prove

- `cloudflare.rs`: `get`, `list_workers`, `list_paginated`,
  `discover`, `worker_state`, `pages_state`,
  `select_worker_attempt`, `latest_pages_deployment`,
  `cloudflare_failure`, `scoped_provider_warning`: HTTP
  200/400/401/403/404/429/5xx, invalid/missing shape, list limits,
  1st/subsequent page, UTC parsing, each latest state, independent
  partial provider failures, redacted error messages. Fake HTTP adapter
  not yet present: end-to-end pagination remains OPEN.
- `analytics.rs`: `worker_name_safe`, `parse_number`,
  `read_aggregate`, `create_query`, `parse_batch`,
  `is_spike`, `query_spikes`: malformed account/worker,
  0/1/8/9/256/257 limits, 0/1/multiple aggregate rows,
  missing NaN/negative/excess errors, ratio thresholds,
  missing GraphQL permissions and partial batches.
- `security.rs`: `valid_id`, `valid_zone_name`,
  `count_group`, `parse_waf_counts`, `parse_zones`,
  `waf_spike`, `list_zones`, `get_zone_counts`, `detect`:
  account foreign/matching, no/multiple groups, zero/invalid counts,
  pagination, per-zone response failures, thresholds, no fake attack.
- `api_budget.rs`: `check`, `observe`, `parse_retry_after`,
  `retry_after_seconds`, `exhausted`: 0,1,60,120,121,
  missing/invalid/overflow header; 429 and repeated calls.
- `polling_policy.rs` and `main.rs`:
  `poll_delay`, `run_polling`, `refresh_once`, `save_settings`,
  `needs_refresh`, `preserve_unverified_catalogs`,
  `preserved_waf`: failure backoff, concurrent Apply, stale state,
  missed/duplicated wakeups, credentials and per-resource recovery.
- `incident_policy.rs`/`incident_store.rs`:
  `describe_transition`, `should_notify`,
  `notification_identity`, `restore`, `snapshot`,
  `insert`, `remove`, `clear`, `resource`:
  all 4 states and spike identities, initial/repeat/resolve,
  offline unknown, restarts and account boundary, bounded IDs.
- `storage.rs`: private atomic JSON write, quarantine,
  missing, symlink, wrong ownership, oversized/invalid record,
  read-only/disk-full fault and partial write recovery.
- Native OS: Windows anchor and Linux Wayland/X11 geometry,
  show/hide, drag interruption, title/icon, Settings keychain,
  autostart and shutdown/upgrade; must retain on-machine PASS/FAIL.
- JS: `classifyWorkerBuild`, `classifyPagesDeployment`,
  `counters`, `filterResources`, `nextResourceIndex`,
  `filterHistory`, `initializeLanguage`, `t`, `plural`,
  `formatDate`, `relativeTime`, `incidentText`,
  `diagnosticText`, `resourceDetail`, Settings Apply/Cancel,
  current warning and search/focus rendering: missing/unknown
  inputs, all statuses, Arabic identifiers, date/DST, live switching,
  malicious plaintext and no unexpected polling.
- Distribution: candidates SHA/identity, signed APT, R2
  404/405/traversal, upload/readback failure, zero cleanup until
  verified receipt, scope and github release without binary.

Each named branch requires its own assertion and boundary cases;
existing relevant unit tests do not automatically satisfy this map.
Full code-level branch enumeration and proof links remain OPEN.

## Stage 6: interaction matrix

| Scenario | Required outcome | Evidence |
| --- | --- | --- |
| Old failed Workers + new queued | New attempt yellow | Rust and JS fixture |
| Pages bad request + verified Workers | Workers remain green, Pages unknown | Source/fixture only |
| 403 Pages scope + Working Analytics | Independent warning, no global outage | Source/fixture only |
| 429 + low remaining budget | Stop and bounded retry | Rust budget unit |
| Runtime spike + Analytics outage | Unknown/stale, no recovery | Rust incident unit |
| Zone WAF success + another zone failed | Recover only verified zone | Rust provider unit |
| Settings Apply while Cloudflare fetch | Discard stale-account result | Source guard; real native pending |
| Corrupt state + restart + keyring | Quarantine; no false green or token leak | Storage Rust; native pending |
| RTL Arabic + provider scope/name | Scope stays LTR, name safe text | Node; physical RTL pending |
| Drag Windows + hide/show + relaunch | No position drift | Scoped owner PASS; full matrix pending |
| R2 public hash mismatch | No cleanup or published success | Python offline; live blocked |

## Stage 7–10: estimation, coverage and mutation gates

For each artifact, estimate cases from executable branch and boundary
inventory, not a global percentage; implement parameterized inputs for
each applicable N-category. Important mutations to kill: invert latest
timestamp/order, change >= to > at limit, treat unknown as healthy,
guess Pages scope from HTTP 400, allow >120 calls, resolve an unverified
spike, accept foreign zone, lose restart incident IDs, insert HTML with
provider names, and delete GitHub artifacts before verified R2 receipt.

These stages are **NOT COMPLETE**. Actual per-function branch results,
boundary variant counts and surviving mutation findings must be recorded.

## Stage 11 and 12: final execution/completion

Run the complete stable suite **once after implementation**, then the
registered known-flaky suite independently (currently no registered
known-flaky tests). Do not reclassify a failing stable test as flaky.
Physical KDE/Windows and real Cloudflare observations are separately
necessary for native/live acceptance.

Completion questions (all public methods; all branches; rules;
invariants; boundaries; applicable N1–N12; interactions) are **NO /
NOT PROVEN** until linked evidence exists. Mode B remains IN PROGRESS.

## Source coverage tranche added on QA branch 2026-10-26

New `tests/mode-b-branch-matrix.test.mjs` exercises table-driven Worker
and Pages state branches, unknown/hostile counter values, resource and
history case-fold search, keyboard boundaries, untrusted partial
snapshots, absolute UTC-offset time across DST, CLDR plural variants,
and queued native drag failures/duplicate sessions without sleeps.
The existing `tests/mode-b-provider-locale.test.mjs` now asserts
the *semantic* latest-success build/deployment wording in every
activated locale, immutable English journal bytes, 32 hexadecimal
Account ID placeholders in all 20 languages, German keychain
diagnostic meaning, Turkish API-token meaning, Russian incident grammar
and Arabic isolated scope identifiers. Production UI status IDs,
network, storage and event serialization remain unchanged.

These suites protect concrete high-risk branches, **not the entire
21-row matrix**. Missing HTTP transport-injected integration, actual
read/write fault injection and native KDE/Windows E2E are still not
proven. Do not mark Mode B complete based on these tests or on CI
success alone.

## 2026-10-26: Mode B resumed after physical client feedback and three scoped UI fixes

**Stage 1 / implementation inventory refresh.** The executable production
surface consists of 16 Rust desktop modules, 12 non-locale JS modules,
20 locale catalogs, GitHub Actions packaging, release/R2 tooling and
the read-only download Worker. Node built-in tests + Rust unit tests +
offline Python release tests are the stable group; no registered flaky
group. Repository CI is intentionally marker-gated; this documentation
and subsequent Mode B test additions **must not start a preflight yet**.

**Stage 2 / business invariants retained.** A successful *build* is not
proof of deployed uptime. Cached Worker/Pages rows are always unverified,
can never cross account boundaries, cannot introduce a fake incident,
and any cached URL must be rebuilt from validated names. Changed-only
settings commits never reapply an unchanged OS keychain token/autostart
or trigger a Cloudflare poll on a pure locale change. The native KDE
history window must request the same installed themed icon as the
package. Frameless About has an accessible Close action and no Minimize;
Cancel/Close does not alter settings. No tests or code change may relax
these guarantees.

**Stage 3 / additional explicit testable artifacts and risk:**

| File / public functions or flows | Business risk | Level | Planned additional deterministic cases |
| --- | --- | --- | ---: |
| `resource_cache.rs`: `safe_name`, `safe_account`, `accepted`, `stale_row`, `from_discovery`, `show_stale` | Cross-account leak, false-green, hostile URL, oversized cache | Rust unit | 18 |
| `settings-transaction.mjs`: `planSettingsApply` | Unwanted keychain/autostart writes, silent config loss | JS unit | 17 |
| `main.rs`: startup `resource-catalog.json`, account switch, `needs_refresh`, `commit_language`, `show_window`, `install_window_icons` | Stale incidents, duplicate polling, cached identity, old History icon | contract/integration/native | 11 |
| `about.html`, `about.mjs`, `about.css`, `tauri.conf.json` | Unclosable frameless dialog, wrong locale, unwanted Minimize | static contract/native smoke | 9 |
| `localization.mjs` and all twenty catalogs | Missing Close translation and changing legacy event bytes | JS unit | 8 |
| `cloudflare.rs` bounded `tokio::join!` four-request batching and `ApiBudget::check` | Storming Cloudflare, order mutation, 429 while concurrent | Rust integration seam + unit | 15 |
| `storage.rs` `read_or_default`, `write_atomic` and quarantine | Lost prior data after permission/disk corruption | Rust integration | 9 |
| `incident_policy.rs`: `describe_transition`, `notification_identity` | False alarm, duplicated recovery or missed WAF spike | Rust unit | 12 |

These new rows **augment**, rather than replace, the original 21-item
inventory. Overall planned additional count above = 99. Counts are
estimates of *unique logical scenarios*, not assertions of executed
tests; parameterized cases count as independent scenarios. Other
original inventory rows and pending scenarios remain in scope.

**Stage 4 / branch-map refinements (all must be linked to tests):**

- Cache name: empty; ASCII length 1, 128, 129; punctuation path/control/
  Unicode; valid hyphen/underscore; mismatched key or kind; account
  0, 31, 32, 33, non-hex; duplicate first/last; catalog length
  0, 1, 512, 513; cross-account; serializing corrupted/legacy JSON;
  trusted URL reconstruction; stale unknown for healthy/failed/deploying.
- Apply draft: same vs changed account; token unchanged/whitespace/new;
  interval same/changed/invalid; supported locale changed/unchanged;
  autostart true/false/null/unavailable; concurrent locale events;
  Cancel vs Apply vs Close; successful save vs keyring/save failure;
  zero Cloudflare calls on locale-only edit. Data changes only on OK.
- Native windows: About decorations false and exactly one accessible
  Close button; Escape action; description absent; 20 localized Close
  strings; History themed KDE hicolor name plus Tauri icon, and icon
  reasserted at show, Windows native ICO (physical checks remain open).
- Provider: four parallel calls share bounded budget; 401/403/429/5xx
  or invalid response for individual Worker/Pages leaves that resource
  unverified; independent failures; no concurrent mutation or replay
  of previous credential results.

**Stage 5 / Positive + N1–N12 decisions.** The added rows above use
the same ✔/?/✖ syntax as the original table. N11 remains ✖ for the
entire free application (no purchase flow). N2, N3 and N12 are ✖
for pure About markup; text/layout are not API requests or disk writes.
The corresponding OS/icon integration failures must be tested under N9
and N7, not invented as file-write tests.

**Stage 6 / interaction rows to validate:**

| Inputs and states | Required result | Level |
| --- | --- | --- |
| First startup after upgrade + empty cache + offline | Empty or journal-only unverified; no green | Rust unit + native |
| Subsequent startup + 512 cached entries + corrupt extra | Strict fail-closed size; no untrusted render | Rust unit |
| 2 accounts with same resource names | No cross-account cache or incidents | Rust unit/integration |
| Locale change with locked keychain + unchanged other settings | Locale commits without reading token; no new poll | JS + native |
| Native tray menu locale changed + About already open | Updated menu and About strings; no native restart | JS source + native |
| Scroll thumb pointer drag + row action | Scroll works without window movement | native E2E |
| Successful prior Workers build + new queued Pages deployment | Yellow latest, never stale green | Rust/API fixture |
| Chrome unsigned browser block + hash mismatch | Abort acceptance; no blanket AV exclusion | distribution procedure |

**Stage 7 / test pyramid and estimation.** Favor pure deterministic
Rust/JS units and bounded contract fakes, followed by a small number of
cross-platform native UI smoke scenarios. No "100% lines" target
substitutes for reaching every executable business branch.

**Stage 8 / coverage map for newly added functions:**

| Source | Branches or transitions | Existing proven | Missing before new Mode B tranche |
| --- | --- | --- | --- |
| `resource_cache.rs` | six functions and bounds/account/name/kind/duplicate/URLs | Three Rust tests | boundary tables, malformed JSON persistence integration |
| `settings-transaction.mjs` | same/changed 5 independent fields | Four focused JS source contracts | coercion/partial/unavailable + combinations |
| `about.mjs` | version success/failure, Close/Escape, website/source failure | Only source contracts | localized Close and lifecycle |
| `main.rs` icon/show | GTK/Tauri success/unavailable; same desktop theme | earlier source-only | Wayland/KDE and Windows taskbar physical |
| `cloudflare.rs` batched reads | worker/page slots 0–4, partial REST failure, 429 | prior provider fixtures | fake HTTP transport and rate limit across concurrency |
| `storage.rs` | absent/valid/corrupt/oversize/symlink/permissions | stable Rust tests | deterministic fault injection (disk full/rename failure) |

**Stage 9 and 10 are next**: implement exact, parameterized scenarios
with explicit negative assertions and mutation kills (boundary > vs >=,
account comparison, checked key, unknown vs green, no-op vs write,
source icon name, accessible Close). Never mock the domain logic.
Integration requiring a real HTTP seam/OS remains clearly documented
as open; tests cannot pretend it passed.

**Stage 11 / 12:** no CI rerun between cosmetic fixes and test writing.
One stable final suite **after** coverage implementation and mutation
review. A green source gate is distinct from owner's native KDE/Windows
acceptance. Mode B is not COMPLETE while any applicable ? and unproven
runtime scenario remain.
