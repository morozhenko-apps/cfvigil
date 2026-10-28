import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const read = path => readFileSync(new URL("../" + path, import.meta.url), "utf8");
const workflow = read(".github/workflows/recover-release.yml");
const normal = read(".github/workflows/protected-release.yml");
const job = name => {
  const start = workflow.indexOf("\n  " + name + ":\n");
  assert.ok(start >= 0);
  const tail = workflow.slice(start + 1);
  const next = tail.slice(1).search(/^  [a-z][a-z-]*:\n/m);
  return next < 0 ? tail : tail.slice(0, next + 1);
};

test("recovery requires its exact branch and explicit marker without changing normal tag admission", () => {
  const trigger = workflow.slice(workflow.indexOf("\non:\n"), workflow.indexOf("\npermissions:"));
  assert.match(trigger, /branches: \[release-recovery\]/);
  assert.match(trigger, /paths: \['ci\/run-release-recovery\.json'\]/);
  assert.doesNotMatch(trigger, /workflow_dispatch|pull_request|tags:|\[dev\]/);
  assert.match(job("validate"), /github\.repository == 'morozhenko-apps\/cfvigil'/);
  assert.match(job("validate"), /github\.ref == 'refs\/heads\/release-recovery'/);
  assert.match(job("validate"), /github\.event\.deleted == false/);
  assert.match(normal, /tags: \['v\*'\]/);
  assert.doesNotMatch(normal, /release-recovery|workflow_dispatch|branches: \[dev\]/);
  assert.match(read(".github/workflows/checks.yml"), /'\.github\/workflows\/recover-release\.yml'/);
});

test("recovery validates source and stable contracts before protected signing access", () => {
  const validation = job("validate");
  const publish = job("publish");
  assert.match(validation, /scripts\/release_recovery\.py --request ci\/run-release-recovery\.json/);
  assert.match(validation, /node --test tests\/\*\.test\.mjs/);
  assert.match(validation, /python3 -m unittest discover -s tests -p 'test_release_\*\.py'/);
  assert.doesNotMatch(validation, /secrets\.|environment:/);
  assert.match(publish, /needs: validate/);
  assert.match(publish, /environment: cfvigil-release/);
  assert.match(publish, /--approved-tag "\$APPROVED_TAG"/);
  assert.match(publish, /\$\{\{ vars\.CFVIGIL_RELEASE_APPROVED_TAG \}\}/);
  assert.ok(publish.indexOf("scripts/release_recovery.py") < publish.indexOf("secrets.CFVIGIL_APT_SIGNING"));
  assert.match(publish, /scripts\/verify_signing_key\.py/);
  assert.match(publish, /if: always\(\)/);
});

test("recovery reuses only admitted archive IDs and original source provenance", () => {
  const publish = job("publish");
  assert.equal((publish.match(/uses: actions\/download-artifact@v4/g) ?? []).length, 2);
  for (const platform of ["linux", "windows"]) {
    assert.ok(publish.includes("artifact-ids: ${{ steps.admit.outputs." + platform + "_artifact_id }}"));
    assert.ok(publish.includes("path: ${{ runner.temp }}/release/" + platform));
  }
  assert.equal((publish.match(/run-id: \$\{\{ steps\.admit\.outputs\.source_run_id \}\}/g) ?? []).length, 2);
  assert.match(publish, /--commit "\$SOURCE_COMMIT" --run-id "\$SOURCE_RUN_ID"/);
  assert.doesNotMatch(workflow, /cargo |tauri build|upload-artifact|gh release upload/);
  assert.doesNotMatch(workflow, /--commit "\$GITHUB_SHA"|--run-id "\$GITHUB_RUN_ID"/);
  assert.doesNotMatch(workflow, /\$\{\{[^}\n]+\}\}\}/);
});

test("recovery retains receipt-gated cleanup and finalizes the original approved tag last", () => {
  assert.match(workflow, /group: cfvigil-release\n  cancel-in-progress: false/);
  assert.match(job("cleanup"), /needs: \[validate, publish\]/);
  assert.match(job("cleanup"), /\$\{\{ needs\.publish\.outputs\.receipt \}\}/);
  assert.match(job("cleanup"), /scripts\/cleanup-published-artifacts\.mjs --verified-receipt/);
  assert.match(job("cleanup"), /actions: write/);
  assert.match(job("github-release"), /needs: \[validate, publish, cleanup\]/);
  assert.match(job("github-release"), /gh release create "\$RELEASE_TAG"/);
  assert.match(job("github-release"), /--verify-tag/);
  assert.match(job("github-release"), /contents: write/);
  assert.equal((workflow.match(/actions: write/g) ?? []).length, 1);
  assert.equal((workflow.match(/contents: write/g) ?? []).length, 1);
  assert.doesNotMatch(workflow, /administration:|gh api.*environments|git push|git tag/);
});
