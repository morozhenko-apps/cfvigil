# Windows 11 native-machine acceptance, CFVigil 0.1.0

Status: NOT TESTED on Windows. GitHub-hosted NSIS build is not an installed-app PASS.

Source: Windows run 40224188456 (SUCCESS), SHA 1b944b0d137ddde952f1e52f4d627e28ed4d1369.
Artifact: cfvigil-windows-unreleased-unsigned-candidate, one-day retention.
Initial expiry shown by GitHub: 2026-10-26 19:18 UTC.
Linux candidate and Preflight passed with application code identical to Windows; comparing
commit e459478dcc960d3e2d3ad0ee7e85603819925a12 against the Windows SHA
shows only docs and the explicit Windows CI marker.

## Safe transfer, checksum, installation

On Windows 11, use ordinary PowerShell and an existing authenticated gh CLI.
Check the run conclusion first, download to a newly empty directory with
gh run download 40224188456 -R morozhenko-apps/cfvigil
  -n cfvigil-windows-unreleased-unsigned-candidate -D <empty-directory>.
Require one .exe, compare SHA256 with its .exe.sha256 sidecar using
Get-FileHash -Algorithm SHA256, and inspect candidate.json: release=false,
signed=false, platform=windows-x64, commit matching the above SHA.
Install only after the hash matches. This is an UNSIGNED private candidate.
Do not disable Defender or SmartScreen globally; if installation is blocked,
record that fact and keep Windows native acceptance open.

## Actual observed acceptance, after installation

1. Installer and app Start menu entry work without elevated privileges.
   Branded tray, Settings and History window icons display properly.
2. Compact card shows via tray, hides only when collapsed and idle;
   expanded list opens upward and can be searched. Move/reopen/quit/relaunch
   the card and check placement is retained.
3. Settings language and token apply ONLY on Apply; Cancel discards drafts.
   Check visible labels, window geometry and translated long strings.
4. With a dedicated read-only Cloudflare token, compare Workers/Pages
   discovery and last failed deploy link to Cloudflare Dashboard.
5. Verify Windows Credential Manager persistence without exposing token.
   History search, local events and config survive full exit and relaunch.
6. Brief offline/online smoke: do not mislabel unknown data green or
   multiply notifications. Restore connectivity.
7. Optional uninstall/reinstall smoke: no unrelated files destroyed;
   user data/keyring retention must be documented, not guessed.

Record PASS/FAIL/NOT TESTED with sanitized evidence. No production
deployments should be intentionally broken. No tags, main, R2 or Mode B
without owner acceptance of real native-machine evidence.

## 2026-10-25: Windows physical first-pass evidence (owner screenshots)

The first unsigned NSIS candidate #40224188456 **installed and ran on
Windows 11**; orange shield in tray and main application launcher works.
Cloudflare shows 26 successful resources, 0 deploying, 1 problem. Owner
reported six defects: repeated window downward drift, lost saved placement
after restart, incorrect click-to-hide behavior with expanded blank drawer,
long Settings Apply without spinner, invisible saved token status in the
password input, and an unexpected console window attached to launch.

The candidate is **FAIL** for native Windows acceptance. Its CI SUCCESS
only proves build/installability, not correctness. The fixes in subsequent
Windows source commits remain **UNTESTED** until a replacement NSIS is
installed on the same Windows machine. Keep original screenshots private;
never request the API token or log it. Re-run: compact->expanded->tray hide
->tray show ten times (no drift/blank content); move then quit/relaunch
(placement retained); input Apply with slow Cloudflare and verify responsive
spinner; reopen Settings and verify masked placeholder + saved-state text,
replace token then verify state; launch from Start Menu with no console.

## 2026-10-25: Windows replacement NSIS retest (owner observation)

The replacement candidate from Preflight #40232194058 and Windows package
#40232194059, source `c17ea2b3`, was exercised on the owner's Windows 11.
Confirmed PASS by owner: no console on launch; saved token has masked
presence in Settings; previously blank resource drawer now retains rows.
This is scoped evidence only, not a full Windows acceptance PASS. The
owner did not explicitly confirm every credential, language, notification
or history scenario on this pass.

Three remaining FAILs on that installed binary:
- After each manual repositioning, the **first** tray hide/show displaces
  the card slightly downward. Subsequent hide/show cycles stay stable
  until another manual move. This supersedes the earlier cumulative-drift
  characterization.
- Card's rounded corner cutouts retain an opaque bevel/white frame.
- The down glyph on the compact Hide button is off-center and visually
  inconsistent with the Breeze-inspired controls.

Development fix `ffb0b85` is not yet physically tested on Windows:
anchor movement uses measured outer height after avoiding unnecessary
native resizes; late Win32 resize event re-anchors the bottom edge.
The main transparent window explicitly disables native shadow (Tauri's
default adds an opaque 1px border to frameless Windows). The hide glyph
is now fixed-size centered SVG instead of a platform-dependent text
character. Native geometry diagnostics are opt-in with
`CFVIGIL_UI_DEBUG=1`, written to bounded local
`windows-card-geometry.log`; no credentials or provider responses.

Physical recheck after the new unsigned NSIS: drag compact card, record
its exact location, tray hide/show once, then ten cycles; drag again and
repeat; verify no vertical delta at both 100% and native display DPI if
practical. Verify both card corner cutouts are visually transparent
against a contrasting desktop and the SVG hide-chevron is centered.
Then expand list, hide/show, check resources and anchoring, quit/relaunch
and check persisted placement. Previous PASS observations need not be
retested at high frequency, but should survive the new install.
Do not close native Windows gate on hosted-CI success alone.
