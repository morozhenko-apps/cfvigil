# Localization contract

CFVigil uses one semantic English reference catalog and locale-specific message catalogs in the desktop UI. Native menu labels and WebView labels must remain consistent.

Keys are stable identifiers; Cloudflare resource names, status enums, URLs and stored incident identities are **not** translation keys. All interpolation is text-safe. Provider-controlled text must never be inserted as trusted HTML.

Date and relative-time formatting use locale-aware internationalization APIs. Plurals follow each locale's CLDR categories, including language-specific forms and safe fallbacks. Arabic supports right-to-left layout while technical identifiers stay directionally isolated. Keyboard and accessibility labels are localized along with visible text.

Translations must preserve placeholders, message meaning and the distinction between deploying, successful, failed and unavailable states. Use `scripts/check-i18n.mjs` and the localization tests for structural verification; language quality additionally requires contextual review. This document does not publish editorial work logs or unfinished-language status lists.
