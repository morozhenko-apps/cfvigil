#!/usr/bin/env python3
"""Prepare and sign a minimal Debian APT snapshot for CFVigil (amd64).

No network access. Signing happens only in a protected release job. Immutable
pool and by-hash objects can be published before mutable InRelease.
"""
import argparse
import datetime as dt
import gzip
import hashlib
from pathlib import Path
import shutil
import subprocess


def invoke(*args, env=None, input_data=None):
    return subprocess.run(args, check=True, capture_output=True, env=env, input=input_data).stdout


def sha256(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def package_field(deb, field):
    return invoke("dpkg-deb", "-f", str(deb), field).decode().strip()


def assemble(deb: Path, out: Path, version: str, gpg_home: Path,
             fingerprint: str, passphrase_file: Path):
    if not deb.is_file() or not gpg_home.is_dir() or not passphrase_file.is_file():
        raise ValueError("Missing package or signing input")
    if out.exists() and any(out.iterdir()):
        raise ValueError("Refusing to overwrite APT snapshot")
    expected = {"Package": "cf-vigil", "Version": version, "Architecture": "amd64"}
    for name, value in expected.items():
        if package_field(deb, name) != value:
            raise ValueError(f"Invalid {name}")
    if "libgtk-layer-shell0" not in package_field(deb, "Depends"):
        raise ValueError("Missing GTK Layer Shell runtime dependency")
    # Do not produce signed metadata for arbitrary or tampered packages.
    if not fingerprint or len(fingerprint) != 40:
        raise ValueError("Missing dedicated CFVigil signing fingerprint")

    pool = out / "pool/main/c/cf-vigil"
    pool.mkdir(parents=True)
    package = pool / f"cfvigil_{version}_amd64.deb"
    shutil.copyfile(deb, package)
    index = out / "dists/stable/main/binary-amd64"
    index.mkdir(parents=True)
    # Filename is relative to the APT repository root.
    proc = subprocess.run(
        ["dpkg-scanpackages", "--multiversion", "pool", "/dev/null"],
        cwd=out, capture_output=True, check=True
    )
    packages = proc.stdout
    if b"Filename: pool/main/c/cf-vigil/" not in packages:
        raise ValueError("APT package index did not contain the expected pool path")
    (index / "Packages").write_bytes(packages)
    # Deterministic gzip (mtime 0) and content-addressed archive.
    with (index / "Packages.gz").open("wb") as f:
        with gzip.GzipFile(filename="", mode="wb", fileobj=f, mtime=0) as g:
            g.write(packages)
    archive = index / "Packages.gz"
    for archive in (index / "Packages", index / "Packages.gz"):
        checksum = sha256(archive)
        by_hash = index / "by-hash/SHA256" / checksum
        by_hash.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(archive, by_hash)

    release_dir = out / "dists/stable"
    release_body = subprocess.run(["apt-ftparchive", "release", "."],
                                  cwd=release_dir, check=True, capture_output=True).stdout
    today = dt.datetime.now(dt.timezone.utc)
    headers = (
        "Origin: CFVigil\nLabel: CFVigil\nSuite: stable\nCodename: stable\n"
        "Architectures: amd64\nComponents: main\nAcquire-By-Hash: yes\n"
        f"Date: {today.strftime('%a, %d %b %Y %H:%M:%S +0000')}\n"
        # Metadata expiration requires an independent refresh workflow,
        # which is not yet configured: do not advertise stale Valid-Until.
    )
    release = release_dir / "Release"
    release.write_bytes(headers.encode() + release_body)
    inrelease = release_dir / "InRelease"
    subprocess.run([
        "gpg", "--homedir", str(gpg_home), "--batch", "--yes", "--no-tty",
        "--pinentry-mode", "loopback", "--passphrase-file", str(passphrase_file),
        "--digest-algo", "SHA256", "--local-user", fingerprint,
        "--clearsign", "--output", str(inrelease), str(release)
    ], check=True, stdout=subprocess.DEVNULL)
    keyring = out / "cfvigil-archive-keyring.gpg"
    keyring.write_bytes(invoke("gpg", "--homedir", str(gpg_home), "--batch",
                               "--export", fingerprint))
    if not keyring.stat().st_size:
        raise ValueError("Empty public signing keyring")
    verify(out, keyring)
    return inrelease


def verify(root, keyring):
    signed = root / "dists/stable/InRelease"
    invoke("gpgv", "--keyring", str(keyring), str(signed))
    archive = root / "dists/stable/main/binary-amd64/Packages.gz"
    by_hash = archive.parent / "by-hash/SHA256" / sha256(archive)
    if archive.read_bytes() != by_hash.read_bytes():
        raise ValueError("APT by-hash mismatch")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--deb", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--version", required=True)
    p.add_argument("--gpg-home", type=Path, required=True)
    p.add_argument("--fingerprint", required=True)
    p.add_argument("--passphrase-file", type=Path, required=True)
    a = p.parse_args()
    assemble(a.deb, a.out, a.version, a.gpg_home, a.fingerprint, a.passphrase_file)


if __name__ == "__main__":
    main()
