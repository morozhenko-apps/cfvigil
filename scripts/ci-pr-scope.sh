#!/usr/bin/env bash
# Complete merge-diff classification; unknown inputs never prove prose-only.
set -euo pipefail
: "${GITHUB_OUTPUT:?GITHUB_OUTPUT must name the job output file}"
required=true
case "${GITHUB_EVENT_NAME:-}" in
  pull_request)
    read -r -a refs <<< "$(git rev-list --parents -n 1 HEAD)"
    [[ ${#refs[@]} -eq 3 ]] || { echo 'Expected a two-parent PR merge' >&2; exit 1; }
    changed=$(mktemp)
    trap 'rm -f -- "$changed"' EXIT
    git diff --no-renames --name-only -z "${refs[1]}" "${refs[0]}" > "$changed"
    [[ -s "$changed" ]] || { echo 'Empty PR merge diff' >&2; exit 1; }
    required=false
    while IFS= read -r -d '' path; do
      case "$path" in
        docs/*.md|doc/*.md|README.md|AGENT.md|AGENTS.md|CHANGELOG.md|CONTRIBUTING.md|.github/agents/*.md|.codex.md) ;;
        *) required=true ;;
      esac
    done < "$changed"
    ;;
  push|workflow_dispatch) ;;
  *) echo 'Unsupported CI event' >&2; exit 1 ;;
esac
printf 'required=%s\n' "$required" >> "$GITHUB_OUTPUT"
