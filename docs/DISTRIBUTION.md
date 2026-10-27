# CFVigil distribution guarantees

This is a technical contract for safe distribution, not a log of individual builds or a guide to private account configuration.

## Artifacts

The release version is consistent across the Rust package, its lockfile and the Tauri bundle. Versioned Linux DEB and AppImage installers and the Windows installer are kept as immutable public artifacts; each has a matching SHA-256 checksum. The same Linux DEB is used for direct download and for the signed APT repository. No duplicate installer is stored in an APT pool.

Release files are grouped by **platform only**. Linux's flat APT index and signed metadata live next to its DEB. A small `latest.json` document points download clients to the approved immutable files instead of maintaining second copies with `latest` filenames. The optional addition of another desktop platform follows the same principle.

## Signed Linux repository

APT consumes a flat `./` repository with `Packages`, `Packages.gz`, a signed `InRelease` and a public keyring. The indexes must reference exactly the validated DEB. Their SHA-256 entries, gzip contents, package identity, architecture and binary checksums must be consistent. Missing metadata, mismatched signatures or package identity failures stop publication; signature enforcement is never bypassed.

The signing workflow imports only an appropriate signing subkey, not a full private primary key. Private key provisioning, account credentials and emergency rotation procedures belong in the owner's private operational records, **not in this public repository**.

## Publication boundaries

The release workflow runs only for the exact approved version tag on the reviewed main branch, within a protected approval environment. Candidates do not publish anything.

The publisher validates source provenance and both platform bundles before uploading. After uploading immutable files, it verifies every public response against expected length and SHA-256. It updates mutable APT indexes, commits signed `InRelease`, and updates `latest.json` **last**. A failure at any step must not produce a successful release receipt.

Temporary CI artifacts may be deleted only after the complete public readback receipt is verified. GitHub Releases carry explanatory metadata, not duplicated installers. The website must reject malformed or unavailable release pointers rather than invent a download URL.

## Limits of verification

Passing source tests does not prove that a native installer is usable. A successful package build does not by itself establish operating-system integration, Windows code signing, a working APT upgrade or public download availability. These have their own acceptance checks.

This document intentionally excludes bucket names, account identifiers, credential inventories, local secret paths and per-run deployment reports. The actual workflow and publisher source are the executable implementation of this contract.
