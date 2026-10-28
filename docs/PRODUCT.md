# CFVigil product contract

CFVigil is a read-only desktop companion for monitoring Cloudflare Workers and Pages. Supported platforms: Linux and Windows.

## Interface

The compact tray card contains five cells: successful, deploying, issues, resources, and hide. It has no duplicate application title. The resource list expands upward, with the bottom edge anchored; search appears between the list and card only while expanded. Each row contains a name, relative event age, status lamp and action to open its Cloudflare page. Long lists scroll; empty lists use their natural content height.

## Status

| Lamp | Meaning |
| --- | --- |
| Green | Latest relevant attempt verified successful |
| Yellow | Latest attempt queued, initializing or running |
| Red | Latest attempt failed, or a separately verified incident is active |
| Grey | Status unavailable, incomplete or unverified |

Pages statuses use **production** deployments, never Preview. Workers exclude explicitly identified pull-request Preview builds; uncertain provider evidence remains unverified. A newer running attempt supersedes an older failure, while a newer failure supersedes an older success. Deployment status is not a website uptime check.

Worker error spikes and zone-level WAF blocking spikes are independent observations. Missing analytics cannot prove recovery; a WAF event cannot be attributed to an individual Worker without supporting evidence.

## Monitoring and local state

Cloudflare requests use one bounded, read-only polling loop and the account permissions granted by the user. Independent provider failures do not erase verified information from other sources or create false success. Rate limits and disconnections preserve unknown states.

Settings are transactional: Cancel discards drafts; Apply changes only the selected fields. A newly configured account displays a pending-sync indicator until the initial request finishes or fails. Localized errors, notifications and searchable event history preserve actual provider outcomes. User-controlled language, polling interval and launch-at-login are supported.

The API token stays in the operating system credential store. Preferences, cached resources, incident journal and event history remain in per-user local storage. Switching accounts must not mix identities or incidents.

Windows upgrade and ordinary uninstall preserve local settings and credentials. Explicit **Delete application data** during Windows uninstall removes CFVigil user data and its credential; failures must be surfaced. Linux package removal does not automatically erase OS keyring entries.

The interface supports keyboard and screen-reader access, right-to-left layouts, localized labels and native window dragging. CFVigil does not deploy, edit Cloudflare configuration, track GitHub Actions or test site availability.

See [QA acceptance](qa/MODE_B_MATRIX.md) and [distribution contract](DISTRIBUTION.md).
