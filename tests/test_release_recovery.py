"""Offline admission contracts for recovery of existing release archives."""
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import release_recovery as recovery

COMMIT = "a" * 40
OTHER = "b" * 40


class RecoveryAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.request = {
            "schema": 1, "source_run_id": 51, "tag": "v0.1.0", "commit": COMMIT,
            "artifacts": {recovery.NAMES[0]: 101, recovery.NAMES[1]: 102},
        }
        self.run = {
            "id": 51, "path": recovery.SOURCE_WORKFLOW, "event": "push",
            "head_branch": "v0.1.0", "head_sha": COMMIT,
            "status": "completed", "conclusion": "failure",
            "repository": {"full_name": recovery.REPO, "id": 17},
            "head_repository": {"full_name": recovery.REPO, "id": 17},
        }
        required = {"validate": "success", "linux": "success", "windows": "success",
                    "publish": "failure", "cleanup": "skipped", "github-release": "skipped"}
        self.jobs = {"total_count": len(required), "jobs": [
            {"name": name, "run_id": 51, "status": "completed", "conclusion": state}
            for name, state in required.items()
        ]}
        self.artifacts = {"total_count": 2, "artifacts": [
            {"id": identity, "name": name, "size_in_bytes": 100, "expired": False,
             "digest": "sha256:" + "c" * 64,
             "workflow_run": {"id": 51, "head_sha": COMMIT, "head_branch": "v0.1.0",
                              "repository_id": 17, "head_repository_id": 17}}
            for name, identity in self.request["artifacts"].items()
        ]}
        self.expected = {
            "version": "0.1.0", "tag": "v0.1.0", "commit": COMMIT,
            "source_run_id": "51", "linux_artifact_id": "101", "windows_artifact_id": "102",
        }

    def verify(self, request=None, run=None, jobs=None, artifacts=None,
               main_sha=COMMIT, tag_sha=COMMIT):
        return recovery.verify_source(
            self.request if request is None else request,
            self.run if run is None else run,
            self.jobs if jobs is None else jobs,
            self.artifacts if artifacts is None else artifacts,
            main_sha, tag_sha)

    def api(self, endpoint):
        resources = {
            recovery.ROOT + "/branches/main": {"commit": {"sha": COMMIT}},
            recovery.ROOT + "/git/ref/tags/v0.1.0": {"object": {"type": "commit", "sha": COMMIT}},
            recovery.ROOT + "/actions/runs/51": self.run,
            recovery.ROOT + "/actions/runs/51/jobs?filter=latest&per_page=100": self.jobs,
            recovery.ROOT + "/actions/runs/51/artifacts?per_page=100": self.artifacts,
        }
        self.assertIn(endpoint, resources, "Only exact read-only source endpoints are permitted")
        return copy.deepcopy(resources[endpoint])

    def test_GivenCompleteFailedPublication_WhenAdmitted_ThenOriginalIdentitiesArePreserved(self):
        # Arrange / Act: exercise the real admission algorithm with a fake API.
        before = copy.deepcopy(self.request)
        result = recovery.admit(self.request, api=self.api)
        # Assert: cleanup must receive the original run, never the recovery run.
        self.assertEqual(result, self.expected)
        self.assertEqual(self.request, before)

    def test_GivenMalformedRequest_WhenValidated_ThenNoSourceIsAdmitted(self):
        cases = [
            ("schema", True), ("schema", 2),
            ("source_run_id", True), ("source_run_id", 0), ("source_run_id", -1),
            ("source_run_id", "51"), ("source_run_id", 2**53),
            ("tag", "v01.1.0"), ("tag", "v0.1.0-rc1"), ("tag", "v0.1.0\n"),
            ("tag", "../main"), ("tag", None),
            ("commit", OTHER.upper()), ("commit", "a" * 39), ("commit", None),
            ("artifacts", []), ("artifacts", {recovery.NAMES[0]: 101}),
            ("artifacts", {recovery.NAMES[0]: True, recovery.NAMES[1]: 102}),
            ("artifacts", {recovery.NAMES[0]: 101, recovery.NAMES[1]: 101}),
            ("artifacts", {recovery.NAMES[0]: 0, recovery.NAMES[1]: 102}),
            ("artifacts", {recovery.NAMES[0]: 101, "unrelated": 102}),
        ]
        for key, value in cases:
            with self.subTest(field=key, value=value):
                request = {**self.request, key: value}
                with self.assertRaises(ValueError):
                    recovery.validate_request(request)
        for request in ([], {}, {**self.request, "extra": "unsafe"}):
            with self.subTest(request=request), self.assertRaises(ValueError):
                recovery.validate_request(request)

    def test_GivenMovedMainOrTag_WhenVerified_ThenRecoveryStops(self):
        for main_sha, tag_sha in ((OTHER, COMMIT), (COMMIT, OTHER), (OTHER, OTHER), (None, COMMIT)):
            with self.subTest(main=main_sha, tag=tag_sha):
                with self.assertRaisesRegex(ValueError, "exact current main"):
                    self.verify(main_sha=main_sha, tag_sha=tag_sha)

    def test_GivenWrongSourceRun_WhenVerified_ThenRecoveryStops(self):
        cases = [
            ("id", 52), ("path", ".github/workflows/package-candidate.yml"),
            ("event", "workflow_dispatch"), ("head_branch", "dev"), ("head_sha", OTHER),
            ("status", "in_progress"), ("conclusion", "success"),
            ("repository", {"full_name": "foreign/cfvigil", "id": 17}),
            ("repository", {"full_name": recovery.REPO, "id": True}),
            ("head_repository", {"full_name": "foreign/cfvigil"}),
        ]
        for key, value in cases:
            with self.subTest(field=key), self.assertRaises(ValueError):
                self.verify(run={**self.run, key: value})

    def test_GivenUnsafeOriginalJobState_WhenVerified_ThenNoArchiveIsAdmitted(self):
        for index, job in enumerate(self.jobs["jobs"]):
            for key, value in (("run_id", 52), ("status", "in_progress"),
                               ("conclusion", "failure" if job["conclusion"] == "success" else "success")):
                with self.subTest(job=job["name"], field=key):
                    jobs = copy.deepcopy(self.jobs)
                    jobs["jobs"][index][key] = value
                    with self.assertRaises(ValueError):
                        self.verify(jobs=jobs)

    def test_GivenMissingOrAmbiguousJobs_WhenVerified_ThenRecoveryStops(self):
        for entries in (self.jobs["jobs"][:-1], self.jobs["jobs"] + [self.jobs["jobs"][0]],
                        [None] + self.jobs["jobs"][1:]):
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                self.verify(jobs={"total_count": len(entries), "jobs": entries})

    def test_GivenPartialMetadataListing_WhenVerified_ThenRecoveryStops(self):
        for key in ("jobs", "artifacts"):
            original = getattr(self, key)
            for count in (0, len(original[key]) + 1, True, "2"):
                with self.subTest(collection=key, count=count):
                    with self.assertRaises(ValueError):
                        self.verify(**{key: {**original, "total_count": count}})
        for malformed in (None, [], {}, {"total_count": 1, "jobs": "invalid"}):
            with self.subTest(payload=malformed), self.assertRaises(ValueError):
                recovery.complete_collection(malformed, "jobs")

    def test_GivenChangedOrExpiredArchive_WhenVerified_ThenRecoveryStops(self):
        for index in (0, 1):
            cases = [
                ("id", 103), ("expired", True), ("expired", None),
                ("size_in_bytes", 0), ("size_in_bytes", -1), ("size_in_bytes", True),
                ("size_in_bytes", 800 * 1024 * 1024 + 1),
                ("digest", None), ("digest", "sha256:" + "f" * 63),
                ("digest", "md5:" + "f" * 64), ("workflow_run", None),
            ]
            for key, value in cases:
                with self.subTest(platform=index, field=key, value=value):
                    artifacts = copy.deepcopy(self.artifacts)
                    artifacts["artifacts"][index][key] = value
                    with self.assertRaises(ValueError):
                        self.verify(artifacts=artifacts)
            for key, value in (("id", 52), ("head_sha", OTHER), ("head_branch", "dev"),
                               ("repository_id", 18), ("head_repository_id", 18)):
                with self.subTest(platform=index, source_field=key):
                    artifacts = copy.deepcopy(self.artifacts)
                    artifacts["artifacts"][index]["workflow_run"][key] = value
                    with self.assertRaises(ValueError):
                        self.verify(artifacts=artifacts)

    def test_GivenArchiveSizeBoundary_WhenVerified_ThenBoundedArchivesAreAccepted(self):
        for size in (1, 800 * 1024 * 1024):
            with self.subTest(size=size):
                artifacts = copy.deepcopy(self.artifacts)
                for item in artifacts["artifacts"]:
                    item["size_in_bytes"] = size
                self.assertEqual(self.verify(artifacts=artifacts), self.expected)

    def test_GivenMissingDuplicateOrAdditionalArchive_WhenVerified_ThenRecoveryStops(self):
        items = self.artifacts["artifacts"]
        for entries in (items[:1], [items[0], items[0]], items + [items[0]], [None, items[1]]):
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                self.verify(artifacts={"total_count": len(entries), "artifacts": entries})

    def test_GivenAnnotatedTag_WhenResolved_ThenItsOriginalCommitIsUsed(self):
        calls = []
        def api(endpoint):
            calls.append(endpoint)
            if endpoint.endswith("/git/ref/tags/v0.1.0"):
                return {"object": {"type": "tag", "sha": OTHER}}
            if endpoint.endswith("/git/tags/" + OTHER):
                return {"object": {"type": "commit", "sha": COMMIT}}
            raise AssertionError(endpoint)
        self.assertEqual(recovery.resolve_tag("v0.1.0", api), COMMIT)
        self.assertEqual(len(calls), 2)

    def test_GivenUnsafeTagReference_WhenResolved_ThenResolutionIsBoundedAndFails(self):
        for obj in (None, {"type": "tree", "sha": COMMIT}, {"type": "commit", "sha": "bad"},
                    {"type": "tag", "sha": OTHER}):
            with self.subTest(object=obj):
                calls = []
                def api(endpoint):
                    calls.append(endpoint)
                    return {"object": obj}
                with self.assertRaises(ValueError):
                    recovery.resolve_tag("v0.1.0", api)
                self.assertLessEqual(len(calls), 2)
        count = []
        def nested(endpoint):
            count.append(endpoint)
            return {"object": {"type": "tag", "sha": f"{len(count):040x}"}}
        with self.assertRaisesRegex(ValueError, "too many nested"):
            recovery.resolve_tag("v0.1.0", nested)
        self.assertEqual(len(count), 5)

    def test_GivenInvalidRequest_WhenAdmitted_ThenNoGitHubRequestIsSent(self):
        def api(endpoint):
            self.fail("Invalid input must never become an API endpoint")
        with self.assertRaises(ValueError):
            recovery.admit({**self.request, "source_run_id": "../other"}, api=api)

    def test_GivenDuplicateRequestField_WhenParsed_ThenRecoveryStops(self):
        for value in ('{"tag":"v0.1.0","tag":"v0.2.0"}',
                      '{"artifacts":{"linux":101,"linux":102}}'):
            with self.subTest(json=value), self.assertRaisesRegex(ValueError, "Duplicate"):
                json.loads(value, object_pairs_hook=recovery.unique_object)

    def test_GivenDedicatedBranch_WhenCLIAdmits_ThenOnlySafeOriginalOutputsAreWritten(self):
        with tempfile.TemporaryDirectory() as directory:
            request = Path(directory) / "request.json"
            request.write_text(json.dumps(self.request))
            output = Path(directory) / "output"
            env = {"GITHUB_REPOSITORY": recovery.REPO, "GITHUB_EVENT_NAME": "push",
                   "GITHUB_REF": "refs/heads/release-recovery", "GITHUB_OUTPUT": str(output)}
            with patch.dict(os.environ, env, clear=True), contextlib.redirect_stdout(io.StringIO()):
                recovery.main(["--request", str(request), "--approved-tag", "v0.1.0"], api=self.api)
            self.assertEqual(dict(line.split("=", 1) for line in output.read_text().splitlines()),
                             self.expected)

    def test_GivenWrongTriggerOrApproval_WhenCLIAdmits_ThenNothingIsFetchedOrWritten(self):
        with tempfile.TemporaryDirectory() as directory:
            request = Path(directory) / "request.json"
            request.write_text(json.dumps(self.request))
            output = Path(directory) / "output"
            env = {"GITHUB_REPOSITORY": recovery.REPO, "GITHUB_EVENT_NAME": "push",
                   "GITHUB_REF": "refs/heads/release-recovery", "GITHUB_OUTPUT": str(output)}
            cases = [({"GITHUB_REPOSITORY": "foreign/cfvigil"}, []),
                     ({"GITHUB_EVENT_NAME": "workflow_dispatch"}, []),
                     ({"GITHUB_REF": "refs/heads/dev"}, []),
                     ({"GITHUB_REF": "refs/tags/v0.1.0"}, []),
                     ({}, ["--approved-tag", "v0.2.0"]), ({}, ["--approved-tag", ""])]
            def api(endpoint):
                self.fail("An unapproved trigger must not query GitHub")
            for overrides, args in cases:
                with self.subTest(overrides=overrides, args=args):
                    with patch.dict(os.environ, {**env, **overrides}, clear=True):
                        with self.assertRaises(ValueError):
                            recovery.main(["--request", str(request), *args], api=api)
                    self.assertFalse(output.exists())

    def test_GivenUnsafeMarker_WhenCLIAdmits_ThenNoGitHubRequestIsSent(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            env = {"GITHUB_REPOSITORY": recovery.REPO, "GITHUB_EVENT_NAME": "push",
                   "GITHUB_REF": "refs/heads/release-recovery", "GITHUB_OUTPUT": str(root / "output")}
            (root / "empty").write_bytes(b"")
            (root / "large").write_bytes(b"x" * 4097)
            (root / "link").symlink_to(root / "empty")
            def api(endpoint):
                self.fail("Unsafe marker must not query GitHub")
            for path in (root / "missing", root / "empty", root / "large", root / "link", root):
                with self.subTest(path=path.name), patch.dict(os.environ, env, clear=True):
                    with self.assertRaisesRegex(ValueError, "unsafe recovery request"):
                        recovery.main(["--request", str(path)], api=api)

    def test_GivenGitHubCLI_WhenMetadataIsRead_ThenAuthenticationRemainsOutOfArguments(self):
        response = subprocess.CompletedProcess([], 0, '{"id":51}', "")
        with patch.object(recovery.subprocess, "run", return_value=response) as run:
            self.assertEqual(recovery.github("repos/example/repo"), {"id": 51})
        self.assertEqual(run.call_args.args[0],
                         ["gh", "api", "--hostname", "github.com", "repos/example/repo"])
        self.assertNotIn("--method", run.call_args.args[0])

    def test_GivenGitHubFailure_WhenMetadataIsRead_ThenFailureRemainsVisibleWithoutRawOutput(self):
        responses = [subprocess.CompletedProcess([], 1, "", "private authentication details"),
                     subprocess.CompletedProcess([], 0, "not json", "")]
        for response in responses:
            with self.subTest(code=response.returncode):
                with patch.object(recovery.subprocess, "run", return_value=response):
                    with self.assertRaises(ValueError) as error:
                        recovery.github("repos/example/repo")
                self.assertNotIn("private authentication", str(error.exception))
        with patch.object(recovery.subprocess, "run", side_effect=OSError("unavailable")):
            with self.assertRaisesRegex(ValueError, "unavailable"):
                recovery.github("repos/example/repo")


if __name__ == "__main__":
    unittest.main()
