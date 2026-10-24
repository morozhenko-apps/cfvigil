# CFVigil — roadmap to v1.0.0

Status: product scope and sequencing contract. `dev` is the working branch;
`main` is reserved for owner-approved stable integration/release.

This roadmap intentionally keeps **localization as the final product feature,
after all other functionality and distribution work, but before the complete
release-candidate test/audit campaign**. Target version numbers are milestones,
not claims that a feature has passed acceptance.

## Feature milestones

### 0.1 — Minimal monitoring foundation
- Tauri 2 / Rust desktop shell, tray card matching RepoTicker, upward drawer,
  search, lamp statuses, relative event time, and context menu.
- Connect Cloudflare via user API token in OS keyring; auto-discover Workers
  and Pages under the configured account.
- Latest build/deployment attempt determines the lamp:
  **new queued/running attempt = yellow even after a prior failed attempt**;
  completed failure = red; success = green; missing/stale = grey.
- No website availability probes and no GitHub Actions integration in CFVigil.
- Source-level prototype exists on `dev`; real Rust build/live Cloudflare
  integration and physical KDE/Windows acceptance are NOT yet proven.

### 0.2 — Reliable Cloudflare provider integration
- Validate Workers Builds, uploaded/manual deployments and Pages deployments
  against actual Cloudflare API response fixtures/live scopes.
- Latest-attempt ordering, page limits, unavailable API permissions, invalid
  credentials, rate limiting and offline recovery.
- Correct account boundaries and deep links directly to the failing
  Cloudflare deployment screen.
- Explicit unknown statuses instead of false healthy states.

### 0.3 — Alerts and event history
- Tray-triggered notifications that bring up the compact card and blink the
  Resources button, then focus the affected resource on upward expansion.
- Readable local incident history, bounded retention, deduplication, recovery,
  stale analytics and restart behavior.
- Right-click tray menu: Show Card, Event History, Settings, About, Exit.

### 0.4 — Error-rate and security monitoring
- Read-only Worker invocation error-spike detection using Cloudflare Analytics
  and conservative, sample-aware baseline comparisons.
- Zone-level WAF block-spike alerts only when API permissions/data allow it.
  Never falsely attribute zone traffic to a particular Worker; do not assert
  a WAF spike proves a successful attack.
- Transparent permissions, rate budget, incident prioritization and stable
  behavior when metrics are unavailable.

### 0.5 — Desktop polish and runtime resilience
- Native KDE Wayland anchored tray surface, X11 fallback, keyboard/focus,
  search, resize, scroll, visual accessibility and system tray lifecycle.
- Configurable polling and optional autostart; no redundant background queries.
- Native platform credential store checks, privacy/security audit, stable
  event persistence and log redaction.

### 0.6 — Distribution and developer documentation
- Follow RepoTicker's Apache-2.0 and NOTICE terms.
- Linux amd64 DEB and AppImage, signed APT publication, Windows x64
  installer, dedicated CFVigil R2 bucket `cfvigil-dist` and scoped credentials,
  controlled Cloudflare R2 path/key separation, deterministic
  versioning and checksums. Tags and optional metadata-only GitHub Releases
  reference R2-hosted downloads; GitHub Releases MUST NOT carry installers,
  checksums, APT repository files or binary attachments.
- GitHub Actions binary artifacts may be used only as temporary handoffs.
  Following successful R2 publication and *public byte-for-byte SHA-256
  readback*, delete the matching GitHub artifacts with Actions:write,
  scoped by source run, commit and artifact name; verify their deletion.
  If upload/readback fails, do not delete the only validated copies.
  Candidate artifacts use the minimum one-day retention, not permanent
  GitHub storage. Never reuse RepoTicker's R2 key or signing credentials.
- Bug-report-only Issue Form, private vulnerability link, no empty issues.
  No third-party PR/feature contribution intake; independent forks allowed.
- Physical installer/upgrade/removal scenarios can be explored here,
  but full release acceptance is after localization.

