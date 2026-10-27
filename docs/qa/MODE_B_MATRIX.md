# CFVigil Mode B: test inventory, branch and interaction matrices

Status: **IN PROGRESS**. Product is a free read-only Cloudflare monitor.
Source authority: `AGENTS.md` and user-supplied `AGENT.md.txt`, Testing Mode
P0–P4 and mandatory Stages 1–12. The prior 88/61-test green Preflight
does **not** prove all branches/categories. Do not chase a numerical
line-coverage percentage or call a partial fixture suite full Mode B.

## Stage 1: repository scan

Tauri 2 Rust native desktop; JS WebView and Node `node --test tests/*.test.mjs`;
Python offline distribution contracts; Rust `cargo check --locked -p cfvigil`
and `cargo test --locked -p cfvigil`. Explicit `ci/run-preflight.txt`
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


## 2026-10-26: Stage 9 first substantive deterministic coverage tranche

The owner asked **not to run a separate check after the three minor
UI changes**; the same instruction is honored during incremental
Stage 9 commits. These tests are committed as source, **not yet
executed**. A single full stable suite belongs to Stage 11, after
the remaining planned Stage 9 and Stage 10 work.

New test modules:

- `tests/mode-b-settings-transaction.test.mjs`: 14 deterministic
  Given/When/Then cases with table-driven account, locale, interval,
  token, autostart and unavailable-state combinations; no mocks of
  the actual `planSettingsApply` domain function.
- `tests/mode-b-about-history-icon.test.mjs`: 8 native-source/UI
  contracts including About no-Minimize, Close/Escape, description
  removal, 20 translated labels, KDE themed History icon and fixed
  safe navigation targets. These are **not** physical native E2E tests.

Existing Rust domain modules expanded (new test *functions*; most
contain multiple exact boundary fixtures):

| Domain file | New test functions | Primary tested risks |
| --- | ---: | --- |
| `resource_cache.rs` | 8 | N1, N2, N6, N7, N10, account crossing |
| `api_budget.rs` | 4 | N2, N3, N5, N7, retry limits |
| `incident_policy.rs` | 5 | P, N4, N5, N6, notification identity |
| `storage.rs` | 5 | N2, N6, N7, N12, rollback |
| `analytics.rs` | 4 | P, N1, N2, N6, N7, spike boundary |
| `security.rs` | 4 | P, N1, N2, N6, N7, zone isolation |
| `polling_policy.rs` | 2 | N2, N4, N5, infinite input safety |
| `incident_store.rs` | 4 | N2, N4, N6, cross-account and eviction |

Source additions total **59 new test functions** (23 JS +
36 Rust; excludes the existing suites). These are code inventory
counts, **NOT pass counts**. Category membership overlaps and should
not be double counted to inflate a global total. Every parameterized
case represents an independent input and has an assertion. The Rust
concurrency tests use scoped threads or watch revision signals,
without clock sleeps or public Cloudflare API calls.

### Stage 10: mutation-to-assertion mapping for this tranche

| Code mutation/defect | Specific tests intended to kill | Status |
| --- | --- | --- |
| `MAX_CACHED >=` inverted or 512 replaced by 511 | `given_0_1_512_513_entries...` | Source authored, not executed |
| Account ID scope comparison bypassed | `given_account_switch_and_cached_history...` | Source authored, not executed |
| Cache reused a persisted foreign URL or prior status | `given_hostile_saved_urls...`; `cached_success_failure...` | Source authored, not executed |
| Malformed key/name accepted into cache | `given_invalid_kinds_and_keys...`; name/hex fixtures | Source authored, not executed |
| Locale-only Apply rewrites token or OS autostart | `Locale-only Apply`, `autostart` JS tables | Source authored, not executed |
| No-op Apply bypassed and saves again | `Given all unchanged Settings...` | Source authored, not executed |
| Remove About Close, restore native minimize, description | `mode-b-about-history-icon` first four cases | Source authored, not executed |
| History no longer matches `Icon=cfvigil-desktop` | `Given installed DEB theme icon...` | Source authored, not executed |
| Shared API budget admits request 121 | `given_parallel_detail_requests...` | Source authored, not executed |
| Retry-After cap or minimum changed | `given_retry_after_boundary_variants...` | Source authored, not executed |
| Unknown analytics accidentally marked recovery | `given_repeated_runtime_spikes...`; GraphQL malformed fixtures | Source authored, not executed |
| Reverse failure→queued precedence | `given_each_latest_attempt_transition...` | Source authored, not executed |
| WAF threshold 100 or slope comparison changed | `given_waf_volume_threshold_edges...` | Source authored, not executed |
| Atomic write replaces old state before bounds check | `given_oversized_replacement...` | Source authored, not executed |
| Retry policy overflows or busy polls | `given_every_retry_backoff_boundary...` | Source authored, not executed |
| Drop account identity or notification eviction | `given_account_missing_changed...`, `given_each_notification_window_boundary...` | Source authored, not executed |

These mappings are intended mutation kills, **not results of an
executed mutation runner**. Changing an assertion until red code passes
would violate P0. If a test reveals a real design defect, address the
production root cause rather than weaken tests.

### Stage 3/4/8 still open for complete regulated Mode B

The repo contains other public flows with unresolved applicable matrix
cells: injected REST/GraphQL response transport and pagination;
Settings Apply under actual failed keyring/OS-autostart writes; separate
verified vs unknown WAF zones; file-system rename/disk-full seams;
Wayland and Windows actual pointer scrollbar and icon acceptance;
signed release environment, R2 readback and verified cleanup.
A source-only test of the existence of a URL, GTK call or workflow
cannot close those real integration/native scenarios. They remain
OPEN rather than declaring Mode B completion.

