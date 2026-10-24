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
