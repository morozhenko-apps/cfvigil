# Approved distribution redesign — 2026-10-27 (authoritative)

**Mode A — release-pipeline architecture refactor.** Owner selected flat
distribution, with **only platform directories**, common existing
Cloudflare R2 bucket `app-dist`, one versioned binary object per platform
artifact, no per-app download Worker, and deferred download rate limits/WAF.
This section supersedes all older `cfvigil-dist`, `apt/pool`,
`apt/dists/stable`, `releases/v...` and dedicated Worker
instructions still present later in historical notes. Those earlier
sections describe the previous, **not yet deployed**, candidate architecture.
No Cloudflare resource, website or live R2 object is changed by source edits.

**Public R2 key namespace** `dist/CFVigil/` only, on existing
`https://download.morozhenko.com` R2 Custom Domain:

```text
dist/CFVigil/latest.json
dist/CFVigil/linux/cfvigil_VERSION_amd64.deb
dist/CFVigil/linux/cfvigil_VERSION_amd64.deb.sha256
dist/CFVigil/linux/CFVigil-VERSION-linux-x64.AppImage
dist/CFVigil/linux/CFVigil-VERSION-linux-x64.AppImage.sha256
dist/CFVigil/linux/Packages
dist/CFVigil/linux/Packages.gz
dist/CFVigil/linux/InRelease
dist/CFVigil/linux/cfvigil-archive-keyring.gpg
dist/CFVigil/windows/CFVigil-Setup-VERSION-win-x64.exe
dist/CFVigil/windows/CFVigil-Setup-VERSION-win-x64.exe.sha256
```

Versioned installers, checksum sidecars and public key are immutable;
`Packages`, `Packages.gz`, `InRelease`, and `latest.json`
are mutable with no-store cache policy. CI generates the
flat signed `Packages.gz` index pointing directly to
`./cfvigil_VERSION_amd64.deb` in that **same** Linux directory.
No second DEB pool copy, `by-hash`, or `stable/main` hierarchy.
The APT source is
`deb [arch=amd64 signed-by=/usr/share/keyrings/cfvigil.gpg] https://download.morozhenko.com/dist/CFVigil/linux/ ./`.
APT relies on signed `InRelease` and exact checksums of the index
and DEB. `Packages` and `Packages.gz` are independently hashed
inside `InRelease`, and their byte contents must agree.

**Publication order:** validate both platform candidates and signed
flat repository offline before S3 writes; upload/check all immutable
platform files and public key; then upload/read back both mutable
indexes; then upload/read back `linux/InRelease` as the APT commit
point; finally upload/read back schema-checked `latest.json`
as the website's **last** commit point. Receipt is emitted only
after full public HTTP body/size/hash readback including both
commit points. Existing exact GitHub Actions handoff cleanup is
authorized solely by that verified receipt; text-only GitHub Release
comes after successful cleanup. Failed/stale/malformed public
readback MUST NOT advance or emit a receipt.

The `latest.json` contains the approved version/tag/commit and
immutable platform URLs. The website resolves those URLs at
runtime, so no duplicate `*-latest-*` binaries or new Cloudflare
Worker are necessary.
Because the website origin (`morozhenko.com`) differs from
`download.morozhenko.com`, browser-side retrieval of `latest.json`
requires an explicit read-only R2 CORS rule for the owning website
origin (GET/HEAD only), or a same-origin server-side proxy.
Do not expose a write endpoint or bypass browser security by wildcard
credentials. The manifest is served as `application/json` and
`Cache-Control: no-store`; validate effective public headers before
switching PersonalLanding to the live manifest. This source-only
refactor does not silently mutate CORS or website code.
 `latest.json` itself is a public pointer,
not an independent signed software-update authority. The app never
installs files on the basis of this metadata without other trust
checks. Candidate CI never advances this pointer.