### Stage 11 execution boundary

No new `ci/run-preflight.txt` trigger, candidate package marker,
`main`, tag or release was modified in this testing tranche.
Final stable (Node, Python, locked Rust check/test and locale/asset
contracts) plus separate known-flaky group is reserved for completion
of Stages 9–10. Previously recorded stable PASS at `edfcc2a`
predates all the new tests and cannot be cited as their execution.


### Stage 9 defect surfaced by testability inspection: About window capability

The new frameless About Close/Escape control invokes
`getCurrentWindow().hide()`. Repo previously granted
`core:window:allow-hide` only to Settings; About had `core:default`
and could be denied. Fixed in narrowly scoped
`apps/desktop/src-tauri/capabilities/about.json` with window
`["about"]` and permission `["core:window:allow-hide"]`.
One additional JS contract case checks that both capability files
grant hide to **only their corresponding windows**, not all WebViews.
This is a real functional prerequisite discovered in Stage 9, not
a test assertion relaxed to accommodate a broken implementation.

No native or full stable suite has been executed after the changes.


## 2026-10-26: Stage 9 transport-backed integration, disk fault and mutation tranche

Mode remains **B (Testing)**. This adds deterministic testability seams,
not new application features or configuration. No external API calls,
application credentials, self-hosted runners, protected release or R2
mutation. No stable/flaky suite has been executed yet in this tranche.

### Real HTTP negative-path integration (new)

| Source and tests | Exact SUT exercised | Positive and negative evidence to obtain |
| --- | --- | --- |
| `cloudflare_http_tests.rs` | Actual `Cloudflare::get`, `list_workers`, `list_paginated`, `discover`, worker/Pages latest-state decisions and shared `ApiBudget`, over loopback GET | 2-page catalog; pending build supersedes deployed version; independent Workers 403 and Pages success; both catalogs unavailable; second page empty; Pages detail 429; Builds scope 403; Pages detail HTTP 400 not mislabeled missing scope; pagination >50; fifth detail request blocked after first-group 429 |
| `analytics_http_tests.rs` | Actual `query_spikes_at` using reqwest POST/GraphQL alias and batch parsing | Nine workers in two batches, first spike verified while second batch 403; incomplete JSON rejects entire affected batch; first-batch 429 prevents subsequent request; empty and 257-worker catalogs do not request external data |
| `security_http_tests.rs` | Actual `detect_at`, account-scoped zones REST and per-zone GraphQL POST, shared budget | Three zones: real spike, one 500/unverified, one normal/verified; foreign account and bad zone URL not queried; zone 429 blocks following queries; invalid account and 31-zone bound fail before unsafe GraphQL |
| `test_http.rs` | Rust test-only 127.0.0.1 HTTP fixture, JSON body and status parser | GET/POST captured without bearer tokens, bounded input, fail if unsafe unknown route. This is **not a mock of the detection algorithms** |

The Cloudflare production base URL stays a compiled fixed constant.
`Cloudflare::fixture` exists under `#[cfg(test)]` only; private
`query_spikes_at` and `detect_at` are invoked in production solely
through their existing public functions passing the fixed Cloudflare
URL. No public API, user-configurable endpoint or credential
boundary changes.

This replaces **source-only verification** of several REST/GraphQL
boundaries with test cases executing the real network client and
provider logic against a local fixture. Still NOT covered: physical
Cloudflare account scope, real API schema drift/permissions and
WebView OS notifications; keep those cells OPEN.

### Persistent catalog and filesystem fault integration

- `resource_cache.rs`: two new tests persist through actual
  `storage::write_atomic` and `read_or_default`, check private
  known resource names come back **grey/unknown** for the correct
  account, verify foreign account empty, and verify corrupted JSON
  is quarantined intact rather than silently rendered.
- `storage.rs`: a private `write_atomic_with_rename` seam retains
  the production `fs::rename` operation; two fault tests inject
  `PermissionDenied`/other rename errors **after a temp file has
  been written and flushed**. Assert previous state byte-for-byte
  preserved, temporary file removed and private filesystem error
  details replaced by stable `STORAGE_PERSIST`. Existing
  `storage-contract.test.mjs` was updated to check the new actual
  atomic boundary and fault tests rather than its obsolete inline
  implementation regex.

Remaining storage gaps: impossible to simulate full disk exhaustion
and every OS-specific fsync failure using this narrow seam; these
must remain N12 OPEN unless a reliable disposable fault-injection
environment can be used.

### Stage 10: executable targeted JS mutation checks

`tests/mode-b-mutation-checks.test.mjs` defines 18 **in-memory**
source mutations across exported pure Settings, Worker/Pages status,
resource search/counters, and native snapshot normalization.
Each altered implementation is loaded separately through a data
URL and must fail a **domain expectation**, not simply differ in
code text. Targets include 5 independent Settings-change decisions,
token suppression, whitespace normalization, autostart, queued
attempt precedence, Pages success/skipped semantics, invalid statuses,
search trimming, readiness and partial warning sanitization.

Status of all mutants: **AUTHORED, NOT EXECUTED**, so there is
no earned mutation kill percentage or 18/18 pass claim yet.
This does not mutate Rust; Rust mutation outcomes remain unverified
until explicitly executed under a safe isolated toolchain. Source
test coverage numbers and mutation kill claims are separate evidence.

### Read-only public download Worker

