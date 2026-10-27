## 2026-10-27: owner selects first public release v1.0.0 (source preparation)

The owner explicitly chose `v1.0.0` rather than an initial
`v0.1.0` public release. No previous public GitHub Release
exists. This is **version selection**, not a direction to
override the separate mandatory exact-main, protected-environment,
native/Mode-B and owner-approval gates. The environment includes
two approved signing secrets, two bucket-access secrets, an APT
primary fingerprint variable and a Cloudflare R2 account ID.
`CFVIGIL_RELEASE_APPROVED_TAG` remains intentionally unset.

Prepare release identity in the single Rust package
`apps/desktop/src-tauri/Cargo.toml`, generated root
`Cargo.lock` CFVigil stanza, and
`apps/desktop/src-tauri/tauri.conf.json`. Do not change the
package name `cf-vigil`, executable `cfvigil`,
bundle identifier `com.morozhenko.cfvigil`,
OS keyring namespace or application data paths.
Upgrade candidate-version gate and product identity
tests to require all three versions agree and reject
a mismatched local Cargo.lock stanza. Leave
`0.1.0` in deliberately synthetic older-version
fixtures and historical QA evidence; those values
are not declarations of the current app version.

One complete source Preflight on a final exact
SHA must confirm the updated stable JavaScript,
Python and locked Rust gates. After that, document
observed counts and exact commit SHA. The
release pipeline's `node scripts/candidate-version.mjs
--check-tag v1.0.0` must pass, and old `v0.1.0`
must fail. Do not run public release workflow,
tag/merge main, write R2, mutate shared credentials,
or fabricate `latest.json` before owner release
authorization and live acceptance. Rollback is a
scoped source revert, not R2 deletion.

## 2026-10-27: optional reuse of an existing APT signing authority

Owner's authenticated CLI confirmed that `cfvigil-release` was
**CREATED** and has the owner-reviewer and `v*` tag policy,
while all three release environment variable NAMES and four
secret NAMES were missing. The dev/main SHA mismatch is
intentionally unsatisfied, pending explicit release approval.
This is owner-supplied CLI output, not an independent private
GitHub environment readback.

**Key clarification:** the checked-in
`scripts/verify_signing_key.py` requires `CFVIGIL_APT_FINGERPRINT`
to be the **40-hex certification PRIMARY fingerprint**, not a
signing subkey fingerprint. It requires a signing-subkey-only
export with the primary secret stripped (one primary stub and
exactly one working signing subkey). This supersedes a mistaken
"subkey fingerprint" description in the prior operational
variables table. Do not change the validator to accept a full
primary export.

The owner has an existing RepoTicker APT keyring under
`$HOME/morozhenko/secrets/RepoTicker/`, including a
`FINGERPRINT`, public keyring and an **encrypted full-secret
backup**. Reusing its existing primary plus its single signing
subkey across the CFVigil APT archive is technically compatible
without changing the cryptographic validator or flat R2 layout.
The 40-hex fingerprint identifies a key; by itself it cannot
sign. The local active `gpg` keyring must actually contain the
corresponding private signing subkey. The encrypted full-secret
backup and unknown `key.txt` must **never** be uploaded into
GitHub Secrets or printed in a chat. Use only
`gpg --export-secret-subkeys` from a verified active local
keyring. Send the signing-subkey export to CFVigil's own
environment secret name and passphrase secret name.

**Trade-off:** GitHub repository environment secrets remain
separate, but the common signing key creates shared trust and
revocation/compromise blast radius: leaking the signing subkey
via either pipeline potentially affects **both** APT archives.
CFVigil remains independently distributable, and later
migration to a CFVigil-only primary/subkey plus updated
signed-by keyring is supported. Do not share R2 API keys merely
because the GPG signing authority is shared. A new independent
signing identity is still the safer default when strict
product-level isolation is required.

Implement a local `scripts/provision_existing_apt_key.sh`
with an audit-only `--check` mode and an explicit `--apply`
mode. Both obtain the actual fingerprint from the selected
public FINGERPRINT file; use GPG's own sealed signing subkey,
validate primary stub/only signing subkey using the existing
validator, then sign a disposable release note using the
owner-entered passphrase and verify it with `gpgv`. The
check-only mode makes no GitHub writes. The explicit apply
mode may write ONLY
`CFVIGIL_APT_FINGERPRINT`,
`CFVIGIL_APT_SIGNING_SUBKEY_BASE64` and
`CFVIGIL_APT_SIGNING_PASSPHRASE` to existing
`cfvigil-release`; it must not touch R2 credentials,
`CFVIGIL_RELEASE_APPROVED_TAG`, app-dist objects, main,
tags or any RepoTicker environment. Never export the full
primary private key and never save the unencrypted subkey
export to disk. Gate all writes on the successful isolated
signing test. Do not execute an owner-side key provisioning
command within hosted CI or agent tools.
Use `bash scripts/provision_existing_apt_key.sh --check "$HOME/morozhenko/secrets/RepoTicker/FINGERPRINT"` on the owner's Ubuntu as the initial **no-write** signing check. Run `--apply` on the same file only after the isolated signing and passphrase are confirmed. The command will not inspect `key.txt` or the encrypted full-secret backup; never paste the signing passphrase in ChatGPT. 
 Validate only the
script's synthetic/static gates in the next single Preflight.

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

### Persistent signed flat APT client gate (2026-10-27)

`tests/test_release_apt_integration.py` now exercises
`apt_repository.assemble` with a synthetic package produced by
actual `dpkg-deb`, a disposable unsigned-to-user GPG test identity,
real `gpg` clearsigning, its exported public keyring and an
isolated `apt-get update` using the approved `./` flat catalog.
The test asserts that the real resolver finds `cf-vigil 0.1.0`
at `Filename: ./cfvigil_0.1.0_amd64.deb`, with no pool/dists
hierarchy, and independently checks that a changed but not
re-signed `InRelease` is rejected. It never installs the dummy
package, changes `/etc/apt`, alters host trust, uses a production
key or accesses Cloudflare. Temporary keyring, lists and
package files are cleaned by the test fixture.

`.github/workflows/checks.yml` installs `dpkg-dev`, `gnupg`
and `gpgv` alongside its existing Linux dependencies and runs
these tests within the ordinary offline release suite. A local
disposable rehearsal against a module mirroring the approved
generator passed **2/2** before this change was pushed.
That rehearsal is **not** evidence that GitHub CI has run the
new checked-in test against exact `dev` source; the final
source SHA and hosted run must be verified independently.

PersonalLanding's source has no CFVigil card or landing route
yet. The first website install URL must remain absent until
a real `latest.json` artifact and public end-to-end byte
readback are available. The website consumer must validate
manifest schema, product identity, release version, canonical
same-product platform URLs and hashes before exposing a
download, and preserve existing attribution/UTM behavior.
If the manifest cannot be loaded, do not invent a version or
link to an unverified guessed file. CORS for browser-side
direct R2 reads remains unconfigured in this source-only task.

### Hosted signed flat APT acceptance — completed 2026-10-27

[Preflight #40589861497](https://github.com/morozhenko-apps/cfvigil/actions/runs/40589861497)
completed **SUCCESS** on exact checked-in source/CI SHA
`07d2eccbc611bade2e1187970bb86964ecb98fe0`.
The job log reports **Node 162/162**, **Python release 24/24**
(including the two real APT integration methods),
**Python icons 3/3** and **Rust 164/164**:
**353/353 PASS, zero failures**. All companion syntax,
localization, icon and locked Cargo checks succeeded.

The new CI cases executed the exact `apt_repository.assemble`
using real Debian tools and GPG under an isolated, temporary
client configuration. An actual APT client accepted the
`./` source and resolved `cf-vigil 0.1.0`; it rejected
a mutated but not re-signed `InRelease`. They validate
the signed *local* flat APT protocol, not a public R2
upload or existing installed Ubuntu package. This run
did not use CFVigil release credentials, issue tags,
other products, production R2 or website settings.

Only documentation changed after the passing code SHA;
no duplicate CI requested. Next operational gates still
require the owner-configured protected environment,
real signed source release tag, Cloudflare publicly
reachable `latest.json` and the eventual
PersonalLanding consumer with safe fallback.

### 2026-10-27: first protected release provisioning preflight (Mode A)

Baseline: dev \`c4909c249d7595b206619c99ef03adab2642226b\`,
main \`08328f75cb54d629b2ac33fb98a42027c4ab2a3c\`,
the complete source Preflight #40589861497 PASS (353/353).
Previously the owner's authenticated CLI returned GitHub REST 404
for \`cfvigil-release\` environment and consequently its variables
and secrets; 404 does not distinguish absent from inaccessible
with certainty. The CLI lacked a globally installed Wrangler.
No release was made. The protected tag workflow exists on dev only.

**Selected architecture:** preserve the already-approved release
workflow, flat signed APT, existing shared \`app-dist\` bucket,
\`dist/CFVigil/\` prefix, direct R2 custom-domain downloads,
Linux/Windows scope and final \`latest.json\` readback. Do not
introduce a new bucket, Worker, duplicate binary, generic GitHub
Actions branch bypass or unprotected release dispatch.

**Root of current operational gap:** hosted CI cannot inspect
the owner's GitHub environment secrets, Cloudflare bucket
ownership and external key material, and our GitHub connector
cannot create GitHub environments or query their private
settings. The previous manual multiline audit was awkward to
paste and did not clearly distinguish unknown from absent.

**Implement source-controlled local one-command setup:**
1. A Python stdlib-only \`scripts/release_readiness.py\`
   that invokes the already authenticated \`gh api\`, **read-only by
   default**. Check exact three variable NAMES and four secret
   NAMES (never fetch, print or persist values), protection
   requiring owner review and deployment tag policy \`v*\`;
   show dev/main heads without modifying Git references.
   A 404 or authorization denial must be reported as
   "missing OR inaccessible", not as proven absent.
2. An **explicit owner-invoked**
   \`python3 scripts/release_readiness.py --init-environment\`
   provisioning mode may create ONLY the missing
   \`cfvigil-release\` GitHub Environment (not other
   repositories) with the current authenticated owner as
   required reviewer, \`prevent_self_review=false\` for
   a solo maintainer and the sole \`v*\` tag deployment
   restriction. Refuse to overwrite any existing environment;
   refuse without confirmed repository admin rights;
   never add credentials, create tags, promote main,
   enable deployments, modify Cloudflare or implicitly
   approve a version. Verify the result by a fresh name-only
   readback. GitHub's API may deny required reviewers on
   private repositories depending on plan; report failure
   rather than silently weaken protection.
3. Add deterministic offline unit tests for a ready environment,
   absent/inaccessible environment, missing or misconfigured
   reviewer/tag gate, missing variable/secret names, and
   non-destructive init refusal. Mock subprocess/API, never
   exercise a live token in hosted CI.
4. Record concise owner instructions for populating the
   three approved variables and four secrets separately
   through GitHub's UI or CLI (values are user-controlled),
   R2 bucket and custom-domain public GET confirmation,
   CORS for eventual website browser use, and real
   dedicated GPG signing subkey provenance. Preserve
   all existing Picturix objects.
5. One final complete source-only Preflight (Node, Python,
   Rust) on exact code SHA; record observed result.
   Do not run the protected publish job or tag/main change
   without the owner separately authorizing that release.

**Rollback:** revert just this helper and its synthetic tests;
no live environment or R2 state is mutated by writing the code.
\`--init-environment\` is an explicit external action to be
run on the owner's machine and is *not* executed by this
source-only implementation.

### Owner runbook: configure the first CFVigil protected release

Use the following **single commands** in a local clone updated from
\`morozhenko-apps/cfvigil\` \`dev\`:

\`\`\`bash
python3 scripts/release_readiness.py
\`\`\`

This is the default **read-only** audit; it prints the existence of
GitHub Environment settings and only *credential names*, never their
values. It exits code 2 if protected reviewers, tag policy, variable
or secret names are absent/inaccessible, or if \`main\` has not yet
received the approved \`dev\` code. A GitHub 404 is deliberately
reported as **missing OR inaccessible**, never as proof that a secret
or environment is absent. A default-branch mismatch is expected
during development and must not be auto-fixed by this helper.

If and only if the owner chooses to create an otherwise absent
GitHub environment, run:

\`\`\`bash
python3 scripts/release_readiness.py --init-environment
\`\`\`

The explicit command requires authenticated \`gh\` CLI **admin**
permissions on \`morozhenko-apps/cfvigil\`, checks the existing
environment list first, refuses to overwrite existing settings,
and creates \`cfvigil-release\` with the initiating account as
required reviewer (solo-maintainer self-review enabled) and only
\`v*\` tag deployment policy. The operation makes a GitHub
environment and tag rule but does **NOT** set secrets, generate
keys, create a tag, merge branches, publish to R2, or run a release.
If GitHub rejects the reviewer policy, stop and inspect the repo
environment rather than silently removing the guard.

Required **Environment Variables** (names only; owner supplies values):

| Name | Correct kind of value |
| --- | --- |
| \`CFVIGIL_R2_ACCOUNT_ID\` | Cloudflare account ID hosting the preexisting shared \`app-dist\` bucket |
| \`CFVIGIL_APT_FINGERPRINT\` | 40-hex **certification primary** fingerprint; the usable signing subkey remains a separate secret export |
| \`CFVIGIL_RELEASE_APPROVED_TAG\` | Exact tag for the specific owner-approved release, for example \`v0.1.0\`; set this **last**, after promotion and native acceptance |

Required **Environment Secrets**, never show/paste values in
issue reports, PRs or chat:

| Name | Purpose |
| --- | --- |
| \`CFVIGIL_R2_ACCESS_KEY_ID\` | Explicitly provisioned R2 API credential for shared bucket \`app-dist\` |
| \`CFVIGIL_R2_SECRET_ACCESS_KEY\` | R2 API secret associated with that key |
| \`CFVIGIL_APT_SIGNING_SUBKEY_BASE64\` | Owner's export of dedicated passphrase-protected CFVigil signing **subkey**, not the offline primary secret key |
| \`CFVIGIL_APT_SIGNING_PASSPHRASE\` | Passphrase for only the dedicated CFVigil signing subkey |

GitHub Environment variables/secrets are **per repository**:
RepoTicker's \`apt-release\` environment and its keys do not appear
in CFVigil automatically. The release workflow checks the approved
tag, cryptographic key inventory and R2 public readback **at
publication time**; the readiness helper validates only metadata
and cannot prove that secrets are correct. Shared \`app-dist\`
credentials are scoped at least to that **entire bucket** by
Cloudflare R2, not to the \`dist/CFVigil/\` prefix; the publisher
itself constrains all object paths to that prefix. Never
automatically reuse or rotate Picturix/RepoTicker credentials.

**Confirm Cloudflare without changing it:** in the R2 dashboard
verify existing \`app-dist\` bucket, its active
\`download.morozhenko.com\` Custom Domain and public GET of a known
Picturix release object. A \`POST\` status of \`401\` or \`405\`
does **not** test public download access; GET/HEAD are the
relevant methods. The actual CFVigil release job independently
downloads every newly published file over public HTTP and
verifies its full SHA256 and length before signaling success.
Do not create a new bucket, Worker or WAF rule here. Later,
for browser-side \`latest.json\` reads from \`morozhenko.com\`,
configure read-only GET/HEAD CORS for the existing bucket
and verify the effective response with an Origin header,
without breaking Picturix.

**Owner-controlled promotion only:** once physical Linux/Windows
acceptance and the separate 20-locale/editorial gate are
approved, fast-forward \`main\` to the validated \`dev\` source,
set \`CFVIGIL_RELEASE_APPROVED_TAG\` to the exact intended
version, create the matching protected \`v*\` tag on that
exact main commit, approve the environment deployment in
GitHub, and verify the public receipt plus binary archive
cleanup. Do not automate these owner approvals in this
preparation task. The first published
\`dist/CFVigil/latest.json\` then becomes the source for
PersonalLanding's future download buttons. No website
buttons should be added before that successful public
readback.

Rollback of explicit environment bootstrap (if needed) is
a manual owner review of the repo's Environments settings
and tag policies; **never delete a pre-existing environment**
just because this helper ran. The check-only command has
no rollback requirements.

### Completed source verification: owner-side release environment helper

[GitHub Actions Preflight #40591766631](https://github.com/morozhenko-apps/cfvigil/actions/runs/40591766631)
completed **SUCCESS** on the exact implementation/CI SHA
\`7f13fa072a59d0ab6a437af8a0c57dc0f78acabe\`.
Observed test log: **162/162 Node**, **35/35 Python
release** (including eleven isolated release-readiness
cases), **3/3 Python icon**, **164/164 Rust**. Total:
**364/364 PASS**, 0 failures, together with JavaScript
syntax, localization, original icon and locked Cargo
verification. No production-privileged tests were
executed. Documentation-only commits after that SHA
do not require another source-gate run.

Exact checked-in helper usage from a current \`dev\` checkout:

\`\`\`bash
python3 scripts/release_readiness.py
\`\`\`

The nonmutating default checks environment protection,
required named variables and secrets and actual
\`main\`/ \`dev\` SHA parity. When the owner explicitly
chooses to provision GitHub environment, only on their
authenticated machine:

\`\`\`bash
python3 scripts/release_readiness.py --init-environment
\`\`\`

Bootstrap is limited to creating a **previously absent**
\`cfvigil-release\` GitHub environment, single owner
reviewer and \`v*\` tag policy. Neither command adds
credential values, signs a release, touches \`app-dist\`,
updates Git refs or publishes installers. The owner must
configure the four secrets and three variables out of
band, retain a dedicated signing subkey, confirm shared
bucket credentials and actual public Cloudflare GET.
\`CFVIGIL_RELEASE_APPROVED_TAG\` remains unset until
a specific final release is approved.

This source gate **does not resolve** the previous
owner-side GitHub 404 nor demonstrate secret validity.
The workflow exists only on \`dev\`; \`main\` promotion
and first \`v*\` tag remain separately owner-authorized.
No site download buttons or WAF rules have been
changed. The public distribution is still unreleased.

### Final source verification — optional shared signing subkey helper

GitHub-hosted [Preflight #40594097832](https://github.com/morozhenko-apps/cfvigil/actions/runs/40594097832) completed **SUCCESS** on exact code/CI source SHA `2f4e3315b6f19466e7b066b485f651af1a26868a`. Full job log: Node **162/162**, Python release **38/38** (including three new signing helper safety checks), Python icon **3/3**, Rust **164/164**, total **367/367 PASS**, zero failures. Existing real flat GPG/APT integration tests remained successful.

This is source and synthetic CLI acceptance, **not** evidence that the owner's existing GPG signing key is available locally, that their passphrase works or that GitHub release signing secrets have been provisioned. The new script has not been run with real key material by the agent, and no R2 or GitHub environment was modified. The owner must run `--check` then explicitly `--apply` for shared key use if the isolated signing preflight passes. Documentation-only commits after the tested code SHA require no second CI run.

### Owner-provided real signing provisioning evidence — 2026-10-27

The owner ran `scripts/provision_existing_apt_key.sh --check` and
`--apply` against their existing RepoTicker public
`FINGERPRINT` file. Both reported that the signing-subkey-only
export imported into the isolated GPG home and signed, with
`gpgv` successfully verifying the disposable signed message.
The reported primary fingerprint is
`757491684BE4A8ED51FB3668DF4D57DD6F1E7B22`.
The `--apply` output confirms that the CLI accepted writes
for `CFVIGIL_APT_SIGNING_SUBKEY_BASE64`,
`CFVIGIL_APT_SIGNING_PASSPHRASE` and environment variable
`CFVIGIL_APT_FINGERPRINT` in
`morozhenko-apps/cfvigil / cfvigil-release`.
These are names and public identity only; no secret values were
observed, retrieved, or committed to source. This evidence
covers owner-side key usability and CLI acknowledgement, not
a GitHub-hosted protected-release import or public R2 publish.

**Next operational gate:** confirm the GitHub Environment secret
**names** with `gh secret list -R morozhenko-apps/cfvigil
-e cfvigil-release`, provision an independent Cloudflare R2
**Object Read & Write** token restricted to existing bucket
`app-dist` (not account admin permissions), and set only
`CFVIGIL_R2_ACCESS_KEY_ID`,
`CFVIGIL_R2_SECRET_ACCESS_KEY` environment secrets and
`CFVIGIL_R2_ACCOUNT_ID` variable. Cloudflare's R2 token
scope is bucket-level, not `dist/CFVigil/` prefix-level;
the publisher's prefix guard does not isolate a stolen R2 key
from sibling objects in `app-dist`. Credentials for RepoTicker
and Picturix must not be re-used by accident. The account ID
must match the account owning `app-dist` and the S3 endpoint
used by `scripts/r2_release.py`. If the bucket has a custom
jurisdiction endpoint such as EU, current default-endpoint
code must be revised and validated before a release.

Do not populate `CFVIGIL_RELEASE_APPROVED_TAG`, advance
`main`, or tag a release until the separate native
acceptance, distribution and review gates are satisfied.

### Owner-selected v1.0.0 source identity acceptance (2026-10-27)

GitHub [Preflight #40598381215](https://github.com/morozhenko-apps/cfvigil/actions/runs/40598381215) completed **SUCCESS** at exactly `ad32baa710a5135393c354b317815804c444d5d5`. Observed logs: Node 163/163, Python release 38/38, Python icon 3/3, Rust 164/164 — **368/368 passed, zero failures**. The release source metadata `Cargo.toml`, root `Cargo.lock` CFVigil package stanza, and Tauri config all report **1.0.0**. The Node release guard rejects `v0.1.0` against the current source, accepts `v1.0.0`, checks exact Cargo.lock parity including Windows CRLF, and the protected tag validation job now explicitly installs `dpkg-dev`, `gnupg`, `gpgv` before running the real APT integration tests.

No production release occurred during the source gate. Main still has its older SHA; `CFVIGIL_RELEASE_APPROVED_TAG` is intentionally absent, and the existing `app-dist` bucket and signing environment are untouched by this version bump. Next validation is *unpublished* Linux/Windows candidate compilation from the same immutable source SHA. A build succeeding is not itself physical install, signing, public distribution or final owner release approval.
