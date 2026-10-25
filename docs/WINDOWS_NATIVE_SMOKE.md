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