Four new Node integration-contract cases exercise the production
`fetch` handler with fake R2 reads: unsupported POST/PUT/PATCH/
DELETE/OPTIONS never touch R2; traversal, encoded URLs and mismatched
versions are 404 without storage access; immutable release versus
mutable APT caching, content-type, ETag and HEAD/body semantics;
missing objects 404 on both GET/HEAD. No live R2 route or bytes touched.

### Execution and completion status

These additions are source commits on `dev`; **not run** and not
CI-verified. Stage 11 remains a single stable gate after coverage
implementation and bounded Stage 10 review. The repository has no
separately declared known-flaky group; report that as "none registered",
not "flaky-free". Previously green Preflight #40398156902 predates
this branch and is not evidence for new tests.

Remaining owner/native and protected-release gates: KDE/Windows real
installed UI, actual Cloudflare API response drift, keychain failure
and concurrency under real OS conditions, protected R2 public GET
readback and signing. Never turn these to PASS from fixture output.


## 2026-10-26 — Follow-on Mode B Stage 9–10 tranche

This continuation advances the **previously identified** gaps in the
21+3-artifact Positive/N1–N12 matrix. It does not create a new product
feature or alter the public Cloudflare API, persistent user formats,
security boundary or release channel. At completion of this tranche,
these are **source-authored, not yet executed**.

### Actual implementation and contracts

- `main.rs`: extracted the existing credential operation decision and
  execution into `credential_action` / `perform_credential_action`,
  reusing **those same helpers in production**. The only native operations
  remain the existing OS keyring read/set, under the established serialized
  `refresh_lock`. No keyring I/O on a locale-only or unchanged Apply;
  a token write or old-token verification error returns **before** JSON
  settings commit or polling. `CredentialAction` intentionally does
  not implement `Debug` to prevent incidental secret formatting.
- `main.rs`: an OS autostart read/change seam `apply_autostart`
  runs the existing real plugin calls while allowing deterministic
  `PermissionDenied` and `read unavailable` failure injection.
  An unchanged preference never asks the OS for its current value.
  The real public command still reports partial settings persistence
  if autostart failed; it does not silently claim rollback.
- `main.rs`: both provider-response publication guards now call the
  same `superseded_revision` helper. Concurrent configuration changes
  invalidate replies collected with outdated account/credentials.
  There is no added poller, retry, timeout or activity check.
- `main.rs` test modules: 13 Settings/keyring/autostart methods and
  5 partial-provider/revision tests, including version change during
  in-flight data, independent Workers/Pages partial-failure statuses
  and WAF's explicit confirmed-zone recovery boundary.
- `tests/smoke-defects-mode-a.test.mjs` was corrected to assert the
  **absence** of the About description, as explicitly requested by
  the owner. The older test incorrectly asserted its presence and
  contradicted the more recent `mode-b-about-history-icon` contract.
  Locale dictionary keys remain intentionally available to avoid
  modifying historical/localized data.
- `tests/artifact-retention.test.mjs`: 4 new fail-closed cleanup
  scenarios for forged/incomplete receipts, 403/malformed/multiple
  GitHub artifacts, DELETE 503/uncleared artifacts, missing token
  and idempotent already-deleted results.
- `tests/test_release_r2.py`: 6 new offline tests use fake S3
  head/upload/get bytes to verify the **real** `r2_release.upload`
  logic: exact CFVigil bucket/prefix, immutable same-hash reuse,
  immutable hash conflict refusing overwrite, mutable index
  `no-store`, private byte-mismatch rejection, and denied
  credentials without a new upload. No R2 or public GET was called.

### Mode B new scenario inventory (this continuation only)

| Test owner | New test functions | Categories exercised (overlap allowed) |
| --- | ---: | --- |
| Cloudflare REST localhost | 9 Rust | Positive, N1, N2, N3, N6, N7 |
| Workers Analytics GraphQL localhost | 4 Rust | Positive, N1, N2, N3, N6, N7 |
| Zone-level WAF localhost | 4 Rust | Positive, N2, N3, N6, N7 |
| Atomic cache persisted/recovery | 2 Rust | Positive, N6, N7, N12 |
| Post-fsync rename failure | 2 Rust | N6, N10, N12 |
| Credential/autostart state + simulated OS failures | 13 Rust | Positive, N4, N5, N7, N10 |
| Partial provider and revision-race policy | 5 Rust | Positive, N3, N4, N5, N6 |
| Public download Worker | 4 Node | Positive, N1, N2, N7 |
| In-memory decision mutants (18 mutations) | 3 Node | Stage 10, Positive, N1, N2, N5, N6 |
| GitHub artifact cleanup receipt and API denials | 4 Node | N1, N3, N4, N6, N7 |
| R2 immutable/mutable byte and permission contracts | 6 Python | Positive, N3, N4, N6, N7, N12 |
| **Total new test functions** | **56 (39 Rust, 11 JS, 6 Python)** | **Unexecuted, never label PASS** |

Scenario categories **not applicable** to this free, read-only
monitoring product: N11 purchases, subscriptions, paywalls and
entitlements (no such production flows exist). N8 (locale/date)
and N9 (UI interruption) have prior dedicated test coverage and
separately pending physical tests; they were not targeted by this
transport/persistence continuation. The table intentionally does
not claim 56 distinct N-category-exclusive checks: many tests cover
multiple categories. Counts are source inventory, NOT results.

### Stage 10 intended mutation kills and evidence boundaries

New assertions should reject: retry after 429, missing pagination
page, wrong latest-attempt selection, treating HTTP 400 as denied
scope, foreign WAF zones, GraphQL partial response claimed recovered,
mutable R2 object treated immutable or vice versa, cleanup before
validated public readback, unpublished receipts, unverified deletes,
cross-account cached identity, publication of superseded account
results, unintended OS autostart write on no-op, or committing settings
after keyring write failure.

