# CFVigil product contract

Status: approved MVP UI/behavior, source implementation in progress.

## Intended audience

Developers maintaining Cloudflare Workers and Pages. A small read-only companion in the system tray, not a Cloudflare administration panel.

## Compact card

No product name or heading inside the card. The bottom strip has exactly five equally sized cells, in this order:

1. Healthy count
2. Deploying count
3. Issues count
4. Resources (toggle drawer)
5. Hide (hide card in tray)

The final two cells match RepoTicker's action-button style. The drawer expands **upwards** while the bottom edge stays anchored. No second title/header is inserted into the expanded view.

## Expanded drawer

A unified alphabetical list of detected Workers and Pages. No type column, branch, sections, Workers/Pages switcher or row expansion. Each row shows:

`resource name | event age | colored lamp | ⋯`

`⋯` opens only a targeted Cloudflare Dashboard action at MVP scope. The text filter is located **between the list and the compact panel**, visible only when expanded. All resources are auto-discovered; no manual resource enrolment.

## Status policy

For a given resource, inspect the **latest Cloudflare build/deployment attempt**, not the last successful deployment:

- Red: most recent attempt failed, a verified Worker invocation-error spike, or a verified zone-level WAF block spike.
- Yellow: most recent attempt is queued/initializing/running, even if the previous attempt was red.
- Green: most recent attempt completed successfully.
- Grey: unknown/unsupported outcome, insufficient permissions, stale response, or unavailable information.

A published version serving traffic does not make a newer failing attempt green. No HTTP health checks or runtime application availability monitoring.

Zone-level WAF monitoring adds a **temporary domain row only while a WAF block spike is detected**; it disappears from the resource drawer when the spike clears. Do not attribute zone security events to Workers/Pages without reliable evidence, and do not label mere WAF blocks as a confirmed attack. The context menu opens the zone Security Events page.

## Worker runtime error spikes (implemented source, unverified live)

Cloudflare GraphQL Analytics supplies Workers invocation request/error counts. To avoid the truncation of sampled raw rows, CFVigil requests each Worker using the documented `workersInvocationsAdaptive(limit: 1, filter: {scriptName: ...})` aggregate pattern, in bounded batches of eight Workers per read-only GraphQL request. Recent (10-minute) and baseline (preceding 30-minute) aggregates use a five-minute ingestion delay.

A fresh spike needs at least 100 requests, at least 10 errors and at least 10% errors. Against a meaningful baseline (100+ historical requests), the rate must be at least three times baseline and at least five percentage points higher. When baseline volume is too low, require 20+ errors and 20% errors. These are initial conservative thresholds subject to real-data tuning.

A repeating incident creates one alert. A missing Analytics response does **not** prove recovery: a previously red error-spike resource becomes grey until metrics are available again. On a new build attempt, a prior build failure no longer determines the status, but a simultaneously active runtime spike stays red.

The token requires **Account Analytics Read**, **Zone Read** and **Zone Analytics Read** in addition to Workers Scripts Read, Workers CI Read and Pages Read.

## Zone WAF block spike monitoring (implemented source, unverified live)

Cloudflare zone discovery is automatic, using an account-scoped Zone Read query. A short-time-window aggregated GraphQL query compares blocked WAF events from the latest 10 minutes with the preceding 30 minutes. To reduce false positives, an incident needs at least 100 recent blocks, a 5× increase relative to the equivalent baseline interval, and at least 80 additional blocks. Adaptive sampling and plan-dependent GraphQL responses require real-account validation.

A WAF incident displays a **domain row**, not a fictitious Worker-level attack. Its Cloudflare link opens `/security/analytics/events?action=block`. The row disappears when metrics confirm recovery, while the incident remains in Event History. Failed GraphQL queries preserve the previous zone alert as unknown (grey) rather than indicating recovery.

## Durable incident continuity (implemented source, awaiting CI)

The native app maintains an account-scoped `incident-state.json` journal
separate from `events.json`. It stores **only** a bounded list of ongoing
Worker execution-error and zone WAF-block incidents plus up to 512 recent
notification identities; API tokens remain exclusively in the operating
system credential store.

After process restart, ongoing incidents return as **unknown (grey)** until a
fresh Cloudflare read confirms whether they are still present. Old successful
resources are never restored as green. A temporary Analytics outage or
reappearance of the same incident must not generate another alert. Verified
recovery removes the active snapshot and re-arms future distinct incidents.

Changing Cloudflare accounts discards the previous account's incident journal
and notification identities; changing the token or polling interval within
the same account retains them. The journal is only rewritten when its meaningful
content changes, not on every poll. If persistence fails, monitoring continues
but a future restart may lose the duplicate-alert guard; the failure is logged
without credentials.

## Event History fidelity (implemented source, awaiting CI)

The native event log retains up to 500 recent records. Each event has a stable ID,
category (`incident`, `recovery`, `update`), timestamp, resource name/key,
message and **the original Cloudflare Dashboard link for that exact historical
attempt**. Clicking an older failed build must not open a newer deployment of
the same resource. URL checks remain on the Rust side.

Event History has search by resource/name/message and category filters for
incidents, recoveries and updates. Legacy `events.json` records without
IDs/categories are normalized in memory; the next persisted history write
preserves their new fields. Existing history records are not discarded.

## Notifications


An important incident brings up the compact card and highlights/blinks **Resources**. **No extra visual banner/toast overlays the five-cell compact strip.** Incident text is exposed through the Resources tooltip and an offscreen screen-reader live region; the expanded resource row has a detail tooltip. Selecting Resources expands the drawer upwards and focuses the affected resource. The user may open the corresponding Cloudflare screen via `⋯`, or ignore it. Repeated polls of an unchanged attempt do not create repeated incidents. A pending alert is stored in native process state until the user opens the resource list or hides the card, so notifications are not lost when WebView event listeners have not started yet.

## Settings and tray menu

Right-click menu: Show Card, Event History, Settings, About, Exit.
Settings: Cloudflare account ID + user-scoped API token, refresh interval.
History: chronological local events and resource link.
The name CFVigil appears in About, not in the card.

## Excluded from MVP

GitHub CI, HTTP probing, automatic rollbacks, WAF management, request graphs, Telegram, paid services, direct edits of Workers/Pages, and a cloud relay.

## Planned before stable v1.0.0

The authoritative phase order is in [ROADMAP_TO_1_0.md](ROADMAP_TO_1_0.md):
complete and validate all monitoring, incident, native desktop and distribution
features first; then add the **same 20 UI locales as PersonalLanding**
(`en`, `ru`, `es`, `pt`, `de`, `fr`, `it`, `pl`, `hi`,
`zh-Hant`, `ja`, `nl`, `sv`, `tr`, `ko`, `bn`, `th`,
`id`, `fil`, `ar`); perform a **contextual and UX audit for every locale**;
only then run the comprehensive Mode B QA, physical acceptance, version/release
audit and stable 1.0.0 release.

`pt` means European Portuguese; `zh-Hant` means Traditional Chinese;
Arabic UI must have correct RTL handling. A simple translation-key existence
check is not sufficient for the contextual audit.

CFVigil remains Apache-2.0, free and open-source, but upstream only accepts
reproducible bug reports through its Issue Form. No outside code contributions,
maintainers or feature request queue. See [CONTRIBUTING.md](../CONTRIBUTING.md).
