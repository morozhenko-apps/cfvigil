#!/usr/bin/env bash
# Reuse a pre-existing GPG signing subkey for CFVigil's separate APT archive.
# Explicit --apply writes only CFVigil's signing secrets and primary fingerprint.
# Never import the encrypted full-secret backup or upload a primary secret key.
set -euo pipefail
umask 077

usage() {
  printf 'Usage: bash scripts/provision_existing_apt_key.sh --check|--apply /path/to/FINGERPRINT\n' >&2
  exit 2
}
[[ $# == 2 ]] || usage
MODE="$1"
FINGERPRINT_FILE="$2"
REPO="morozhenko-apps/cfvigil"
ENVIRONMENT="cfvigil-release"
[[ "$MODE" == "--check" || "$MODE" == "--apply" ]] || usage
[[ -f "$FINGERPRINT_FILE" && ! -L "$FINGERPRINT_FILE" ]] || {
  echo "[BLOCKED] Fingerprint file not found or is a symlink." >&2
  exit 2
}
for executable in gpg gpgv gpgconf python3; do
  command -v "$executable" >/dev/null || {
    printf '[BLOCKED] Missing %s.\n' "$executable" >&2
    exit 2
  }
done
[[ -t 0 ]] || {
  echo "[BLOCKED] Run directly in an interactive terminal, never in CI or via a pipe." >&2
  exit 2
}

FPR="$(tr -d '[:space:]' < "$FINGERPRINT_FILE" | tr '[:lower:]' '[:upper:]')"
[[ "$FPR" =~ ^[A-F0-9]{40}$ ]] || {
  echo "[BLOCKED] Expected a 40-hex PRIMARY GPG fingerprint." >&2
  exit 2
}
echo "[INFO] Existing GPG certification-primary fingerprint: $FPR"

# The user may keep an encrypted backup independently. This helper reads
# only the public fingerprint file and the active GPG keyring, not backups.
gpg --batch --list-secret-keys "$FPR" >/dev/null 2>&1 || {
  echo "[BLOCKED] Existing signing subkey is not available in the local GPG keyring." >&2
  echo "[NOTE] The encrypted full-secret backup is not a CI signing-subkey export." >&2
  exit 2
}

TEMP="$(mktemp -d)"
cleanup() {
  gpgconf --homedir "$TEMP/gnupg" --kill gpg-agent >/dev/null 2>&1 || true
  rm -rf -- "$TEMP"
}
trap cleanup EXIT
mkdir -m 700 "$TEMP/gnupg"

# Export the signing subkey directly into a disposable GPG home through a pipe.
# GnuPG's --export-secret-subkeys replaces the primary secret with a stub.
# Do NOT replace this with --export-secret-keys or save the full key in GitHub.
if ! (gpg --batch --export-secret-subkeys "$FPR" |
       gpg --homedir "$TEMP/gnupg" --batch --import >/dev/null 2>&1); then
  echo "[BLOCKED] GPG signing-subkey-only export/import failed." >&2
  exit 2
fi

gpg --homedir "$TEMP/gnupg" --batch --with-colons \
  --list-secret-keys "$FPR" > "$TEMP/inventory"
python3 "$(dirname "$0")/verify_signing_key.py" \
  --listing "$TEMP/inventory" --fingerprint "$FPR"

printf 'Signing subkey passphrase (not the encrypted backup password): ' >&2
IFS= read -rs SIGNING_PASSPHRASE
printf '\n' >&2
[[ -n "$SIGNING_PASSPHRASE" ]] || {
  echo "[BLOCKED] The signing passphrase cannot be empty." >&2
  exit 2
}
printf '%s' "$SIGNING_PASSPHRASE" > "$TEMP/passphrase"
unset SIGNING_PASSPHRASE

printf 'CFVigil local signing verification\n' > "$TEMP/Release"
if ! gpg --homedir "$TEMP/gnupg" --batch --yes --no-tty \
    --pinentry-mode loopback --passphrase-file "$TEMP/passphrase" \
    --digest-algo SHA256 --local-user "$FPR" \
    --clearsign --output "$TEMP/InRelease" "$TEMP/Release" \
    >/dev/null 2>&1; then
  echo "[BLOCKED] This passphrase cannot sign with the exported subkey." >&2
  exit 2
fi
gpg --homedir "$TEMP/gnupg" --batch --export "$FPR" > "$TEMP/public.gpg"
gpgv --keyring "$TEMP/public.gpg" "$TEMP/InRelease" >/dev/null 2>&1 || {
  echo "[BLOCKED] Test signature did not validate with the public keyring." >&2
  exit 2
}

echo "[PASS] The isolated signing subkey imported, signed, and verified."
if [[ "$MODE" == "--check" ]]; then
  echo "[NOTE] Read-only check complete. No GitHub values changed."
  exit 0
fi

command -v gh >/dev/null || {
  echo "[BLOCKED] GitHub CLI gh is missing." >&2
  exit 2
}
gh auth status >/dev/null 2>&1 || {
  echo "[BLOCKED] Log into gh on this computer first." >&2
  exit 2
}
CURRENT_ENV="$(gh api "repos/$REPO/environments/$ENVIRONMENT" --jq '.name' 2>/dev/null)" || {
  echo "[BLOCKED] Existing protected GitHub environment is inaccessible." >&2
  exit 2
}
[[ "$CURRENT_ENV" == "$ENVIRONMENT" ]] || {
  echo "[BLOCKED] Environment identity does not match." >&2
  exit 2
}
# Fail closed on stale/partial setup: an existing signing secret might be
# deliberate and must not be silently replaced by a shared key.
EXISTING="$(gh secret list -R "$REPO" -e "$ENVIRONMENT" \
  --json name --jq '.[].name' 2>/dev/null)" || {
  echo "[BLOCKED] Could not inspect existing environment secret names." >&2
  exit 2
}
for key in CFVIGIL_APT_SIGNING_SUBKEY_BASE64 CFVIGIL_APT_SIGNING_PASSPHRASE; do
  if grep -Fqx "$key" <<< "$EXISTING"; then
    echo "[BLOCKED] $key already exists; refusing to overwrite existing signing authority." >&2
    exit 2
  fi
done

# Export from the exact isolated keyring just verified, not another local key
# and not the original encrypted full-secret backup. Pipe private data only
# into gh; stdout/stderr never echo it.
gpg --homedir "$TEMP/gnupg" --batch --export-secret-subkeys "$FPR" |
  base64 -w0 |
  gh secret set CFVIGIL_APT_SIGNING_SUBKEY_BASE64 \
    -R "$REPO" -e "$ENVIRONMENT"
cat "$TEMP/passphrase" |
  gh secret set CFVIGIL_APT_SIGNING_PASSPHRASE \
    -R "$REPO" -e "$ENVIRONMENT"
gh variable set CFVIGIL_APT_FINGERPRINT \
  -R "$REPO" -e "$ENVIRONMENT" --body "$FPR"

echo "[PASS] Configured CFVigil's two signing secrets and PRIMARY fingerprint."
echo "[NOTE] No R2 secret, approved tag, main ref or RepoTicker setting changed."
