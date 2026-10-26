# CFVigil: feature-freeze localization architecture (0.7)

Status: 0.7 source and Preflight completed. 0.8 runtime localization implementation started.
Only English is enabled; translation/editorial acceptance remains milestone 0.8.
Scope: centralize all desktop presentation strings without changing provider
status decisions, incident persistence semantics, Cloudflare query logic, or
existing saved event messages. The 20 locale translations belong exclusively
to the following 0.8 milestone.

## Architecture

- One English UI catalog at `apps/desktop/ui/locales/en.mjs`. Keys are
  semantic, stable dot-separated identifiers; application code never derives
  keys from untrusted Cloudflare names.
- `localization.mjs` is the presentation boundary: `t`, `plural`,
  `formatDate`, `relativeTime`, `localizeDocument`, and diagnostics.
  Missing keys and malformed interpolations fail fast in development/tests.
  No raw HTML is interpolated: rendered user/API values use `textContent`.
- HTML templates contain `data-i18n`, `data-i18n-title`,
  `data-i18n-placeholder`, and `data-i18n-aria-label`. The same catalog
  owns interactive, history, settings, accessibility and about labels.
- English is the *only enabled* UI locale in 0.7. The 20 planned locale
  codes are a future contract, not a claim that translated dictionaries
  exist. A locale must be activated only after matching keys and tokens.
- Translatable event meanings are independent of Cloudflare resource names,
  URLs, API status enums, history IDs and technical product identifiers.
  Existing Rust history message strings and `classify_history` stay byte
  compatible until a versioned event-code migration is designed.
- Date and age formatters use `Intl.DateTimeFormat` and
  `Intl.RelativeTimeFormat`, not English-only string assembly. Plurals use
  `Intl.PluralRules` and require explicit `other` plus available
  language categories. Formats and required interpolation parameters are
  statically checked.
- Layout direction is explicit on `html`. RTL-aware style and `bdi`
  isolation protect machine identifiers, resource names, URLs and timestamps.
  No automatic translation of Cloudflare API fields occurs.
- `scripts/check-i18n.mjs` is a source-validation gate, called by
  Preflight and by local tests. It validates HTML declarative keys,
  supported attribute kinds, catalog entry shapes and placeholder safety.
  Future dictionaries must also pass baseline key/parameter/plural parity.

## Delivery stages

1. Record the contract and risk boundaries (this document).
2. Refactor desktop presentation HTML/JavaScript to the English catalog;
   preserve exact monitoring behavior and tray card geometry.
3. Implement source diagnostics with negative tests for key mismatch, plural
   categories, interpolation mismatch, dates and RTL direction.
4. Update native presentation labels separately, without changing persisted
   incident messages or provider classification.
5. Run a single stable JS/Python/Rust Preflight. Keep `main`, tags, release
   credentials and Cloudflare R2 untouched.

## Risks / rollback

- A missing localization key can hide an accessible label. Prevent by an
  explicit validation gate and English text fallback only for known keys.
- Casing and translation width may overflow the 390px card. The 0.8 locale
  acceptance will include screenshot and font/RTL layout evidence.
- Do not mutate `events.json` or incident identity format as part of i18n.
  Future event-code migration requires backward compatibility for all old
  English messages and exact historic deployment URLs.
- Roll back the presentation commit if WebView or source tests regress;
  independent documentation and CI commits stay separate.

## Exit criteria (0.7)

Static and dynamic UI labels share a stable catalog, key diagnostics are
automated, date/age/plural and RTL formatting contracts are tested, and the
stable Preflight passes. This does not imply 20 completed translations,
native-speaker approval, a live Cloudflare API acceptance, or physical KDE
and Windows smoke evidence.

## Source evidence and implemented boundaries (2026-10-24)

- `locales/en.mjs` is the active English WebView catalog. The four HTML
  documents and dynamic app/history/settings/about text now resolve semantic
  keys through `localization.mjs`; names are inserted as text, never HTML.
- `native_messages.rs` centralizes the Rust menu labels, provider diagnostic
  literals and incident strings, including formatting helpers for runtime and
  WAF spikes. Legacy `events.json` messages and incident classification are
  unchanged, with a JS regression parity contract.
