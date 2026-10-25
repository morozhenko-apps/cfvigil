# CFVigil native-machine checkpoint before Mode B

## Current checkpoint (2026-10-25)

The original BLOCKED/NOT EXECUTED introduction below is historical. Later
records establish **v5 KDE geometry/Settings/position PASS only**. The overall
native-machine gate remains IN PROGRESS, with real Windows, locale/editorial,
security/credential/offline, connected-provider and release cases still open.
The [connected smoke](NATIVE_CLOUDFLARE_SMOKE.md) now records actual Worker rows
and a partial result; old "NOT TESTED" connected summaries apply to their
checkpoint. Corrected Pages permission warnings, verified partial counters and
polling/drag source require installed retest. Pages Read is now owner-verified;
the later HTTP 400 request/pagination defect must not reopen that permission
question. The corrected 20-item query is not yet live accepted. Do not replay a completed geometry
fix or infer missing scopes, a successful Pages response, yellow-state capture
or incident correctness without evidence. Preserve all earlier FAILs and cases.

The [roadmap](ROADMAP_TO_1_0.md) and [delivery record](DELIVERY.md) retain order:
no broad test expansion or Mode B until explicit owner machine acceptance.
This editorial clarification triggers no package, installation or CI marker.


Status: **BLOCKED / NOT EXECUTED**. Source and hosted CI evidence is not
proof of a usable KDE/Windows application. The owner requires an actual
machine inspection **after feature/localization implementation and initial
stable tests, but before broad test-coverage expansion and Mode B**.

## Dependency chain (do not reorder)

1. Freeze remaining 0.8 strings, implement **all 20** catalogs and check
   structural/parity/CLDR/RTL basics with the already established Preflight.
2. Produce deliberate, **unsigned, unreleased** candidate artifacts from
   the exact same `dev` commit, with no release tags or R2 publication:
   `ci/run-package-candidate.txt` and `ci/run-windows-candidate.txt`.
   Those marker workflows use GitHub-hosted runners. Capture both run IDs,
   commit SHA, artifact names and SHA-256 verification.
3. On the owner's **actual Ubuntu KDE computer** (Wayland, X11 when
   available), and a real Windows 11 machine, install/run the candidates.
   Record screenshot or observable behavior and log evidence, with personal
   Cloudflare tokens and machine identifiers redacted.
4. Fix observed runtime/UI/installation defects **architecturally** in
   `dev`, keep code and documentation commits separate, re-run only the
   relevant gate(s), and repeat the machine inspection against the corrected
   candidate. Do not build regression coverage for unstable behavior.
5. Only after the owner has accepted recorded results, expand the stable /
   flaky/negative test matrix, then begin final 0.9 **Mode B**.

**Hard stop**: if either platform has an unverified blocking scenario, the
corresponding platform cannot be marked accepted and Mode B cannot begin.
A successful hosted build does not waive this gate. No automation tool in
this chat currently has control of the owner's KDE or Windows desktop;
the check must be performed there, not simulated as a successful run.

## Candidate retrieval (after those workflows exist and succeed)

Use the CI run URLs and exact SHA, then from a trusted local terminal:

```bash
gh run download <LINUX_RUN_ID> -R morozhenko-apps/cfvigil \
  -n cfvigil-linux-unreleased-candidate -D ./cfvigil-linux-smoke
cd cfvigil-linux-smoke
sha256sum --check ./*.sha256
# Inspect package name and dependencies before optionally installing:
dpkg-deb --info ./*.deb
```

Windows: download the `cfvigil-windows-unreleased-unsigned-candidate`
artifact for the **same source SHA**, verify its included SHA-256 using
PowerShell `Get-FileHash`, inspect the unsigned NSIS installer, then
install under a test user. No production API token in public logs.

Artifact retention is one day: deliberately initiate the candidate workflow
when the owner is able to examine it; otherwise the evidence expires.
Do not change self-hosted runner settings or use Alcotester Actions runners.

## Native acceptance checklist

| Area | Physical action and expected behavior |
| --- | --- |
| First launch | No crash; a tray icon appears; card does not steal focus/taskbar |
| Tray behavior | Left-click toggles the card; right-click menus act correctly; Exit exits |
| Card geometry | No visible app title; 5 items/counters/actions in RepoTicker order; drawer opens upward, search only when expanded |
| Data states | Unknown/unavailable stays grey, latest queued build yellow, latest completed failure red, latest successful attempt green; no uptime assumption |
| Accessibility | Keyboard list navigation, Escape, screen-reader labels, focus return, resize and 15 s auto-hide behavior as actually implemented |
| Cloudflare | Scoped read-only token, Worker/Pages discovery, correct per-build deep links; no token exposure or API writes |
| Incident UX | New incident blinks Resources, reveals exact affected item, acknowledgements and recovery/dedup persist |
| Analytics | Worker error spikes vs deployment failures, zone WAF scope and unavailable-metrics behavior distinguished |
| Languages | Change each of 20 languages in Settings, card/history/about/tray/window titles update, no app restart, no extra polling |
| Layout/fonts | Check narrow 390px card, long German/Polish names, CJK/Indic/Thai/Bengali glyphs, Arabic RTL and bidirectional identifiers |
| Persistence | Language, account, polling interval and events persist through restart; keyring token is protected; corrupt JSON backed up |
| OS integration | KDE Wayland then X11 fallback; Windows tray, autostart opt-in, install/upgrade/uninstall with settings-retention semantics |
| Shutdown/offline | Disconnect network, restart, return online; stale state not shown healthy, no duplicate notification storm |

