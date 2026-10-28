"""Offline R2 publisher integration tests. Never connect to Cloudflare."""
import gzip
import hashlib
import io
import json
import sys
import tempfile
from types import ModuleType
import unittest
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.request import Request

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
        pool = self.apt / "cfvigil_0.1.0_amd64.deb"
        self.apt.mkdir(parents=True)
        pool.write_bytes((self.bundle / "linux" / pool.name).read_bytes())
        (self.apt / "cfvigil-archive-keyring.gpg").write_bytes(b"public test key")
        index = self.apt
        package_bytes = pool.read_bytes()
        control = (
            "Package: cf-vigil\n"
            f"Version: {VERSION}\n"
            "Architecture: amd64\n"
            f"Filename: ./{pool.name}\n"
            f"Size: {len(package_bytes)}\n"
            f"SHA256: {sha(package_bytes)}\n"
            "Description: Minimal test package\n"
            " fixture continuation line\n"
            "\n"
        ).encode("utf8")
        files = {
            "Packages": control,
            "Packages.gz": gzip.compress(control, mtime=0),
        }
        for name, data in files.items():
            (index / name).write_bytes(data)
        self.seal_offline_indexes()

    def seal_offline_indexes(self):
        """Re-sign test-only flat metadata using fake GPG.

        This never exercises a private key, network or real signature.
        """
        index = self.apt
        signed_lines = [
            "-----BEGIN PGP SIGNED MESSAGE-----",
            "Hash: SHA256",
            "",
            "Origin: CFVigil",
            "SHA256:",
        ]
        for item in (index / "Packages", index / "Packages.gz"):
            data = item.read_bytes()
            relative = item.name
            signed_lines.append(f" {sha(data)} {len(data)} {relative}")
        signed_lines.extend([
            "-----BEGIN PGP SIGNATURE-----",
            "offline-fixture-signature; cryptography stubbed in tests",
            "-----END PGP SIGNATURE-----",
            "",
        ])
        (self.apt / "InRelease").write_text(
            "\n".join(signed_lines), encoding="utf8"
        )

    def rewrite_package_indexes(self, plain, compressed=None):
        """Intentionally construct internally wrong but signed test catalogs."""
        index = self.apt
        (index / "Packages").write_bytes(plain)
        (index / "Packages.gz").write_bytes(
            gzip.compress(plain, mtime=0) if compressed is None else compressed
        )
        self.seal_offline_indexes()

    def planned(self):
        with patch.object(release.subprocess, "run"):
            return release.source_map(self.bundle, self.apt, VERSION, COMMIT)

    def test_isolated_exact_source_map_contains_signed_apt_and_release_files(self):
        immutable, mutable, platforms = self.planned()
        self.assertEqual(set(platforms), {
            "cfvigil-release-linux", "cfvigil-release-windows"
        })
        self.assertIn("linux/cfvigil_0.1.0_amd64.deb", immutable)
        self.assertIn("linux/cfvigil-archive-keyring.gpg", immutable)
        self.assertEqual(set(mutable), {"linux/Packages", "linux/Packages.gz",
                                        "linux/InRelease"})
        self.assertEqual(sum(k.endswith(".deb") for k in immutable), 1)
        self.assertFalse(any(k.startswith(("apt/", "releases/")) for k in immutable))
        self.assertEqual(len(platforms["cfvigil-release-linux"]), 4)
        self.assertEqual(len(platforms["cfvigil-release-windows"]), 2)
        for key in [*immutable, *mutable]:
            self.assertNotIn("RepoTicker", key)
            self.assertNotIn("..", key)


    def test_signed_apt_indexes_match_actual_sizes_and_digest_with_lf_and_crlf(self):
        inrelease = self.apt / "InRelease"
        index = self.apt
        release.verify_signed_apt_indexes(inrelease, index)
        initial = inrelease.read_bytes()
        inrelease.write_bytes(initial.replace(b"\n", b"\r\n"))
        release.verify_signed_apt_indexes(inrelease, index)
        self.planned()

    def test_old_signed_apt_index_must_reject_new_indexes(self):
        index = self.apt
        packages = index / "Packages"
        new_content = packages.read_bytes() + b"Version: unexpected\n"
        packages.write_bytes(new_content)
        with self.assertRaisesRegex(ValueError, "Signed APT index"):
            self.planned()

    def test_signed_apt_index_rejects_wrong_size_digest_missing_and_duplicate(self):
        signed = self.apt / "InRelease"
        original = signed.read_text(encoding="utf8")
        lines = original.splitlines()
        checksum_line = next(
            row for row in lines if row.endswith("Packages.gz")
        )
        digest, size, path = checksum_line.split()
        scenarios = {
            "wrong-size": original.replace(checksum_line,
                f" {digest} {int(size) + 1} {path}"),
            "wrong-digest": original.replace(checksum_line,
                f" {'f' if digest[0] != 'f' else 'e'}{digest[1:]} {size} {path}"),
            "missing-index": original.replace(checksum_line + "\n", ""),
            "duplicate-index": original.replace(checksum_line + "\n",
                checksum_line + "\n" + checksum_line + "\n"),
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


    def test_packages_control_exactly_binds_signed_index_to_approved_deb(self):
        """Realistic package fields and folded Description pass."""
        index = self.apt
        pool = self.apt / "cfvigil_0.1.0_amd64.deb"
        control = (index / "Packages").read_bytes()
        self.assertIn(b" fixture continuation line\n", control)
        release.verify_apt_package_catalog(index / "Packages",
                                           index / "Packages.gz", pool, VERSION)
        self.planned()

    def test_signed_but_wrong_package_control_fields_fail_before_publication(self):
        index = self.apt
        original = (index / "Packages").read_text(encoding="utf8")
        pool = self.apt / "cfvigil_0.1.0_amd64.deb"
        expected_name = "./" + pool.name
        variants = {
            "foreign package": original.replace("Package: cf-vigil",
                                                  "Package: repoticker"),
            "wrong version": original.replace(f"Version: {VERSION}",
                                               "Version: 0.1.1"),
            "wrong architecture": original.replace("Architecture: amd64",
                                                    "Architecture: arm64"),
            "wrong filename": original.replace(f"Filename: {expected_name}",
                                                "Filename: ./evil.deb"),
            "traversal filename": original.replace(f"Filename: {expected_name}",
                                                   "Filename: ../../private.deb"),
            "wrong sha256": original.replace(f"SHA256: {sha(pool.read_bytes())}",
                                              "SHA256: " + "f" * 64),
            "wrong size": original.replace(f"Size: {pool.stat().st_size}",
                                            f"Size: {pool.stat().st_size+1}"),
            "duplicate critical SHA256": original.replace(
                "SHA256:", "SHA256: " + sha(pool.read_bytes()) + "\nSHA256:", 1),
            "missing filename": original.replace(
                f"Filename: {expected_name}\n", ""),
            "extra second package stanza": original + (
                "Package: cf-vigil\nVersion: 0.1.1\n"
                "Architecture: amd64\nFilename: " + expected_name + "\n\n"
            ),
            "folder substring outside filename": original.replace(
                f"Filename: {expected_name}",
                f"X-Directory: {expected_name}\nFilename: external/replaced.deb"),
            "continued filename": original.replace(
                f"Filename: {expected_name}",
                f"Filename: {expected_name}\n unexpected-extension"),
        }
        for label, value in variants.items():
            with self.subTest(case=label):
                self.rewrite_package_indexes(value.encode("utf8"))
                with self.assertRaisesRegex(ValueError, "APT package catalog"):
                    self.planned()

    def test_separately_signed_gzip_must_expand_to_identical_packages(self):
        index = self.apt
        plain = (index / "Packages").read_bytes()
        candidates = {
            "different signed gzip contents": gzip.compress(
                plain.replace(b"Package: cf-vigil", b"Package: foreign!"),
                mtime=0),
            "truncated signed gzip": gzip.compress(plain, mtime=0)[:-6],
            "invalid signed gzip": b"not-gzip",
            "two concatenated signed gzip members": (
                gzip.compress(plain, mtime=0) + gzip.compress(plain, mtime=0)
            ),
        }
        for label, payload in candidates.items():
            with self.subTest(case=label):
                self.rewrite_package_indexes(plain, payload)
                with self.assertRaisesRegex(ValueError, "APT package catalog"):
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

    def test_staged_apt_deb_must_match_validated_linux_deb(self):
        pool = self.apt / "cfvigil_0.1.0_amd64.deb"
        pool.write_bytes(b"tampered pool")
        with self.assertRaisesRegex(ValueError, "staged DEB"):
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
            good = release.public_readback(file, "linux/" + file.name,
                                           lambda url, timeout: Reply(file.read_bytes()))
            self.assertEqual(good["readback_sha256"], good["sha256"])
            self.assertEqual(good["readback_size"], file.stat().st_size)
            with self.assertRaisesRegex(ValueError, "does not match"):
                release.public_readback(
                    file, "linux/" + file.name,
                    lambda url, timeout: Reply(b"corrupted"))

    def test_latest_manifest_points_only_to_immutable_approved_artifacts(self):
        immutable, mutable, _ = self.planned()
        manifest = release.latest_manifest(immutable, VERSION, COMMIT)
        self.assertEqual(manifest["version"], VERSION)
        self.assertEqual(manifest["tag"], "v" + VERSION)
        self.assertEqual(manifest["commit"], COMMIT)
        values = [
            manifest["files"]["linux"]["deb"],
            manifest["files"]["linux"]["appimage"],
            manifest["files"]["windows"]["installer"],
        ]
        self.assertEqual(len(values), 3)
        for item in values:
            self.assertIn(item["url"].removeprefix(release.BASE), immutable)
            self.assertEqual(item["sha256"],
                release.digest(immutable[item["url"].removeprefix(release.BASE)]))
            self.assertGreater(item["size"], 0)
            self.assertNotIn("/releases/", item["url"])
        self.assertEqual(release.BUCKET, "app-dist")
        self.assertNotIn("latest.json", mutable)

    def offline_publication(self, fail_latest=False):
        """Emulate cloud clients while running real upload/publish ordering."""
        client = UploadSafetyTests.FakeS3()
        boto = ModuleType("boto3")
        boto.client = lambda *args, **kwargs: client
        config = ModuleType("botocore.config")
        config.Config = lambda **kwargs: None
        botocore = ModuleType("botocore")
        botocore.__path__ = []
        botocore.config = config
        seen = []

        def public(file, key):
            seen.append(key)
            if key == "latest.json" and fail_latest:
                raise ValueError("intentional last-pointer readback failure")
            size = file.stat().st_size
            checksum = release.digest(file)
            return {
                "url": release.BASE + key, "sha256": checksum,
                "readback_sha256": checksum, "size": size,
                "readback_size": size, "readback_status": 200,
            }

        destination = Path(self.temp.name) / "verified-publication-receipt.json"
        with patch.dict(sys.modules, {
            "boto3": boto, "botocore": botocore, "botocore.config": config
        }), patch.object(release.subprocess, "run"), patch.object(
                release, "public_readback", side_effect=public):
            if fail_latest:
                with self.assertRaisesRegex(ValueError, "last-pointer"):
                    release.publish(self.bundle, self.apt, VERSION, COMMIT, 51,
                                    destination, "a" * 32, "test-key", "test-secret")
            else:
                release.publish(self.bundle, self.apt, VERSION, COMMIT, 51,
                                destination, "a" * 32, "test-key", "test-secret")
        return client, seen, destination

    def test_flat_publication_preserves_single_binary_and_publishes_latest_last(self):
        client, seen, destination = self.offline_publication()
        self.assertTrue(destination.exists())
        self.assertEqual(seen[-1], "latest.json")
        self.assertEqual(seen[-2], "linux/InRelease")
        self.assertEqual(sum(k.endswith(".deb") for k in client.objects), 1)
        self.assertTrue(all(k.startswith(release.PREFIX) for k in client.objects))
        self.assertEqual(len(client.objects), 11)
        self.assertEqual([key for _, key, _ in client.writes][-1],
                         release.PREFIX + "latest.json")
        data = json.loads(destination.read_text(encoding="utf8"))
        self.assertEqual(data["latest"]["url"], release.BASE + "latest.json")
        self.assertEqual(len(data["artifacts"]), 2)
        self.assertEqual(
            json.loads(client.objects[release.PREFIX + "latest.json"][0])["version"],
            VERSION
        )

    def test_failed_final_latest_readback_never_emits_deletion_receipt(self):
        client, seen, destination = self.offline_publication(fail_latest=True)
        self.assertEqual(seen[-1], "latest.json")
        self.assertIn(release.PREFIX + "linux/InRelease", client.objects)
        self.assertFalse(destination.exists())

    def test_receipt_cannot_be_emitted_with_missing_credentials(self):
        with self.assertRaisesRegex(ValueError, "credentials"):
            release.publish(self.bundle, self.apt, VERSION, COMMIT, 10,
                            Path(self.temp.name) / "receipt.json",
                            "invalid", "", "")



class PublicReadbackTests(unittest.TestCase):
    """Exercise real HTTP request construction and fail-closed byte checks."""

    class Reply:
        def __init__(self, data, status=200):
            self.status = status
            self.body = io.BytesIO(data)
            self.read_sizes = []

        def __enter__(self):
            return self

        def __exit__(self, *args):
            self.body.close()

        def read(self, size):
            self.read_sizes.append(size)
            return self.body.read(size)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.file = Path(self.temp.name) / "approved-package"
        self.payload = b"approved CFVigil package bytes"
        self.file.write_bytes(self.payload)
        sleep_patch = patch.object(release.time, "sleep")
        sleep_patch.start()
        self.addCleanup(sleep_patch.stop)

    def assert_request(self, request, timeout, relative):
        self.assertIsInstance(request, Request)
        self.assertEqual(request.full_url, release.BASE + relative)
        self.assertEqual(request.get_method(), "GET")
        self.assertIsNone(request.data)
        self.assertEqual(timeout, 45)
        self.assertEqual(dict(request.header_items()), {
            "User-agent": (
                "CFVigil-ReleaseVerifier/1.0 "
                "(+https://github.com/morozhenko-apps/cfvigil)"
            ),
        })

    def test_GivenReleaseFiles_WhenReadbackRuns_ThenRequestIdentifiesCFVigil(self):
        for relative in (
            "linux/CFVigil-0.1.0-linux-x64.AppImage",
            "windows/CFVigil-Setup-0.1.0-win-x64.exe",
            "linux/InRelease",
            "latest.json",
        ):
            with self.subTest(relative=relative):
                calls = []

                def opener(request, timeout):
                    calls.append(request)
                    self.assert_request(request, timeout, relative)
                    return self.Reply(self.payload)

                # Act: the production verifier constructs the HTTP request.
                result = release.public_readback(self.file, relative, opener)

                # Assert: no receipt replaces the complete byte proof.
                self.assertEqual(len(calls), 1)
                self.assertEqual(result, {
                    "url": release.BASE + relative,
                    "sha256": sha(self.payload),
                    "readback_sha256": sha(self.payload),
                    "size": len(self.payload),
                    "readback_size": len(self.payload),
                    "readback_status": 200,
                })

    def test_GivenLargeBody_WhenReadbackRuns_ThenAllChunksAreHashed(self):
        # Arrange: exceed a single bounded read, with a distinct final byte.
        self.payload = b"a" * (1024 * 1024) + b"z"
        self.file.write_bytes(self.payload)
        reply = self.Reply(self.payload)

        # Act.
        result = release.public_readback(
            self.file, "linux/package", lambda request, timeout: reply)

        # Assert: neither a prefix hash nor an unbounded read is accepted.
        self.assertEqual(result["readback_sha256"], sha(self.payload))
        self.assertEqual(result["readback_size"], len(self.payload))
        self.assertEqual(reply.read_sizes, [1024 * 1024] * 3)

    def test_GivenTransientHTTPFailure_WhenRetried_ThenIdentificationPersists(self):
        # Arrange: emulate an external HTTP failure without real time/network.
        relative = "linux/package"
        calls = []

        def opener(request, timeout):
            calls.append(request)
            self.assert_request(request, timeout, relative)
            if len(calls) == 1:
                raise HTTPError(request.full_url, 503, "Unavailable", {}, None)
            return self.Reply(self.payload)

        # Act.
        result = release.public_readback(self.file, relative, opener)

        # Assert.
        self.assertEqual(len(calls), 2)
        self.assertEqual(result["readback_sha256"], sha(self.payload))

    def test_GivenNon200Reply_WhenReadbackRuns_ThenNoBodyProofIsAccepted(self):
        for status in (201, 206, 403):
            with self.subTest(status=status):
                replies = []

                def opener(request, timeout):
                    reply = self.Reply(self.payload, status)
                    replies.append(reply)
                    return reply

                # Act / Assert: status must be exactly 200 even for valid bytes.
                with self.assertRaisesRegex(ValueError, "returned non-200"):
                    release.public_readback(self.file, "linux/package", opener)
                self.assertEqual(len(replies), 4)
                self.assertTrue(all(not reply.read_sizes for reply in replies))

    def test_GivenWrongPublicBytes_WhenReadbackRuns_ThenVerificationFails(self):
        for label, body in (
            ("empty", b""),
            ("truncated", self.payload[:-1]),
            ("overlong", self.payload + b"extra"),
            ("same-size corruption", b"x" * len(self.payload)),
            ("HTML challenge", b"<html>Complete a challenge</html>"),
        ):
            with self.subTest(body=label):
                # Act / Assert: an identified client never bypasses SHA256.
                with self.assertRaisesRegex(ValueError, "does not match source"):
                    release.public_readback(
                        self.file, "linux/package",
                        lambda request, timeout: self.Reply(body))

    def test_GivenPersistent403_WhenReadbackRuns_ThenHTTPErrorRemainsVisible(self):
        # Arrange.
        relative = "linux/package"
        error = HTTPError(release.BASE + relative, 403, "Forbidden", {}, None)
        calls = []

        def opener(request, timeout):
            self.assert_request(request, timeout, relative)
            calls.append(request)
            raise error

        # Act / Assert: failure still blocks publication after bounded attempts.
        with self.assertRaises(HTTPError) as raised:
            release.public_readback(self.file, relative, opener)
        self.assertIs(raised.exception, error)
        self.assertEqual(len(calls), 4)


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
        self.key = "linux/" + self.file.name
        self.object_key = release.PREFIX + self.key

    def test_missing_file_is_uploaded_to_isolated_bucket_and_sha_verified(self):
        release.upload(self.client, self.key, self.file, True)
        self.assertEqual(len(self.client.writes),1)
        bucket, key, options = self.client.writes[0]
        self.assertEqual((bucket,key),("app-dist",self.object_key))
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
