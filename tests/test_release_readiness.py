"""Offline safety tests for the owner's GitHub release environment helper."""
import contextlib
import io
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import release_readiness as ready


class ReleaseReadinessTests(unittest.TestCase):
    def setUp(self):
        self.environment = {
            "name": ready.ENVIRONMENT,
            "protection_rules": [{
                "type": "required_reviewers",
                "reviewers": [{"type": "User", "reviewer": {"login": "offline"}}],
                "prevent_self_review": False,
            }, {"type": "branch_policy"}],
            "deployment_branch_policy": {
                "protected_branches": False, "custom_branch_policies": True,
            },
        }
        self.policies = [{"name": "v*", "type": "tag"}]

    def test_only_explicit_tag_policy_and_all_credential_names_pass(self):
        self.assertEqual(ready.evaluate(self.environment, self.policies,
                                        set(ready.VARIABLES), set(ready.SECRETS)), [])

    def test_missing_secret_and_variable_names_report_only_names(self):
        problems = ready.evaluate(
            self.environment, self.policies,
            set(ready.VARIABLES) - {"CFVIGIL_R2_ACCOUNT_ID"},
            set(ready.SECRETS) - {"CFVIGIL_APT_SIGNING_PASSPHRASE"})
        self.assertEqual(problems, [
            "Missing variable: CFVIGIL_R2_ACCOUNT_ID",
            "Missing secret: CFVIGIL_APT_SIGNING_PASSPHRASE",
        ])
        self.assertNotIn("value", " ".join(problems))

    def test_misconfigured_reviewer_tag_and_branch_policy_fail_closed(self):
        unprotected = {
            **self.environment,
            "protection_rules": [{"type": "branch_policy"}],
            "deployment_branch_policy": None,
        }
        cases = [
            (unprotected, self.policies, "Required release reviewer"),
            (self.environment, [], "Missing v* release tag"),
            (self.environment,
             self.policies + [{"name": "main", "type": "branch"}],
             "Unexpected extra"),
        ]
        for env, policies, expected in cases:
            with self.subTest(expected=expected):
                errors = ready.evaluate(env, policies, set(ready.VARIABLES),
                                        set(ready.SECRETS))
                self.assertTrue(any(expected in error for error in errors))

    def test_absent_or_inaccessible_environment_never_claims_ready(self):
        with patch.object(ready, "gh_api",
                          side_effect=ready.GitHubUnavailable("HTTP 404")):
            result = io.StringIO()
            with contextlib.redirect_stdout(result):
                self.assertFalse(ready.assess_remote())
        self.assertIn("missing or inaccessible", result.getvalue())

    def test_existing_environment_init_is_non_destructive(self):
        seen = []
        def github(path, **kwargs):
            seen.append((path, kwargs))
            if path == ready.ROOT:
                return {"permissions": {"admin": True}}
            if path == ready.ENV:
                return self.environment
            raise AssertionError("No other requests allowed")
        with patch.object(ready, "gh_api", side_effect=github):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(ready.init_environment())
        self.assertEqual([path for path, _ in seen], [ready.ROOT, ready.ENV])
        self.assertTrue(all(not kwargs for _, kwargs in seen))

    def test_init_requires_confirmed_repo_admin_permission(self):
        with patch.object(ready, "gh_api",
                          return_value={"permissions": {"admin": False}}) as api:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(ready.init_environment())
            api.assert_called_once_with(ready.ROOT)

    def test_explicit_init_adds_only_self_reviewer_and_tag_pattern(self):
        writes = []
        def github(path, **kwargs):
            if path == ready.ROOT:
                return {"permissions": {"admin": True}}
            if path == ready.ENV and not kwargs:
                raise ready.GitHubUnavailable("404")
            if path == f"{ready.ROOT}/environments?per_page=100":
                return {"total_count": 0, "environments": []}
            if path == "user":
                return {"id": 123}
            writes.append((path, kwargs))
            return {"name": ready.ENVIRONMENT}
        with patch.object(ready, "gh_api", side_effect=github):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertTrue(ready.init_environment())
        self.assertEqual(len(writes), 2)
        self.assertEqual(writes[0][0], ready.ENV)
        self.assertEqual(writes[0][1]["method"], "PUT")
        self.assertEqual(writes[0][1]["body"]["reviewers"],
                         [{"type": "User", "id": 123}])
        self.assertFalse(writes[0][1]["body"]["prevent_self_review"])
        self.assertEqual(writes[1], (
            ready.ENV + "/deployment-branch-policies",
            {"method": "POST", "form": {"name": "v*", "type": "tag"}},
        ))
        self.assertFalse(any("secret" in str(v).lower() for v in writes))

    def test_missing_full_environment_list_refuses_creation(self):
        def github(path, **kwargs):
            if path == ready.ROOT:
                return {"permissions": {"admin": True}}
            if path == ready.ENV:
                raise ready.GitHubUnavailable("404")
            if path == f"{ready.ROOT}/environments?per_page=100":
                return {"total_count": 101, "environments": []}
            raise AssertionError("Unexpected mutating request")
        with patch.object(ready, "gh_api", side_effect=github):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(ready.init_environment())


if __name__ == "__main__":
    unittest.main()
