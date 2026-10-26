# CFVigil Cloudflare-connected native smoke (before broad test coverage)

## Current checkpoint (2026-10-25)

The initial READY/NO LIVE PASS introduction below predates the actual connected
KDE probe retained later in this file. Worker discovery/search/green lamps were
observed; missing Pages and partial readiness produced an incomplete result.
Do not treat either old "not observed" text or partial success as full PASS.
Corrected source exposes redacted per-provider warnings, retains verified
Worker counters and normal polling while one catalog works, and fixes static
text selection/row dragging. Installed retest remains pending.

Read all original minimum-permission and redaction rules below. Pages 403 must
be distinguished from a verified 200/empty catalog; no token scope is inferred
from an incomplete screenshot. Counters describe resource states, not lifetime
build counts; a short running attempt may fall between polls. Do not induce
failures/traffic or broaden API rights for a synthetic red/yellow observation.
Latest evidence below verifies Pages Read and reports HTTP 400, not a 403
scope denial. The 20-item pagination and bounded numeric-error correction
await installed retest; earlier requests to recheck the scope are historical.
Overall [machine acceptance](NATIVE_MACHINE_GATE.md) is still IN PROGRESS;
Windows, incident, offline/keyring and locale cases retain their own gaps.


**Status: READY FOR MANUAL EXECUTION; NO LIVE CLOUD­FLARE PASS CLAIMED.**
Owner's previously installed and checksum-verified KDE candidate is Linux
Actions run [#40137438889](https://github.com/morozhenko-apps/cfvigil/actions/runs/40137438889)
(`0.1.0`, source SHA `913f5d8e382369947e7a22b38bd97027ad2daf81`).
Earlier KDE geometry, compact hide, and Settings Apply tests were accepted;
**connected-provider behavior has not been observed**.

## Access, credentials and safe test conditions

Use a **separate user-scoped, read-only Cloudflare API token** for the exact
Cloudflare account being tested. Account-scoped tokens cannot be used with
the Workers Builds API. Create a token in the authenticated Cloudflare
profile's API Tokens page. Restrict it to the account and zones under test.
The CFVigil Settings help presently requires:

- Account: Workers Scripts — Read
- Account: Workers CI — Read
- Account: Pages — Read
- Account: Account Analytics — Read
- Zone: Zone — Read
- Zone: Analytics — Read (Cloudflare's permission UI may use this label,
  also described as *Zone Analytics Read* in the API documentation)

Do **not** grant Edit/Write privileges. Use the 32-character Account ID
from the Cloudflare account Overview. Never commit, paste, screenshot,
screen-share or send the token itself to a chat or CI log.
The application stores the token in the OS keyring, not its JSON settings.
Do not run independent curl commands that echo tokens or save credentials
in shell history. Leave autostart disabled during the first smoke.

Cloudflare API reference:
- https://developers.cloudflare.com/api/resources/workers_builds/methods/get_latest_builds/
- https://developers.cloudflare.com/workers/ci-cd/builds/api-reference/
- https://developers.cloudflare.com/fundamentals/api/reference/permissions/

## On the installed Alcotester candidate

1. **Initial state:** start the installed `cf-vigil` normally (no sudo).
   With no configured Cloudflare account, expand Resources. Expect a prompt
   to connect in Settings, **not** fabricated green counts. Capture
   `NOT TESTED` instead if the account was already configured.
2. **Connect:** tray > Settings. Enter account ID + dedicated user API token,
   leave refresh at 3 minutes, keep autostart off, choose a locale. Press
   **Apply once**, verify the Saved response. If it fails, preserve only
   the *error text*, never the token or Account ID. Other preferences and
   token persistence should not be inferred from seeing "Saved".
3. **First read:** dismiss Settings, open the card, expand Resources and
   record approximate number of distinct Worker and Pages entries, three
   compact counters, and any grey Unknown items. The Settings save signals
   the existing poller; no manual application restart is necessary. Allow
   the API request to finish; temporary Unknown while data is unverified
   must **not** be marked a failure by itself.
4. **Compare actual Cloudflare dashboard:** choose one existing Worker and
   one Pages project (if available). Search/filter the list. Compare the
   *latest build/deployment attempt* in Dashboard with the app status:
   running/queued = yellow, latest failed = red, successful = green,
   unavailable/unverified = grey. A previous successful production deploy
   must **not** mask a newer failed build.
5. **Navigation:** for each verified Worker/Pages row, use its contextual
   `⋯` action and confirm the external browser opens the correct
   Cloudflare dashboard **for that resource/latest attempt**. The app must
   not open unrelated URLs or request write access.
6. **History:** on first connection, zero events can be correct because
   existing builds should not generate spurious *new incident* alerts.
   Review history search, event labels and empty states; do not fabricate
   incident evidence. Only naturally occurring later state transitions
   count as live incident tests.
7. **Refresh and persistence:** observe the next normal 3-minute update;
   close and reopen the card, then exit CFVigil through the tray and relaunch.
   Account ID, language, refresh interval and resource discovery should
   survive. The token field is intentionally empty on reopen: that means
   the secret is concealed, **not** that the keyring token was deleted.
8. **Partial scope / errors:** if Workers load but Pages do not (or vice
   versa), record both independently. If an API error appears, preserve
   redacted error text and time. Never infer full access or zero resources
   from a failed fetch. Do not deliberately break production deployments
   or generate malicious traffic to create red incident states.
9. **Unexecuted scenarios:** Worker runtime error spike, zone WAF block
   spike, detection/recovery deduplication, offline network failure,
   keyring lock, restart persistence of incidents, 20-language glyph
   and RTL, and Windows installation remain `NOT TESTED` until physical
   evidence exists.

## Optional nonsecret runtime trace

Exit existing CFVigil first. Launch its installed binary as the user:

```bash
APP=$(dpkg -L cf-vigil | grep -E '^/usr/bin/[^/]+$' | head -n 1)
CFVIGIL_UI_DEBUG=1 "$APP" 2>&1 | tee "$HOME/Downloads/cfvigil-provider-smoke.log"
```

The UI_DEBUG flag diagnoses native layout; **it does not guarantee any
per-request API tracing**. Do not imply that absence of an error line
confirms a successful Cloudflare request. Inspect/redact the log before
sharing it. In particular, remove tokens, account IDs, private hostnames,
resource names, query parameters and user directory paths as appropriate.

## Evidence record

| Scenario | Owner result | Evidence needed |
| --- | --- | --- |
| Settings account/token Apply and keyring | NOT TESTED | Saved/error state, no secret |
| Workers discovery | NOT TESTED | App list vs dashboard, approximate count |
| Pages discovery | NOT TESTED | App list vs dashboard, approximate count |
| Latest attempt and counter colors | NOT TESTED | A verified real Worker/Pages state |
| Per-resource dashboard navigation | NOT TESTED | Correct destination, URL redacted |
| History empty/initial state | NOT TESTED | Normal empty vs unexpected alert |
| Normal refresh and restart | NOT TESTED | Before/after observed |
| Unknown/partial error handling | NOT TESTED | Only if observed naturally |
| Incident transitions/recovery | NOT TESTED | Real transition; never induce failures |
| Windows native smoke | NOT TESTED | Installed unsigned Windows candidate |

### Gate outcome

**KDE geometry PASS, live Cloudflare connected smoke NOT TESTED.
Full native-machine gate remains IN PROGRESS; Mode B BLOCKED.**

## 2026-10-25: real connected KDE probe — partial failure

The owner confirmed an actual Cloudflare connection, populated Worker rows,
search and green lamps on the installed KDE candidate. After a manual
Cloudflare rebuild, the owner did not observe a yellow state or a change
to the counters. **Counters represent the number of resources in a state,
not the lifetime number of builds.** An already-healthy Worker succeeding
again legitimately leaves the green count unchanged. A short build can
also finish between polls, so the yellow interval is not guaranteed.
The resource's latest `changed_at` and attempt identity remain the
stronger acceptance evidence; they have not yet been compared with
Cloudflare for the reported rebuild.

The owner reports **zero Pages entries**, with no visible explanation.
The existing discovery code tolerates an independent Pages failure, but
drops its error status and publishes the Worker-only snapshot as
`ready=false`. Existing UI hides all three counters when `ready=false`,
even if individual Worker states are verified. Distinguish these cases:
(a) Pages Read scope missing -> 403; (b) Pages catalog legitimately empty;
(c) malformed/unavailable Pages response; (d) Pages names/data filtered out.
Cloudflare API confirms `GET /accounts/{id}/pages/projects` requires
Pages Read or Pages Write. The token screenshot earlier in the session
showed an empty final permission selector; scope must be verified in the
Cloudflare dashboard and never inferred.

Owner also reports selectable text in the card resource list and Settings,
interfering with the drag gesture. Source change should disable selection
for static UI text in all windows, while retaining selection/normal cursor
behavior in actual text-entry controls (search, Account ID, API token).

**Implementation ordering:** CSS selection fix, expose redacted per-provider
HTTP/scope diagnostics in the snapshot and expanded card (not only a transient
toast), preserve truthful verified Worker counters on partial API failure,
then compile and make another unsigned candidate. Do not invent a successful
Pages response, artificially increment status counters, or force more polling.
Do not begin Mode B until this installed result is rechecked.

### 2026-10-25 corrective source batch pending installed retest

The user confirmed Worker rows are visible. They also reported no Pages
entries, no observed yellow state or change in status totals after a rebuild,
and selection of static UI labels blocking drag.

Root cause confirmed in source: a denied Pages request was discarded when
Worker discovery succeeded; the snapshot then set `ready=false` for *all*
counters, even when Worker states were verified. The partial result also
returned `false` from `refresh_once`, triggering exponential backoff up to
30 minutes despite healthy Workers discovery. All of this may make a
successful rebuild appear to be ignored.

Changes: keep endpoint-specific *redacted* diagnostic lines for Workers,
Workers Builds, Worker deployments, Pages projects, Pages deployments in
each snapshot; show them atop the expanded drawer; keep verified
per-status Worker totals on partial success; use normal configured poll
interval while either provider catalog is available; reserve backoff for
full discovery outage or exhausted API budget. Removed static-text selection
from card/Settings/history/about, preserving actual editable text controls.
Allow dragging from non-interactive resource rows. No API writes or
Cloudflare credential exposure.

**Remaining owner check:** inspect the native warning banner. In the Cloudflare
token editor verify **Account → Cloudflare Pages → Read** is actually granted
(the preceding token screenshot's final scope was unselected). A 403 must
appear as a Pages permission error, not be presented as zero Pages projects.
If Pages catalog is 200 + empty array, check the account has Pages projects
under the configured Account ID. A newly completed build may not increment
any metric count if its Worker was already green; inspect the row's
`changed_at`/latest attempt rather than lifetime build counts. A yellow
interval shorter than the polling cadence may be missed.

The next candidate's source is **not yet accepted** on Alcotester.
Mode B and broad test coverage remain blocked.

### 2026-10-25: Pages Read verified, HTTP 400 requires request correction

The owner supplied the token's Cloudflare permissions. **Account /
Cloudflare Pages / Read is present.** The other options named
`Access: Custom Pages` and `Account Custom Pages` are not substitutes.
The native client reported **Pages: Cloudflare HTTP 400**, which is a bad
request response, not a confirmed missing-read-scope denial. The previous
diagnostic incorrectly attached `(requires Pages Read)` to every Pages
failure, including HTTP 400.

Source inspection found `list_paginated()` sending
`GET /accounts/{id}/pages/projects?page=1&per_page=100` and reusing that
aggressive default on subsequent pages. Cloudflare's documented Pages list
API permits `page`/`per_page` and its example uses 20 per page. The
project catalog is now queried with **20** items per page; pagination and
bounded resource/count limits remain enforced. The hypothesis is that
the provider rejected the previous oversized request. **Not yet proved
on the live account:** real API acceptance of the corrected query.

New diagnostics distinguish 400 from 403. Only 403 suggests checking
Pages Read; both include only Cloudflare's numeric HTTP and error codes
without copying free-form API error bodies, account IDs or credentials.
On the next installed Linux candidate: confirm Pages projects appear,
or report the new exact sanitized error line. Existing Workers and their
configured refresh cadence must remain unaffected. This is still a native
machine-gate failure pending another observed result, not Mode B acceptance.

### 2026-10-25: live 8000024 identifies rejected Pages list options

After installing the 20-item query candidate, the user observed
`Pages: Cloudflare HTTP 400 (code 8000024)` with verified account-scoped
`Cloudflare Pages / Read`. The code is associated with Cloudflare
`Invalid list options provided`; simply reducing `per_page` was
insufficient. This is a provider request-parameter defect, not evidence
that the user needs more permissions.

CFVigil now fetches the Pages project catalog from the **plain**
`GET /accounts/{id}/pages/projects` endpoint without `page=1` or
`per_page`. If the response advertises `result_info.total_pages > 1`,
the client requests the next pages by `?page=N` only, with existing
resource/page upper bounds. A rejected later page or an empty advertised
additional page remains an explicit failure; no partial catalog is called
fully synchronized. The latest Pages deployments endpoint is likewise
queried without optional list parameters and the latest returned attempt
is selected by parsed RFC3339 timestamp rather than trusting array order.
All calls remain GET-only; no token/account changes.

**Native retest still required.** Confirm Pages rows appear and the
8000024 banner disappears, or report new sanitized status/code. Do not
call this PASS based on source or hosted CI alone. Other known KDE
drag/position fixes are unchanged, Mode B blocked.

## 2026-10-25: installed KDE Pages catalog — PASS

After GitHub Linux candidate #40185446460 was installed, the owner shared
a live CFVigil screenshot confirming that previously missing Pages projects
now appear alongside Workers; the `Pages: Cloudflare HTTP 400 (code 8000024)`
banner is gone. The list visibly includes Pages-style project names such as
`bannermatch` and `chromabankshot`. The card displays 26 successful
resources, 1 failed resource and 0 running builds, with some individually
unverified grey rows. These are resource-state counters, not cumulative
build counters.

**Native Pages project discovery: PASS** on Alcotester (owner observed).
The previous source-only `NOT TESTED` / error paragraphs above are
historical and superseded for this scenario only. **Not yet accepted:**
latest-attempt transitions and build start/end timing, Pages deployment
deep-links, event history and recovery, full token/restart persistence,
offline/error handling, locale review, Windows installation. Do not infer
any of those from this screenshot or mark the full machine gate passed.

## 2026-10-25: latest failed Pages deployment deep-link — PASS

On the same installed KDE candidate, the owner opened the red
`bannermatch` row's context action. It opened the **latest failed
deployment** for that project in Cloudflare. This confirms an actual
resource-specific failed status and correct latest-attempt navigation,
rather than merely a link to a generic product dashboard.

**PASS (observed):** Pages discovery; one failed deployment state and its
latest-failure deep-link. This does **not** establish that an in-progress
yellow build was observed, that a new failure was captured during polling,
that all Worker/Pages deep-links work, or that incidents/recovery/persistence
passed. Those remain pending native smoke checkpoints.

## 2026-10-25: History native UI and search — PASS, restart remains pending

Owner screenshot shows History search for "bann", with 4 of 51 recorded
events, localized timestamps/descriptions and resource-specific
Cloudflare action controls. Owner stated the screen works. Mark the
rendering/filtering portion PASS only. Persistence after a complete
quit/restart, newly observed incident lifecycle and recovery remain
NOT TESTED. Do not synthesize missing failure history for an older
pre-monitoring build.

## 2026-10-26: owner-directed deterministic provider acceptance before candidates

Mode B source fixtures now verify HTTP 400/403/429 separation, bounded
request/backoff policy, absolute-time latest-attempt priority, failed-build
vs queued-new-build behavior, account-isolated WAF, and partial GraphQL
responses that **cannot** silently clear prior alerts. This is a
**controlled deterministic provider-contract check**, not a live API pass.

Previous owner-observed KDE Pages discovery and the latest-failure browser
link remain PASS. The still unobserved live dynamic transitions, Workers
Analytics/WAF actual samples, read-only token keyring/offline negative
states, and cross-platform incident recovery must be checked through
the newly built candidate without inducing a production failure.
No claim of live REST/GraphQL calls or user-token access in this batch.
