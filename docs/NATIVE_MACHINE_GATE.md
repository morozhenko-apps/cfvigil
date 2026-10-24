# CFVigil native-machine checkpoint before Mode B

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
