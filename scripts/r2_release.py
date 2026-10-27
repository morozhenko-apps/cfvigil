#!/usr/bin/env python3
"""CFVigil-only signed R2 publication. Never invoked by candidate workflows.

Publishing requires an owner-approved exact-main tag, a protected environment,
a separately provisioned signing subkey, and scoped CFVigil R2 credentials.
The verified receipt is written ONLY after every public download matches.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import urllib.request

BUCKET = "cfvigil-dist"
PREFIX = "dist/CFVigil/"
BASE = "https://download.morozhenko.com/" + PREFIX
VERSION = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\Z")
COMMIT = re.compile(r"[a-f0-9]{40}\Z")
MIMES = {
    ".deb": "application/vnd.debian.binary-package",
    ".AppImage": "application/octet-stream",
    ".exe": "application/octet-stream",
    ".gz": "application/gzip",
    ".gpg": "application/pgp-keys",
    ".sha256": "text/plain",
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
            key = "releases/v" + version + "/" + platform + "/" + item.name
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
        "main/binary-amd64/Packages": index / "Packages",
        "main/binary-amd64/Packages.gz": index / "Packages.gz",
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


def apt_files(apt, linux_deb, version):
    if apt.is_symlink() or not apt.is_dir():
        raise ValueError("Incomplete signed APT directory")
    pool = apt / f"pool/main/c/cf-vigil/cfvigil_{version}_amd64.deb"
    keyring = apt / "cfvigil-archive-keyring.gpg"
    root = apt / "dists/stable"
    index = root / "main/binary-amd64"
    packages = index / "Packages"
    compressed = index / "Packages.gz"
    inrelease = root / "InRelease"
    files = [pool, keyring, packages, compressed, inrelease]
    for file in files:
        safe_file(file)
    if digest(pool) != digest(linux_deb):
        raise ValueError("Signed APT pool does not match validated CFVigil DEB")
    # Verify actual signed metadata, not merely an InRelease filename.
    subprocess.run(["gpgv", "--keyring", str(keyring), str(inrelease)],
                   check=True, stdout=subprocess.DEVNULL)
    verify_signed_apt_indexes(inrelease, index)
    if b"pool/main/c/cf-vigil/" not in packages.read_bytes():
        raise ValueError("APT package index points outside CFVigil pool")
    immutable = {}
    mutable = {}
    for item in (pool, keyring):
        immutable["apt/" + item.relative_to(apt).as_posix()] = item
    for item in (packages, compressed):
        hashed = index / "by-hash/SHA256" / digest(item)
        safe_file(hashed)
        if digest(hashed) != digest(item):
            raise ValueError("APT by-hash corruption")
        immutable["apt/" + hashed.relative_to(apt).as_posix()] = hashed
        mutable["apt/" + item.relative_to(apt).as_posix()] = item
    mutable["apt/dists/stable/InRelease"] = inrelease
    return immutable, mutable


def source_map(release, apt, version, commit):
    if not VERSION.fullmatch(version) or not COMMIT.fullmatch(commit):
        raise ValueError("Invalid signed-release version or commit")
    linux = platform_files(release, "linux", version, commit)
    windows = platform_files(release, "windows", version, commit)
    package = linux["releases/v" + version + "/linux/cfvigil_" + version + "_amd64.deb"]
    apt_immutable, apt_mutable = apt_files(apt, package, version)
    immutable = {**linux, **windows, **apt_immutable}
    return immutable, apt_mutable, {
        "cfvigil-release-linux": linux,
        "cfvigil-release-windows": windows,
    }


def content_type(path):
    if path.name == "InRelease" or path.name == "Packages":
        return "text/plain; charset=utf-8"
    return MIMES.get(path.suffix, "application/octet-stream")


def public_readback(path, relative, opener=urllib.request.urlopen):
    # The public Worker must already be independently deployed. No fallback
    # to private R2 readback is accepted as proof of public distribution.
    expected_size = path.stat().st_size
    expected_sha = digest(path)
    for attempt in range(4):
        try:
            with opener(BASE + relative, timeout=45) as reply:
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


def upload(client, key, file, immutable):
    object_key = PREFIX + key
    sha = digest(file)
    try:
        existing = client.head_object(Bucket=BUCKET, Key=object_key)
    except Exception as error:
        status = getattr(error, "response", {}).get("ResponseMetadata", {}).get("HTTPStatusCode")
        if status != 404:
            raise
        existing = None
    if existing and immutable:
        if existing.get("Metadata", {}).get("sha256") != sha:
            raise ValueError("Refusing to overwrite immutable CFVigil R2 object")
    else:
        client.upload_file(str(file), BUCKET, object_key, ExtraArgs={
            "ContentType": content_type(file),
            "Metadata": {"sha256": sha},
            "CacheControl": "public,max-age=31536000,immutable" if immutable else "no-store",
        })
    obj = client.get_object(Bucket=BUCKET, Key=object_key)
    hasher = hashlib.sha256()
    length = 0
    for piece in obj["Body"].iter_chunks(chunk_size=1024 * 1024):
        if piece:
            hasher.update(piece)
            length += len(piece)
    if length != file.stat().st_size or hasher.hexdigest() != sha:
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
    verified = {}
    for key, file in sorted(immutable.items()):
        upload(client, key, file, True)
        verified[key] = public_readback(file, key)
    # Mutable indexes are committed after all immutable files are reachable.
    # Signed InRelease MUST be the final published object.
    for key, file in sorted(mutable.items()):
        if key.endswith("/InRelease"):
            continue
        upload(client, key, file, False)
        verified[key] = public_readback(file, key)
    inrelease = "apt/dists/stable/InRelease"
    upload(client, inrelease, mutable[inrelease], False)
    verified[inrelease] = public_readback(mutable[inrelease], inrelease)
    receipt = {
        "schema": 1, "product": "CFVigil", "base_url": BASE,
        "version": version, "commit": commit,
        "artifacts": [
            {"name": name, "run_id": int(run_id),
             "files": [verified[key] for key in sorted(files)]}
            for name, files in platforms.items()
        ],
    }
    output.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print("CFVigil R2 publication and public checksum verification completed")


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
