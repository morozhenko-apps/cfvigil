"""Real disposable GPG + dpkg + APT integration for CFVigil's flat release.

No Cloudflare credentials, host APT enrollment, package installation or
production signing keys. Unlike mock-based publisher tests this executes the
actual APT generator and the system package manager.
"""
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import apt_repository

VERSION = "0.1.0"


def command(*args):
    result = subprocess.run(args, text=True, capture_output=True,
                            env={**os.environ, "LC_ALL": "C"})
    if result.returncode:
        raise AssertionError(
            f"{args[0]} exited {result.returncode}: " +
            (result.stdout + result.stderr)[-1800:])
    return result.stdout


class RealFlatAptClientTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory(prefix="cfvigil-apt-test-")
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)
        self.gpg_home = self.root / "gnupg"
        self.gpg_home.mkdir(mode=0o700)
        self.passphrase = self.root / "passphrase"
        self.passphrase.write_text("", encoding="utf8")
        for executable in ("dpkg-deb", "dpkg-scanpackages", "gpg", "gpgv",
                           "apt-get", "apt-cache"):
            self.assertIsNotNone(shutil.which(executable),
                                 f"Required test executable: {executable}")
        command("gpg", "--homedir", str(self.gpg_home), "--batch", "--yes",
                "--pinentry-mode", "loopback", "--passphrase", "",
                "--quick-generate-key",
                "CFVigil disposable test signer <offline@example.invalid>",
                "ed25519", "sign", "0")
        listing = command("gpg", "--homedir", str(self.gpg_home),
                          "--batch", "--with-colons", "--list-keys")
        self.fingerprint = next(
            line.split(":")[9] for line in listing.splitlines()
            if line.startswith("fpr:"))
        self.assertRegex(self.fingerprint, r"^[0-9A-F]{40}$")

        self.source_deb = self.root / f"cfvigil_{VERSION}_amd64.deb"
        package = self.root / "package"
        control = package / "DEBIAN"
        control.mkdir(parents=True)
        (control / "control").write_text(
            "Package: cf-vigil\n"
            f"Version: {VERSION}\n"
            "Architecture: amd64\n"
            "Maintainer: CFVigil Offline CI <offline@example.invalid>\n"
            "Depends: libgtk-layer-shell0\n"
            "Description: disposable flat APT integration test package\n",
            encoding="utf8")
        data = package / "usr/share/doc/cf-vigil"
        data.mkdir(parents=True)
        (data / "README").write_text("offline test; not real CFVigil\n",
                                       encoding="utf8")
        command("dpkg-deb", "--build", "--root-owner-group",
                str(package), str(self.source_deb))

        self.staging = self.root / "staging"
        apt_repository.assemble(self.source_deb, self.staging, VERSION,
                                self.gpg_home, self.fingerprint, self.passphrase)
        self.public = self.root / "public" / "linux"
        self.public.mkdir(parents=True)
        for filename in (self.source_deb.name, "Packages", "Packages.gz",
                         "InRelease", "cfvigil-archive-keyring.gpg"):
            origin = (self.source_deb if filename == self.source_deb.name
                      else self.staging / filename)
            shutil.copyfile(origin, self.public / filename)

    def apt_options(self, case):
        client = self.root / "apt-client" / case
        client.mkdir(parents=True)
        source_list = client / "sources.list"
        source_list.write_text(
            f"deb [arch=amd64 signed-by={self.public / 'cfvigil-archive-keyring.gpg'}] "
            f"file:{self.public}/ ./\n", encoding="utf8")
        lists = client / "lists"
        (lists / "partial").mkdir(parents=True)
        archives = client / "archives"
        (archives / "partial").mkdir(parents=True)
        (client / "status").write_text("", encoding="utf8")
        return [
            "-o", f"Dir::Etc::sourcelist={source_list}",
            "-o", "Dir::Etc::sourceparts=-",
            "-o", f"Dir::State::lists={lists}",
            "-o", f"Dir::Cache::archives={archives}",
            "-o", f"Dir::State::status={client / 'status'}",
            "-o", "Debug::NoLocking=1",
            "-o", "Acquire::Languages=none",
            "-o", "APT::Get::List-Cleanup=0",
            "-o", "Acquire::AllowInsecureRepositories=false",
        ]

    def test_signed_flat_catalog_is_accepted_by_real_apt(self):
        options = self.apt_options("valid")
        output = command("apt-get", *options, "update")
        self.assertIn("Reading package lists", output)
        policy = command("apt-cache", *options, "policy", "cf-vigil")
        self.assertIn("Candidate: 0.1.0", policy)
        metadata = command("apt-cache", *options, "show", "cf-vigil")
        self.assertIn(f"Filename: ./cfvigil_{VERSION}_amd64.deb", metadata)
        self.assertFalse((self.public / "pool").exists())
        self.assertFalse((self.public / "dists").exists())

    def test_tampered_inrelease_is_rejected_by_real_apt(self):
        signed = self.public / "InRelease"
        original = signed.read_bytes()
        self.assertIn(b"Origin: CFVigil", original)
        signed.write_bytes(original.replace(b"Origin: CFVigil",
                                            b"Origin: CFVigix", 1))
        options = self.apt_options("tampered")
        result = subprocess.run(["apt-get", *options, "update"],
                                capture_output=True, text=True,
                                env={**os.environ, "LC_ALL": "C"})
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