### 0.7 — Feature freeze and pre-localization readiness
- Resolve feature gaps, provider contracts and key architecture defects.
- All user-facing source strings are centralized and stable for translation.
- Implement diagnostics for missing localized keys, plural forms, date/time
  formats and right-to-left layout, without building additional product UIs.
- No more functionality is added after this freeze unless a blocking defect
  demands it.

### 0.8 — All 20 standard locales (final product feature)
Use exactly the same **20 UI locale codes as PersonalLanding**, preserving
their meanings and casing:

`en`, `ru`, `es`, `pt`, `de`, `fr`, `it`, `pl`, `hi`,
`zh-Hant`, `ja`, `nl`, `sv`, `tr`, `ko`, `bn`, `th`,
`id`, `fil`, `ar`.

`pt` is **European Portuguese (pt-PT)**, never automatic pt-BR copy.
`zh-Hant` is Traditional Chinese; do not silently replace it with
Simplified Chinese (`zh-Hans`). `ar` needs proper RTL, while code, URLs,
dates, IDs and shell commands remain directionally isolated as appropriate.

Localization must cover all user-facing app strings, not only the compact
card: counters, resource search, tooltips and menu actions; settings,
connection and permission guidance; history, notifications, error and
recovery text; about/licensing; installation/help messages and accessibility
labels. Never translate literal API status IDs, CLI commands or Cloudflare
product names incorrectly.

**Contextual audit for every one of the 20 languages**, performed after
translations and before exhaustive release testing:

1. Check conceptual meanings of Healthy, Deploying, Issues, failed builds,
   pending/running attempt, WAF blocks, error spikes, history and recovery
   (monitoring context; not uptime or proof of attack).
2. Verify natural terminology for actual developer users, including
   capitalization, grammar, case, plurals and culturally appropriate tone.
3. Validate European Portuguese wording; Traditional Chinese glyphs/terms;
   RTL Arabic ordering and punctuation; CJK/Indic/Thai/Bengali/Filipino font
   coverage and clipping.
4. Verify complete translation-key parity, interpolation, parameter escaping,
   plural/select forms, empty states, error-message consistency and
   non-leakage of raw English fallback where a translation is required.
5. Validate locale-aware relative time, dates, numbers, sorting,
   bidirectional isolation and window text width using screenshot or
   equivalent UI evidence on representative desktop configurations.
6. Record a per-language audit result and corrections in
   `docs/i18n/CONTEXT_AUDIT.md`; do not equate automated key checks with
   native-speaker editorial certification.

No locale can be silently omitted from the release build.

### 0.9 — Final Mode B QA and release candidate (after i18n)
- Full feature/branch/negative test matrix and known stable/flaky separation.
- Source/CI checks plus real-app tests for all 20 locales and language-context
  regression cases.
- Real Cloudflare API acceptance, credential lifecycle, incident dedup,
  permissions, offline/restart, security and notification state transitions.
- Physical KDE Wayland/X11 and Windows smoke, installer, upgrade, uninstall,
  signed APT/R2/download checks, untrusted data and accessibility checks.
- Final documentation/traces, observed test evidence and explicitly declared
  known limitations. No unverified claims.

### 1.0.0 — Owner-approved stable release
- Owner approves the validated release candidate, and only then is `dev`
  fast-forwarded into `main`. Release tag points to the exact `main` SHA.
- Publish checked artifacts and metadata only through the proven controlled
  distribution pipeline. Verify live download endpoints.
- Close any temporary feature PRs and remove their merged branches;
  the long-lived `dev` and `main` branches remain.
- No external developers added. Public bug reports and private security
  vulnerability reports remain the only upstream feedback channels.

## Project policies

License: **Apache License 2.0**, matching RepoTicker, with `NOTICE` and
proper attribution on forks. Distribution is free and open source.

Development: work directly in `dev` unless another agent is actively
modifying it; only then create a temporary branch/PR, merge, close and clean
it up when done. Keep code/config and docs-only commits separate.

No GitHub CI integration in the **application**. GitHub Actions is used only
for **building/testing/publishing this public repository**, using standard
free GitHub-hosted runners.
