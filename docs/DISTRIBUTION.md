# CFVigil distribution guarantees

This is a technical contract for safe distribution, not a log of individual builds or a guide to private account configuration.

## Artifacts

The release version is consistent across the Rust package, its lockfile and the Tauri bundle. Versioned Linux DEB and AppImage installers and the Windows installer are kept as immutable public artifacts; each has a matching SHA-256 checksum. The same Linux DEB is used for direct download and for the signed APT repository. No duplicate installer is stored in an APT pool.

Release files are grouped by **platform only**. Linux's flat APT index and signed metadata live next to its DEB. A small `latest.json` document points download clients to the approved immutable files instead of maintaining second copies with `latest` filenames. The optional addition of another desktop platform follows the same principle.

## Signed Linux repository

APT consumes a flat `./` repository with `Packages`, `Packages.gz`, a signed `InRelease` and a public keyring. The indexes must reference exactly the validated DEB. Their SHA-256 entries, gzip contents, package identity, architecture and binary checksums must be consistent. Missing metadata, mismatched signatures or package identity failures stop publication; signature enforcement is never bypassed.

The signing workflow imports only an appropriate signing subkey, not a full private primary key. Private key provisioning, account credentials and emergency rotation procedures belong in the owner's private operational records, **not in this public repository**.

## Publication boundaries

The normal release workflow runs only for the exact approved version tag on the reviewed main branch, within a protected approval environment. Candidates do not publish anything.

The publisher validates source provenance and both platform bundles before uploading. After uploading immutable files, it verifies every public response against expected length and SHA-256. It updates mutable APT indexes, commits signed `InRelease`, and updates `latest.json` **last**. A failure at any step must not produce a successful release receipt.

Public verification requests must identify the CFVigil release verifier with a descriptive `User-Agent`. They use ordinary unauthenticated GET requests and stream the complete response; a successful HTTP status alone never replaces length and SHA-256 verification. Client identification must not impersonate a browser or require disabling download security controls.

Temporary CI artifacts may be deleted only after the complete public readback receipt is verified. GitHub Releases carry explanatory metadata, not duplicated installers. The website must reject malformed or unavailable release pointers rather than invent a download URL.

## Failed-publication recovery

Before retrying a failed publication, retain the original platform archives and establish whether the failure affected builds, upload or public verification. Correct and regression-test the failed publisher contract before starting another publication. Complete builds need not be repeated while their original archives remain available and their source provenance is still valid.

A job re-run uses its original source revision, so a later publisher correction does not enter that re-run automatically. Running corrected recovery code must preserve the approved artifact version, source revision, signatures, public readback and cleanup gates. It must not silently move an existing release tag, promote another commit to main or grant a development branch access to the protected signing environment. A recovery route that changes those authorization boundaries requires a separate owner decision before implementation or execution.

The one-off recovery workflow is retired after accepted publication and is
retained in Git history. Its temporary signing-environment exception must also
be absent. Do not carry that development-branch authority into normal mainline
publication. A future recovery must receive its own reviewed plan and keep
original archive identities, signatures, exact-byte readback, final pointer and
receipt-gated cleanup. No historical failure permits weakening these controls.

## Limits of verification

Passing source tests does not prove that a native installer is usable. A successful package build does not by itself establish operating-system integration, Windows code signing, a working APT upgrade or public download availability. These have their own acceptance checks.

This document intentionally excludes bucket names, account identifiers, credential inventories, local secret paths and per-run deployment reports. The actual workflow and publisher source are the executable implementation of this contract.

## Atomic publication safety

The publisher uses per-object conditional PutObject.
Absent objects require If-None-Match; mutable replacements require the exact
previous ETag. Snapshot latest.json before changing APT metadata and compare
that original snapshot at final promotion. Any conflict stops without a receipt
or deletion authority; it is never retried as an unconditional overwrite.
Keep immutable same-byte reuse, bounded streamed readback, signatures, mainline
admission and published release identity. Offline storage-contract tests must
exercise intervening creators/replacements, malformed/missing ETags, denied
reads, stale latest snapshots, corrupt bytes and stopped final receipts.
Source verification does not prove a new production publication; its live
readback and owner release approval remain separate gates.

Local readiness validates both permanent dev and main source identities and
requires their reviewed tips to match before tagging. It also inspects existing
release-environment controls. Equality cannot establish CI or owner approval;
required exact-source verification and tag authorization remain independent.

## Build toolchain maintenance integration

The owner approved a scoped integration of the already validated maintenance into the current release branch through a reviewable PR. This authorization supersedes ordinary temporary-branch restrictions for this integration only. Build/CI tool versions change; application behavior, public contracts, data, credentials and immutable release identities remain unchanged. Upstream Actions keep their supported internal runtime; project Node tooling uses the exact maintained LTS pin.

Plan:

- [x] Inspect current release source, owning instructions, prior maintenance evidence and deployment triggers. Start from the current release head to preserve its unique commits.
- [x] Record this plan before executable edits, with documentation and executable changes committed separately.
- [x] Apply only the reviewed Actions, Node and required compatibility/security corrections; adapt references to existing release workflows. Exclude unrelated accumulated development work.
- [x] Review complete diff and source invariants, run applicable final stable checks once, and reuse previous suite evidence only with exact source/lock equivalence. Keep native/store and release-only acceptance separate.
- [ ] Create the scoped PR; integrate only after its exact-head required checks and existing production-admission conditions pass. Preserve concurrent release changes and never bypass branch protection.

Rollback uses an ordinary revert of the scoped maintenance commits; no tag movement, resource recreation or provider/data rollback is part of this task. Existing automatic Git deployment ownership and triggers are preserved. A source/PR pass alone does not establish effective production Node or native/store acceptance.

Validation result: PASS. Source review preserves workflow events, permission/runner/release conditions and existing release-side commits. Executable source/lock bytes match the previously validated development maintenance, excluding explicitly reviewed documentation, workflow and non-executable housekeeping differences; its complete consumer-suite evidence remains applicable. New source contracts and diff checks were executed on this release-based branch. Remote PR/release results are tracked separately and are not implied by local checks.

- `python3 -m unittest discover -s tests -v` — exit0 (STABLE_SOURCE).
- `git diff --check 4e7d107f184469222429edebd308ac610b0c7382` — exit0 (STABLE_SOURCE).
