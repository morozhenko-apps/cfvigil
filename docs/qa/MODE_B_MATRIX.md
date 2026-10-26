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
