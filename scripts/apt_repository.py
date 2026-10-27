#!/usr/bin/env python3
"""Prepare and sign a minimal Debian APT snapshot for CFVigil (amd64).

No network access. Signing happens only in a protected release job.
Creates a flat signed index next to the one approved versioned Linux DEB.
The temporary local DEB copy is never published a second time to R2.
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

    out.mkdir(parents=True, exist_ok=True)
    # dpkg-scanpackages emits "Filename: ./cfvigil_VERSION_amd64.deb"
    # when scanning "." from the flat APT root. The temporary copy is
    # validated for parity but never uploaded to R2 as a second object.
    package = out / f"cfvigil_{version}_amd64.deb"
    shutil.copyfile(deb, package)
    proc = subprocess.run(
        ["dpkg-scanpackages", ".", "/dev/null"],
        cwd=out, capture_output=True, check=True
    )
    packages = proc.stdout
    if (packages.count(b"Package: cf-vigil\n") != 1 or
            b"Filename: ./cfvigil_" + version.encode() + b"_amd64.deb\n" not in packages):
        raise ValueError("Flat APT index did not contain the exact CFVigil DEB")
    (out / "Packages").write_bytes(packages)
    with (out / "Packages.gz").open("wb") as f:
        with gzip.GzipFile(filename="", mode="wb", fileobj=f, mtime=0) as g:
            g.write(packages)
    release_body = subprocess.run(["apt-ftparchive", "release", "."],
                                  cwd=out, check=True, capture_output=True).stdout
    today = dt.datetime.now(dt.timezone.utc)
    headers = (
        "Origin: CFVigil\nLabel: CFVigil\n"
        "Architectures: amd64\n"
        f"Date: {today.strftime('%a, %d %b %Y %H:%M:%S +0000')}\n"
        # Metadata expiration requires an independent refresh workflow,
        # which is not yet configured: do not advertise stale Valid-Until.
    )
    release = out / "Release"
    release.write_bytes(headers.encode() + release_body)
    inrelease = out / "InRelease"
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
    signed = root / "InRelease"
    invoke("gpgv", "--keyring", str(keyring), str(signed))
    with gzip.open(root / "Packages.gz", "rb") as compressed:
        if compressed.read() != (root / "Packages").read_bytes():
            raise ValueError("Flat APT Packages and Packages.gz mismatch")


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