**Future macOS extension (not released):** this flat scheme remains
unchanged except for an additional `dist/CFVigil/macos/` platform
directory. The actual release will need macOS binary naming/architecture
and SHA256 sidecars, platform candidate provenance, protected workflow
build/sign/notarize steps, independent macOS acceptance, new R2 source-map
and receipt entries, an updated `latest.json` platform entry, plus
website links/tests. Do not add an empty directory or claim that the
current two-platform release receipt already accepts macOS. Apple
credential custody and release approval are independent; Linux APT
remains under `linux/` and does not acquire macOS package metadata.

**Trade-offs explicitly accepted:** dropping APT by-hash means
clients concurrently downloading during an index switch may see
a transient hash mismatch and safely retry. Do not weaken
`signed-by`, checksums or `gpgv` to hide that failure. The
`app-dist` R2 bucket shares the bucket-level credential boundary
with Picturix; ensure the CI credential is deliberately provisioned
for that scope. Keep all private/debug/build artifacts **outside**
the public bucket. WAF/rate limits are an **owner-deferred**
post-implementation task: never globally challenge/deny native APT
or curl, which are nonbrowser clients.

**Implementation sequence & rollback:** (1) document before code,
(2) update signed APT generation/verification, (3) adapt publisher,
receipt and ordering, (4) adapt workflow and offline contracts;
(5) run a **single final** complete hosted source CI, record exact
SHA and counts. Revert the scoped implementation commits if the
gate fails; no production rollback needed because this task does
not execute a tag or upload files. Remove historical, never-deployed `infra/download-worker/` source
and its Worker-only tests during this same source refactor; retain
older Worker discussion only in explicitly superseded historical notes. Protect the `main`/tag/environment gate. Final
real R2 readback and signing still require owner-configured secrets,
approved tag and a separate explicitly authorized release.

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

### 2026-10-27 — implemented two-platform cleanup preflight (CI pending)

`validateR2Receipt` now requires the exact Linux/Windows archives
to reference a **single positive source run ID**; this is the
existing emission behavior of `r2_release.py`. The cleanup
operation now makes a single read-only listing of the source run,
checks **all** present platform records for correct name, unique
numeric artifact ID, matching `workflow_run.id`, matching source
`head_sha`, and unambiguous matches BEFORE the first destructive
API call. A missing named archive remains idempotently skipped.
After each authorized deletion the operation independently
re-lists and rejects a still-present archive.

The shared-run test fixture now represents the actual publisher
instead of two artificial unrelated run IDs. New regression tests
cover a cross-run receipt with zero requests, an otherwise-good
Linux archive plus foreign Windows commit, a duplicate Windows
archive, malformed Windows artifact ID, and two platform names
sharing one GitHub artifact ID. All negative tests require
**zero DELETE calls**; most require a single read-only listing.

Important failure-mode limit: GitHub's delete API has no
cross-artifact transaction. If the second DELETE fails *after*
the first succeeds, the workflow is marked failed; no untrue
rollback or atomicity is claimed. Read-only preflight eliminates
preventable partial deletion from corrupt/foreign **initial**
provenance, not mid-operation network failures.

Source modifications: `scripts/cleanup-published-artifacts.mjs`
and `tests/artifact-retention.test.mjs` only. No schema version
change, Cloudflare calls, R2 writes or GitHub artifact DELETE
is performed in this test batch. Full dev Preflight must run
once after both files are finalized; do not cite the previous
`#40563249933` for this additional change.

### 2026-10-27 — final apt signed-index content hardening (Stage 1–12 release slice)

