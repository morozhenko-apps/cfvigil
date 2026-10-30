#!/usr/bin/env python3
"""One-command, owner-side CFVigil protected release readiness audit.

Default is read-only. --init-environment explicitly creates an otherwise
missing GitHub Environment and its v* tag policy; it never writes secrets,
Cloudflare state, tags, branches or releases. Do not run in hosted CI.
"""
import argparse
import json
import re
import subprocess
import sys

REPO = "morozhenko-apps/cfvigil"
ENVIRONMENT = "cfvigil-release"
ROOT = f"repos/{REPO}"
ENV = f"{ROOT}/environments/{ENVIRONMENT}"
VARIABLES = frozenset((
    "CFVIGIL_RELEASE_APPROVED_TAG",
    "CFVIGIL_APT_FINGERPRINT",
    "CFVIGIL_R2_ACCOUNT_ID",
))
SECRETS = frozenset((
    "CFVIGIL_APT_SIGNING_SUBKEY_BASE64",
    "CFVIGIL_APT_SIGNING_PASSPHRASE",
    "CFVIGIL_R2_ACCESS_KEY_ID",
    "CFVIGIL_R2_SECRET_ACCESS_KEY",
))


class GitHubUnavailable(Exception):
    """Do not confuse 404 / permission errors with a proven absent resource."""


