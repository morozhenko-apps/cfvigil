# CFVigil

**Cloudflare deployment status in your desktop tray.** Free, open source, and available for Linux and Windows.

CFVigil shows the latest relevant deployment attempts for Cloudflare Workers and Pages in a compact, searchable panel. Keep track of what's deploying, what succeeded, and what needs attention without repeatedly opening the Cloudflare dashboard.

## Download

- [Product page and downloads](https://morozhenko.com/cfvigil/)
- [Linux DEB](https://morozhenko.com/cfvigil/install/linux) · [Linux AppImage](https://download.morozhenko.com/dist/CFVigil/linux/CFVigil-1.0.0-linux-x64.AppImage)
- [Windows x64 installer](https://morozhenko.com/cfvigil/install/windows)
- [Release manifest and checksums](https://download.morozhenko.com/dist/CFVigil/latest.json)

Windows NSIS installers are currently unsigned. Check the published SHA-256 checksum before installing. Linux packages include a DEB and AppImage; the signed Linux APT repository is described on the product page.

## Features

- **Production deployment status:** a compact tray card summarizing successful, running and failed attempts; an expanded searchable list for individual Workers and Pages.
- **Latest attempt first:** a new in-progress attempt supersedes an earlier failure. Unavailable information stays unverified rather than appearing successful.
- **Incident signals:** optional Worker execution-error spikes and zone-level WAF blocking activity, where the Cloudflare account makes those metrics available.
- **Local event history:** searchable events, notifications, configurable refresh interval and optional start-at-login.
- **Native desktop integration:** Linux KDE and Windows, with 20 interface languages.

CFVigil does not trigger deployments, modify Cloudflare settings, monitor GitHub workflow jobs, or perform website uptime checks.

## Connect Cloudflare

Open **Settings**, enter your Cloudflare Account ID and a **read-only API token**, then select **Apply**. The card shows a pending-sync indicator while fetching the initial results.

Grant only the API permissions required for the features you use: Workers Scripts Read, Workers CI Read and Pages Read for deployments; Account Analytics Read, Zone Read and Zone Analytics Read for optional Worker error and WAF metrics.

Your token is stored in your operating system's credential manager. Application preferences, cached resource names and event history remain on your device. CFVigil does not send app usage telemetry or your Cloudflare data to developer-operated analytics servers.

## Privacy, license and support

- [Privacy Policy](https://morozhenko.com/cfvigil/privacy-policy)
- [Terms of Use](https://morozhenko.com/cfvigil/terms-of-use)
- [Apache License 2.0](LICENSE) · [Notices](NOTICE)
- [Report a bug](https://github.com/morozhenko-apps/cfvigil/issues/new?template=bug_report.yml) · [Suggest an improvement](https://github.com/morozhenko-apps/cfvigil/issues/new?template=feature_request.yml)
- [Report a security issue privately](https://github.com/morozhenko-apps/cfvigil/security/advisories)

Do not include Cloudflare tokens, account identifiers or private project information in issue screenshots or logs.
