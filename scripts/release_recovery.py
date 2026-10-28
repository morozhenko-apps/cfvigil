#!/usr/bin/env python3
"""Admit exact archives from a failed approved release; never build or publish.

The separate recovery workflow retains protected owner review and delegates
signing, publication and receipt-gated cleanup to the normal release scripts.
All GitHub requests here are read-only and use the runner's scoped token.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

REPO = "morozhenko-apps/cfvigil"
ROOT = "repos/" + REPO
BRANCH = "release-recovery"
SOURCE_WORKFLOW = ".github/workflows/protected-release.yml"
NAMES = ("cfvigil-release-linux", "cfvigil-release-windows")
TAG = re.compile(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\Z")
SHA = re.compile(r"[a-f0-9]{40}\Z")


def positive_id(value):
    return type(value) is int and 0 < value <= 2**53 - 1


def validate_request(request):
    expected = {"schema", "source_run_id", "tag", "commit", "artifacts"}
    if (not isinstance(request, dict) or set(request) != expected or
            type(request["schema"]) is not int or request["schema"] != 1 or
            not positive_id(request["source_run_id"]) or
            not isinstance(request["tag"], str) or not TAG.fullmatch(request["tag"]) or
            not isinstance(request["commit"], str) or not SHA.fullmatch(request["commit"])):
        raise ValueError("Invalid recovery request or source identity")
    artifacts = request["artifacts"]
    if (not isinstance(artifacts, dict) or set(artifacts) != set(NAMES) or
            not all(positive_id(value) for value in artifacts.values()) or
            len(set(artifacts.values())) != len(NAMES)):
        raise ValueError("Recovery requires two exact distinct release archive IDs")
    return request


def complete_collection(payload, key):
    if (not isinstance(payload, dict) or not isinstance(payload.get(key), list) or
            type(payload.get("total_count")) is not int or
            payload["total_count"] != len(payload[key]) or len(payload[key]) > 100):
        raise ValueError("Incomplete GitHub recovery metadata: " + key)
    return payload[key]


def verify_source(request, run, jobs_payload, artifacts_payload, main_sha, tag_sha):
    """Pure provenance checks, independent of tokens, clocks and the network."""
    validate_request(request)
    commit, tag, run_id = request["commit"], request["tag"], request["source_run_id"]
    if main_sha != commit or tag_sha != commit:
        raise ValueError("Recovery source tag must still be the exact current main HEAD")
    if (not isinstance(run, dict) or not positive_id(run.get("id")) or
            run.get("id") != run_id or
            run.get("path") != SOURCE_WORKFLOW or run.get("event") != "push" or
            run.get("head_branch") != tag or run.get("head_sha") != commit or
            run.get("status") != "completed" or run.get("conclusion") != "failure" or
            not isinstance(run.get("repository"), dict) or
            run["repository"].get("full_name") != REPO or
            not positive_id(run["repository"].get("id")) or
            not isinstance(run.get("head_repository"), dict) or
            run["head_repository"].get("full_name") != REPO):
        raise ValueError("Source is not a completed failed approved release in this repository")

    required = {"validate": "success", "linux": "success", "windows": "success",
                "publish": "failure", "cleanup": "skipped", "github-release": "skipped"}
    jobs = complete_collection(jobs_payload, "jobs")
    if (len(jobs) != len(required) or
            any(not isinstance(job, dict) or not isinstance(job.get("name"), str)
                for job in jobs) or
            {job.get("name") for job in jobs} != set(required)):
        raise ValueError("Unexpected original release jobs")
    for name, conclusion in required.items():
        matching = [job for job in jobs if isinstance(job, dict) and job.get("name") == name]
        if (len(matching) != 1 or not positive_id(matching[0].get("run_id")) or
                matching[0].get("run_id") != run_id or
                matching[0].get("status") != "completed" or
                matching[0].get("conclusion") != conclusion):
            raise ValueError("Original release job is not safely recoverable: " + name)

    artifacts = complete_collection(artifacts_payload, "artifacts")
    if len(artifacts) != len(NAMES):
        raise ValueError("Recovery requires both original release archives only")
    for name in NAMES:
        matching = [item for item in artifacts
                    if isinstance(item, dict) and item.get("name") == name]
        if len(matching) != 1:
            raise ValueError("Ambiguous or missing release archive")
        artifact = matching[0]
        source = artifact.get("workflow_run")
        size = artifact.get("size_in_bytes")
        digest = artifact.get("digest")
        if (not positive_id(artifact.get("id")) or
                artifact.get("id") != request["artifacts"][name] or
                artifact.get("expired") is not False or
                type(size) is not int or not 0 < size <= 800 * 1024 * 1024 or
                not isinstance(digest, str) or
                not re.fullmatch(r"sha256:[a-f0-9]{64}", digest) or
                not isinstance(source, dict) or source.get("id") != run_id or
                source.get("head_sha") != commit or source.get("head_branch") != tag or
                source.get("repository_id") != run["repository"]["id"] or
                source.get("head_repository_id") != run["repository"]["id"]):
            raise ValueError("Expired, foreign or changed original release archive")
    return {
        "version": tag[1:], "tag": tag, "commit": commit,
        "source_run_id": str(run_id),
        "linux_artifact_id": str(request["artifacts"][NAMES[0]]),
        "windows_artifact_id": str(request["artifacts"][NAMES[1]]),
    }


def github(endpoint):
    """Use GitHub CLI authentication without logging tokens or raw errors."""
    try:
        response = subprocess.run(
            ["gh", "api", "--hostname", "github.com", endpoint],
            text=True, capture_output=True, timeout=30, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValueError("GitHub recovery metadata is unavailable") from error
    if response.returncode:
        raise ValueError("GitHub recovery metadata request failed")
    try:
        return json.loads(response.stdout)
    except json.JSONDecodeError as error:
        raise ValueError("GitHub recovery metadata is not JSON") from error


def resolve_tag(tag, api):
    reference = api(ROOT + "/git/ref/tags/" + tag)
    seen = set()
    for _ in range(4):
        obj = reference.get("object") if isinstance(reference, dict) else None
        if (not isinstance(obj, dict) or not isinstance(obj.get("sha"), str) or
                not SHA.fullmatch(obj["sha"]) or obj["sha"] in seen):
            raise ValueError("Invalid recovery source tag reference")
        seen.add(obj["sha"])
        if obj.get("type") == "commit":
            return obj["sha"]
        if obj.get("type") != "tag":
            raise ValueError("Recovery source tag does not resolve to a commit")
        reference = api(ROOT + "/git/tags/" + obj["sha"])
    raise ValueError("Recovery source tag has too many nested references")


def admit(request, api=github):
    validate_request(request)  # No unvalidated input enters an API endpoint.
    main = api(ROOT + "/branches/main")
    main_commit = main.get("commit") if isinstance(main, dict) else None
    main_sha = main_commit.get("sha") if isinstance(main_commit, dict) else None
    tag_sha = resolve_tag(request["tag"], api)
    source = ROOT + "/actions/runs/" + str(request["source_run_id"])
    return verify_source(request, api(source),
                         api(source + "/jobs?filter=latest&per_page=100"),
                         api(source + "/artifacts?per_page=100"), main_sha, tag_sha)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Duplicate recovery request field")
        result[key] = value
    return result


def main(argv=None, api=github):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--approved-tag")
    args = parser.parse_args(argv)
    if (os.environ.get("GITHUB_REPOSITORY") != REPO or
            os.environ.get("GITHUB_EVENT_NAME") != "push" or
            os.environ.get("GITHUB_REF") != "refs/heads/" + BRANCH):
        raise ValueError("Recovery admission requires the exact dedicated branch push")
    if (args.request.is_symlink() or not args.request.is_file() or
            not 0 < args.request.stat().st_size <= 4096):
        raise ValueError("Missing or unsafe recovery request marker")
    request = validate_request(json.loads(
        args.request.read_text(encoding="utf8"), object_pairs_hook=unique_object))
    if args.approved_tag is not None and args.approved_tag != request["tag"]:
        raise ValueError("Protected owner approval does not match the original source tag")
    result = admit(request, api=api)
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf8") as output:
        for key, value in result.items():
            output.write(key + "=" + value + "\n")
    print("CFVigil recovery admitted the exact original platform archives")


if __name__ == "__main__":
    main()
