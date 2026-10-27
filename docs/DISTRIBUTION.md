# CFVigil isolated distribution (not deployed)

Status: release implementation staged on dev; NO live R2 publication,
Cloudflare route, signing credential or GitHub Release has been configured.

## Deployment boundaries

GitHub is for versioned source, protected CI, source tags, and text-only
release notes. The only binary distribution storage is the Cloudflare R2
bucket **cfvigil-dist** under **dist/CFVigil/**. This is a separate bucket
from RepoTicker and must have a dedicated S3 API credential restricted to that
bucket. Never share signing private keys, S3 credentials, or R2 prefixes
with RepoTicker.

The public download base is:
https://download.morozhenko.com/dist/CFVigil/

The dedicated read-only Worker is in
infra/download-worker/src/worker.mjs. Its wrangler.jsonc contains a
separate download route on that domain and binds cfvigil-dist. It rejects
POST, PUT and DELETE, unexpected object keys, path traversal and queries.
Public endpoints serve only exact versioned packages and sidecars plus signed
APT index objects. No Cloudflare API token used for CFVigil *monitoring*
is used for publishing.

Cloudflare configuration (requires owner action; do not auto-deploy):
1. Create bucket **cfvigil-dist** in the intended Cloudflare account.
2. Provision an R2 S3 token with object read/write privileges on ONLY
   cfvigil-dist. Keep it separate from all other product secrets.
3. Check that the download Worker route does not overlap other Workers.
   Configure Cloudflare Git integration for this Worker to deploy only from
   owner-approved main (root infra/download-worker). Provide
   CLOUDFLARE_ACCOUNT_ID through its secure integration configuration.
4. Confirm the public Worker returns 404 for unknown keys, 405 for POST,
   and 200 on a verified test object through GET and HEAD. Cloudflare
   Browser Integrity Check must not reject legitimate APT/curl/Python GET
   requests on this narrow download route. Any exception must be
   hostname + path + method scoped, not global.
5. Never publish a production release until the download Worker is
   reachable and exact-byte public readback succeeds.

## GitHub protected environment

Create a GitHub environment named **cfvigil-release** and require owner
approval before use. Do NOT grant this environment to arbitrary branches,
unreviewed contributors or tags.

Environment variables:
- CFVIGIL_RELEASE_APPROVED_TAG: exact tag of the one specifically approved
  release, e.g. v1.0.0, updated manually only after acceptance.
- CFVIGIL_APT_FINGERPRINT: 40-hex GPG certification-primary fingerprint.
- CFVIGIL_R2_ACCOUNT_ID: 32-hex Cloudflare account ID.

Environment secrets (CFVigil-specific, never reuse RepoTicker's values):
- CFVIGIL_APT_SIGNING_SUBKEY_BASE64
- CFVIGIL_APT_SIGNING_PASSPHRASE
- CFVIGIL_R2_ACCESS_KEY_ID
- CFVIGIL_R2_SECRET_ACCESS_KEY

APT signing uses a dedicated offline certification primary and one signing
subkey, imported into the protected GitHub runner as a signing-subkey-only
export. The script rejects full-primary secret key material. Securely store
the encrypted full-primary backup outside GitHub and verify independent
backup readability. The private primary must never be placed in GitHub
Secrets, source control, build artifacts or R2.

Do not set these production secrets on dev or on a public CI test workflow.

## Release execution sequence

.github/workflows/protected-release.yml is triggered only by a strict
vMAJOR.MINOR.PATCH git tag; the admission job rejects a tag that is not
the exact current main HEAD, or versions not matching committed Cargo.lock
and Tauri/Cargo metadata. It runs Node and offline Python publication
tests, then builds Linux and Windows on standard GitHub-hosted runners.

The Linux job creates amd64 DEB/AppImage, Windows builds unsigned x64 NSIS.
Both validate package identities, sizes and SHA256 before staging temporary
artifacts with 1-day retention. Physical OS acceptance is a separate release
prerequisite; CI success does not imply it.

The protected publish job requires environment approval and matching
CFVIGIL_RELEASE_APPROVED_TAG, independently validates the CFVigil GPG
signing subkey, and creates the signed APT metadata (gpgv, Packages,
Packages.gz, by-hash). Signed APT is stored under
dist/CFVigil/apt and published with InRelease LAST. Artifacts under
dist/CFVigil/releases/vVERSION/ are immutable; hash mismatches on an
existing object cause a hard failure.

The publisher verifies each R2 object privately *and* retrieves it from
the public read-only Worker, comparing full SHA256 and byte length to
the source. If any public GET/size/hash fails, it emits no success receipt
and cleanup does not run. No GitHub binary download mirror is permitted.

The cleanup job has GitHub Actions:write solely to remove the TWO named
temporary release archives belonging to the exact source run and commit,
after reading the verified public R2 receipt. It re-lists artifacts to
confirm deletion; unrelated artifacts are never removed. There are no
permanent binary attachments to the text-only GitHub Release, created
only after cleanup succeeds.

If cleanup or metadata-only GitHub Release fails after R2 upload, the
workflow reports failure; R2 objects already published remain and MUST
be reconciled before attempting any rerun. Mutable APT indexes should
only advance under the serialized release gate.

## Remaining acceptance gates

The release workflow, signed APT and publisher have been implemented in
source, not executed against live credentials. The protected environment,
dedicated R2 bucket, Cloudflare download Worker route, signing key and
WAF exception are not provisioned by this source change. Require real KDE
Wayland/X11 and Windows installer/upgrade/uninstall smoke; Windows NSIS
is currently unsigned. Full cross-platform validation, 20 locale audit,
and Mode B acceptance remain before approving stable 1.0.0.

## Distribution workflow artifact-path hardening (2026-10-24)

Root cause observed by source inspection: four Linux/Windows upload/download
artifact paths in the protected tag workflow contain one extra closing brace
after the GitHub Actions `runner.temp` expression. The extra character is a
literal part of the resolved path, so both source archives and publish job
download destinations can diverge from the validated staging directories.
The preflight source tests previously checked only that some valid expressions
were present; they did not reject valid expressions followed by stray braces.

Implementation plan (code changes separate from this documentation commit):
1. Correct all four upload/download paths in
   `.github/workflows/protected-release.yml` with the same exact
   `runner.temp` expression as the existing Linux/Windows candidate jobs.
2. Add regression assertions in `tests/protected-release.test.mjs` covering
   both platforms, both upload/download steps, exact target directories and
   rejection of extra closing braces in GitHub expressions.
3. Run the final stable check through a single explicit `dev` Preflight marker.
   Do not trigger a release tag, protected publish job or R2/Cloudflare changes.

Risks: static expression checking is not a replacement for the owner-approved
protected release dry run with real installers and public R2 SHA256 readback.
Rollback: revert only the code/test commit if a subsequent validation exposes
an unexpected packaging contract. The R2-only distribution invariants and
release permissions remain unchanged.

Status: analysis complete; implementation and final CI verification pending.

## 2026-10-26: protected release source-contract checkpoint

The current source already has an exact-expression regression assertion
in `tests/protected-release.test.mjs` preventing a literal stray brace
after `${{ runner.temp }}`, plus exact upload/download paths for both
staging directories. The protected workflow currently matches those paths.
This supersedes the previous pre-implementation status note for the
**source-path** problem only. The protected environment, public R2 readback,
actual APT signing, live publishing and subsequent artifact deletion have
NOT been executed or accepted. Keep production publication disabled.


## 2026-10-26: Chrome/Defender warning on unsigned Windows smoke builds

Owner observed a Chrome threat/download warning against the intentionally
**unsigned** Windows NSIS candidate. A warning is NOT evidence that the
installer is safe, nor proof that it contains malware. Do not state
"false positive" until file provenance, integrity and scanner evidence
have been checked. Chrome's per-download override is not a permanent
application allowlist. Never instruct users to globally disable Safe
Browsing, SmartScreen, Defender or add blanket browser/downloads
directory exemptions.

**Owner diagnostic path:**
1. Read the exact Chrome warning category (uncommon / suspicious /
   dangerous / malware) in Downloads and record it without clicking
   through. Obtain the candidate only from the trusted GitHub Actions
   run, not a third-party mirror.
2. Verify the exact SHA-256 against the accompanying CI sidecar; on
   Windows: `Get-FileHash .\\CFVigil*.exe -Algorithm SHA256`,
   compare with the `.exe.sha256` file. Verify intentional
   `Get-AuthenticodeSignature` result `NotSigned` and scan with
   Microsoft Defender. A valid checksum proves artifact fidelity, not
   malware freedom.
3. If the downloaded NSIS is independently verified and Chrome
   identifies only an *uncommon/unfamiliar* executable, its Downloads
   warning menu may offer an individual keep decision depending on
   browser version/policy. Do not bypass a detection classified as
   active malware or a corporate block without security review.
4. If Defender quarantines a verified expected binary, use **Windows
   Security → Virus & threat protection → Protection history → the
   exact detection → Actions → Allow on device** only after
   independent verification; otherwise leave in quarantine. The
   exception can later be revoked under Allowed threats. Do not grant
   folder-wide exclusions or switch off real-time protection.
5. For distribution, replace unsigned NSIS with **identity-verified
   Authenticode signing** and build long-term publisher reputation.
   Signing does not instantly guarantee SmartScreen acceptance; even
   newly signed files may warn. Microsoft Store is an optional signed
   distribution channel. Code signing credentials and workflow require
   separate owner approval and must stay outside source control.

Google Chrome help: https://support.google.com/chrome/answer/6261569
Microsoft Defender Protection History:
https://support.microsoft.com/en-us/windows/security/windows-security/protection-history-in-the-windows-security-app
Microsoft SmartScreen reputation guidance:
https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation

No exception has been added, no malware scan or Chrome/Windows
security decision was remotely performed, and no production signing
configuration was modified.

## 2026-10-27 — protected-distribution offline finalization after Mode B source gate

The immediately preceding source gate [#40562000345](https://github.com/morozhenko-apps/cfvigil/actions/runs/40562000345)
succeeded at SHA `7738dbcd4c4cc6850338b26d8c37c7e970889b92`:
163 Node, 16 Python and 164 Rust tests, zero failed; locked
Rust check and source/localization/icon steps passed.
This source evidence belongs to the owner-approved Cloudflare
simulation / 20-locale package. Physical owner inspection is
already accepted and is not to be repeatedly requested.

### Root-cause and existing contract

Before cleanup, `validateR2Receipt` verifies product, version,
commit, R2 base, per-file SHA/size equality and 200 public readback,
but currently accepts **any file URL below the version prefix**
and **any nonempty platform file list**. As a result, a syntactically
plausible but incomplete or cross-platform receipt can authorize
deletion of temporary GitHub archives, even though the publisher
should issue exactly two Linux installers with their sidecars and
one Windows installer with its sidecar.

Potential failure sources screened: broad prefix match; missing exact
platform filename set; missing sidecar; duplicate receipt URL; swapped
Linux/Windows path; GitHub artifact run/commit mismatch; incomplete
public-byte verification. The last two already have separate checks,
and the R2 publisher already emits complete exact platform manifests.
The **first four receipt input-validation gaps** are this targeted
repair. No evidence of an actual compromised publisher, deletion,
or bad R2 upload was observed.

### Implementation and branch/negative matrix

1. Require exact literal path (scheme, origin, prefix, version,
   platform, filename) for every expected Linux/Windows binary and
   SHA256 sidecar. Require all four Linux and both Windows entries
   exactly once. Retain current numeric size/hash/readback checks.
2. Update the test fixtures to the **actual source_map publisher
   shape**, never weaken the production requirement to keep synthetic
   legacy test shortcuts.
3. Parameterized negative tests: omitted/duplicate sidecar, filename
   from wrong OS, unexpected extra filename, percent encoding,
   user-info, query/fragment, traversal, alternate host/port and
   receipt with mismatching provenance. Check zero requests before
   any DELETE.
4. Reuse existing offline Python publication and Worker tests; run
   all stable Node/Python/Rust only once after implementation through
   the explicit dev CI marker. No release tag, branch promotion,
   production keyring, R2 object, Cloudflare Worker deploy or
   GitHub artifact deletion.

**Mutation review:** any substitution of exact URL equality with
prefix matching, removal of expected-name equality, or acceptance of
incomplete/duplicate file lists must fail a negative test. No mutation
score is claimed. Rollback affects only the receipt validator, its
offline fixture/tests and this separate documentation milestone.
The valid protected publisher receipt format does not change.

**External release gate:** Owner-approved isolated R2 bucket,
download Worker route, dedicated secrets, valid subkey and signing,
protected environment and public GET readback remain necessary to
publish. Windows NSIS is intentionally unsigned; Authenticode signing
cannot be represented as completed without credentials and an actual
verified signature. This effort is an offline release hardening gate,
not a public v1.0.0 release authorization.

### Offline receipt invariant implemented (2026-10-27)

The release-only cleanup validator now requires exactly four
Linux file URL entries (DEB, AppImage and their SHA256 sidecars)
and exactly two Windows entries (unsigned NSIS and its SHA256
sidecar). URLs must match the correct version, platform and
canonical public path exactly. Existing per-file SHA256, byte-size
and public 200-readback proofs are retained.

The offline fixture was corrected to represent the actual
`r2_release.py` receipt, not synthetic incomplete one-file
archives. A new table-driven negative test covers fourteen
missing, duplicated, wrong-platform, extra, encoded and
ambiguous URL shapes and asserts **zero GitHub requests**
before validation. Code changes are confined to the release
cleanup validator and its offline test fixture; production
publisher and protected workflow remain unchanged.

**Status: authored, final dedicated CI evidence pending.**
No live R2 upload, release tag, branch promotion, GitHub
artifact deletion, Worker deployment or signing occurred.

### 2026-10-27 — release cleanup preflight/atomicity hardening (Mode B regression-first)

**Observed source defect, no actual production deletion:** the current
`deletePublishedArtifacts` lists, validates and deletes each platform
archive inside the same loop. A first valid Linux archive is destroyed
before the function discovers an invalid/malformed/foreign Windows
archive. This weakens the already documented "only delete exact matching
temporary artifacts after verified publication" invariant and makes a
malformed complete receipt able to cause partially destructive cleanup.

**Root-cause alternatives evaluated:** (1) misleading receipt files
(the previous stricter six-file check now blocks these); (2) a mismatched
second artifact head SHA (not preflighted before first deletion);
(3) duplicate Windows artifact name (detected only after first deletion);
(4) a malformed/missing second artifact listing (detected only after first
deletion); (5) mixed platform run IDs (validator currently accepts, although
the publisher emits one common `GITHUB_RUN_ID`); (6) subsequent deletion
API outage (cannot be made cross-request transactional and must remain
explicit release failure). Causes 2–5 stem from interleaving external
reads/validation with destructive actions and the missing common-run
receipt invariant, not from R2 hashes.

**Plan (documented before implementation):**

1. Use the actual publisher contract: both platform entries must share
   exactly one positive `run_id`. Existing synthetic test fixture
   currently uses 51 and 52; correct it to one run and one artifact list
   before introducing new acceptance tests.
2. Regression-first negative tests must reject, with **zero DELETE**,
   foreign Windows SHA after valid Linux, duplicate Windows archive,
   malformed listing and mixed run IDs, while preserving unrelated
   archives and idempotent already-absent named archives.
3. Implement a read-only, full-run preflight: list once, check every
   planned archive's name/ID/run/commit/uniqueness, then and only then
   delete the approved exact IDs. Re-list to verify each deletion, keep
   existing fail-closed HTTP error handling.
4. Final review of 0, 1 and 2 present archive cases, 401/403/503
   list errors, POST/DELETE success/failure and restored idempotency.
   Report any unavoidable partial cleanup after an actual mid-delete
   transport failure as partial, never as an atomic rollback.
5. After the entire batch, trigger one deliberate dev preflight run
   covering JS, Python, Rust, localization and icon. No native-machine
   repeat, main/tag, GitHub Release or production R2 actions.

**Risk/compatibility:** release-only internal validator and its contract
fixtures change; all bona fide receipts already share one run ID.
Rejecting invalid mixed-run receipts is intentional fail-closed behavior,
not a new public receipt version. No persistent data migration or API
contract change. Rollback is a scoped revert of the release validator,
its tests, and this separate documentation amendment.

**Completion criteria:** a real final stable CI PASS on the exact code
SHA, clear count/citation; leave protected production deployment gated.