## Evidence template

- Candidate exact SHA, Linux run ID, Windows run ID, installation version.
- OS name/version, desktop session (Wayland/X11), WebView version, architecture.
- Per-scenario PASS/FAIL/BLOCKED; observation and screenshot/log reference.
- Defects: reproduction steps, expected/observed, priority, fix commit,
  repeat-run result.
- Final owner decision: **PASS or NOT ACCEPTED**, timestamp and remaining
  risks. Do not use `Accepted` without a real machine result.

All stable/flaky regression coverage written **after** this checkpoint should
target observed and agreed app behavior. The preexisting basic unit/contract
checks remain necessary to build safe candidates.

## 2026-10-24: Alcotester native KDE smoke — FAILED, remediation 0.8.1

Owner installed the Linux 0.1.0 DEB from candidate run 40125762836;
SHA256 for DEB/AppImage matched the delivered sidecars. Actual machine
screenshots of the compact card and the Russian Settings window identify:

1. Card cannot be dragged in compact or expanded mode.
2. Position is not durable; top/bottom coordinates drift during collapse.
3. Compact native popup retains unwanted empty area above the metrics.
4. Settings window contains excessive unused vertical space.
5. Language selection takes effect on dropdown change, before Apply.
6. Icon provenance/rights are not yet documented (needs evidence, not guess).
7. Card lacks the 15-second compact-only idle auto-hide used by RepoTicker.

**Acceptance status: FAIL**. The eighth numbered item has no description.
Do not expand Mode B test coverage until an updated Linux candidate is
installed and these issues rechecked. Compare implementation against
RepoTicker's native Wayland drag and Layer Shell anchoring; avoid CSS-only
fixes for GTK geometry. Keep runtime/config changes separate from docs.

### Remediation batch after first live KDE screenshots

Source remediation includes the RepoTicker-compatible GTK/GDK Wayland
relative-pointer bridge (Apache-2.0; see NOTICE), persistent bottom/right
Layer Shell margins in dedicated `card-position.json`, explicit compact
geometry reallocation on show/collapse, pointer-background drag gestures,
15-second compact-only inactivity hide, localized Settings content-fit,
and a single Apply-only write path for language/Cloudflare options.

The old `0.1.0` installed binary is unchanged. **All fixes are UNVERIFIED
until new Linux/Windows candidate(s) are built and installed.** No Mode B
coverage may proceed while these reports remain open.

Icon provenance: the repository contains a 1,308-byte `icon.png` with no
original vector source, explicit creation record or asset attribution. The
same icon is NOT byte-identical to RepoTicker's current `icon.png`.
This does **not** establish that the icon is licensed or independently
created. Treat provenance as **UNRESOLVED**, review/replace it before release;
do not claim copyright clearance on the basis of source ownership alone.

## 2026-10-24: second Alcotester inspection — three native geometry FAILs

Source build #40129376669 was installed with SHA256 verification and ran on
the owner's KDE workstation. **Observed**: drag PASS; anchor retention FAIL;
compact card gap FAIL; Settings bottom gap FAIL; Apply-only language PASS;
15-second compact-only autohide PASS. The native-machine gate stays **FAIL**.

RepoTicker historical fixes were examined before changing more code:
- `801c74fb65`: serialize viewport and embedded WebView geometry on GTK's
  main thread; do not infer the new height from asynchronous callers.
- `e65e1dfa9d`, `79bae25eca`: change GTK minimum/maximum heights in the
  correct order to avoid transient invalid constraints.
- RepoTicker's final `apply_card_height` returns immediately when height
  is unchanged, whereas CFVigil's earlier `apply_height` repeatedly writes
  geometry, queues allocation and resets layer anchors even at the same
  height. CFVigil also resizes redundantly on each `show()`.
- `49287b9e60`: the popup must restore the *saved* compositor margins on
  show without introducing a fresh geometry transaction.

Implementation decisions: native GTK uses one idempotent main-thread
height transaction, with bounded viewport and WebView requests. A tray
reopen can queue compact resize **once** before show. Show only reapplies
stored bottom/right margins; it cannot resize again. Add opt-in geometry
logs to compare actual allocation with requested dimensions. Do not reset
user coordinates or use absolute Wayland positioning.