def gh_api(endpoint, *, method=None, body=None, form=None):
    cmd = ["gh", "api"]
    if method:
        cmd.extend(["--method", method])
    cmd.append(endpoint)
    if body is not None:
        cmd.extend(["--input", "-"])
    if form:
        for key, value in form.items():
            cmd.extend(["-f", f"{key}={value}"])
    try:
        result = subprocess.run(
            cmd, input=json.dumps(body) if body is not None else None,
            capture_output=True, text=True, timeout=20, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GitHubUnavailable("gh command unavailable or timed out") from error
    if result.returncode:
        # GH's stderr can contain URLs/account context. Never log it,
        # especially not while the owner is configuring release secrets.
        raise GitHubUnavailable(f"GitHub API returned an error (exit {result.returncode})")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise GitHubUnavailable("GitHub API returned invalid JSON") from error


def evaluate(environment, policies, variable_names, secret_names):
    """Assess provider metadata without ever examining credential values."""
    problems = []
    if not isinstance(environment, dict) or environment.get("name") != ENVIRONMENT:
        return ["Expected GitHub environment is unavailable"]
    rules = environment.get("protection_rules")
    if not isinstance(rules, list):
        return ["Environment protection rules cannot be verified"]
    reviewer_rules = [r for r in rules
                      if isinstance(r, dict) and r.get("type") == "required_reviewers"]
    if not reviewer_rules or not any(
            any(isinstance(person, dict) and person.get("type") in ("User", "Team")
                for person in rule.get("reviewers", []))
            for rule in reviewer_rules):
        problems.append("Required release reviewer is not configured")
    settings = environment.get("deployment_branch_policy")
    if settings != {"protected_branches": False, "custom_branch_policies": True}:
        problems.append("Environment must use custom tag/branch restrictions")
    if not isinstance(policies, list) or not any(
            isinstance(rule, dict) and rule.get("name") == "v*"
            and rule.get("type") == "tag" for rule in policies):
        problems.append("Missing v* release tag deployment policy")
    elif any(not isinstance(rule, dict) or
             rule.get("name") != "v*" or rule.get("type") != "tag"
             for rule in policies):
        problems.append("Unexpected extra environment deployment policies")
    if not isinstance(variable_names, set):
        problems.append("Environment variable names could not be checked")
    else:
        problems.extend(f"Missing variable: {name}"
                        for name in sorted(VARIABLES - variable_names))
    if not isinstance(secret_names, set):
        problems.append("Environment secret names could not be checked")
    else:
        problems.extend(f"Missing secret: {name}"
                        for name in sorted(SECRETS - secret_names))
    return problems


def read_names(endpoint, collection):
    payload = gh_api(endpoint)
    if (not isinstance(payload, dict) or not isinstance(payload.get(collection), list) or
            payload.get("total_count") != len(payload[collection])):
        raise GitHubUnavailable("GitHub did not return a complete names-only listing")
    return {row["name"] for row in payload[collection]
            if isinstance(row, dict) and isinstance(row.get("name"), str)}


def assess_remote():
    try:
        environment = gh_api(ENV)
    except GitHubUnavailable:
        print("[BLOCKED] cfvigil-release is missing or inaccessible; check gh auth and repository admin access.")
        return False
    try:
        policies_response = gh_api(f"{ENV}/deployment-branch-policies?per_page=100")
        policies = policies_response.get("branch_policies")
        if not isinstance(policies, list) or policies_response.get("total_count") != len(policies):
            raise GitHubUnavailable("Incomplete environment deployment policy listing")
        variable_names = read_names(f"{ENV}/variables?per_page=100", "variables")
        secret_names = read_names(f"{ENV}/secrets?per_page=100", "secrets")
    except (GitHubUnavailable, AttributeError):
        print("[BLOCKED] Cannot verify environment protection or credential NAMES (not values).")
        return False
    problems = evaluate(environment, policies, variable_names, secret_names)
    if problems:
        for problem in problems:
            print("[BLOCKED]", problem)
    else:
        print("[PASS] Required reviewer and v* tag-only environment policy.")
        print("[PASS] All 3 release variable names and all 4 secret names are present.")
        print("[NOTE] Presence does not prove secret values, R2 permissions or GPG key validity.")
    return not problems


def init_environment():
    """Explicit bootstrap only; never overwrite an existing environment."""
    try:
        repository = gh_api(ROOT)
        if repository.get("permissions", {}).get("admin") is not True:
            print("[BLOCKED] Confirmed repository admin permission is required.")
            return False
        try:
            gh_api(ENV)
        except GitHubUnavailable:
            # A failed GET could mean auth failure or 404. Check that the
            # repository is readable and that environment list succeeded.
            listing = gh_api(f"{ROOT}/environments?per_page=100")
            present = listing.get("environments")
            if not isinstance(present, list):
                print("[BLOCKED] Cannot establish existing environment inventory.")
                return False
            if any(row.get("name") == ENVIRONMENT for row in present
                   if isinstance(row, dict)):
                print("[BLOCKED] Environment exists; access is incomplete.")
                return False
            if listing.get("total_count", 0) > len(present):
                print("[BLOCKED] Environment listing is incomplete; refusing to create.")
                return False
        else:
            print("[BLOCKED] Existing environment must be reviewed manually; no overwrite.")
            return False
        identity = gh_api("user")
        user_id = identity.get("id")
        if not isinstance(user_id, int) or user_id <= 0:
            print("[BLOCKED] Cannot confirm GitHub owner ID.")
            return False
        gh_api(ENV, method="PUT", body={
            "prevent_self_review": False,
            "reviewers": [{"type": "User", "id": user_id}],
            "deployment_branch_policy": {
                "protected_branches": False, "custom_branch_policies": True,
            },
        })
        gh_api(f"{ENV}/deployment-branch-policies", method="POST",
               form={"name": "v*", "type": "tag"})
    except GitHubUnavailable as error:
        print(f"[BLOCKED] GitHub refused setup: {error}. Inspect environment in Settings.")
        return False
    print("[CREATED] Owner-reviewed cfvigil-release environment and v* tag policy.")
    print("[NEXT] Populate approved variables and secrets yourself; values are never requested by this script.")
    return True


def branch_state():
    # The sole long-lived branch replaces the retired dev/main equality check.
    # This read-only preflight is not CI or release authorization evidence.
    try:
        result = gh_api(f"{ROOT}/branches/main")
        sha = result.get("commit", {}).get("sha", "")
        if not isinstance(sha, str) or not re.fullmatch(r"[a-f0-9]{40}", sha):
            raise GitHubUnavailable("Missing or malformed main commit SHA")
    except (GitHubUnavailable, AttributeError):
        print("[UNKNOWN] Cannot inspect main branch identity.")
        return False
    print(f"[INFO] main {sha[:12]}")
    print("[PASS] Mainline identity is readable; tested source and owner-approved tagging remain separate.")
    return True


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--init-environment", action="store_true",
                   help="EXPLICIT write: create missing owner-reviewed environment and v* tag policy")
    args = p.parse_args(argv)
    print("CFVigil release readiness (GitHub names and protection only, no secret values)")
    if args.init_environment:
        if not init_environment():
            return 2
    branch_ok = branch_state()
    passed = assess_remote()
    print("[NOTE] No tag, main promotion, R2 upload or signing occurs here.")
    return 0 if passed and branch_ok else 2


if __name__ == "__main__":
    sys.exit(main())
