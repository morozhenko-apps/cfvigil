# CFVigil

CFVigil is a small, free desktop app for keeping an eye on Cloudflare Workers and Pages deployments. It lives in the system tray, so you can see what's happening across your projects without keeping the Cloudflare dashboard open all day.

## Why I built it

I work on several apps and games, and quite a few of them use Cloudflare. When I ship an update, I want to know whether the new deployment is still running, finished successfully or failed. Opening every project in a browser to check that gets old quickly, especially when I'm already busy building or testing something else.

I made CFVigil to solve that problem for myself. I wanted one small card that tells me when something needs attention, with a quick way to open the right Cloudflare page when I need the details. It isn't meant to replace the dashboard or become a large monitoring platform.

## What it does

CFVigil discovers the Workers and Pages projects available to your Cloudflare API token and shows their latest known deployment states in a compact tray card. You can expand the resource list upwards, search by name and open a specific resource in Cloudflare. A deployment that is queued or running shows as in progress, even if the previous deployment failed. A failed attempt shows as a problem, a successful attempt as healthy, and missing or unverified data as unknown.

The app can also watch for unusual Worker error rates and zone-level WAF activity when the relevant Cloudflare analytics and permissions are available. It notifies you about detected incidents and keeps a small local event history, so you can look back at what happened. Those analytics are not available in every account, and a green deployment status does **not** mean that a website is guaranteed to be online. CFVigil checks Cloudflare's deployment and incident data, not arbitrary website health.

The app is read-only. It doesn't deploy Workers, change Cloudflare settings or manage your projects. Your API token stays in the operating system's credential store, and the event history is kept locally. You can choose how often it checks for updates and whether it starts with your computer. There is no separate CFVigil account, subscription or hosted monitoring backend.

## Platforms and current status

CFVigil is still in development. Linux (especially KDE Plasma) and Windows are the current desktop targets, but the first public installer and full release acceptance have not happened yet. The GitHub `dev` branch contains working source and automated tests; that isn't the same as a finished, publicly supported release.

macOS may be added later. The distribution layout is designed to add a `macos/` directory next to `linux/` and `windows/`, but actual macOS builds, Apple signing, notarization and native testing are separate work. Please don't treat macOS as available yet.

The interface has 20 language catalogs in source. They are still draft translations undergoing context and desktop-layout checks, not 20 fully reviewed translations.

When the first release is approved, downloads will be hosted under `https://download.morozhenko.com/dist/CFVigil/`. The planned layout keeps versioned installers in platform folders and uses a small `latest.json` file to point the website to the latest verified release. Linux will also have a signed APT repository using the same DEB as the direct download, without a second copy in an APT pool. Until a release is actually published, there are no official installer links to offer.

## Bugs and ideas

If you find something broken, please use the [bug report form](https://github.com/morozhenko-apps/cfvigil/issues/new?template=bug_report.yml). Include enough detail to reproduce it, but remove any private Cloudflare information before posting logs or screenshots.

If you have an idea that would make the app easier to use, use the [improvement suggestion form](https://github.com/morozhenko-apps/cfvigil/issues/new?template=feature_request.yml). I'm interested in real workflows and problems, even if you don't have a technical solution. I can't promise to add every suggestion.

**I don't accept outside pull requests or code fixes for the main project.** I develop and maintain CFVigil myself, but you're free to fork it under the Apache 2.0 license. More details are in [CONTRIBUTING.md](CONTRIBUTING.md). Please report security issues [privately](https://github.com/morozhenko-apps/cfvigil/security/advisories).

## Building from source

CFVigil uses Rust and Tauri 2, with a small WebView interface. You need the platform dependencies required by Tauri 2, a Rust toolchain and Node.js 24 for its repository checks. The source branch is `dev`.

```sh
cargo check --locked -p cfvigil
cargo test --locked -p cfvigil
node --test tests/*.test.mjs
python3 -m unittest discover -s tests -p 'test_release_*.py'
```

To start a development build locally, use `cargo run -p cfvigil` in a correctly configured development environment. This doesn't connect to your Cloudflare account until you choose to configure access. The application is not designed to make Cloudflare changes.

For the current feature details and known limits, see [PRODUCT.md](docs/PRODUCT.md), [development and acceptance status](docs/DELIVERY.md), [the roadmap](docs/ROADMAP_TO_1_0.md) and [the distribution design](docs/DISTRIBUTION.md).

## License

CFVigil is open source under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for attribution and the [contribution policy](CONTRIBUTING.md) for feedback and fork rules.

<!-- Older root README section links remain available for existing references. -->
<a id="known-limitations-of-this-initial-implementation"></a>
<a id="principles"></a>
[Earlier development notes and historical checkpoints](docs/DELIVERY.md).
