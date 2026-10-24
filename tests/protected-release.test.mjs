import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const read=p=>readFileSync(new URL("../"+p,import.meta.url),"utf8").replace(/\r\n/g,"\n");
const release=read(".github/workflows/protected-release.yml");
const worker=read("infra/download-worker/wrangler.jsonc");
const publisher=read("scripts/r2_release.py");

test("no dev push or workflow dispatch can publish CFVigil",()=>{
  assert.match(release,/^on:\n  push:\n    tags: \['v\*'\]/m);
  assert.match(release,/git fetch --no-tags origin main/);
  assert.match(release,/git rev-parse origin\/main\^\{commit\}/);
  assert.match(release,/scripts\/candidate-version\.mjs --check-tag/);
  assert.doesNotMatch(release,/workflow_dispatch:|branches: \[dev\]/);
  assert.match(release,/APPROVED_TAG.*CFVIGIL_RELEASE_APPROVED_TAG/);
  assert.match(release,/environment: cfvigil-release/);
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
  assert.match(publisher,/BUCKET = "cfvigil-dist"/);
  assert.match(publisher,/base_url/);
  assert.match(publisher,/public_readback/);
  assert.match(publisher,/apt\/dists\/stable\/InRelease/);
});

test("dedicated download Worker is read only and not a RepoTicker mirror",()=>{
  const config=JSON.parse(worker);
  assert.equal(config.name,"cfvigil-download");
  assert.equal(config.workers_dev,false);
  assert.equal(config.preview_urls,false);
  assert.equal(config.r2_buckets[0].bucket_name,"cfvigil-dist");
  assert.equal(config.routes[0].pattern,"https://download.morozhenko.com/dist/CFVigil/*");
  assert.doesNotMatch(worker,/dist\/RepoTicker|repoticker-download/);
  const source=read("infra/download-worker/src/worker.mjs");
  assert.doesNotMatch(source,/\.put\(|\.delete\(|\.list\(/);
  assert.match(source,/request\.method !== "GET" && request\.method !== "HEAD"/);
});
