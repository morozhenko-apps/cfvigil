import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const workflow = readFileSync(
  new URL("../.github/workflows/checks.yml", import.meta.url), "utf8"
).replace(/\r\n/g, "\n");
function section(start, end) {
  const first = workflow.indexOf(start);
  const last = workflow.indexOf(end, first + start.length);
  assert.ok(first >= 0 && last > first, "Expected workflow section");
  return workflow.slice(first + start.length, last);
}

test("dev push only runs preflight on the explicit verification marker", () => {
  const push = section("  push:\n", "  pull_request:\n");
  assert.match(push, /branches: \[dev\]/);
  assert.match(push, /paths:\s*\n\s*- 'ci\/run-preflight\.txt'/);
  assert.equal((push.match(/^\s*- '/gm) || []).length, 1);
  assert.doesNotMatch(push, /docs\/|README\.md|Cargo\.toml|apps\/desktop/);
});

test("temporary agent PRs validate code changes but ignore docs-only changes", () => {
  const pr = section("  pull_request:\n", "  workflow_dispatch:\n");
  assert.match(pr, /branches: \[dev, main\]/);
  for (const source of ["Cargo.toml", "Cargo.lock", "apps/desktop/**", "tests/**"]) {
    assert.ok(pr.includes("'" + source + "'"), source);
  }
  assert.doesNotMatch(pr, /'docs\/|README\.md|AGENTS\.md|CONTRIBUTING\.md/);
});

test("one hosted job runs UI and native Rust checks in a single preflight", () => {
  assert.match(workflow, /runs-on: ubuntu-24\.04/);
  assert.doesNotMatch(workflow, /self-hosted|container:\s*\n/);
  assert.equal((workflow.match(/^    runs-on:/gm) || []).length, 1);
  assert.match(workflow, /node --test tests\/\*\.test\.mjs/);
  assert.match(workflow, /cargo check --locked -p cfvigil/);
  assert.match(workflow, /cargo test --locked -p cfvigil/);
  assert.match(workflow, /cancel-in-progress: true/);
});
