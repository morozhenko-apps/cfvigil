# CFVigil: feature-freeze localization architecture (0.7)

Status: implementation in progress on `dev`; English baseline only.
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
