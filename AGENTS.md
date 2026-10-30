# CFVigil engineering contract

This file defines the current rules for coding agents. Do not append dated task diaries, personal environment details, CI run histories or private deployment instructions here.

## Product boundary

CFVigil is a read-only desktop companion for monitoring Cloudflare Workers and Pages. The compact tray card shows counts and opens an upward-expanding, searchable resource list. Status must come from the latest relevant attempt: running overrides an older failure; only verified success is green; missing or stale evidence is unknown. Worker execution-error and zone-level WAF spikes are separate, evidence-based incidents. Zone security activity must not be attributed to an unrelated Worker. No Cloudflare mutations or synthetic uptime claims.

Keep application credentials in the OS credential store and ordinary settings/history in local per-user storage. Never expose tokens in the UI, logs, tests or diagnostics. Do not assume uninstalling a binary removes OS keyring entries.

## Source workflow

- `main` is the sole long-lived branch. Use reviewed short-lived PR branches, inspect current ownership, preserve required checks and retire task branches after their work is preserved. Do not recreate permanent `dev`, force-push, rewrite other work or publish tags without explicit release authorization.
- Read the relevant product and technical contract before changing behavior. Document the intent and rollback boundary before an architectural change.
- Keep code and documentation in **separate commits**. Prefer small, maintainable fixes rather than workarounds.
- Run focused diagnostics when necessary. Run the stable suite once, at the final validation gate.
- CI runs by an **explicit marker** only after the code changes are finished. If CI or an external build is started, **do not poll it**. Report the run URL and exact pending validation, then stop; continue after a completion event or the owner's next instruction.
- Documentation-only edits must not trigger build markers or gratuitous CI. Existing code-level required checks remain authoritative.
- Do not edit or consume the owner's other products, machines, runners, secrets or cloud settings as part of CFVigil work.

## Packaging and release

Linux DEB, AppImage and Windows NSIS use one canonical version and identity. The DEB also serves the signed flat APT index; do not publish duplicates of versioned installers. A release requires an owner-approved tag at the reviewed `main` HEAD, protected signing credentials, complete platform checks and publicly verified checksums before temporary archives can be deleted. Release pipeline code is the executable source of truth.

Owner-approved failed-publication recovery may use the exact temporary `release-recovery` branch and an explicit recovery request marker. It reuses successful platform archives from the original approved tag at current `main`, validates their source run and artifact identities, and retains the protected environment and mandatory owner review. It must not build replacement installers, move the release tag or promote recovery code to `main`. After verified publication and successful finalization, remove the exact recovery environment policy and then the branch. Ordinary `dev` pushes never publish.

Never infer release acceptance from source tests alone. Real machine acceptance, security of user data and the release approval are separate gates. Never publish packages, create tags, change production storage or weaken signing gates without explicit owner approval.

## Documentation boundaries

- `README.md` is **user-facing**: why the app exists, what it does, how to report problems and its license. No build status, draft translations, platform promises, CI, storage topology or deployment credentials.
- `docs/PRODUCT.md` describes behavior and invariants; `docs/DISTRIBUTION.md` explains generic release guarantees; `docs/qa/MODE_B_MATRIX.md` defines acceptance requirements. Other docs cover focused development topics.
- Keep docs short and current. Never paste terminal output, full failure investigations, commit/run ledgers, private directory paths, hostnames, cloud account IDs, private bucket names, signing fingerprints or GitHub environment secret inventories. These belong in private operational records, if needed.
- Public source and Git history remain visible. Removing private operational details from Markdown does **not** erase earlier commits or conceal implementation visible in code and workflows.
- Existing bug and improvement Issue Forms accept feedback; unsolicited outside PRs are not accepted. Forks remain permitted under Apache 2.0.

## Completion

Summarize actual changes, verified checks and remaining limitations. Never claim an unrun test passed or promise to resume an external operation without a supported scheduler.
