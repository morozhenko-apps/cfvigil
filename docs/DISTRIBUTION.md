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

The owner may approve the exact temporary `release-recovery` branch as a recovery exception. Its workflow starts only when its explicit request marker changes. Implement recovery admission and offline regression checks before starting it: require a completed failed normal release, successful original validation and platform jobs, two unexpired archives with matching artifact identities and source revision, and an unchanged approved tag at current main. Read-only admission and the stable distribution tests precede access to signing credentials; repeat admission inside the protected publication job after owner review.

The recovery publisher downloads those exact archive IDs without rebuilding installers. It uses the original source revision and run ID in its manifest and verified receipt so the existing cleanup gate can delete only the original archives. It preserves signing, every public checksum readback, the final latest-pointer commit and the text-only release finalizer.

Treat the branch and its environment policy as a bounded exception. Keep the normal tag policy and required reviewer unchanged. After all recovery jobs and release finalization succeed, remove the exact branch policy before deleting the branch; verify the branch still points to the completed recovery revision. On failure, retain recovery state and source archives for diagnosis. Reverting the recovery source changes must not change existing public installers or weaken verification.

## Limits of verification

Passing source tests does not prove that a native installer is usable. A successful package build does not by itself establish operating-system integration, Windows code signing, a working APT upgrade or public download availability. These have their own acceptance checks.

This document intentionally excludes bucket names, account identifiers, credential inventories, local secret paths and per-run deployment reports. The actual workflow and publisher source are the executable implementation of this contract.

## Atomic publication safety

Repair plan: replace check-then-upload with per-object conditional PutObject.
Absent objects require If-None-Match; mutable replacements require the exact
previous ETag. Snapshot latest.json before changing APT metadata and compare
that original snapshot at final promotion. Any conflict stops without a receipt
or deletion authority; it is never retried as an unconditional overwrite.
Keep immutable same-byte reuse, bounded streamed readback, signatures, mainline
admission and published release identity. Offline storage-contract tests must
exercise intervening creators/replacements, malformed/missing ETags, denied
reads, stale latest snapshots, corrupt bytes and stopped final receipts.
Documentation and implementation remain independently revertible.

Local readiness inspects the sole main branch identity and existing release
environment controls. It cannot infer CI or owner approval from branch presence;
required exact-source verification and tag authorization remain independent.
