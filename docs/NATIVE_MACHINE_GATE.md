# Native desktop acceptance

Source tests and automated builds do not replace installation and usability checks on actual supported operating systems.

## Linux

Install a validated DEB or run a checksum-verified AppImage. Check tray appearance, compact/expanded geometry, anchored list behavior, search, keyboard controls, Settings, notifications, event history and application restart. Confirm launch-at-login can be enabled and disabled without duplicate registrations. Test both reconnecting a known account and operating without a token.

Verify package identity, desktop entry, icons and an upgrade without duplicate application identities. Confirm that removing the package does not accidentally remove unrelated user data.

## Windows

Install a validated NSIS candidate as an ordinary user. Check tray/menu integration, native window icons, Settings, event history, autostart, upgrade identity and uninstall behavior. Respect OS installer-security warnings; never disable system-wide protection to make a candidate pass.

## Privacy and account checks

Use a dedicated, limited-permission testing token. Never capture it in logs or screenshots. Verify that account switching does not mix incidents or resources and that old or inaccessible provider states are not reported as healthy.

**Credential-store persistence is separate from package uninstall.** Check local files and saved OS credentials independently when evaluating an explicit full-data-removal flow. Do not claim they are erased merely because the executable disappeared.

Record observations and failures in the testing issue or CI system, not in dated Markdown appendices. See [NATIVE_CLOUDFLARE_SMOKE.md](NATIVE_CLOUDFLARE_SMOKE.md) for connected-provider checks and [WINDOWS_NATIVE_SMOKE.md](WINDOWS_NATIVE_SMOKE.md) for Windows-specific points.
