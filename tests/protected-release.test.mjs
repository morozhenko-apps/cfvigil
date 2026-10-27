import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const read=p=>readFileSync(new URL("../"+p,import.meta.url),"utf8").replace(/\r\n/g,"\n");
const release=read(".github/workflows/protected-release.yml");
const publisher=read("scripts/r2_release.py");

test("no dev push or workflow dispatch can publish CFVigil",()=>{
  assert.match(release,/^on:\n  push:\n    tags: \['v\*'\]/m);
  assert.match(release,/git fetch --no-tags origin main/);
  assert.match(release,/git rev-parse origin\/main\^\{commit\}/);
  assert.match(release,/scripts\/candidate-version\.mjs --check-tag/);
  assert.doesNotMatch(release,/workflow_dispatch:|branches: \[dev\]/);
  assert.match(release,/APPROVED_TAG.*CFVIGIL_RELEASE_APPROVED_TAG/);
  assert.match(release,/environment: cfvigil-release/);
  // GitHub Actions uses double-brace expressions; malformed single braces
  // must fail source checks before an approved production tag ever exists.
  assert.match(release,/\$\{\{\s*github\.ref_name\s*\}\}/);
  assert.match(release,/\$\{\{\s*vars\.CFVIGIL_RELEASE_APPROVED_TAG\s*\}\}/);
  assert.match(release,/\$\{\{\s*needs\.publish\.outputs\.receipt\s*\}\}/);
  assert.doesNotMatch(release,/\$\{\s+(?:github|vars|secrets|needs|runner|steps)\./);
});

test("publication orders R2 readback ahead of cleanup and text-only GitHub release",()=>{
  assert.match(release,/python" scripts\/r2_release\.py/);
  assert.match(release,/--receipt "\$RUNNER_TEMP\/publication-receipt\.json"/);
  assert.match(release,/needs: \[validate, linux, windows, publish\]/);
  assert.match(release,/needs: \[validate, publish, cleanup\]/);
  assert.match(release,/scripts\/cleanup-published-artifacts\.mjs/);
  assert.match(release,/name: cfvigil-release-linux/);
  assert.match(release,/name: cfvigil-release-windows/);
  assert.match(release,/retention-days: 1/);
  assert.match(release,/gh release create/);
  assert.doesNotMatch(release,/gh release upload|gh release create.*\*\.deb/);
});

test("publication scopes credentials, R2 prefix and GitHub write permissions",()=>{
  assert.match(release,/CFVIGIL_R2_ACCESS_KEY_ID/);
  assert.match(release,/CFVIGIL_R2_SECRET_ACCESS_KEY/);
  assert.match(release,/CFVIGIL_APT_SIGNING_SUBKEY_BASE64/);
  assert.doesNotMatch(release,/REPOTICKER_|repoticker-release|self-hosted/);
  assert.equal((release.match(/actions: write/g)||[]).length,1);
  assert.match(publisher,/PREFIX = "dist\/CFVigil\/"/);
  assert.match(publisher,/BUCKET = "app-dist"/);
  assert.match(publisher,/base_url/);
  assert.match(publisher,/public_readback/);
  assert.match(publisher,/linux\/InRelease/);
  assert.match(publisher,/latest_manifest/);
  assert.match(publisher,/latest\.json/);
});

test("release uses direct app-dist custom domain and no Worker dependency",()=>{
  assert.match(publisher,/BUCKET = "app-dist"/);
  assert.match(publisher,/PREFIX = "dist\/CFVigil\/"/);
  assert.match(publisher,/BASE = "https:\/\/download\.morozhenko\.com\/"/);
  assert.match(publisher,/upload\(client, "latest\.json"/);
  assert.doesNotMatch(release,/wrangler deploy|cfvigil-download|cloudflare worker/);
  assert.doesNotMatch(publisher,/cfvigil-dist|pool\/main\/|dists\/stable/);
});

test("release artifact handoff paths match the verified Linux and Windows staging directories", () => {
  // GitHub interpolates double-braced expressions; any trailing brace is a
  // literal path character and silently breaks artifact upload/download.
  assert.doesNotMatch(release,/\$\{\{[^}\n]+\}\}\}/);
  const handoffPaths = [...release.matchAll(/^\s*path: (\$\{\{ runner\.temp \}\}\/release\/(?:linux|windows)(?:\/\*)?)\s*$/gm)]
    .map(match => match[1]).sort();
  assert.deepEqual(handoffPaths, [
    "${{ runner.temp }}/release/linux",
    "${{ runner.temp }}/release/linux/*",
    "${{ runner.temp }}/release/windows",
    "${{ runner.temp }}/release/windows/*"
  ]);
  assert.equal(release.split("\n").filter(line => /^\s*path:.*runner\.temp/.test(line)).length, 4);
  assert.match(release,/--target "\$RUNNER_TEMP\/release\/linux"/);
  assert.match(release,/--target "\$folder"/);
});
