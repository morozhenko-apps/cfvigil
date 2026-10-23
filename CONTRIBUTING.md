# Bug reports and forks

CFVigil is an open-source project maintained by Stanislav Moseyko (morozhenko).

## Reporting defects

Only actionable, reproducible bug reports are accepted in upstream GitHub Issues.
Use the **Bug report** issue form and include the CFVigil version or Git SHA,
operating system and desktop session, reproduction steps, expected result and
actual result. Verify whether the defect has already been reported.

Sanitize all diagnostics. Never publish Cloudflare API tokens, credentials,
account or zone IDs, private Worker names, private domains, request headers,
personal paths or private screenshots. Report security vulnerabilities privately
via GitHub Security Advisories, not public Issues.

Upstream is not an open feature-request queue. Feature proposals and general
support requests are outside the scope of Issues.

## External development

The upstream maintainer does **not accept third-party code contributions,
feature PRs, additional maintainers or external development assignments**.

You may fork CFVigil and independently develop, build and distribute your
version under Apache License 2.0, observing its attribution/NOTICE requirements.
Do not represent a fork as an official CFVigil / morozhenko-apps release.

An unsolicited pull request is not a commitment to review, merge, support, or
maintain the proposed code. Prefer independent forks rather than upstream PRs.

## Owner's development process

The integration/work branch is `dev`. Release branch `main` is updated only
after owner-authorized validation and release decisions. The final accepted
release tag must point to the exact `main` commit.

Continuous validation is allowed while implementing features, but final
end-to-end acceptance and exhaustive quality audit occur **after** the full
20-locale implementation and contextual translation audit.

Documentation-only changes and code changes must be committed separately.
