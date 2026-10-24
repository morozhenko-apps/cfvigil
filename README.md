# CFVigil

Free, local-first desktop monitoring for Cloudflare Workers and Pages.

**Development status:** initial v0.1.0 source implementation on `dev`, not a published or physically accepted installer. The `main` branch is the initial repository baseline.

Workspace `Cargo.lock` is committed for reproducible Rust dependency
resolution. Separate opt-in workflows build unsigned Linux DEB/AppImage and
Windows x64 NSIS **candidates only**; neither publishes a release.

## What the current source implements

- Minimal system tray card inspired by RepoTicker: no product title, three counters (Healthy, Deploying, Issues), then **Resources** and **Hide** buttons.
- One combined, scrollable list of Workers and Pages. It grows upward; search appears between the list and the compact card.
- Row layout: resource name, relative event time, state lamp, and `⋯` menu for opening the Cloudflare Dashboard.
- Green: latest Cloudflare build/deployment succeeded; yellow: a newer attempt is running; red: latest attempt failed. Grey means unknown. A new running attempt takes precedence over a failed previous attempt.
- Automatic resource discovery via Cloudflare REST APIs and a configurable polling interval.
- Native API-token handling with OS keychain, **account-scoped durable incident deduplication across restarts**, **searchable/filterable local event history with exact event-specific links**, deployment/Worker-error/WAF-block notifications, and direct browser links to specific builds, Pages deployment details or zone Security Events. Pending incident focus is retained by Rust until the user opens the resource drawer.

### Known limitations of this initial implementation

- Worker execution error-spike detection is implemented through **read-only Cloudflare GraphQL Analytics**, with per-script aggregate queries (Cloudflare's documented `limit: 1`, `scriptName` filter), batched up to 8 Workers per GraphQL request. It compares a recent 10-minute window against the preceding 30-minute baseline with a 5-minute ingestion buffer, and requires at least 100 recent requests, 10 errors, and a substantial increase in error rate. It has not yet been validated against a live account.
- **Zone-scoped WAF block-spike detection is implemented in source.** It queries Cloudflare Security Events aggregate counts, compares the latest 10-minute interval to a 30-minute baseline, and adds temporary zone alerts to the unified resource list. This feature has not been validated against a live Cloudflare account. A zone's WAF alert is never misattributed to a Worker.
- A previous failed build followed by a newer running build is yellow. A currently serving deployment is not used as proof that an application is healthy.
- Linux KDE Wayland anchoring now has a native GTK Layer Shell implementation and KSNI tray integration adapted from RepoTicker, but has **not** been physically tested on KDE; fallback window repositioning can be rejected by unsupported Wayland compositors.
- No signed APT, DEB/AppImage/Windows release workflow, production icons, autostart configuration or localized interface has been accepted for CFVigil.
- Rust application compilation, Cloudflare live-account API responses and installers have not yet been verified.
- An account ID and a **user-scoped** API Token with Workers Scripts Read, Workers CI Read, Pages Read, Account Analytics Read, Zone Read and Zone Analytics Read are required. Availability of APIs can vary by token and account.
- The source currently supports one configured Cloudflare account at a time.

## Localization readiness

The desktop development branch contains **all 20 target-language UI dictionaries** as drafts. No language is considered editorially or physically accepted yet. Its user interface uses semantic
keys in `apps/desktop/ui/locales/en.mjs` with native Rust diagnostic strings
centralized in `native_messages.rs`. Validation for keys, interpolation,
plural rules, date formatting and RTL direction is available via:

```sh
node scripts/check-i18n.mjs
```

The 20-language product milestone and full per-language context audit are
planned **after feature freeze**; the presence of locale codes is not a
claim that the additional dictionaries are implemented.

## Development

Prerequisites: Rust toolchain, Tauri 2 OS dependencies and Node.js 22+.

```sh
node --test tests/*.test.mjs
cargo test -p cfvigil-desktop
cargo run -p cfvigil-desktop
```

For Ubuntu KDE development, install Tauri's WebKitGTK and GTK dependencies before building. No published installers are available yet.

## Principles

- No GitHub integration; Cloudflare is the authoritative provider for Cloudflare builds and deployments.
- Never attempt to infer app uptime by requesting the deployed application URL.
- Missing API permissions, stale data and unknown outcomes must not be shown as green.
- No destructive Cloudflare API requests. Native code owns the API token, network requests and browser destination validation.
- No hosted backend or paid tier.

## Project and contribution policy

[Product contract](docs/PRODUCT.md) · [Roadmap to 1.0.0](docs/ROADMAP_TO_1_0.md) ·
[Release/testing gates](docs/DELIVERY.md) · [Working rules](AGENTS.md)

- Open source, free, Apache License 2.0 (identical to RepoTicker), with [NOTICE](NOTICE).
- **Working branch: `dev`.** `main` is for owner-approved stable integrations/releases.
- One serialized background poller wakes on settings changes, marks stale/locked-account data unknown, and backs off after core Cloudflare API failures.
- Optional OS-native launch at sign-in is available through Settings, off by default; it does not trigger Cloudflare polling when only that setting is changed.
- Keyboard-accessible resource drawer and menus, stable focus during polling, and resource-specific incident focus.
- Local settings, incident journal and event history use private atomic JSON writes; corrupted files are backed up rather than silently overwritten.
- Explicit, nonpublishing Linux amd64 DEB/AppImage candidate workflow with source/version checks, Debian metadata validation, SHA-256 sidecars and short-lived GitHub Actions artifacts; no release or R2 upload.
- Release distribution policy: **binary packages and checksums live only in CFVigil's isolated Cloudflare R2 namespace**. GitHub Actions artifacts expire after one day in candidate jobs and must be deleted immediately after a later release's verified R2 public readback. GitHub Releases, if used, provide text/links only, never binary attachments.
- Partial Workers/Pages/Analytics/WAF outages preserve verified results and show unavailable data as unknown; one shared 120-call budget per polling cycle respects REST 429 Retry-After. No overlapping per-save refreshes.
- CI uses one **GitHub-hosted Ubuntu 24.04** job. Normal `dev` code/docs commits **do not launch GitHub Actions**. After a completed feature or fix, update [`ci/run-preflight.txt`](ci/run-preflight.txt) once to request JavaScript and Rust checks on the resulting head; temporary PRs are checked automatically for code changes. This avoids repeated queued jobs. CI is not a CFVigil product integration.
- Dedicated `cfvigil-dist` R2 bucket, read-only CFVigil download Worker and owner-protected main-tag publication workflow are implemented **in source only**. No live bucket, signing credentials, route, R2 upload or GitHub Release was created. See [Distribution setup and safety gates](docs/DISTRIBUTION.md).
- **20 UI languages are planned** as the last functionality milestone, after all other MVP features and before the final end-to-end QA campaign; the required locale set and per-language contextual audit are in the roadmap.
- Upstream is designed for **structured bug reports only**; feature requests and third-party PRs are not accepted, while independent forks are permitted. The [Bug report form](.github/ISSUE_TEMPLATE/bug_report.yml) is prepared on `dev` but only becomes visible in GitHub's Issues chooser once it reaches the repository's default branch (`main` at present). See [CONTRIBUTING.md](CONTRIBUTING.md).

© 2026 Stanislav Moseyko (morozhenko).