- `check-i18n.mjs` validates source keys, interpolation, HTML attributes,
  20 planned codes and every activated locale's dictionary/plural parity.
  The currently activated set is exactly `en`.
- Locale-aware `Intl` formatting uses explicit `pt-PT` semantics for
  future `pt`, and explicit `ar` RTL direction with `bdi` isolation.
  Native window titles and native menus remain English until multi-locale
  runtime wiring in 0.8; do not claim localized native menus in 0.7.
- No locale was translated or reviewed; no installed binary was physically
  checked; no history event codes or persistent schema were changed.
- Preflight (JS syntax, source checker, JS/Python/Rust suites) is the pending
  acceptance gate. No tags, R2 uploads, Worker deploys or `main` changes.

## Stage 0.8 incremental delivery (2026-10-24)

- Two complete non-English draft dictionaries: `ru.mjs` and `pt.mjs`.
- New native `LanguagePreference` in `language.json`, using existing private
  atomic JSON persistence and fail-closed language-code validation. Language
  changes emit `cfvigil-language-changed` without touching the provider poller.
- WebViews update their own locale and formatted timestamps when they receive
  the native language event. Settings allows switching before Cloudflare
  credentials are configured. The native tray labels and window titles update
  for the enabled locales.
- Validation now explicitly enforces the CLDR `many` plural category in
  European Portuguese. The initial CI gate correctly rejected missing forms;
  the catalog and negative/positive tests were repaired before rerun.
- The 17 remaining target-language dictionaries, native error message
  presentation localization, all editorial audits, screenshot/accessibility
  checks, and full release QA are not yet complete.

## 2026-10-24: 0.8 continuation, runtime consistency and locale batch

The 0.8 RU/pt-PT Preflight #40120200642 completed successfully at source level.
The next safe batch retains the existing language setting and history schema.
No release, packaging or Cloudflare provider behavior changes are authorized.

Implementation stages:

1. Repair the multi-WebView boot race: a native language-change event received
   during the asynchronous initial `get_language` must win over that older
   read. A failed initial read falls back to English only if no valid newer
   event was already observed. Keep the new behavior event-driven and add
   deterministic race-order tests, not arbitrary retries or waits.
2. Render translated event names in history search and translated resource
   status names in screen-reader labels. Preserve original provider strings,
   saved events, status IDs and historical URLs. Do not expose those strings
   as translation keys.
3. Add complete `es`, `de` and `fr` draft dictionaries to the 20-code
   contract, with locale-specific CLDR plural categories and native menu/title
   labels. Enable a language only when the baseline-key / interpolation /
   plural validation succeeds.
4. Keep source commits separate from docs, add automated source and localized
   behavior tests, then trigger a single stable Preflight with the existing
   explicit marker. Do not run experimental release or publish jobs.

Tradeoffs and rollback:

- A local language switch must never trigger Cloudflare polling, change
  account/token settings or modify incident history. The language reader is
  authoritative at bootstrap unless a newer native change event has arrived.
- No implicit English dictionary inheritance. Translation drafts undergo
  contextual/editorial review later; source completeness is not editorial
  acceptance. Layout/window width and fonts require physical smoke.
- If regressions occur, revert the independent locale/race batch and keep
  the previously validated EN/RU/PT runtime. Do not advance `main`.

Acceptance evidence: repository source parity; focused race/search/ARIA
tests; a single successful JS/Python/Rust stable CI run. Physical Windows,
KDE, RTL and native-review evidence are separate future requirements.

## 0.8 source continuation: five languages beyond English

`ru`, `pt`, `es`, `de`, and `fr` are fully keyed draft dictionaries.
The editor/UI can choose among those plus `en`, with native shell labels.
The initial-language read race now prefers a newer native event, and
translated event terms are searchable without rewriting stored incident
strings. The remaining 14 non-English language dictionaries are not yet
implemented. Full editorial, physical, RTL and release acceptance is not claimed.

## 0.8 third draft tranche (2026-10-24)

Complete independently authored `it`, `pl`, `nl` and `sv` draft catalogs
and matching native tray/window labels were added. Automated checks cover
91 catalog keys per language, baseline interpolation compatibility and
language-specific CLDR categories. The Polish event count uses distinct
`one`, `few`, `many`, `other` grammar; Italian `many` is present.
English is not inherited in any dictionary. Ten of twenty language codes
are now source-enabled, but no source-only result substitutes for an actual
native contextual / font / width / accessibility audit.

