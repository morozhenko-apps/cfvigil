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
        # The test stubs only the GPG cryptographic check; the signed
        # Release text itself must carry coherent index SHA256 records.
        signed_lines = [
            "-----BEGIN PGP SIGNED MESSAGE-----",
            "Hash: SHA256",
            "",
            "Origin: CFVigil",
            "SHA256:",
        ]
        for item in (index / "Packages", index / "Packages.gz"):
            data = item.read_bytes()
            relative = "main/binary-amd64/" + item.name
            signed_lines.append(f" {sha(data)} {len(data)} {relative}")
        signed_lines.extend([
            "-----BEGIN PGP SIGNATURE-----",
            "offline-fixture-signature; cryptography stubbed in tests",
            "-----END PGP SIGNATURE-----",
            "",
        ])
        (self.apt / "dists/stable/InRelease").write_text(
            "\n".join(signed_lines), encoding="utf8"
        )

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


    def test_signed_apt_indexes_match_actual_sizes_and_digest_with_lf_and_crlf(self):
        inrelease = self.apt / "dists/stable/InRelease"
        index = self.apt / "dists/stable/main/binary-amd64"
        release.verify_signed_apt_indexes(inrelease, index)
        initial = inrelease.read_bytes()
        inrelease.write_bytes(initial.replace(b"\\n", b"\\r\\n"))
        release.verify_signed_apt_indexes(inrelease, index)
        self.planned()

    def test_old_signed_apt_index_must_reject_new_indexes_even_if_by_hash_matches(self):
        index = self.apt / "dists/stable/main/binary-amd64"
        packages = index / "Packages"
        new_content = packages.read_bytes() + b"Version: unexpected\\n"
        packages.write_bytes(new_content)
        new_hash = index / "by-hash/SHA256" / sha(new_content)
        new_hash.write_bytes(new_content)
        with self.assertRaisesRegex(ValueError, "Signed APT index"):
            self.planned()

    def test_signed_apt_index_rejects_wrong_size_digest_missing_and_duplicate(self):
        signed = self.apt / "dists/stable/InRelease"
        original = signed.read_text(encoding="utf8")
        lines = original.splitlines()
        checksum_line = next(
            row for row in lines if row.endswith("main/binary-amd64/Packages.gz")
        )
        digest, size, path = checksum_line.split()
        scenarios = {
            "wrong-size": original.replace(checksum_line,
                f" {digest} {int(size) + 1} {path}"),
            "wrong-digest": original.replace(checksum_line,
                f" {'f' if digest[0] != 'f' else 'e'}{digest[1:]} {size} {path}"),
            "missing-index": original.replace(checksum_line + "\\n", ""),
            "duplicate-index": original.replace(checksum_line + "\\n",
                checksum_line + "\\n" + checksum_line + "\\n"),
            "missing-sha256": original.replace("SHA256:", "SHA999:"),
            "malformed-hash": original.replace(checksum_line,
                f" corrupt {size} {path}"),
            "invalid-envelope": original.replace(
                "-----BEGIN PGP SIGNED MESSAGE-----", "invalid signed header"),
        }
        for name, mutated in scenarios.items():
            with self.subTest(name=name):
                signed.write_text(mutated, encoding="utf8")
                with self.assertRaisesRegex(ValueError, "Signed APT index"):
                    self.planned()
        signed.write_text(original, encoding="utf8")
        self.planned()

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



class UploadSafetyTests(unittest.TestCase):
    """Exercise the actual R2 upload logic with offline S3 byte-contract fakes."""

    class NotFound(Exception):
        response = {"ResponseMetadata": {"HTTPStatusCode":404}}

    class Forbidden(Exception):
        response = {"ResponseMetadata": {"HTTPStatusCode":403}}

    class Body:
        def __init__(self, value):
            self.value = value
        def iter_chunks(self, chunk_size):
            for i in range(0, len(self.value), chunk_size):
                yield self.value[i:i + chunk_size]

    class FakeS3:
        def __init__(self):
            self.objects = {}
            self.writes = []
            self.deny_head = False
        def head_object(self, Bucket, Key):
            if self.deny_head:
                raise UploadSafetyTests.Forbidden()
            if Key not in self.objects:
                raise UploadSafetyTests.NotFound()
            return {"Metadata":self.objects[Key][1]}
        def upload_file(self, filename, Bucket, Key, ExtraArgs):
            self.writes.append((Bucket, Key, ExtraArgs))
            self.objects[Key] = (Path(filename).read_bytes(), ExtraArgs["Metadata"])
        def get_object(self, Bucket, Key):
            return {"Body":UploadSafetyTests.Body(self.objects[Key][0])}

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.file = Path(self.temp.name) / "CFVigil-0.1.0-linux-x64.AppImage"
        self.file.write_bytes(b"isolated validated binary")
        self.client = self.FakeS3()
        self.key = "releases/v0.1.0/linux/" + self.file.name
        self.object_key = release.PREFIX + self.key

    def test_missing_file_is_uploaded_to_isolated_bucket_and_sha_verified(self):
        release.upload(self.client, self.key, self.file, True)
        self.assertEqual(len(self.client.writes),1)
        bucket, key, options = self.client.writes[0]
        self.assertEqual((bucket,key),("cfvigil-dist",self.object_key))
        self.assertEqual(options["Metadata"]["sha256"],release.digest(self.file))
        self.assertIn("immutable",options["CacheControl"])
        self.assertEqual(self.client.objects[self.object_key][0],self.file.read_bytes())

    def test_immutable_identical_blob_is_reused_without_upload(self):
        self.client.objects[self.object_key] = (
            self.file.read_bytes(),{"sha256":release.digest(self.file)}
        )
        release.upload(self.client,self.key,self.file,True)
        self.assertEqual(self.client.writes,[])

    def test_immutable_conflict_rejects_without_overwriting_existing_bytes(self):
        self.client.objects[self.object_key] = (b"other data",{"sha256":"f"*64})
        with self.assertRaisesRegex(ValueError,"Refusing to overwrite immutable"):
            release.upload(self.client,self.key,self.file,True)
        self.assertEqual(self.client.writes,[])
        self.assertEqual(self.client.objects[self.object_key][0],b"other data")

    def test_mutable_index_updates_with_no_store_cache_control(self):
        self.client.objects[self.object_key] = (b"old index",{"sha256":"f"*64})
        release.upload(self.client,self.key,self.file,False)
        self.assertEqual(self.client.writes[0][2]["CacheControl"],"no-store")
        self.assertEqual(self.client.objects[self.object_key][0],self.file.read_bytes())

    def test_private_readback_mismatch_fails_before_public_proof(self):
        self.client.objects[self.object_key] = (b"corrupted",{"sha256":release.digest(self.file)})
        with self.assertRaisesRegex(ValueError,"Private R2 readback failed"):
            release.upload(self.client,self.key,self.file,True)
        self.assertEqual(self.client.writes,[])

    def test_forbidden_head_does_not_turn_into_an_unverified_upload(self):
        self.client.deny_head = True
        with self.assertRaises(self.Forbidden):
            release.upload(self.client,self.key,self.file,True)
        self.assertEqual(self.client.writes,[])


if __name__ == "__main__":
    unittest.main()
