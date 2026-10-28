# Windows desktop verification

Verify the checksum of the Windows installer before executing it. Do not bypass SmartScreen or disable antivirus globally to force installation.

On a real Windows desktop, verify installation and Start menu identity, tray icon, compact and expanded card, Settings and Event History, keyboard focus, search, notifications and correct resizing on varying display scaling.

Confirm that language and token changes happen only when Apply is used, that Cancel does not persist drafts and that native credential storage never exposes the token to the WebView. Check autostart enable/disable, relaunch, account switching and brief offline recovery.

Test an upgrade, ordinary uninstall, and uninstall with **Delete the application data** checked. Upgrade and ordinary uninstall must preserve the app's credential. Explicit full-data removal must erase CFVigil's own generic entry from Windows Credential Manager in addition to app-specific Roaming/Local user data. Check for the credential by target name before and after; never print its secret value. Confirm that reinstall does not inherit an old token. Test an already-absent credential and failure to delete a locked/inaccessible entry: a failure must be visible rather than falsely reported as complete erasure. This must be observed on a real Windows installation.

Record reproducible defects as issues with sanitized screenshots and steps. Release-check details belong in CI logs, not this document.