## 2026-10-24: all 20 source catalogs available

The planned locale set now matches the activated WebView and Rust `ACTIVE`
lists. All 20 dictionaries are standalone 91-key drafts with CLDR plural
categories and placeholders; no implicit English fallback. Native shell
menu and window titles are wired for each, while provider/backend diagnostic
errors and installer surfaces may still contain English. Source structural
checks passed; full CI/native machine acceptance must be assessed separately.
Owner-requested checkpoint `docs/NATIVE_MACHINE_GATE.md` strictly precedes
comprehensive stable/flaky/Mode B test expansion.


## 2026-10-26: Mode A, bounded Cloudflare diagnostics presentation

Status: IN PROGRESS. This is the next scoped 0.8 source milestone. The existing
20 activated draft locales, Tauri IPC messages, Cloudflare read-only behavior,
saved incident messages and package gates remain authoritative.

1. [x] Add explicit, exact-match presentation translations for the frequently
   surfaced Cloudflare network/JSON/credential/partial/stale/pending/rate-limit/
   discovery failures. Retain the original native English constants, status IDs,
   account/token state, event identities and polling cadence.
2. [x] Format **only** bounded numeric `Cloudflare HTTP NNN (code N)` diagnostic
   variants and their known Workers/Pages provider prefixes. Translate the
   display-only `code` / `check scope` labels in all 20 existing UI catalogs;
   never translate the numeric codes or the exact Cloudflare permission names.
   Treat unknown or future error text as opaque, never as a translation key.
3. [x] Route card provider warnings, sync messages, resource details and
   Settings command errors through one pure formatter. Preserve textContent
   rendering and live locale switching; do not reinterpret 400 as a 403
   permissions failure, hide warnings, or synthesize successful states.
4. [ ] Add deterministic positive/negative regression cases for RU/pt-PT,
   other activated locales, numeric HTTP 400/403 distinctions and unknown
   text; validate catalog parity and native constants. Run one final stable
   verification gate with the explicit `ci/run-preflight.txt` marker.
5. [ ] Preserve the native KDE/Windows acceptance checklist. This source
   milestone does **not** close broader provider/analytics/WAF diagnostic
   translation, localization editorial review, the KDE launcher icon,
   incident/offline/language device acceptance, or 0.9 Mode B.

Risks: unsafe free-text interpolation, silently treating HTTP 400 as missing
Pages Read, false translated success, or changing historical messages. The
formatter works solely on exact known native diagnostics and a strict numeric
HTTP grammar, leaves other text unchanged and only emits plain strings.
Rollback: independently reverse the one presentation/catalog/test commit;
there is no persistent schema or public Cloudflare contract migration.

### Implementation and validation checkpoint

Source: twenty complete draft catalogs gained ten known diagnostic labels
(`37a0662`, `f44e478`, `a18236f`, `91d12c6`), then one pure formatter,
card/Settings routing and regression tests (`754a358`). No native API call,
serialized JSON, incident classification, provider status or release gate changed.

First stable Preflight #40332875940 at `35afb88`: syntax, all source locale
catalogs and icon checks passed; Node 81 PASS / 1 FAIL. The sole failure in
`tests/polling.test.mjs` was an exact-source-text assertion expecting the
pre-localization rendering literal `entry.textContent = item`. The actual
updated renderer continues to use `textContent` safely but now invokes
`diagnosticText(item)`. Test expectation was updated to enforce that safe
localized path (`7649604`); no production code or test semantics were
weakened. Rerun requested once by `aef1a49` (#40333108575, queued at this
checkpoint). Python and Rust steps were skipped in the failed first run.
Full gate, native install, editorial locale acceptance and Mode B are OPEN.


## 2026-10-26: Mode A, resource detail and analytics/WAF presentation

Status: IN PROGRESS. Previous bounded HTTP/credential localization source gate
#40333108575 completed SUCCESS at `aef1a49` (after the stale test assertion
correction). The native-machine gate remains open; no Mode B or release.

