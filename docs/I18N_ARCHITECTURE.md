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
