# CFVigil

Free, local-first desktop monitoring for Cloudflare Workers and Pages.

**Development status:** initial v0.1.0 source implementation on `dev`, not a published or physically accepted installer. The `main` branch is the initial repository baseline.

## What the current source implements

- Minimal system tray card inspired by RepoTicker: no product title, three counters (Healthy, Deploying, Issues), then **Resources** and **Hide** buttons.
- One combined, scrollable list of Workers and Pages. It grows upward; search appears between the list and the compact card.
- Row layout: resource name, relative event time, state lamp, and `⋯` menu for opening the Cloudflare Dashboard.
- Green: latest Cloudflare build/deployment succeeded; yellow: a newer attempt is running; red: latest attempt failed. Grey means unknown. A new running attempt takes precedence over a failed previous attempt.
- Automatic resource discovery via Cloudflare REST APIs and a configurable polling interval.
- Native API-token handling with OS keychain, local event history and failed-attempt notifications that focus the affected row.

### Known limitations of this initial implementation

- **No attack detection or error-spike monitoring yet.** These will be implemented independently, without HTTP health checks or guessed zone-to-Worker attribution.
- A previous failed build followed by a newer running build is yellow. A currently serving deployment is not used as proof that an application is healthy.
- Linux KDE Wayland tray anchoring still needs the native Layer Shell implementation/physical acceptance; Tauri window repositioning can be rejected by Wayland.
- No signed APT, DEB/AppImage/Windows release workflow, production icons, autostart configuration or localized interface has been accepted for CFVigil.
- Rust application compilation, Cloudflare live-account API responses and installers have not yet been verified.
- An account ID and a **user-scoped** API Token with Workers Scripts Read, Workers CI Read and Pages Read are required. Availability of APIs can vary by token and account.
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

## Project

[Product contract](docs/PRODUCT.md) · [Release/testing gates](docs/DELIVERY.md)

Apache License 2.0, with [NOTICE](NOTICE). © 2026 Stanislav Moseyko (morozhenko).
