# CFVigil — 20-locale contextual audit ledger

**Status: IN PROGRESS. Russian, European Portuguese, Spanish, German and French draft dictionaries are implemented, not context-approved. No native language-specific audits have been completed.** This ledger will be populated in the final product-feature
milestone, after all other MVP functionality/distribution work and before
the exhaustive v1.0.0 test campaign.

The locale set is sourced from
`morozhenko-apps/PersonalLanding/src/app/i18n/language.ts`; do not substitute
the device/OS supported-language list or add/remove languages without an
owner-approved product decision.

| Locale | Display language | Context audit | Required attention |
| --- | --- | --- | --- |
| en | English | Pending | Developer terminology baseline |
| ru | Русский | Draft dictionary; audit pending | Deploy/build distinction, plural forms |
| es | Español | Draft dictionary; audit pending | Neutral software/monitoring terminology |
| pt | Português (Portugal) | Draft dictionary; audit pending | **pt-PT only**, not Brazilian Portuguese |
| de | Deutsch | Draft dictionary; audit pending | Technical compounds, clipped labels |
| fr | Français | Draft dictionary; audit pending | Native technical terms, articles/elision |
| it | Italiano | Pending | Technical terms, gender agreement |
| pl | Polski | Pending | Cases, complex plurals |
| hi | हिन्दी | Pending | Devanagari fonts, natural IT terms |
| zh-Hant | 繁體中文 | Pending | Traditional characters, region-appropriate terminology |
| ja | 日本語 | Pending | Japanese developer terms and CJK layout |
| nl | Nederlands | Pending | Idiomatic software terminology |
| sv | Svenska | Pending | Error vs issue terminology |
| tr | Türkçe | Pending | Agglutination and locale casing |
| ko | 한국어 | Pending | Natural technical phrasing, CJK layout |
| bn | বাংলা | Pending | Bengali fonts, grammar and digit behavior |
| th | ไทย | Pending | Thai line wrapping and fonts |
| id | Bahasa Indonesia | Pending | Concise monitoring vocabulary |
| fil | Filipino | Pending | Natural Filipino usage, avoid English-only fallback |
| ar | العربية | Pending | **RTL**, LTR isolation for IDs/URLs, plurals |

## Required review record per locale

At localization time, record reviewer/date, dictionaries and version,
completeness (no missing keys/English leaks), context review for each
status/incident term, plural/interpolation/sorting/relative-time validation,
RTL/font/clipping/accessibility screenshots or equivalent evidence, defects and
their resolved commit SHAs. Distinguish automated checks from human editorial
review. Do not mark a locale `Accepted` without evidence.

**Critical distinctions across all locales:** successful build does not
guarantee service uptime; an in-progress newer build overrides the previous
failed attempt; Worker error spikes are not individual failed deployments;
zone WAF blocks are not proof of an attack or necessarily attributable to
a Worker; grey is unknown/unverified, not healthy.

## 2026-10-24: 0.8 implementation checkpoint

Complete baseline-key dictionaries have been committed for `ru` and `pt`
(European Portuguese) with no English-fallback inheritance. The `pt-PT`
`many` plural forms are included; both dictionaries remain **editorial
drafts**. Switching via Settings persists independently of account settings
in `language.json`, updates WebViews, native window titles and native tray
labels, and does not start a Cloudflare network refresh.

Source-level parity and representative content tests are not equivalent to
native reviewer sign-off. Run context audit, screenshot/font-width review,
relative-time, accessibility, RTL, plural and device acceptance when the
remaining 17 dictionaries are implemented. `en` is the baseline, not an
accepted target-language editorial review.

## 2026-10-24: next locale tranche and runtime regression safeguards

- Added complete source-level `es`, `de` and `fr` draft catalogs, including
  their CLDR plural forms (Spanish/French `many`, German `one/other`).
  Enabled them in WebView and native tray/window title mapping only after
  matching baseline keys and placeholder structure. Language selection is
  independent of Cloudflare credentials, refresh cadence and event storage.
- Repaired the initial-language IPC race. During asynchronous `get_language`,
  a newer `cfvigil-language-changed` event takes precedence. Added controlled
  tests for out-of-order result and bootstrap-read failure.
- History search now matches translated event names without rewriting saved
  legacy English records. Screen-reader labels use localized status names,
  including a distinct unverified/unknown state.
- Static source key/plural checks are **not** native-speaker editorial approval.
  Physical screenshots, width/RTL/font checks, localized native diagnostics,
  and real Cloudflare smoke remain pending.
