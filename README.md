# CFVigil

Free, local-first desktop monitoring for Cloudflare Workers and Pages.

**Development status:** initial v0.1.0 source implementation on `dev`, not a published or physically accepted installer. The `main` branch is the initial repository baseline.

## What the current source implements

- Minimal system tray card inspired by RepoTicker: no product title, three counters (Healthy, Deploying, Issues), then **Resources** and **Hide** buttons.
- One combined, scrollable list of Workers and Pages. It grows upward; search appears between the list and the compact card.
- Row layout: resource name, relative event time, state lamp, and `⋯` menu for opening the Cloudflare Dashboard.
- Green: latest Cloudflare build/deployment succeeded; yellow: a newer attempt is running; red: latest attempt failed. Grey means unknown. A new running attempt takes precedence over a failed previous attempt.
- Automatic resource discovery via Cloudflare REST APIs and a configurable polling interval.
- Native API-token handling with OS keychain, local event history, deployment/Worker-error/WAF-block notifications, and direct browser links to specific builds, Pages deployment details or zone Security Events. Pending incident focus is retained by Rust until the user opens the resource drawer.

### Known limitations of this initial implementation

- Worker execution error-spike detection is implemented through **read-only Cloudflare GraphQL Analytics**, with per-script aggregate queries (Cloudflare's documented `limit: 1`, `scriptName` filter), batched up to 8 Workers per GraphQL request. It compares a recent 10-minute window against the preceding 30-minute baseline with a 5-minute ingestion buffer, and requires at least 100 recent requests, 10 errors, and a substantial increase in error rate. It has not yet been validated against a live account.
- **Zone-scoped WAF block-spike detection is implemented in source.** It queries Cloudflare Security Events aggregate counts, compares the latest 10-minute interval to a 30-minute baseline, and adds temporary zone alerts to the unified resource list. This feature has not been validated against a live Cloudflare account. A zone's WAF alert is never misattributed to a Worker.
- A previous failed build followed by a newer running build is yellow. A currently serving deployment is not used as proof that an application is healthy.
- Linux KDE Wayland anchoring now has a native GTK Layer Shell implementation and KSNI tray integration adapted from RepoTicker, but has **not** been physically tested on KDE; fallback window repositioning can be rejected by unsupported Wayland compositors.
- No signed APT, DEB/AppImage/Windows release workflow, production icons, autostart configuration or localized interface has been accepted for CFVigil.
- Rust application compilation, Cloudflare live-account API responses and installers have not yet been verified.
- An account ID and a **user-scoped** API Token with Workers Scripts Read, Workers CI Read, Pages Read, Account Analytics Read, Zone Read and Zone Analytics Read are required. Availability of APIs can vary by token and account.
- The source currently supports one configured Cloudflare account at a time.

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
- CI uses standard **GitHub-hosted Actions** on the public repository, following RepoTicker's preflight structure. CI is not a CFVigil product integration.
- **20 UI languages are planned** as the last functionality milestone, after all other MVP features and before the final end-to-end QA campaign; the required locale set and per-language contextual audit are in the roadmap.
- Upstream [Issues](https://github.com/morozhenko-apps/cfvigil/issues/new/choose) accept **bug reports only** through the structured template. Feature requests and third-party PRs are not accepted; independent forks are permitted under Apache-2.0. See [CONTRIBUTING.md](CONTRIBUTING.md).

© 2026 Stanislav Moseyko (morozhenko).