Settings root cause is distinct: `document.body.scrollHeight` includes
viewport height, so deriving a native window height from it creates
self-referential sizing, especially after a prior oversize. Measure the
actual last form element's DOM bounds instead and deduplicate native
resize commands. Keep the form scrollable where the translation exceeds
usable screen height.

**Retest both compact and expanded card**, repeated reopen, process restart,
and Settings on each language; do not mark any of these three PASS based
on CI or a source diff. Leave Mode B gated.

### Third candidate native retest protocol (AFTER successful build)

On Alcotester, exit the old `cf-vigil` process through the tray before
updating its .deb, then confirm checksum and install the matching candidate.
Launch the new binary with `CFVIGIL_UI_DEBUG=1` and capture stderr locally.
Diagnostic messages are opt-in, never include Cloudflare API tokens.

1. Open compact ten times, hide ten times, and note actual height vs requested
   `122`; inspect whether the five compact actions still have a top gap.
2. Move it to a distinguishable position, expand/collapse five times, hide/show
   five times, then exit/relaunch. The same bottom/right margins and the same
   physical bottom edge should survive every action.
3. Open Settings in Russian and English. Check footer at the intrinsic bottom
   margin without a large empty area; close/reopen and check again. Confirm
   language still changes only after Apply.
4. Record compact/expanded screenshots and the opt-in allocation/Settings-fit
   log. If a GTK allocation is oversized while its requested height is 122,
   fix the native surface; if allocation is 122 but WebView contains blank
   content, fix WebKit/DOM measurement instead.

Source CI is not the result of the above physical experiment. Mode B
remains blocked until actual owner-observed results are captured.

## 2026-10-24: third Alcotester physical smoke (candidate #40132472841) — FAIL

The owner installed the SHA256-verified 0.1.0 DEB. Actual screenshot still
shows an approximately 200px compact card with an empty upper area and an
oversized Settings window; Settings native trace reports
`current=460x774 target=460x655 scale=1` but native shrinking does not
take effect. The window loses its previous placement on reopen and the
Settings titlebar has unwanted KDE window-management controls.

**Critical diagnosis from RepoTicker HEAD and its historic root-cause
fix**: RepoTicker's `configure_gdk_for_native_wayland()` runs before
`tauri::Builder`, enforcing `GDK_BACKEND=wayland` only in a verified Wayland
session with a nonempty `WAYLAND_DISPLAY`. It specifically records that a
GDK XWayland fallback yields a 390x200 window, position drift and ineffective
stored layer-shell margins. CFVigil lacks this pre-GTK configuration.
Additionally RepoTicker always logs the selected surface mode at startup;
CFVigil only emitted diagnostics from an enabled Layer Shell surface, so
absence of lines was not actionable.

Fix architecture:
- Port pre-Tauri native Wayland preference and unconditional backend report;
  preserve X11 fallback where Wayland is genuinely not running.
- Use the same native Layer Shell window for card positioning, not absolute
  Wayland XDG window coordinates. Saved compositor margins are authoritative.
- For Settings, eliminate conflicting GTK/WebKit minimum requisitions with a
  bounded native scrolled viewport and exact intrinsic form geometry.
  Remove platform decoration controls using a custom, accessible topbar
  with one close action and compositor-managed drag; do not paint faux
  controls over the existing KDE window frame.
- Keep Settings positioning stable on subsequent show/hide (and during
  content-size updates); no repeated centering or new window construction.
- Verify logs explicitly say `surface mode=gtk-layer-shell` for KDE Wayland,
  and compare card requested/actual height 122 before assessing gaps.

All symptoms remain FAIL until checked on another installed candidate.

### Next source candidate: native Wayland activation and Settings geometry

