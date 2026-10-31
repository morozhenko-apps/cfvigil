#!/usr/bin/env python3
"""CFVigil flat signed R2 publication. Never invoked by candidate workflows.

Publishing requires an owner-approved exact-main tag, a protected environment,
a separately provisioned signing subkey, and scoped CFVigil R2 credentials.
The verified receipt is written ONLY after every public download matches.
"""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import urllib.request
import zlib

BUCKET = "app-dist"
PREFIX = "dist/CFVigil/"
BASE = "https://download.morozhenko.com/" + PREFIX
PUBLIC_READBACK_USER_AGENT = (
    "CFVigil-ReleaseVerifier/1.0 (+https://github.com/morozhenko-apps/cfvigil)"
)
VERSION = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\Z")
COMMIT = re.compile(r"[a-f0-9]{40}\Z")
MIMES = {
    ".deb": "application/vnd.debian.binary-package",
    ".AppImage": "application/octet-stream",
    ".exe": "application/octet-stream",
    ".gz": "application/gzip",
    ".gpg": "application/pgp-keys",
    ".sha256": "text/plain",
    ".json": "application/json",
}


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for part in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(part)
    return h.hexdigest()


def safe_file(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError("Missing or unsafe release file: " + path.name)
    size = path.stat().st_size
    if not 0 < size <= 350 * 1024 * 1024:
        raise ValueError("Invalid release file size: " + path.name)
    return size


def platform_files(release, platform, version, commit):
    names = {
        "linux": [f"cfvigil_{version}_amd64.deb",
                  f"CFVigil-{version}-linux-x64.AppImage"],
        "windows": [f"CFVigil-Setup-{version}-win-x64.exe"],
    }
    folder = release / platform
    if folder.is_symlink() or not folder.is_dir():
        raise ValueError("Missing platform directory: " + platform)
    manifest = folder / "candidate.json"
    safe_file(manifest)
    content = json.loads(manifest.read_text(encoding="utf8"))
    expected_platform = {"linux": "linux-amd64", "windows": "windows-x64"}[platform]
    if any([
        content.get("schema") != 1, content.get("product") != "CFVigil",
        content.get("version") != version, content.get("commit") != commit,
        content.get("platform") != expected_platform, content.get("release") is not False,
    ]):
        raise ValueError("Candidate provenance mismatch: " + platform)
    if platform == "windows" and content.get("signed") is not False:
        raise ValueError("Windows installer signature status not confirmed")
    expected = {"candidate.json"}
    entries = {}
    manifest_entries = {}
    for item in content.get("files", []):
        if not isinstance(item, dict) or not isinstance(item.get("name"), str):
            raise ValueError("Invalid candidate file manifest")
        name = item["name"]
        if name in manifest_entries:
            raise ValueError("Duplicate manifest entry")
        manifest_entries[name] = (item.get("sha256"), item.get("size"))
    for name in names[platform]:
        file = folder / name
        sidecar = folder / (name + ".sha256")
        actual_size = safe_file(file)
        safe_file(sidecar)
        sha = digest(file)
        if manifest_entries.get(name) != (sha, actual_size):
            raise ValueError("Candidate checksum/size mismatch: " + name)
        if sidecar.read_text(encoding="ascii").strip() != sha + "  " + name:
            raise ValueError("Candidate checksum sidecar mismatch: " + name)
        expected.update((name, name + ".sha256"))
        for item in (file, sidecar):
            key = platform + "/" + item.name
            entries[key] = item
    if set(manifest_entries) != set(names[platform]) or {
        child.name for child in folder.iterdir()
    } != expected:
        raise ValueError("Unexpected artifact in release folder: " + platform)
    return entries



def verify_signed_apt_indexes(inrelease, index):
    """Validate the bytes APT's verified InRelease SHA256 section commits to.

    Caller must verify the signature with gpgv first. A valid signature
    over OLD index hashes is not proof of the new Packages files.
    """
    try:
        lines = inrelease.read_text(encoding="utf8").splitlines()
        blank = lines.index("", 1)
        signed_end = lines.index("-----BEGIN PGP SIGNATURE-----", blank + 1)
    except (UnicodeError, ValueError) as error:
        raise ValueError("Signed APT index has a malformed cleartext envelope") from error
    if (not lines or lines[0] != "-----BEGIN PGP SIGNED MESSAGE-----"
            or "Hash: SHA256" not in lines[1:blank]
            or signed_end <= blank + 1
            or "-----END PGP SIGNATURE-----" not in lines[signed_end + 1:]):
        raise ValueError("Signed APT index has a malformed cleartext envelope")

    required = {
        "Packages": index / "Packages",
        "Packages.gz": index / "Packages.gz",
    }
    found = {}
    in_sha256 = False
    saw_sha256 = False
    for line in lines[blank + 1:signed_end]:
        if line == "SHA256:":
            if saw_sha256:
                raise ValueError("Signed APT index repeats SHA256 section")
            saw_sha256, in_sha256 = True, True
            continue
        if not in_sha256:
            continue
        if not line.startswith((" ", "\t")):
            in_sha256 = False
            continue
        match = re.fullmatch(r"\s+([a-fA-F0-9]{64})\s+([0-9]+)\s+(\S+)\s*", line)
        if match is None:
            raise ValueError("Signed APT index has a malformed SHA256 row")
        expected_sha, size, name = match.groups()
        if name in required:
            if name in found:
                raise ValueError("Signed APT index repeats a package-index entry")
            found[name] = (expected_sha.lower(), int(size))
    if not saw_sha256 or set(found) != set(required):
        raise ValueError("Signed APT index lacks the required SHA256 entries")
    for name, package_index in required.items():
        if not package_index.is_file():
            raise ValueError("Signed APT index is missing a referenced file")
        signed_sha, signed_size = found[name]
        if signed_size != package_index.stat().st_size or signed_sha != digest(package_index):
            raise ValueError("Signed APT index SHA256 or size does not match the file")



def verify_apt_package_catalog(packages, compressed, pool, version):
    """Bind APT's signed single-package index to the exact approved DEB.

    The caller must first authenticate InRelease and confirm its SHA256
    records for both index files. Independent signatures over two files
    do not establish that either describes the correct package, nor
    that the compressed variant contains the same index bytes.
    """
    try:
        lines = packages.read_text(encoding="utf8").splitlines()
    except UnicodeError as error:
        raise ValueError("APT package catalog is not valid UTF-8") from error
    critical = {"Package", "Version", "Architecture", "Filename", "Size", "SHA256"}
    entries = []
    fields = {}
    previous = None
    for line in lines:
        if not line:
            if fields:
                entries.append(fields)
                fields = {}
            previous = None
            continue
        if line[0] in " \t":
            # Debian control files permit folded Description and other
            # informational fields, but critical identities cannot fold.
            if previous is None or previous in critical:
                raise ValueError("APT package catalog has unsafe continuation")
            continue
        key, separator, value = line.partition(":")
        if (not separator or
                not re.fullmatch(r"[A-Za-z][A-Za-z0-9-]*", key) or
                key in fields):
            raise ValueError("APT package catalog contains malformed or duplicate fields")
        fields[key] = value.strip()
        previous = key
    if fields:
        entries.append(fields)
    if len(entries) != 1:
        raise ValueError("APT package catalog must contain one exact CFVigil package")
    record = entries[0]
    expected = {
        "Package": "cf-vigil",
        "Version": version,
        "Architecture": "amd64",
        "Filename": f"./cfvigil_{version}_amd64.deb",
        "Size": str(pool.stat().st_size),
        "SHA256": digest(pool),
    }
    if any(record.get(key) != value for key, value in expected.items()):
        raise ValueError("APT package catalog does not match approved DEB")

    # Compare bounded output chunks to the signed plain index. Never
    # unboundedly inflate a compressed file into memory.
    try:
        with packages.open("rb") as plain, gzip.open(compressed, "rb") as packed:
            while chunk := plain.read(1024 * 1024):
                if packed.read(len(chunk)) != chunk:
                    raise ValueError("APT package catalog gzip differs from Packages")
            if packed.read(1):
                raise ValueError("APT package catalog gzip contains additional data")
    except (EOFError, OSError, zlib.error) as error:
        raise ValueError("APT package catalog has invalid gzip") from error


def apt_files(apt, linux_deb, version):
    if apt.is_symlink() or not apt.is_dir():
        raise ValueError("Incomplete signed flat APT directory")
    # apt_repository.py stages a local copy for dpkg-scanpackages;
    # the published object is only linux/cfvigil_VERSION_amd64.deb
    # from the platform candidate, never this second staging file.
    staged = apt / f"cfvigil_{version}_amd64.deb"
    keyring = apt / "cfvigil-archive-keyring.gpg"
    packages = apt / "Packages"
    compressed = apt / "Packages.gz"
    inrelease = apt / "InRelease"
    for file in (staged, keyring, packages, compressed, inrelease):
        safe_file(file)
    if digest(staged) != digest(linux_deb):
        raise ValueError("Flat APT staged DEB does not match validated CFVigil DEB")
    subprocess.run(["gpgv", "--keyring", str(keyring), str(inrelease)],
                   check=True, stdout=subprocess.DEVNULL)
    verify_signed_apt_indexes(inrelease, apt)
    verify_apt_package_catalog(packages, compressed, linux_deb, version)
    return ({"linux/" + keyring.name: keyring},
            {"linux/" + packages.name: packages,
             "linux/" + compressed.name: compressed,
             "linux/" + inrelease.name: inrelease})


def source_map(release, apt, version, commit):
    if not VERSION.fullmatch(version) or not COMMIT.fullmatch(commit):
        raise ValueError("Invalid signed-release version or commit")
    linux = platform_files(release, "linux", version, commit)
    windows = platform_files(release, "windows", version, commit)
    package = linux["linux/cfvigil_" + version + "_amd64.deb"]
    apt_immutable, apt_mutable = apt_files(apt, package, version)
    immutable = {**linux, **windows, **apt_immutable}
    if set(immutable).intersection(apt_mutable):
        raise ValueError("Overlapping R2 publication keys")
    return immutable, apt_mutable, {
        "cfvigil-release-linux": linux,
        "cfvigil-release-windows": windows,
    }


def latest_manifest(immutable, version, commit):
    """A small mutable pointer; never a second binary or an update signature."""
    def item(key):
        file = immutable[key]
        return {"url": BASE + key, "sha256": digest(file), "size": file.stat().st_size}
    return {
        "schema": 1, "product": "CFVigil",
        "version": version, "tag": "v" + version, "commit": commit,
        "files": {
            "linux": {
                "deb": item(f"linux/cfvigil_{version}_amd64.deb"),
                "appimage": item(f"linux/CFVigil-{version}-linux-x64.AppImage"),
            },
            "windows": {
                "installer": item(f"windows/CFVigil-Setup-{version}-win-x64.exe")
            },
        },
    }


def content_type(path):
    if path.name == "InRelease" or path.name == "Packages":
        return "text/plain; charset=utf-8"
    return MIMES.get(path.suffix, "application/octet-stream")


def public_readback(path, relative, opener=urllib.request.urlopen):
    # Direct R2 custom-domain readback is mandatory; private S3 reads alone
    # cannot demonstrate that visitors can download this exact object.
    expected_size = path.stat().st_size
    expected_sha = digest(path)
    # Identify the application rather than urllib's generic client, which
    # download-edge integrity checks may reject before reaching R2.
    request = urllib.request.Request(
        BASE + relative, headers={"User-Agent": PUBLIC_READBACK_USER_AGENT},
        method="GET",
    )
    for attempt in range(4):
        try:
            with opener(request, timeout=45) as reply:
                if reply.status != 200:
                    raise ValueError("Public R2 download returned non-200")
                hasher = hashlib.sha256()
                length = 0
                while chunk := reply.read(1024 * 1024):
                    hasher.update(chunk)
                    length += len(chunk)
                if length != expected_size or hasher.hexdigest() != expected_sha:
                    raise ValueError("Public R2 download does not match source")
                return {
                    "url": BASE + relative, "sha256": expected_sha,
                    "readback_sha256": hasher.hexdigest(), "size": expected_size,
                    "readback_size": length, "readback_status": 200,
                }
        except (OSError, ValueError):
            if attempt == 3:
                raise
            time.sleep(2 * (attempt + 1))


_NO_SNAPSHOT = object()


def object_head(client, key):
    """Strongly consistent S3 snapshot; only an explicit 404 is absent."""
    try:
        return client.head_object(Bucket=BUCKET, Key=PREFIX + key)
    except Exception as error:
        status = getattr(error, "response", {}).get("ResponseMetadata", {}).get("HTTPStatusCode")
        if status == 404:
            return None
        raise


def object_etag(head):
    """Validate the exact opaque, quoted tag required by S3 IfMatch."""
    if head is None:
        return None
    etag = head.get("ETag")
    if not isinstance(etag, str) or not re.fullmatch(r'"[^"\r\n]{1,128}"', etag):
        raise ValueError("Missing or malformed R2 object ETag")
    return etag


def upload(client, key, file, immutable, expected_latest=_NO_SNAPSHOT):
    """Single-part, per-object conditional write with mandatory private readback."""
    object_key = PREFIX + key
    size = safe_file(file)
    sha = digest(file)
    existing = object_head(client, key)
    if expected_latest is not _NO_SNAPSHOT:
        # Never promote a release based on an ETag observed AFTER
        # the mutable APT index publication has already begun.
        if object_etag(existing) != object_etag(expected_latest):
            raise ValueError("Concurrent latest pointer changed during publication")
        if existing is not None and existing.get("Metadata", {}).get("sha256") == sha:
            # Same version/SHA recovery: do not rewrite an already committed
            # pointer merely because the public CDN verification previously failed.
            pass
    if existing and immutable:
        if existing.get("Metadata", {}).get("sha256") != sha:
            raise ValueError("Refusing to overwrite immutable CFVigil R2 object")
    elif not (existing and existing.get("Metadata", {}).get("sha256") == sha):
        conditional = ({"IfNoneMatch": "*"} if existing is None else
                       {"IfMatch": object_etag(existing)})
        # boto3 upload_file has no per-object atomic creation contract.
        # R2 supports single-part streamed PutObject for these bounded files.
        with file.open("rb") as source:
            client.put_object(
                Bucket=BUCKET, Key=object_key, Body=source,
                ContentLength=size, ContentType=content_type(file),
                Metadata={"sha256": sha},
                CacheControl=("public,max-age=31536000,immutable"
                              if immutable else "no-store"),
                **conditional,
            )
    elif not immutable:
        # Even idempotent mutable reuse needs a valid ETag for the
        # next writer's compare-and-swap contract.
        object_etag(existing)
    obj = client.get_object(Bucket=BUCKET, Key=object_key)
    hasher = hashlib.sha256()
    length = 0
    for piece in obj["Body"].iter_chunks(chunk_size=1024 * 1024):
        if piece:
            length += len(piece)
            if length > size:
                raise ValueError("Private R2 readback failed")
            hasher.update(piece)
    if length != size or hasher.hexdigest() != sha:
        raise ValueError("Private R2 readback failed")


def publish(release, apt, version, commit, run_id, output, account, access_key, secret):
    if not re.fullmatch(r"[a-f0-9]{32}", account) or not access_key or not secret:
        raise ValueError("Missing separately scoped CFVigil R2 credentials")
    if not str(run_id).isdigit() or int(run_id) < 1:
        raise ValueError("Invalid GitHub release run")
    if output.exists():
        raise ValueError("Refusing to overwrite any prior publication receipt")
    immutable, mutable, platforms = source_map(release, apt, version, commit)
    import boto3
    from botocore.config import Config
    client = boto3.client("s3",
        endpoint_url="https://" + account + ".r2.cloudflarestorage.com",
        region_name="auto", aws_access_key_id=access_key,
        aws_secret_access_key=secret,
        config=Config(signature_version="s3v4", retries={"max_attempts":3,"mode":"standard"})
    )
    latest_before = object_head(client, "latest.json")
    object_etag(latest_before)
    verified = {}
    for key, file in sorted(immutable.items()):
        upload(client, key, file, True)
        verified[key] = public_readback(file, key)
    # Two mutable index representations first; signed InRelease commits
    # APT afterwards. Without by-hash a concurrent client may see a
    # transient signed-index mismatch and must retry, never bypass GPG.
    for key, file in sorted(mutable.items()):
        if key == "linux/InRelease":
            continue
        upload(client, key, file, False)
        verified[key] = public_readback(file, key)
    inrelease = "linux/InRelease"
    upload(client, inrelease, mutable[inrelease], False)
    verified[inrelease] = public_readback(mutable[inrelease], inrelease)

    # latest.json is the FINAL commit point. No duplicate "latest" binary
    # aliases: the site reads this pointer and links immutable binaries.
    manifest = latest_manifest(immutable, version, commit)
    latest_path = output.parent / "cfvigil-latest-publish.json"
    if latest_path.exists():
        raise ValueError("Refusing to overwrite an existing latest manifest")
    latest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    upload(client, "latest.json", latest_path, False, expected_latest=latest_before)
    latest_verified = public_readback(latest_path, "latest.json")
    receipt = {
        "schema": 1, "product": "CFVigil", "base_url": BASE,
        "version": version, "commit": commit, "latest": latest_verified,
        "artifacts": [
            {"name": name, "run_id": int(run_id),
             "files": [verified[key] for key in sorted(files)]}
            for name, files in platforms.items()
        ],
    }
    output.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print("CFVigil flat R2 publication, APT and latest manifest readback completed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", type=Path, required=True)
    parser.add_argument("--apt", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--run-id", type=int, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    publish(args.release, args.apt, args.version, args.commit,
            args.run_id, args.receipt,
            os.environ["CFVIGIL_R2_ACCOUNT_ID"],
            os.environ["CFVIGIL_R2_ACCESS_KEY_ID"],
            os.environ["CFVIGIL_R2_SECRET_ACCESS_KEY"])


if __name__ == "__main__":
    main()
