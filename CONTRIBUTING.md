# Feedback, bug reports and forks

CFVigil is an open-source project developed and maintained by Stanislav Moseyko (morozhenko). I'm happy to hear about problems people find and about things that could make the app easier to use. I work on the project myself, though, so I keep responsibility for its code and technical direction in one place.

## Found a bug?

Please use the [bug report form](https://github.com/morozhenko-apps/cfvigil/issues/new?template=bug_report.yml). Include the CFVigil version or commit, operating system, how you installed or ran it, and what happened. Steps to reproduce are especially helpful. Check for an existing issue first so that several reports of the same problem can stay together.

Before attaching logs or screenshots, remove API tokens, Cloudflare account and zone IDs, private Worker names, private domains, request headers, usernames, local paths and other sensitive information. Please report security vulnerabilities [privately through GitHub Security Advisories](https://github.com/morozhenko-apps/cfvigil/security/advisories), not through a public issue.

## Have an idea?

Use the [improvement suggestion form](https://github.com/morozhenko-apps/cfvigil/issues/new?template=feature_request.yml). You don't need to know how to implement it. Explaining the situation, what is inconvenient today and what you would like to do is usually more useful than a detailed technical specification.

I read suggestions as feedback, not as a promised roadmap. A feature may not fit the purpose of CFVigil, or I may not have time to add it. Please don't include private Cloudflare information in public requests.

## Code contributions

**I don't accept outside pull requests, patches or requests to join the upstream maintenance team.** This includes unsolicited bug fixes as well as new features. Please report a problem or describe your idea in Issues instead of sending a PR. I may fix or implement it in my own way, but I'm not committing to review or merge outside code.

CFVigil is available under the [Apache License 2.0](LICENSE), so you are welcome to fork it, change it and distribute your own version while following the license and [NOTICE](NOTICE) obligations. Please don't present an unofficial fork as an official CFVigil or morozhenko-apps release.

## Development and release boundaries

The active source branch is `dev`. The `main` branch is reserved for reviewed releases, and public binaries appear only after the relevant release checks and approval. Source builds and passing CI are not a claim that a particular platform installer has been publicly released.

Detailed internal build, quality and release rules live in [AGENTS.md](AGENTS.md), [the roadmap](docs/ROADMAP_TO_1_0.md) and [distribution documentation](docs/DISTRIBUTION.md). Documentation-only updates and code changes are committed separately.
