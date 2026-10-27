# CFVigil product direction

CFVigil is designed to make routine Cloudflare deployment checks less distracting. Product decisions favor clear status semantics, native desktop behavior and small, predictable interfaces over a large administrative dashboard.

## Core scope

- Tray monitoring for Workers and Pages, showing the most recent deployment attempt.
- Searchable resource list, contextual links and incident-aware notifications.
- Worker error-rate and zone-level WAF blocking signals when provider data is available.
- Local event history, read-only permissions, application settings and localization.
- Linux and Windows builds with verifiable distribution metadata.

## Future considerations

Additional desktop platforms, improved incident presentation and usability refinements may be considered from user feedback. Platform support is introduced only with native installation and security verification. New capabilities must not expand Cloudflare permissions into write access or blur the difference between deployment state and site uptime.

The living behavior contract is [PRODUCT.md](PRODUCT.md). Public suggestions belong in [GitHub Issues](https://github.com/morozhenko-apps/cfvigil/issues).
