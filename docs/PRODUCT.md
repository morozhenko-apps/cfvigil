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

## Polling and network resilience (implemented source; native acceptance pending)

Exactly one asynchronous polling loop owns Cloudflare network refreshes. Saving
valid Settings serializes OS-keyring and account configuration changes with
in-flight refreshes, marks any previously verified resources unknown, then
notifies the existing loop through a coalescing Tokio watch channel. Settings
never spawn a competing refresh task; latest account/token state wins.

An ordinary successful poll uses the configured cadence of 1, 3, 5 or 10
minutes. A credentials failure or failed Workers/Pages discovery makes previous
data **unknown (grey)**, clears verified readiness, publishes that change to
the card and emits an error message. Repeated core failures back off
exponentially up to 30 minutes; updating settings wakes the poller immediately
instead of waiting for backoff. A successful core read resets the delay.

Cloudflare Analytics/WAF partial availability remains handled separately:
missing metrics cannot prove recovery and do not automatically mark a
successful independent deployment read stale. Polling never makes an uptime
probe, retries at sub-minute frequency, or launches overlapping network jobs.

## Partial API failures and request budget (implemented source; acceptance pending)

Workers, Pages, Workers Analytics and zone WAF analytics are independent
read-only sources. A failed resource catalog does not discard the other
catalog's verified results. Previously known resources from an unavailable
catalog remain in the drawer as **unknown (grey)**, rather than disappearing.
When any Workers/Pages discovery is incomplete, aggregate counters are
unverified, and the poller applies its normal error backoff. A malformed
single Pages deployment result is distinct from a valid empty array.
Unavailable build batches and per-resource deployment reads likewise keep
the affected Worker unknown, not green.

Workers Analytics reads are batched (maximum 8 Workers per GraphQL query).
One batch failing no longer cancels valid measurements from other batches.
An unresolved previous runtime spike becomes unknown only for its failed
batch. Zone WAF queries are likewise independent: an old alert may clear
only when that exact zone was successfully queried and no spike remains;
a different zone's failure does not hide a confirmed active WAF spike.

One **shared per-poll budget** is enforced across REST and GraphQL,
at most 120 outbound API calls in one poll. Requests exceeding the budget
are rejected locally with no HTTP retry storm. `429 Too Many Requests`
blocks any further requests in that poll, observes REST `Retry-After`
seconds (bounded to 60–1800 seconds), and extends the next poll delay.
The fallback for a missing/invalid Retry-After header is 300 seconds.
Normal successful API reads continue at the user-selected 1/3/5/10 minute
interval, subject to the shared budget; a provider failure may back off
further. Cloudflare's GraphQL-specific cost limits remain separate from
its global REST/API-token limit and are not represented as guaranteed
quota or rate-limit immunity.

Catalog processing is bounded to at most 256 Workers and 256 Pages
projects; exceeding the bound is reported as an incomplete provider,
never silently truncated to a fake complete result. Analytics retains
its 256 Worker scan cap, and WAF its 30-zone scan cap. A large account
may need a future explicitly designed scalable strategy; never
claim all resources have been verified if the bounded scan cannot finish.

Cloudflare's documented limits and REST headers:
https://developers.cloudflare.com/fundamentals/api/reference/limits/
and https://developers.cloudflare.com/api/resources/workers_builds/methods/get_latest_builds/.

## Optional launch at login (implemented source; physical acceptance pending)

Settings includes an opt-in **Launch CFVigil when I sign in** checkbox.
It starts unchecked for users who have not already enabled the operating
system's startup registration. CFVigil never silently turns on autostart
during installation or first launch.

The native Tauri 2 autostart plugin reads and updates Linux desktop-session
autostart and Windows user sign-in registration. Only Rust commands can
perform that action; there are no direct JavaScript autostart permissions.
The checkbox reads actual OS state, not a potentially stale JSON flag.
If it cannot be read, Settings disables the checkbox and leaves existing
registration untouched. Failure to update the OS setting is displayed to
the user rather than silently treated as success.

CFVigil starts hidden in the tray as it does on manual launch. An
autostart-only Settings change does **not** invalidate resource statuses,
reset incident deduplication or trigger an unnecessary Cloudflare request.
Changing credentials, account or polling interval still wakes the single
poller as before.

Native startup registration in KDE Wayland/X11 and Windows, desktop
login/reboot behavior, DEB/AppImage move/upgrade paths and disable/uninstall
cleanup must be physically checked before release.

## Keyboard and resource focus behavior (source implemented; desktop acceptance pending)

The expanded drawer uses native HTML buttons for each resource's single
Cloudflare action; Tab/Shift+Tab remain standard browser navigation. When
focus is on a resource action, Up/Down move between visible resource actions
(wrapping at either end), and Home/End jump to the first/last action. Down
from the search field moves to the first visible resource action. Enter or
Space activates the focused native button. The menu itself contains one
actual `menuitem`, is keyboard focused when opened, and Escape closes the
menu **before** it collapses or hides the main card.

The menu flyout is anchored to the five-cell card viewport, not to a child
of the resource list's scrolling region, so scrolling cannot cut it off.
When an update re-renders the list, keyboard focus is recovered by resource
key when still present. Incident focus resets a stale text search, reveals
the affected resource and puts keyboard focus on its action after expansion.
Without an incident, expanding the drawer focuses search. Closing the
drawer returns focus to Resources. A concurrent second resize is ignored.

All resource entries expose list semantics and status text to screen readers.
The Resources button reports `aria-expanded`, and the flyout trigger reports
its own expanded state. The compact strip remains **three counters and two
actions**, with its original dimensions and no extra banner.

The source-level tests cover arrow boundary/wrap rules and presence of
accessibility semantics. Keyboard/scroll/focus behavior on actual KDE
Wayland/X11 and Windows WebViews must still pass physical acceptance.

## Durable local state and recovery (source implemented; native acceptance pending)

Local app state uses three private JSON records in the operating system's
per-app configuration folder: `settings.json`, `incident-state.json`,
and `events.json`. The Cloudflare token remains **exclusively in the OS
credential store**; it is never copied into any JSON or recovery file.

All JSON writes are bounded to 4 MiB and use a fresh private temporary file
in the same directory, file flush and atomic rename. On Unix, the new file
mode is `0600`, and existing state files are restricted to `0600` when
read. An interrupted write must not truncate the previous complete record.

Malformed or oversized JSON is **moved intact to a uniquely named
`.corrupt.*` sibling** before any empty/default replacement is accepted.
The previous bytes remain available for manual recovery. Regular missing
files still use defaults on first launch; OS permission errors and unsafe
symlinked state files fail closed rather than being silently interpreted
as missing. A sanitized diagnostic is logged, and the startup snapshot
reports a non-secret recovery warning, shown accessibly on the card and in
Settings. No token or damaged JSON content appears in the warning.

Loaded event history is bounded to the latest 500 records; older valid
record formats remain supported. Local damaged-state recovery preserves
the original file but does not reconstruct data automatically or prove
that event history survived a prior disk failure. Recovery, file permissions
and interruption behavior still require physical acceptance on KDE and
Windows filesystems before distribution.

## Settings and tray menu

Right-click menu: Show Card, Event History, Settings, About, Exit.
Settings: Cloudflare account ID + user-scoped API token, refresh interval,
and optional launch-at-login registration (off unless enabled explicitly).
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
