"""Offline R2 publisher integration tests. Never connect to Cloudflare."""
import hashlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import r2_release as release

VERSION = "0.1.0"
COMMIT = "a" * 40


def sha(data):
    return hashlib.sha256(data).hexdigest()


class PublicationFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.bundle = root / "release"
        self.apt = root / "apt"
        names = {
            "linux": ["cfvigil_0.1.0_amd64.deb", "CFVigil-0.1.0-linux-x64.AppImage"],
            "windows": ["CFVigil-Setup-0.1.0-win-x64.exe"],
        }
        for platform, items in names.items():
            directory = self.bundle / platform
            directory.mkdir(parents=True)
            manifest = {
                "schema": 1, "product": "CFVigil", "version": VERSION,
                "platform": {"linux": "linux-amd64", "windows": "windows-x64"}[platform],
                "commit": COMMIT, "release": False, "files": []
            }
            if platform == "windows":
                manifest["signed"] = False
            for name in items:
                content = ("validated package " + name).encode()
                (directory / name).write_bytes(content)
                checksum = sha(content)
                (directory / (name + ".sha256")).write_text(
                    checksum + "  " + name + "\n"
                )
                manifest["files"].append({
                    "name": name, "sha256": checksum, "size": len(content)
                })
            (directory / "candidate.json").write_text(json.dumps(manifest))
        pool = self.apt / "pool/main/c/cf-vigil/cfvigil_0.1.0_amd64.deb"
        pool.parent.mkdir(parents=True)
        pool.write_bytes((self.bundle / "linux" / pool.name).read_bytes())
        (self.apt / "cfvigil-archive-keyring.gpg").write_bytes(b"public test key")
        index = self.apt / "dists/stable/main/binary-amd64"
        index.mkdir(parents=True)
        files = {
            "Packages": b"Filename: pool/main/c/cf-vigil/cfvigil_0.1.0_amd64.deb\n",
            "Packages.gz": b"fake gzip for path validation",
        }
        for name, data in files.items():
            (index / name).write_bytes(data)
            hashed = index / "by-hash/SHA256" / sha(data)
            hashed.parent.mkdir(parents=True, exist_ok=True)
            hashed.write_bytes(data)
        (self.apt / "dists/stable/InRelease").write_bytes(b"signed test placeholder")

    def planned(self):
        with patch.object(release.subprocess, "run"):
            return release.source_map(self.bundle, self.apt, VERSION, COMMIT)

    def test_isolated_exact_source_map_contains_signed_apt_and_release_files(self):
        immutable, mutable, platforms = self.planned()
        self.assertEqual(set(platforms), {
            "cfvigil-release-linux", "cfvigil-release-windows"
        })
        self.assertTrue(any(k.startswith("apt/pool/main/c/cf-vigil/") for k in immutable))
        self.assertTrue(any("by-hash/SHA256/" in k for k in immutable))
        self.assertIn("apt/dists/stable/InRelease", mutable)
        self.assertNotIn("apt/dists/stable/InRelease", immutable)
        self.assertEqual(len(platforms["cfvigil-release-linux"]), 4)
        self.assertEqual(len(platforms["cfvigil-release-windows"]), 2)
        for key in [*immutable, *mutable]:
            self.assertNotIn("RepoTicker", key)
            self.assertNotIn("..", key)

    def test_corrupt_manifest_and_wrong_git_provenance_rejected(self):
        original = self.bundle / "windows" / "candidate.json"
        manifest = json.loads(original.read_text())
        manifest["commit"] = "f" * 40
        original.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError, "provenance"):
            self.planned()

    def test_modified_bundle_bytes_rejected(self):
        target = self.bundle / "linux/cfvigil_0.1.0_amd64.deb"
        target.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "checksum/size"):
            self.planned()

    def test_unexpected_release_sidefile_and_symlink_rejected(self):
        extra = self.bundle / "windows" / "unexpected.txt"
        extra.write_text("untrusted")
        with self.assertRaisesRegex(ValueError, "Unexpected artifact"):
            self.planned()
        extra.unlink()
        package = self.bundle / "windows/CFVigil-Setup-0.1.0-win-x64.exe"
        package.unlink()
        package.symlink_to(self.bundle / "linux/CFVigil-0.1.0-linux-x64.AppImage")
        with self.assertRaisesRegex(ValueError, "unsafe release file"):
            self.planned()

    def test_signed_apt_pool_must_match_validated_linux_deb(self):
        pool = self.apt / "pool/main/c/cf-vigil/cfvigil_0.1.0_amd64.deb"
        pool.write_bytes(b"tampered pool")
        with self.assertRaisesRegex(ValueError, "APT pool"):
            self.planned()

    def test_public_readback_verifies_full_body_and_size(self):
        file = self.bundle / "linux/cfvigil_0.1.0_amd64.deb"
        class Reply:
            status = 200
            def __init__(self, data): self.data = io.BytesIO(data)
            def __enter__(self): return self
            def __exit__(self, *args): pass
            def read(self, n): return self.data.read(n)
        with patch.object(release.time, "sleep"):
            good = release.public_readback(file, "releases/v0.1.0/linux/" + file.name,
                                           lambda url, timeout: Reply(file.read_bytes()))
            self.assertEqual(good["readback_sha256"], good["sha256"])
            self.assertEqual(good["readback_size"], file.stat().st_size)
            with self.assertRaisesRegex(ValueError, "does not match"):
                release.public_readback(
                    file, "releases/v0.1.0/linux/" + file.name,
                    lambda url, timeout: Reply(b"corrupted"))

    def test_receipt_cannot_be_emitted_with_missing_credentials(self):
        with self.assertRaisesRegex(ValueError, "credentials"):
            release.publish(self.bundle, self.apt, VERSION, COMMIT, 10,
                            Path(self.temp.name) / "receipt.json",
                            "invalid", "", "")


if __name__ == "__main__":
    unittest.main()
