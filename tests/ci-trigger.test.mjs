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

test("mainline push only runs preflight on the explicit verification marker", () => {
  const push = section("  push:\n", "  pull_request:\n");
  assert.match(push, /branches: \[main\]/);
  assert.match(push, /paths: \['ci\/run-preflight\.txt'\]/);
  assert.equal((push.match(/paths:/g) || []).length, 1);
  assert.doesNotMatch(push, /docs\/|README\.md|Cargo\.toml|apps\/desktop/);
});

test("all PRs report required status while the complete merge classifier resolves prose", () => {
  const pr = section("  pull_request:\n", "  workflow_dispatch:\n");
  assert.match(pr, /branches: \[main\]/);
  assert.doesNotMatch(pr, /paths(?:-ignore)?:/);
  assert.match(workflow, /fetch-depth: 0/);
  assert.match(workflow, /run: bash scripts\/ci-pr-scope\.sh/);
});

test("three independent hosted workers feed the strict existing preflight result", () => {
  assert.doesNotMatch(workflow, /self-hosted|container:\s*\n/);
  assert.equal((workflow.match(/^    runs-on: ubuntu-24\.04$/gm) || []).length, 5);
  for (const name of ["javascript", "distribution", "rust"]) {
    const job = section("  " + name + ":\n", "\n  " + (name === "javascript" ? "distribution" : name === "distribution" ? "rust" : "stable") + ":\n");
    assert.match(job, /needs: scope/);
    assert.match(job, /if: needs\.scope\.outputs\.required == 'true'/);
  }
  assert.match(workflow, /needs: \[scope, javascript, distribution, rust\]/);
  assert.match(workflow, /name: Stable UI and Rust verification/);
  assert.match(workflow, /if: always\(\)/);
  assert.match(workflow, /node --test tests\/\*\.test\.mjs/);
  assert.match(workflow, /cargo check --locked -p cfvigil/);
  assert.match(workflow, /cargo test --locked -p cfvigil/);
  assert.match(workflow, /cancel-in-progress: true/);
});
