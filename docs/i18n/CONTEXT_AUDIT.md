# CFVigil — 20-locale contextual audit ledger

**Status: PLANNED. No CFVigil translations or language-specific audits have
been completed.** This ledger will be populated in the final product-feature
milestone, after all other MVP functionality/distribution work and before
the exhaustive v1.0.0 test campaign.

The locale set is sourced from
`morozhenko-apps/PersonalLanding/src/app/i18n/language.ts`; do not substitute
the device/OS supported-language list or add/remove languages without an
owner-approved product decision.

| Locale | Display language | Context audit | Required attention |
| --- | --- | --- | --- |
| en | English | Pending | Developer terminology baseline |
| ru | Русский | Pending | Deploy/build distinction, plural forms |
| es | Español | Pending | Neutral software/monitoring terminology |
| pt | Português (Portugal) | Pending | **pt-PT only**, not Brazilian Portuguese |
| de | Deutsch | Pending | Technical compounds, clipped labels |
| fr | Français | Pending | Native technical terms, articles/elision |
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