The existing 18 JS source mutants now have an executable test harness
under `tests/mode-b-mutation-checks.test.mjs`, but **their actual kill
results are not yet recorded**. The Rust mutation descriptions are
test-to-invariant mappings, not a recorded automated mutation run.

### Remaining mandatory gaps before Mode B COMPLETE

Stage 11 full stable suite remains **NOT EXECUTED** for these commits:
run Node stable, i18n, offline Python, locked Rust check and test
**only after** all planned changes/mutation reviews. Known-FLAKY
suite: none registered; do not infer perfect stability. If errors
occur, use root cause and rerun only affected checks plus mandatory
stable gate, never weaken assertions.

Actual locked/denied OS keyring under KDE and Windows; real OS
autostart-denial behavior; screen-reader/RTL and visual scroll; real
Cloudflare schema/permission drift; Windows SmartScreen warning
classification; production protected signing/R2 upload/public GET
readback and cleanup are **NOT OBSERVED**. A fake HTTP or S3 provider
does not close those physical/production gates. No self-hosted runners,
CI markers, candidate builds, `main`, release or R2 publication
were touched in this continuation.

Rollback: each change is scoped to a source/test commit. The
keyring/autostart helpers preserve existing action ordering. The
new R2 and GitHub tests are test-only and require no user-state
migration or cleanup.

## 2026-10-27: identity drift remediation added to Mode B

A source audit found that, although the Tauri product was always
`CFVigil`, the Cargo workspace member was named `cfvigil-desktop`
and its derived native executable and Linux theme icon inherited
that suffix. This risks leaking an internal directory concept into
installed binaries, KDE task-switcher and downstream manifests.

Architecture-level correction on `dev`: Rust crate `cfvigil`
plus matching committed `Cargo.lock`, all hosted Linux/Windows/
protected CI selectors `-p cfvigil`, `main.rs` GTK
`set_icon_name("cfvigil")` at window creation/show, and new
package validation requiring the exact DEB `Name=CFVigil`,
`Exec=cfvigil` and `Icon=cfvigil` with installed PNG.
`scripts/candidate-version.mjs` now rejects any noncanonical
crate/lock identity. Tests protect version drift, old suffix,
installed launcher identity, preserved bundle ID/keyring, and
the release pipeline. The historical 0.1.0 filename, `cf-vigil`
Debian package name, `com.morozhenko.cfvigil` app ID and
`dist/CFVigil/` R2 prefix remain stable.

