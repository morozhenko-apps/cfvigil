# CFVigil — 20-locale contextual audit ledger

**Status: SOURCE CATALOGS 20/20; EDITORIAL AND PHYSICAL ACCEPTANCE PENDING. Russian, European Portuguese, Spanish, German, French, Italian, Polish, Dutch and Swedish draft dictionaries are implemented, not context-approved. No native language-specific audits have been completed.** This ledger will be populated in the final product-feature
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
| it | Italiano | Draft dictionary; audit pending | Technical terms, gender agreement |
| pl | Polski | Draft dictionary; audit pending | Cases, complex plurals |
| hi | हिन्दी | Draft dictionary; audit pending | Devanagari fonts, natural IT terms |
| zh-Hant | 繁體中文 | Draft dictionary; audit pending | Traditional characters, region-appropriate terminology |
| ja | 日本語 | Draft dictionary; audit pending | Japanese developer terms and CJK layout |
| nl | Nederlands | Draft dictionary; audit pending | Idiomatic software terminology |
| sv | Svenska | Draft dictionary; audit pending | Error vs issue terminology |
| tr | Türkçe | Draft dictionary; audit pending | Agglutination and locale casing |
| ko | 한국어 | Draft dictionary; audit pending | Natural technical phrasing, CJK layout |
| bn | বাংলা | Draft dictionary; audit pending | Bengali fonts, grammar and digit behavior |
| th | ไทย | Draft dictionary; audit pending | Thai line wrapping and fonts |
| id | Bahasa Indonesia | Draft dictionary; audit pending | Concise monitoring vocabulary |
| fil | Filipino | Draft dictionary; audit pending | Natural Filipino usage, avoid English-only fallback |
| ar | العربية | Draft dictionary; audit pending | **RTL**, LTR isolation for IDs/URLs, plurals |

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

## 2026-10-24: Italian, Polish, Dutch and Swedish drafts

The third locale tranche added standalone full-key `it`, `pl`, `nl`, and
`sv` dictionaries and matching native tray/window labels. PL category
`few` was corrected for 22/23/24 events; its `many` case covers 5/10/11.
`it` includes CLDR `many`; NL/SV use `one` and `other`. All six previously
available locales remain enabled. Source tests cover key parity, native/JS
locale parity and representative localized plural outputs. The planned set
still contains 20 codes; **10 active including English, 10 unavailable**.

Editorial/terminology review, actual translated Windows dialogs, native
Cloudflare diagnostic strings, font/width checks and screenshot evidence are
not completed. A successful source CI result does not confer native/editorial
acceptance.

## 2026-10-24: 20 of 20 source dictionaries committed, no editorial acceptance

The remaining Turkish, Indonesian, Filipino, Traditional Chinese, Japanese,
Korean, Hindi, Bengali, Thai and Arabic dictionaries were added to `dev`.
All 20 catalogs have exactly 91 keys, required interpolation tokens, correct
plural category sets, and exact PersonalLanding locale codes. The Arabic
dictionary implements `zero/one/two/few/many/other`; right-to-left direction
is set by the runtime and resource names are isolated in `bdi` elements.

All are **drafts**. Do not imply semantic validity, complete native diagnostic
translations, translated installation dialogs, correct native font metrics,
RTL interaction, screen reader acceptance, or successful installed execution.
The user explicitly requires a real Ubuntu KDE/Windows checkpoint between
initial source validation and broad QA/Mode B. Exact machine gate:
`docs/NATIVE_MACHINE_GATE.md`. No reviewer or device evidence yet.

## 2026-10-25: RU / pt-PT monitoring terminology source spot-check

Scope: targeted **source-level** terminology review only, commit `3f6f8a2`.
This is NOT a complete contextual audit, human-native review, window-width
or installed-app verification. Both locales remain **draft; audit pending**.
No other language, native Rust diagnostic, Cloudflare data/status contract or
persisted event message was changed.

Findings and source corrections:

- RU `card.deploying`: `Сборка` incorrectly implied only a Worker build,
  although the yellow counter also includes in-progress deployments.
  Now `В процессе`.
- RU accessible `card.optionsLabel`: `Настройки ресурса` described a
  non-existent settings operation; the menu contains a Cloudflare action.
  Now `Действия с ресурсом: {name}`.
- PT `card.healthy`: `Concluídos` could include failed completed
  deployments. Now `Sucesso`, tied only to the verified successful state,
  not to HTTP health or uptime.
- PT `card.statusUnknown`: `Por verificar` suggested a resource never
  checked; now `Não verificado`, also suitable for previously known but
  currently unverified/stale state.
- PT no-account guidance now says `Configure o Cloudflare nas Definições`,
  not `Ligue o Cloudflare`.
- RU/PT account placeholder specifies **32 hexadecimal characters**,
  matching native input validation, not just any 32 characters.

A focused regression test asserts these specific semantic choices and
checks full dictionary/parmeter/plural compatibility against the English
baseline. Native font-fit/accessibility screenshots, locale-specific
provider diagnostics, polished translations for the remaining strings,
new incident transitions, and reviewer sign-off remain pending.
Do not change the audit table to Accepted on the basis of this source
review or a green Preflight.
