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

- Red: most recent attempt failed, or future confirmed error-traffic/security incident.
- Yellow: most recent attempt is queued/initializing/running, even if the previous attempt was red.
- Green: most recent attempt completed successfully.
- Grey: unknown/unsupported outcome, insufficient permissions, stale response, or unavailable information.

A published version serving traffic does not make a newer failing attempt green. No HTTP health checks or runtime application availability monitoring.

A zone-level security event must not be assigned to a Worker/Pages resource without reliable attribution. Future security monitoring should represent zone-level events truthfully.

## Notifications

An important incident brings up the compact card and highlights/blinks **Resources**. Selecting Resources expands the drawer upwards and focuses the affected resource. The user may open the corresponding Cloudflare screen via `⋯`, or ignore it. Repeated polls of an unchanged attempt do not create repeated incidents.

## Settings and tray menu

Right-click menu: Show Card, Event History, Settings, About, Exit.
Settings: Cloudflare account ID + user-scoped API token, refresh interval.
History: chronological local events and resource link.
The name CFVigil appears in About, not in the card.

## Excluded from MVP

GitHub CI, HTTP probing, automatic rollbacks, WAF management, request graphs, Telegram, paid services, direct edits of Workers/Pages, and a cloud relay.

## Planned before stable v1.0

Security/error traffic monitoring using available Cloudflare analytics, reliable alert thresholds and deduplication, clear zone/account scope, native KDE Layer Shell anchoring, native Windows smoke acceptance, signed APT/R2 distribution like RepoTicker, autostart, EN/RU UI, negative tests and a final Mode B QA campaign.