**Observed result before change:** Preflight
[#40568546486](https://github.com/morozhenko-apps/cfvigil/actions/runs/40568546486)
SUCCEEDED on exact SHA `b72f4c654b504eebee772adddc11cb37b563b359`.
Source/localization/icon checks PASS, Node **167/167**,
offline Python **13+3/16**, Rust **164/164**; zero failures,
total **347/347**. The two-platform cleanup preflight and all
three new regression cases passed. Its reported status is now
CLOSED, replacing the prior "CI pending" checkpoint.

**Next production risk:** `scripts/r2_release.py::apt_files` executes
`gpgv` against `InRelease`, compares each by-hash object to
its corresponding Packages/Packages.gz, and checks that Packages
mentions the CFVigil pool, but it **never checks that the verified
signature's SHA256 index entries refer to those same index bytes**.
An older valid signed `InRelease` plus new indexes/by-hash can
pass source_map and be uploaded, leaving APT clients unable to
verify the advertised package indexes. No such corrupted live
publication is known; this is a failure found in source audit.

Root-cause alternatives screened: missing signature validity
(existing gpgv check); missing by-hash blob parity (existing check);
wrong pool DEB hash (existing check); stale signed index digest
and size (not checked); malformed/duplicate/missing SHA256 index
rows (not checked); unrelated package claims (current broad
substring check). Scope this fix to stale/missing/duplicate
signed index checks and associated test fixtures; do not
change public URLs, APT metadata format or publication order.

**Planned implementation:**
1. Add a pure parser/validator for RFC4880 clear-signed `InRelease`
   checking a single complete `SHA256:` section and one canonical
   size+digest record each for
   `main/binary-amd64/Packages` and
   `main/binary-amd64/Packages.gz`. Handle LF/CRLF only with
   exact canonical paths, fail closed on missing/malformed/duplicate
   target rows, mismatch in byte count or digest; never trust
   `by-hash` alone. Execute it only **after gpgv succeeds**.
2. Update mock InRelease in the existing Python offline
   publisher fixture to contain coherent signed Release records;
   keep the fake GPG execution limited to fixtures.
3. Add negative tests for old valid signature with newer index,
   size-only tamper, digest-only tamper, duplicate/missing path,
   missing SHA256 section and malformed clear-signed envelope.
   Ensure R2 `source_map` rejects prior to any network
   upload or receipt emission. Add actual parser-positive
   coverage including LF/CRLF.
4. Run one final full hosted Ubuntu Preflight (Node, Python,
   Rust and source checks) **after** the full code/test
   implementation; update this dated evidence, not previous
   release claims. No provider calls, no production R2,
   tag/main, owner OS retest or signing-key operations.

**Rollback:** scoped script/test commit plus documentation
and final CI marker. No state migration, auth change, or
release-source schema change. Existing genuine
`apt-ftparchive release .` signed indexes contain these
SHA256 rows and should pass unchanged.

**Release boundary:** a green offline gate does not create
bucket, route, protected environment, signing subkey or
public readback; first live publication still requires
explicit authorization and provisioned CFVigil-only secrets.

### Signed InRelease index verification implemented (2026-10-27)

`scripts/r2_release.py::apt_files` now verifies both that `gpgv`
accepts the clear-signed `InRelease` **and** that its authenticated
`SHA256:` section matches the exact local bytes and byte lengths of
`main/binary-amd64/Packages` and `Packages.gz`. The parser requires
a clear-signed envelope with SHA256 hash declaration, signature trailer,
one SHA256 section and exactly one matching canonical record for each
index. Missing, duplicate, malformed, stale or tampered index records
fail before any S3 upload or public success receipt.

The existing offline publication fixture now includes coherent
`InRelease` records while cryptographic signing remains a test-only
mock. Newly authored integration cases verify LF/CRLF, stale signed
indexes despite a matching newly generated by-hash file, wrong SHA256,
wrong byte size, a missing row, duplicates, malformed hash, missing
SHA256 section and malformed signed envelope. Existing `gpgv`,
pool-binary SHA256 parity and by-hash equality checks are preserved.

This is a **source-level safety improvement**, not evidence of
production APT signing or live R2 readback. Its final CI results
must be documented below after the post-change verification run.

### Verified signed-index publisher source gate (2026-10-27)

[Hosted Preflight #40571683067](https://github.com/morozhenko-apps/cfvigil/actions/runs/40571683067)
completed **SUCCESS** at exact commit
`7d9e24c4996b378a6d68d5c41b17d5b01dc3cf5b`.
Its log confirms all source check steps passed, Node **167/167**,
Python release **16/16**, Python icon **3/3**, and Rust
**164/164**, zero failures/ignored Rust: **350 tests passed**.
All three newly added Python signed-index test methods passed
inside the 16-test release suite. This supersedes the earlier
"CI pending" label for the signed-index step.

Distribution source gate: ACCEPTED for publisher checksum,
signed-index consistency, exact receipt file inventory, and
cross-platform cleanup preflight. This is a bounded code-level
acceptance, not evidence of actual provisioned key, genuine
GPG signing exercise, production R2 upload, public Worker
reachability, real public SHA256 readback, GitHub archive
deletion or a publicly available release. Those separate
externally protected operations still require authorization
and actual execution with owner-provisioned CFVigil-only
credentials. Do not conflate the mocked offline GPG validation
with signing a real production APT repository.

No extra test trigger was run for this documentation-only
status update, and no user machine, main branch, production
Cloudflare object or GitHub release was changed.

### 2026-10-27 — signed APT package-to-DEB and gzip consistency (Mode A)

**Baseline:** `dev` at `6bf807e58b7fec97accd253ef314d8e3d900be15`;
most recent full Preflight #40571683067 at exact unchanged-code
`7d9e24c4996b378a6d68d5c41b17d5b01dc3cf5b`:
Node 167/167; Python 19/19; Rust 164/164; 350/350 PASS.
This task does not assume live signing or production R2 publication.

**Observed source defect:** `scripts/r2_release.py::apt_files`
cryptographically validates the signed `InRelease` to local
`Packages` and `Packages.gz`, and verifies that the APT pool DEB
bytes equal the release's validated DEB. However, `Packages`
is only checked for the occurrence of the pool directory substring.
A signed index can name the wrong package, version or architecture,
or claim a different `Filename`, `Size` or `SHA256`.
Also, `Packages.gz` need not actually expand to the separately
signed `Packages`. Either inconsistency yields unusable or
contradictory package acquisition even when all existing signature,
by-hash and R2 integrity tests succeed.

**Root-cause analysis:** (a) signed metadata tamper is addressed by
the previous index SHA256 verification; (b) binary pool mismatch is
already checked; (c) internal Packages control fields, sole paragraph
identity and gzip-to-plain equivalence have **no current verification**.
Therefore add deterministic checks at the signed-index publishing
boundary, after `gpgv` and before any external S3/public operation.
No additional Cloudflare API, signing, HTTP or security boundary is
introduced.

**Implementation plan:**

1. Build an explicit exact-single-package Debian control index
   validator in `r2_release.py`. Require one and only one nonempty
   package stanza; `Package=cf-vigil`,
   `Version=<approved release version>`,
   `Architecture=amd64`,
   `Filename=pool/main/c/cf-vigil/cfvigil_<version>_amd64.deb`,
   `SHA256=<actual pool digest>`, `Size=<actual pool size>`.
   Reject malformed and duplicated control keys rather than taking
   whichever field appears last. Permit legitimate folded
   Description lines and unrelated standard control fields, but
   never accept untrusted package names, extra package stanzas,
   traversal or inline substring matches.
2. Require `gzip.decompress(Packages.gz)` to equal `Packages`
   byte-for-byte and reject invalid/truncated gzip, not merely
   validate both independent SHA256 rows.
3. Correct offline R2 publication fixtures to realistic
   `dpkg-scanpackages`-style Packages bytes and real deterministic
   gzip; regenerate the signed-index fixture SHA256 and by-hash
   exactly as already done by fixture setup.
4. Regression tests for valid matching package, folded descriptions,
   wrong name/version/architecture/path/hash/size, duplicate
   critical field, additional stanza, deceptive path substrings,
   mismatched yet signed compressed index and invalid gzip.
   Re-signing in tests is simulated by regenerating coherent
   mocked signed-index fixture records after metadata changes,
   so failures are proven at the package consistency layer,
   not caused incidentally by stale SHA256 or by-hash.
5. Final single full CI marker after implementation; independently
   record actual Node/Python/Rust results and SHA. Do not run
   production signing, R2, Worker, main promotion, release tag,
   existing owner's KDE/Windows native check or human review.

**Compatibility:** legitimate `apt_repository.py` output includes
the fields above because it is produced by
`dpkg-scanpackages` from an already identified CFVigil amd64
DEB. This is a fail-closed validation of the preexisting one-DEB
distribution contract; no public URL, package format, storage
schema, API or UI change. This task does not grant a release
authorization. **Rollback:** scoped revert of the Python validator
and unit fixtures/tests; docs/CI marker are separate commits.
**Completion evidence:** final hosted CI proof at exact code SHA,
not a pre-change run. Partial signed apt/source checks should never
be labeled a fully live validated public release.

### 2026-10-27 — APT package-to-DEB catalog checks implemented (awaiting CI)

Source change: `scripts/r2_release.py` adds
`verify_apt_package_catalog`, invoked by `apt_files`
after signature verification and exact signed-index digest/size
checks. It enforces one Debian control stanza with
`Package: cf-vigil`, exact release `Version`,
`Architecture: amd64`, canonical release pool `Filename`,
`SHA256` and `Size` agreeing with the actual validated
pool DEB; rejects duplicate/malformed fields and folded
critical identities while accepting normal folded descriptive
fields. The two independently signed index representations
must agree: `Packages.gz` is decompressed in bounded
chunks and compared byte-for-byte to `Packages`; invalid,
truncated and additional gzip output fails closed.

Test update: realistic `dpkg-scanpackages`-shaped test
catalog with genuine deterministic `gzip.compress(mtime=0)`
and coherently generated by-hash plus mocked signed
InRelease. Explicit offline negatives: wrong package
identity/version/arch/path/digest/size, duplicate critical
SHA256, missing filename, extra package stanza, deceptive
pool-substring field, folded Filename, signed but divergent
gzip, truncated gzip, invalid gzip and concatenated extra
gzip content. All failures are required before any network
call, signing key use or public receipt creation.

**Execution status:** tests authored, not yet accepted by
final full hosted CI on the changed Python code. The prior
350-test run predates this package-catalog safety change
and must not be cited as its validation. No public release
operation authorized or performed.

### Verified APT package catalog source gate (2026-10-27)

[Hosted Preflight #40574280521](https://github.com/morozhenko-apps/cfvigil/actions/runs/40574280521)
completed **SUCCESS** on exact source-and-CI SHA
`cd49390dc9b38585509a2f5ff695b8914de7b382`.
Observed CI job log: Node **167/167** (0 failed),
Python release **19/19**, Python icon **3/3**,
Rust **164/164** (0 failed, 0 ignored); locked Rust check,
JavaScript syntax, all 20 i18n source checks and icon
consistency steps passed. **353/353 tests passed, zero failures.**
This run includes all three new APT catalog contract test
methods and their table-driven negative cases.

**Scoped gate CLOSED:** the signed APT Packages control stanza
now binds exactly one CFVigil amd64 package to the validated
DEB pool's canonical file path, version, size and SHA256.
`Packages.gz` must expand to the exact signed `Packages`
bytes. The mismatch, duplicate, smuggled pathname, truncated
and concatenated gzip cases are rejected before publication.
No real key signing or S3 publication was exercised; fixture
GPG verification was intentionally mocked, and R2 storage
remained untouched.

**Remaining protected release prerequisites:** actual separately
provisioned bucket/Worker route/secrets, valid owner-supplied
APT subkey and passphrase, protected environment and approved
main tag, public GET body SHA256 readback and exact Actions
archive cleanup. The code-level release package validation
PASS does NOT assert deployed service availability or grant
authorization for tag/main, release, credential or Cloudflare
production mutations. No separate native acceptance or
human-language reviewers need to be solicited for this
offline source-only change. This documentation-only
receipt requires no duplicate CI trigger.

## 2026-10-27 signed flat APT local acceptance

An isolated, test-only flat APT repository was exercised with real
`dpkg-deb`, `dpkg-scanpackages`, GPG clear-signing and
`apt-get update` using private temporary APT source/list/cache
directories and a disposable test public key. The test package was
`cf-vigil_0.1.0_amd64.deb`, where the generated control-index
`Filename` was exactly `./cfvigil_0.1.0_amd64.deb`.
APT successfully fetched the signed flat `InRelease` and
`Packages` and reported the `cf-vigil 0.1.0` candidate at
`file:/...` distribution `./`. No host APT sources or installed
packages were changed and the temporary package/GPG home was removed.
The test emitted a nonfatal `_apt` sandbox warning due to the
owner-private temporary directory; the resolver still confirmed the
signed source. This proves the **flat format in real APT**, not
the latest CFVigil application build, live R2/Cloudflare routing
or an end-user install/upgrade.

Final source-run marker `9fd77fa64b600c7c333039273dc7b8a151bbf811`
requested GitHub Actions Preflight
[run 40585813847](https://github.com/morozhenko-apps/cfvigil/actions/runs/40585813847).
The run remains in progress at this documentation checkpoint.
No passing CI outcome is inferred until terminal status and logs
are independently inspected.

### Final flat distribution source acceptance — 2026-10-27

GitHub-hosted [Preflight #40585813847](https://github.com/morozhenko-apps/cfvigil/actions/runs/40585813847)
completed **SUCCESS** on exact release/CI source SHA
`9fd77fa64b600c7c333039273dc7b8a151bbf811`.
Job log shows **162/162 Node**, **22/22 Python release**,
**3/3 Python icon**, **164/164 Rust**, all PASS:
**351/351 tests, zero failures**.
JS syntax, 20-locale key contracts, icon validation and
locked Rust check also passed. Fewer tests than the
previous 353/353 gate reflect intentional removal of
six obsolete download Worker tests and new exact
flat-package/manifest tests; it is not a coverage regression
claim or complete Mode B/physical acceptance.

The protected tag pipeline is **not run**: the public
`app-dist` bucket was not written, no Cloudflare DNS/WAF
or R2 security settings were changed, no release credentials
were imported, and `main`/tag/release were not changed.
GitHub `cfvigil-release` environment remains a
separate owner-provisioning gate. Website CORS and
`latest.json` consumer integration are outside this
repository change and must be checked before offering
dynamic public downloads. Only documentation was
updated after the successful CI SHA, so no additional
source-gate CI was requested.

### 2026-10-27: persistent real-APT CI acceptance plan

Source baseline: `dev` `174cfc6da934f391591f478471cf431e3004bb4b`.
The flat APT distribution and its publisher were approved and
source-tested in full Preflight #40585813847 (351/351), and a
one-off disposable local APT read was previously documented.
However, the persistent Python suite mocks `gpgv` for publisher
tests; no checked-in CI case currently executes an actual generated
Debian archive, signing command, `apt-get update` or resolver
against the exact `apt_repository.py` flat output.

Root cause: the pipeline's source-level signed index tests cannot
establish real package-manager compatibility. This is a test-evidence
gap, not permission to change the adopted flat R2 storage layout,
remove signature enforcement, or introduce a new Worker.

Scope: add one offline integration test module generating a synthetic
`cf-vigil_0.1.0_amd64.deb`, a **disposable** GPG key in a private
temporary directory, real `apt_repository.assemble`, and an isolated
`file:` apt source with dedicated lists/status/cache directories.
Require a successful real `apt-get update`, candidate resolution and
exact `./cfvigil_0.1.0_amd64.deb` index Filename; deliberately tamper
signed `InRelease` and require apt to reject it. Publish no package,
alter no host APT source, install no system package, and never use or
print real release credentials. Keep the existing package boundary,
`latest.json`, GitHub cleanup, docs/Issue forms and PersonalLanding
unchanged. Include these tools in the hosted Preflight prerequisites:
`dpkg-dev`, `gnupg`, `gpgv` (same apt update/install phase).
Run exactly one final complete source Preflight and record the exact
code SHA plus observed Node, Python and Rust totals. No main/tag/R2/WAF,
credential or production site mutation.

Website continuation: PersonalLanding currently has no CFVigil
catalog entry. Do not publish dead install links before the first
R2 release; after actual public `latest.json` is available, validate
the manifest against a strict same-origin/CORS reader and connect
its versioned URLs with the existing attribution contract. The
absence of a website button during this source-only gate is
intentional, not a passing public distribution check.