Motivation: the current UI shows plain-English native resource details for
unverified Workers/Pages deployment reads and preserved Workers Analytics /
WAF anomalies. These are actually visible to users, unlike internal errors
from the Analytics/WAF GraphQL parsers, which are swallowed behind an
unverified status. Translating all hidden parser error strings without a
presentation path would create unused catalogs and a false completeness claim.

Plan and invariants:

1. [x] Introduce a small, exact-literal display catalog for the native
   resource details seen in the drawer: latest Worker build/deployment,
   latest Pages deployment, builds unavailable, missing/unverified latest
   deployment, malformed/unavailable Pages response, stale resource catalog,
   and unverified Workers Analytics / WAF. Include the known generic invalid/
   unexpected Cloudflare provider-response warnings, preserving prefixes.
2. [x] Extend each of the exact 20 existing draft dictionaries with the same
   baseline IDs and interpolations. Keep Cloudflare names, numeric status
   codes, URLs and resource identifiers unchanged; Arabic isolates
   directional identifiers. Unknown text must remain opaque, never HTML.
3. [x] Ensure the live language-change event refreshes the actual visible
   provider warnings and outstanding sync error without Cloudflare polling,
   changing incidents, or treating unknown as success. Keep the five-cell
   compact layout and Tauri native API untouched.
4. [x] Add focused positive/negative source regression tests for RU/pt-PT,
   all-locale parity, preservation of untrusted text and no HTML rendering.
   Run precisely one explicit stable Preflight after the completed batch.
5. [ ] Record actual checks, deficiencies and the still-open real KDE/Windows,
   translation-editorial, accessibility, launcher-icon and Mode B gates.

No native API, scope, serialized state, status classification, event journal,
Cloudflare querying, HTTP permission handling, publication, `main` or release
change is authorized. Rollback: reverse only the standalone UI/locales/tests
commit(s); no schema or provider migration is needed.

### Scoped implementation checkpoint (awaiting final stable source gate)

All 20 draft catalogs now contain 13 additional exact-match native
resource-detail IDs (source commits `969bdf1`, `b88bda6`, `e429d9f`,
`25c1e80`): latest Worker build/deployment and Pages deployment,
missing/unverified deployment attempts, malformed/unavailable Pages result,
unverified stale resource catalog, preserved Workers error spike, and
preserved WAF alert. Known invalid/unexpected Cloudflare response tails
are localized under existing bounded provider-prefix handling.
The source catalogs now have 114 message IDs per locale (91 initial,
10 previous diagnostics, 13 new). These translations remain editorial
DRAFTS pending on-device text/RTL/font review.

Pure WebView diagnostic mapping and runtime refresh were integrated with
focused RU/pt-PT/other-locale native-contract tests (`25e8767`).
Existing native English literals, persisted event bytes, request budget,
Cloudflare read-only access, poller, status and recovery rules are unchanged.
The live locale event recomputes provider warnings and outstanding sync
errors without an API call. Unknown/provider-specific messages stay
uninterpreted and rendered through `textContent`. Error HTTP status/code
and exact Cloudflare permission names are still preserved.

CI marker remains to be requested once for the complete batch. No stable
Rust, Python, desktop reinstall or Windows/KDE smoke is claimed yet.
Remaining native parse-specific messages and physical/editorial checks
are explicitly outside this source milestone.


### First Preflight failure and focused correction

Explicit Preflight #40374098821 at `d3b83bf` completed FAILURE:
JavaScript syntax, 20-locale structural/i18n checks, and icon checks PASS.
JS tests found exactly two stale source-test assertions (other stable
JavaScript cases passed); Python/Rust steps were skipped by the job.

- Existing unknown-diagnostic test still classified `Workers: unexpected
  Cloudflare response` as intentionally opaque. It now has an explicitly
  translated exact native warning suffix. Preserve the unknown-message
  security test using `Workers: unrecognized Cloudflare response`.
- Existing renderer source test still expected `payload ?
  diagnosticText(payload)` inline. The new implementation stores the raw
  `lastSyncError` for correct recomputation on language changes. Assert the
  new rendering path instead.

Both tests were corrected in source commit `5802fa0` without modifying
the native/UI implementation or weakening unknown-message/HTML safety.
The final full stable gate still requires a fresh explicit run; no success
is inferred from the earlier failures.