Prior Mode B hosted Preflight
[#40470097481](https://github.com/morozhenko-apps/cfvigil/actions/runs/40470097481)
completed SUCCESS for **the previous SHA**, before identity changes.
That pass proves only the prior test tranche and is NOT acceptance
of the renamed executable/launcher. A fresh STABLE gate and matched
Linux/Windows package checks must verify the current changes.
Real owner KDE/Windows upgrade/autostart smoke remains OPEN.
N1/N6/N7/N9 are applicable to upgrade/package identity; N11 still
inapplicable. No live Cloudflare, production release, R2, signing,
`main` or self-hosted runners touched.


## 2026-10-27 — Owner Linux acceptance, remaining Mode B contract slice

The owner installed the canonical `cfvigil` Linux candidate from the
same-SHA `b20d13278d4ebfac5128ee495a52d0eece5c54da` packaging
runs and reported the visible Ubuntu experience satisfactory with no
new UI complaints. Record this as **owner visual Linux candidate
acceptance in the reported scope**, not as a complete verification of
KWallet permission denials, arbitrary network failures, signed APT,
Windows runtime, all 20 locale screens or live Cloudflare analytics.

GitHub-hosted source gate [#40484906417](https://github.com/morozhenko-apps/cfvigil/actions/runs/40484906417)
and matched Linux/Windows candidate packaging runs
[#40485912007](https://github.com/morozhenko-apps/cfvigil/actions/runs/40485912007)
and [#40485912042](https://github.com/morozhenko-apps/cfvigil/actions/runs/40485912042)
are green **at that earlier exact SHA**, with 157 JavaScript,
16 Python and 139 Rust tests. No claim that these already covered
every testable branch.

### Targeted plan: high-risk real HTTP boundary and fault cases

This Mode B increment closes concrete **missing transport branches** from
Stage 4 and row `Provider REST, pagination, latest attempt`, retaining
the existing production implementation. It uses real `reqwest`
against an isolated `127.0.0.1` `Loopback` test server.
Test only the dependency (HTTP server), never mock
`Cloudflare::get`/`list_paginated`/`list_workers`.

Inventory and branch map for this tranche:

| Method / artifact | Uncovered branches and boundaries | Categories | Expected scenarios |
| --- | --- | --- | ---: |
| `Cloudflare::get` | 200 valid/invalid JSON, missing `success`, `success:false`, non-object envelope, short/long body; 2 MiB exact/max+1 | P, N1, N2, N6 | 9 |
| `Cloudflare::get` errors | HTTP 400/401/403/404/429/500/503, string/malformed/oversized error code, no secret-body leakage; 429 absent/malformed Retry-After | N1, N3, N7, N10 | 11 |
| `Cloudflare::list_workers` | 0/1/256/257 entries, non-array result, forbidden Workers pagination | P, N1, N2, N6 | 5 |
| `Cloudflare::list_paginated` | 0/1/50/51 advertised pages, empty later pages, malformed subsequent page, 256/257 cumulative entries, existing query delimiter | P, N1, N2, N3, N6 | 9 |

Interaction matrix: a malformed reply must never be accepted as
verified/green; one provider can fail while the other is retained;
limit-triggered rejection must stop further REST traversal;
429 without usable Retry-After must inhibit subsequent requests and
use the bounded 300-second fallback, without exposing the bearer
token, account, or server-supplied free text.

Stage 10 mutation expectations: invert `> 2 MiB`, `> 256` or
`> 50`; bypass `success === true`; log untrusted HTTP error body;
remove budget stop on 429; follow a nonexistent extra Pages page.
The new tests must reject these mutations. Source implementation
and any necessary loopback fault-test seam changes will be in
test-only commits. No feature expansion, changed public API,
persistent format, auth boundary, or production network destination.

Execute one complete STABLE group after the additions: Node syntax,
20-locale and icon contracts, JavaScript tests, offline Python,
locked Rust check and tests. Physical Windows/live Cloudflare/R2
release and human editorial 20-locale review stay separate,
unverified gates. No known-FLAKY group is registered.


### Implementation evidence: bounded Cloudflare REST fault suite

Source commit `db3edeb9` adds ten deterministic `#[tokio::test]`
functions with multiple independent table-driven cases to
`apps/desktop/src-tauri/src/cloudflare_http_tests.rs`. A test-only
`Loopback::with_retry_header` controls external Retry-After headers,
while continuing to exercise the actual production `ApiBudget::observe`
and `Cloudflare::get` logic. No production feature/configuration or
real Cloudflare endpoint was changed.

| New test | Branch proof intended | N categories |
| --- | --- | --- |
| `given_rest_success_envelopes...` | nine true/false/missing/null/invalid result envelopes | P, N1, N6 |
| `given_rest_payload_at_two_megabytes_boundary...` | 2 MiB−1, exactly 2 MiB, 2 MiB+1 | P, N2 |
| `given_untrusted_http_errors...` | 400/401/403/404/500/503, code type, 8192-byte error cutoff, secret redaction | N1, N3, N7, N10 |
| `given_429_without_usable_retry_after...` | absent/bad/overflow retry; 300-second backoff, no second HTTP | N3, N4, N7 |
| `given_0_1_256_257_workers...` | exact Worker count bounds; never add unsupported params | P, N2 |
| `given_worker_list_as_nonarray...` | null/object/string/boolean list rejected | N1, N6 |
| `given_pages_catalog_0_1_50_51_pages...` | 50-page cap plus 0/1 case, reject 51 without traversal | P, N2, N3 |
| `given_cumulative_pages_count_at_256_257...` | 256 accepted, 257 rejected across pages | P, N2 |
| `given_malformed_later_pages...` | bad second-page payload, false success, malformed shape | N1, N3, N6 |
| `given_existing_query_in_pages_path...` | pagination separator with pre-existing query | P, N1 |

These are **implemented, not yet executed** for the new commit.
The strict branch-level mutation candidates are:
`> 2 MiB` → `>= 2 MiB`; `> 256` → `>= 256`;
`> 50` → `>= 50`; `success == true` bypass;
`retry_after` fallback removed; error body accidentally leaked;
first/last Pages query delimiter corrupted. Each must be killed by
one of the explicit assertions. No automated Rust mutation runner
has been executed, so no mutation kill percentage is claimed.

Next stage is ONE GitHub-hosted full stable verification on the final
code commit. N11 remains not applicable; Windows native, real
Cloudflare authorization, human 20-locale semantic acceptance and
protected R2 release remain outside this offline suite and OPEN.


### Stage 11 verified results for the REST boundary tranche

**Source commit:** `3455ba8f56e59648e34023c121543b0845458d35`
(test + documentation + explicit CI marker; no candidate, release
or distribution production change).

**GitHub-hosted STABLE:** [#40511403562](https://github.com/morozhenko-apps/cfvigil/actions/runs/40511403562)
completed **SUCCESS** on 2026-10-27.

| Exact CI command/stage | Verified result |
| --- | --- |
| JS syntax and `node scripts/check-i18n.mjs` | PASS |
| `node scripts/check-icon-assets.mjs` | PASS |
| `node --test tests/*.test.mjs` | **157 passed, 0 failed** |
| `python3 -m unittest discover -s tests -p 'test_release_*.py'` | **13 passed** |
| `python3 -m unittest discover -s tests -p 'test_icon_import.py'` | **3 passed** |
| `cargo check --locked -p cfvigil` | PASS |
| `cargo test --locked -p cfvigil` | **149 passed, 0 failed, 0 ignored** |

**10 new Rust tests / 41 table-driven cases**, all 10 passed.
Non-exclusive test-function coverage by negative category for
this increment only (do not add as disjoint counts): P=6,
N1=5, N2=4, N3=4, N4=1, N5=0, N6=3,
N7=2, N8=0, N9=0, N10=1, N11=0 (N/A),
N12=0. These are code review classifications, not profiler-driven
statement coverage and not a claim all applicable categories are
complete. N5/N8/N9/N12 have separately authored suites and still
need artifact-level coverage closure evidence.

**FLAKY:** no separately registered FLAKY suite. Therefore no
independent FLAKY test execution was possible; this is *not*
a claim that the codebase has no flaky cases. This tranche adds no
sleep, nondeterministic production network, or current-time
assertion. The CI run used exclusively GitHub-hosted Ubuntu.

**Stage 12:** REST transport branches listed in this tranche
are source-tested and CI-verified. End-to-end Mode B remains
**PARTIAL / IN PROGRESS**: exhaustive per-public-method
branch map, uncovered matrix `?` cells, real Windows/Cloudflare,
the independent 20-locale human/RTL audit, signed APT/R2
protected release and public readback remain unproven.
The owner's Ubuntu visual approval is scoped to the installed client,
not a blanket operational/security/regression sign-off.

No `main`, release tag, protected publishing, user self-hosted
runner, user keyring, or on-device installed application was
modified by these new tests.


## 2026-10-27 — Mode B Windows-native and locale matrix (planned)

### Scope / repository scan

Mode B continues after source STABLE #40511403562:
157 JavaScript, 16 Python and 149 Rust tests PASS at
`3455ba8f`. That gate does not execute Windows-only
`windows_card.rs` tests on Ubuntu, even though the Windows candidate
workflow can run `cargo test --locked -p cfvigil` on `windows-2025`.
The owner's earlier Ubuntu visual acceptance is separate.

Risk-first targeted Stage 3 inventory: nontrivial **Windows-native**
`anchored_position` (critical physical bottom-edge invariant),
`SavedPosition` persistent JSON (upgrade/corruption), and
`NativeCard` default drag/anchor state; **native language**
`ACTIVE`/`supported`/`sanitize`/`menu`/`window_title`
(all 20 locales). No changes to public API, UI, credentials or
persistent data format are planned. Existing Windows real-world
drag/keyring/scroll/notification and native speaker semantic review
remain pending.

### Stage 4/5/6/7/8 exact branch and interaction plan

| Artifact / function | Positive branches | Negative and boundary variants | Categories | Planned test function count |
| --- | --- | --- | --- | ---: |
| `windows_card::anchored_position` | compact 92 and expanded 510, identical bottom edge for saved physical anchor; negative x/multi-monitor position; first reopen measured outer height | measured physical heights 0/1/92/510/511/other; bottom at 0, negative, i32 extrema; arithmetic overflow must return Err, not wrap | P, N1, N2, N9 | 3 |
| `windows_card::SavedPosition` | encoded same x/bottom exact after repeated serialize/restore; absent anchor as `None` | partial missing x/bottom, invalid/float/out-of-range coordinate; old null anchor and legacy fields cannot fabricate non-null position | P, N1, N6, N12 | 2 |
| `windows_card::NativeCard` | default has no persisted anchor, not dragging, no checkpoint | fresh state independent of another instance; no stale in-memory state after restart | P, N4, N5, N6 | 1 |
| `language::ACTIVE`, `supported`, `sanitize` | all 20 supported codes; 20 durable roundtrip locale choices; same account/settings not implicitly involved | empty/case/whitespace/unsupported/legacy locales fall back to en; malformed preference JSON rejects, missing locale defaults | P, N1, N6, N8 | 3 |
| `language::menu` | 20 locales × five native actions | unknown action falls back to CFVigil, unsupported locale defaults to English; no empty strings | P, N1, N8, N9 | 2 |
| `language::window_title` | all 20 × main/settings/history/about labels | unknown window fallback, unsupported locale English; title matches corresponding menu action for Settings/History | P, N1, N8, N9 | 2 |

Total **13 planned test functions**, each covers table-driven
independent assertions. Existing tests retain authority and are not
counted again. Windows work area behavior, DPI and true OS drag
interruption cannot be proven by pure coordinate functions; reserve
real Windows acceptance.

### Stage 10 intended mutation kills

- Change `bottom - measured_outer_height` to `+`, use top edge,
  or apply logical height instead of measured physical height:
  anchored screen-bottom invariant fails.
- Replace checked `i32::try_from` with unchecked narrowing:
  exact overflowing combinations must return Err.
- Swap persisted `x` with `bottom` or serialize only one coordinate:
  serialized roundtrip and corrupted legacy matrix fail.
- Accept unsupported locale on sanitize, break one of 20 action
  translations, treat `zh-Hant` as `zh-Hans` or alter fallback:
  native locale matrix fails.
- Tie native window title to a global/fixed English label:
  per-locale Settings/History/About assertions fail.

These are planned mutation **kills** until executed; no mutation
score is claimed. No new test suite runs are permitted before the
final full STABLE gate.

### Stage 11 and Stage 12 gates

After test implementation, trigger **once** GitHub-hosted
Ubuntu Preflight for all stable suites and separate **Windows
candidate** workflow to execute Windows-only Rust tests. The
Windows workflow also packages an unsigned candidate as previously
specified; this is not a production release, tag or R2 publication.
Document exact CI run and counts; do not interpret host-level
source tests as a physical Windows OS/owner smoke. No registered
FLAKY group, so no distinct runnable FLAKY suite exists.


### Stage 9–10 implementation and mutation map (Windows/native locale)

Windows-only test source commit `f05e722a` adds six
`#[test]` functions to `windows_card.rs`:

1. Physical bottom invariant at compact 92 and expanded 510 with
   negative/multi-monitor screen coordinates.
2. Physical outer height 0,1,92,100,510,511,1000 and first-reopen
   distinction from logical height.
3. `i32::MIN/MAX` bottom, `u32::MAX` height and exact
   checked-overflow/fail-closed boundaries.
4. Repeated serialized `x/bottom` roundtrip at both signed extrema.
5. Null legacy anchor and invalid/missing/float/out-of-range saved
   coordinate shapes reject instead of fabricating an active anchor.
6. Two fresh native card sessions have independent drag,
   persistence/checkpoint and anchor state.

Native locale source commit `243cc6c0` adds seven Rust test
functions to `language.rs`: all 20 language codes; invalid/
noncanonical locales; durable 20-locale JSON roundtrip; defaulted
legacy/malformed preference cases; **all 100 native menu labels**
across five actions; unknown action/fallback; and **all 20 × 4
native window titles**. All assertions use production logic and
explicit expected translations; they establish consistency and
correct code wiring, **not native-speaker semantic certification**.
They do not modify any localized presentation strings.

Category/function counts for this 13-function tranche (overlapping,
not a claim of 13 unique N-exclusive paths):

| Category | Authored test functions | Evidence/limits |
| --- | ---: | --- |
| Positive | 11 | Defined translations, anchor behavior, saved state |
| N1 | 7 | Invalid action/locale/JSON/type/coordinate |
| N2 | 2 | Height, integer boundary and overflow |
| N3 | 0 | No network dependency in these artifacts |
| N4 | 1 | Restart state/idempotence |
| N5 | 1 | Independent session state, not real OS race |
| N6 | 4 | Saved JSON, older state, malformed locale |
| N7 | 2 | Fail-closed locale/action path input |
| N8 | 7 | 20 locale menu/title/preference variants |
| N9 | 4 | Restored size, display title, navigation state |
| N10 | 0 | No credential/log paths in these methods |
| N11 | N/A | Product has no monetization |
| N12 | 1 | Corrupted persisted position; OS disk failures covered elsewhere |

No new test-only DI, new production symbols or public contract change
was necessary: existing `anchored_position` and native language
functions were suitable as implemented. Intended operator mutations
listed in the preceding Stage 10 section now have direct value-
and state-specific assertions. An automated Rust mutation tool
has **not** been run, so no numerical Rust mutation score is earned.

**Status before STABLE gate:** authored, not yet executed. Windows-only
test code cannot run on Ubuntu; both Linux Preflight and Windows
candidate workflow `cargo test --locked -p cfvigil` are required.
No known registered FLAKY group exists; keep physical Windows
screen/UI acceptance and 20-language human editorial audit OPEN.


## 2026-10-27 — Mode B: confirmed persisted-shape failures in Linux and Windows CI

Evidence from exact SHA `a88832d9b7f3a29f4ff8291f53143d970bb8f09b`:
- Ubuntu Preflight [#40513445192](https://github.com/morozhenko-apps/cfvigil/actions/runs/40513445192): **155/156 Rust tests**, `language::tests::given_legacy_or_corrupt_locale_json...` fails on `[]`.
- Windows native [#40513445237](https://github.com/morozhenko-apps/cfvigil/actions/runs/40513445237): **161/163 Rust tests**, same `language.rs` failure plus `windows_card::tests::given_corrupt_or_legacy_saved_position...` fails for `{}`.
- Earlier source-only JS/Python tests and Rust check reached PASS, but **the full gate FAILED**; Windows NSIS packaging was not reached.

### Root-cause protocol (R1–R5)

Expected: serialized language preferences use an object `{"locale":"pt"}`; the empty **object** `{}` is a deliberate legacy-compatible default to English, while an empty **array** `[]` is malformed and must be quarantined. Windows position uses `{"anchor":null}` before a drag, or `{"anchor":{"x":-40,"bottom":700}}` afterward; a missing required `anchor` key must not be silently treated as an intentionally absent anchor. Existing `storage::read_or_default` quarantines files **only if deserialization returns Err**.

Candidate explanations evaluated: serde derive's sequence visitor; implicit serde default for `Option<T>`; a normal zero-length file; storage error swallowing; Windows path normalization; JSON wrong-type coercion; incorrect test expectations. The logs identify the first two as the reproducible causes; both operating systems show the first defect, so this is not a Windows path-specific failure. Blank files and arbitrary filesystem errors are not involved in the observed tests.

**Mode B remediation plan before coding:** introduce *map-object-only* deserialization for `LanguagePreference` and Windows `SavedPosition`, preserving the existing **Serialize** output, persisted field names, in-memory state, default `{}` language compatibility, and valid `{"anchor":null}` Windows preference. Reject JSON arrays, wrong field types, and `SavedPosition` maps with no `anchor`. Unknown extra object fields remain accepted for forward/legacy compatibility. This is an input-validation repair, NOT a persistent data-format or public API migration. No app ID, username, keyring, locale string or window geometry change.

Test branches: `{}` language -> en; `[]` language -> parse Err; valid 20-language map roundtrip; invalid locale strings still sanitized -> en; `{"anchor":null}` -> None; `{}` Windows position -> Err; partial/malformed coordinates, bad scalar/sequence, overflow -> Err; valid negative/extreme positions -> same roundtrip. Add `storage::read_or_default` integration proving malformed files are quarantined intact and not silently treated as valid empty preferences.

**Stage 10 intended mutations:** allow `[]` as empty language; strip required `anchor`; reinterpret null as default coordinates; silently ignore wrong-type locale; disable corrupt-file quarantine. All must be detected by assertions at the final STABLE gate.

The Linux and Windows compile/test failures remain **OPEN** until fresh same-SHA GitHub-hosted stable Rust and Windows-native Rust suites pass. Do not run intermediate suites per commit or claim Windows package PASS. No real Cloudflare, R2, `main`, release tag, owner runners or credentials required.


### Stage 9 implementation completed; verification now required

**Production parsing root cause fixed** at `f10b2f6b`:
`LanguagePreference` and `SavedPosition` use
`#[serde(try_from = "serde_json::Map<String, serde_json::Value>")]`
to require JSON objects, rather than allow `serde` to deserialize
empty `[]` as a field sequence. The exact existing serialized JSON
shape is unchanged. Language `{}` remains the documented default
`en`; Windows `{"anchor":null}` remains its default-on-first-run.
A Windows map with no `anchor` is rejected, not treated as a
deliberate null position. Unknown extra fields are accepted, preserving
valid old files. Existing `storage::read_or_default` quarantines
invalid bytes without silently replaying foreign or fabricated state.

**New cross-platform regression** in `storage.rs` writes invalid
language `[]` to a real disposable JSON file, requires quarantine,
verifies original bytes intact, writes valid `pt` and verifies reload.
**New Windows-native integration regression** in `windows_card.rs`
writes invalid `{}`, verifies quarantine and no old anchor, writes
valid negative X/bottom coordinates and verifies recovery. Commits:
`f10b2f6b` production format hardening and Linux integration,
`17186b4b` Windows persisted-fault integration.

One new cross-platform and one Windows-only Rust test function:
P=2; N1=2; N2=0; N3=0; N4=1; N5=0; N6=2;
N7=1; N8=1; N9=1; N10=0; N11=N/A; N12=2.
Counts overlap: these are per-function category allocations, not
statement/branch profiler coverage. The previously failed 2 source
tests must also be verified unchanged on both OS.

**Stage 10 checks:** turning map-only `Deserialize` back into
plain derived structs must fail the `[]` language case; deleting the
required `anchor` key check must fail the `{}` position case;
bypassing quarantine or overwriting broken bytes fails integration
assertions; rejecting `{}` language or `{"anchor":null}`
Windows would break documented backwards compatibility tests.
No automated Rust mutation run is claimed.

**Status:** committed but STABLE not yet executed for these changes.
Run same-SHA Ubuntu Preflight and Windows candidate workflows once;
Windows native gate must reach Rust test PASS and NSIS packaging,
while user-machine/Cloudflare/R2/signed release gates remain
explicitly OPEN.

## 2026-10-27 — Mode B HTTP quota observation contract (planned)

**Baseline:** dev `239ab1457f`, Ubuntu Preflight #40534964282 and Windows
candidate #40534964001 both SUCCESS. Previous malformed persisted JSON
regressions are now build-verified, but that does not close native/live gates.
Mode B remains IN PROGRESS.

### Stage 1–2: risk and production owner

The production `api_budget::ApiBudget::observe` consumes a real
`reqwest::Response`, accepts only HTTP 429 as a throttling signal and
applies a bounded Retry-After. Earlier tests reach the parsing helper
directly and existing provider suites cover some 429 paths; this
slice proves the **actual response-to-budget boundary**, without
mocking the function under test.

Invariant: 200/400/401/403/500/503 cannot trigger rate-limit
state or a retry; a real 429 with Retry-After must block further
admissions and expose no response-body secret; a new poll budget must
not inherit another poll's rate-limit flag.

### Stages 3–8: inventory, branch/negative and interaction matrix

| Production path | P | N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9 | N10 | N11 | N12 | Planned test |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ApiBudget::observe` non-429 statuses through real HTTP | ✔ | ✖ | ✖ | ✔ | ✔ | ✖ | ✖ | ✔ | ✖ | ✖ | ✔ | ✖ | ✖ | 1 matrix test (6 response statuses) |
| `ApiBudget::check/observe` on real HTTP 429 | ✔ | ✖ | ✔ | ✔ | ✔ | ✖ | ✖ | ✔ | ✖ | ✖ | ✔ | ✖ | ✖ | 1 contract test (admission, Retry-After, stop, redaction) |
| Two distinct per-poll `ApiBudget` instances with 429/200 | ✔ | ✖ | ✖ | ✔ | ✔ | ✔ | ✖ | ✔ | ✖ | ✖ | ✔ | ✖ | ✖ | 1 isolation test |

`✖` rationale: N1 invalid header and N2 malformed/overflow parsing are
already addressed by the existing bounded `parse_retry_after` matrix;
these particular HTTP probes deliberately supply valid response types.
N6 persisted data, N8 locale/time, N9 UI and N12 disk faults are
outside this in-memory HTTP-observer boundary. N11 is not applicable
to this free product. N5 means *independent polling cycles*, not a claim
to model all concurrent in-flight requests or native process scheduling.

**Interaction plan:** issue reqwest GETs only to the existing
127.0.0.1 `FixtureHttp` server; assert exact observed statuses,
count, absence of speculative follow-up calls, `retry_after_seconds`,
`exhausted`, native error constant, and independent budgets.
No Cloudflare access, credentials, OS keyring, production behavior,
public API or persistence changes.

**Stage 9:** three new `#[tokio::test]` cases in
`apps/desktop/src-tauri/src/api_budget.rs`; use existing HTTP fixture
and actual reqwest client. Do not change `test_http.rs` or shared runtime.

**Stage 10 mutation plan:** changing the 429-only predicate to
4xx/5xx, ignoring Retry-After, forgetting to mark `limited`, returning
OK from `check` after 429, or accidentally sharing budget state
across polls must break at least one test. These are designed kills;
no mutation runner or score is claimed.

**Stage 11 validation:** after the single source-test commit, request
exactly one dev Preflight by updating `ci/run-preflight.txt`.
The full GitHub-hosted stable JS, Python and Rust gate must pass on the
same head. Run no intermediate CI and no self-hosted runners.
There is no declared separately executable flaky suite.

**Stage 12 limitations:** even a green CI does not close real Cloudflare
authorization, installed KDE/Windows smoke, per-locale human review,
signed APT/R2 publication or release-owner approval. Rollback: revert
the one test-only commit, the marker and this separate documentation
commit as appropriate; no production or user-data restoration.
