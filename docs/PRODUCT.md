# CFVigil product behavior

This document describes what CFVigil should do, rather than recording development activity. The implementation and tests determine exact algorithms and provider contracts.

## Purpose and interface

CFVigil gives developers a compact, read-only view of Cloudflare Workers and Pages in the desktop tray. The main card has five equally sized controls or counters, in this order: successful, deploying, issues, resources and hide. The title is kept out of the card.

The resource list expands **upwards** without moving the card's anchored bottom edge. Search appears between the expanded list and the compact card; the list has no separate application heading, resource-type column, branch column or per-resource collapsible sections. Each row shows the resource name, relative age, status lamp and a context action that opens the relevant Cloudflare page.

## Status semantics

| Status | Meaning |
| --- | --- |
| Green | The latest relevant deployment attempt is confirmed successful |
| Yellow | The latest attempt is queued, initializing or running |
| Red | The latest attempt failed, or a separately verified incident needs attention |
| Grey | Information is unavailable, unverified or too stale to establish a reliable outcome |

A new running attempt takes precedence over an older failed attempt. An earlier successful production deployment does not disguise a newer failed attempt. Cloudflare deployment status is **not** a website uptime check.

Worker execution-error spikes and zone-level WAF blocking spikes are independent signals. An unavailable analytics response is not evidence of recovery. WAF observations belong to the zone and are not attributed to a specific Worker without reliable data. Avoid describing WAF blocks as proof of an attack.

## Monitoring and notifications

Workers and Pages are discovered within the permissions of the user's chosen account and read-only token. Provider failures are isolated: one unavailable data source must not erase other verified results or produce a false green count. Queries are bounded and coordinated by one polling loop. Rate limiting, offline connectivity and partial responses preserve truthful unknown states.

Notifications identify meaningful new issues without repeating the same alert indefinitely. Event History is stored locally, is searchable and retains incident context across restarts. Historic resource names and provider identifiers remain opaque; they are never interpreted as code or trusted URLs.

## Settings and data

The account identifier, polling interval, language and optional start-at-login setting are controlled in Settings. Applying an unchanged field does not provoke unrelated credential writes or Cloudflare requests. Cancel discards unsaved changes.

The API token is held in the OS credential store, not inside configuration or history JSON. Local preferences, resource cache and event data use per-user application storage. Changing accounts must not retain incidents from the previous account.

**Privacy boundary:** Windows NSIS preserves the existing credential and per-user data during ordinary uninstall or upgrade. If the user explicitly checks **Delete the application data** during an interactive uninstall, NSIS removes the application's per-user folders and the single CFVigil generic credential from Windows Credential Manager. An absent credential counts as already clean; a deletion failure is reported to the user rather than silently claiming success. This needs native Windows acceptance before considering the behavior verified on an installed system. Linux package removal does not erase OS keyring entries.

## Product scope

Linux and Windows use the same application identity and behavior. The interface supports localization, keyboard interaction, accessible status labels and platform-native tray integration. CFVigil does not deploy Workers, edit Cloudflare settings, monitor GitHub jobs or assert site availability.

[Development checks](qa/MODE_B_MATRIX.md) and [distribution guarantees](DISTRIBUTION.md) document the relevant engineering contracts.
