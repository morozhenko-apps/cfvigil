# Windows desktop verification

Verify the checksum of the Windows installer before executing it. Do not bypass SmartScreen or disable antivirus globally to force installation.

On a real Windows desktop, verify installation and Start menu identity, tray icon, compact and expanded card, Settings and Event History, keyboard focus, search, notifications and correct resizing on varying display scaling.

Confirm that language and token changes happen only when Apply is used, that Cancel does not persist drafts and that native credential storage never exposes the token to the WebView. Check autostart enable/disable, relaunch, account switching and brief offline recovery.

Test upgrade and uninstall separately. Windows Credential Manager entries and per-user app data must be checked independently; removing the installer does not itself prove complete credential erasure.

Record reproducible defects as issues with sanitized screenshots and steps. Release-check details belong in CI logs, not this document.