Owner's verified v3 screenshot and `CFVigil Settings fit: current=460x774
target=460x655 scale=1` demonstrate that ordinary Tauri geometry changes
do not force a smaller window. Only Settings fit logged; the previous
CFVigil Layer Shell allocation diagnostics were absent.

RepoTicker main.rs has a pre-initialization `configure_gdk_for_native_wayland`
function and unconditional surface-mode log. Its comment explicitly ties
the XWayland fallback to 390x200 card geometry, drifting and unsaved
Layer Shell anchor. CFVigil now ports these two features verbatim in
principle, only overriding `GDK_BACKEND` for an actual Wayland session
with a populated display socket.

Settings now uses a bounded native GTK ScrolledWindow around Tauri's
existing WebView VBox, with exact viewport/window constraints and
content-height requests; no new Tauri windows are created. Native OS
decorations are removed; a custom localized titlebar provides one
close-as-Cancel button and OS-managed window dragging.

Source and hosted CI do not prove either fix. The next Alcotester session
must confirm `CFVigil surface mode=gtk-layer-shell` and 390x122
compact allocation, preserved bottom/right position through repeat
toggle/restart, no Settings bottom gap, a single close control, and
window dragging. If native surface is still a fallback, inspect the
session/GDK report rather than iterating CSS.

### Alcotester fourth observation and compact/Settings visual polish

The owner confirms the KDE Wayland candidate now preserves card position.
Only a small remaining compact top gap is visible. Settings' undecorated
window works but its flat white surface blends into a light desktop.

The compact UI consists of fixed 70px action cells within 10px vertical
padding and 1px borders, so its canonical exact native and Tauri window
height is now **92px** (previously 122px); preserve the existing card
bottom/right margins and all drag and expansion logic. The Settings
surface now has an explicit contrasting background, dark neutral border,
subtle inset shadow and distinct header/content regions, without
introducing a second window or resetting its position.

Acceptance **not yet granted**: inspect the new Linux candidate, verify
there is no compact top gap and no clipping on EN/RU, that Settings has a
visible perimeter against a light browser/desktop, and that expand/collapse,
position retention and close/reopen remain correct. No Mode B before this.

## 2026-10-24: Alcotester v5 KDE geometry acceptance — PASS

Owner installed the unreleased Linux candidate from run
[#40137438889](https://github.com/morozhenko-apps/cfvigil/actions/runs/40137438889).
The download's `cfvigil_0.1.0_amd64.deb` and AppImage SHA-256 checks were
reported OK. The first install attempt was interrupted by sudo timeout;
the retry used the already downloaded files and the owner subsequently
confirmed the installed v5 UI behavior.

**Observed on the real Alcotester KDE machine, not inferred from CI:**
- **PASS** — remaining top blank space in compact 92px card eliminated.
- **PASS** — Settings background/border provide clear contrast against
  desktop and neighboring windows.
- **PASS** — card position remains preserved on hiding/reopening.

Earlier Alcotester confirmations that remain in force: native card dragging
PASS, Apply-only language change PASS and compact-only 15-second auto-hide
PASS. The previous FAIL reports remain above as historical failure/evidence,
not current geometry status. The RepoTicker-derived native backend and
geometry code should be frozen unless a new reproducible failure appears.

**Scope of acceptance is KDE geometry/interaction only. Overall native
machine gate remains IN PROGRESS**, not a Mode B approval. Still not
accepted: full connected Cloudflare/incident workflow, multi-locale visual
and RTL/font checks, accessibility/offline/persistence negative cases,
physical Windows installation and runtime checks, icon provenance,
and remaining release-readiness scenarios. Test these on installed builds,
fix observed defects, then expand broad regression coverage and Mode B
only after explicit owner acceptance of the machine gate.

## 2026-10-24: next physical checkpoint — connected Cloudflare smoke

Owner accepted the three v5 KDE geometry/Settings changes. The next
**required real-device** stage is a read-only Cloudflare connection and
navigation check on the installed candidate. The complete minimum-privilege
token setup, expected initial Workers/Pages resource behavior, status
comparison, browser navigation, history, persistence and redacted evidence
template are in [NATIVE_CLOUDFLARE_SMOKE.md](NATIVE_CLOUDFLARE_SMOKE.md).

The current status is **NOT TESTED**, not an inferred success from source or
CI. No production deploy failure or high-volume attack traffic should be
induced to exercise red incident states. Follow with native localization,
offline/keyring negative cases and physical Windows testing. Do not enlarge
test coverage or begin Mode B before the machine gate has an accepted
result.

## 2026-10-25: live Pages discovery accepted on Alcotester

The owner installed the Linux candidate and supplied a screenshot showing
the Pages catalog populated without the previous Cloudflare HTTP 400
8000024 banner. **Pages discovery: PASS (physical KDE observation).**
Worker discovery and the verified colored resource totals remain visible;
the screenshot shows 26 green, 1 red and zero in-progress resources,
plus grey rows. This does not verify the interpretation of a specific
red/grey state against Cloudflare's dashboard or a live build transition.

The overall native machine gate remains **IN PROGRESS**. Verify latest
build/deploy, navigation, history, persistence, offline behavior, language
rendering, and a real Windows installation before expanding regression
coverage and Mode B. Keep existing KDE geometry fixes unchanged.

## 2026-10-25: live latest-failure link verified

On Alcotester, the owner confirmed the red `bannermatch` entry opens
its **latest failed Cloudflare deployment** through the resource's action
menu. Record **PASS** for this one observed failed-state and latest-attempt
deep-link scenario. Pages discovery remains PASS. Dynamic build
yellow-to-green/red transitions, incident history/recovery, runtime
persistence, locale and Windows checks are still pending. The overall
native-machine gate is **IN PROGRESS**; Mode B remains blocked.
