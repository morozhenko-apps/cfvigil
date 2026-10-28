import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {emptyResourceMessageKey} from "../apps/desktop/ui/connection-state.mjs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";

test("unconfigured account is not shown as connecting",()=>{
  assert.equal(emptyResourceMessageKey({configured:false,pending:false,ready:false}),"card.noAccount");
});
test("first provider request shows pending, not disconnected",()=>{
  const snapshot=normalizeSnapshot({configured:true,ready:false,pending:true,resources:[]});
  assert.equal(snapshot.pending,true);
  assert.equal(emptyResourceMessageKey(snapshot),"diagnostic.pending");
});
test("completed empty or failed reads differ from pending",()=>{
  assert.equal(emptyResourceMessageKey({configured:true,pending:false,ready:true}),"card.noResources");
  assert.equal(emptyResourceMessageKey({configured:true,pending:false,ready:false}),"card.noData");
  assert.equal(normalizeSnapshot({configured:true,ready:false}).pending,false);
});

test("pending indicator uses a centered vector arc without font-dependent rotation",()=>{
  const read=path=>readFileSync(new URL("../"+path,import.meta.url),"utf8");
  const html=read("apps/desktop/ui/index.html");
  const css=read("apps/desktop/ui/style.css");
  const app=read("apps/desktop/ui/app.mjs");
  assert.match(html,/<span class="glyph glyph-sync" aria-hidden="true">/);
  assert.match(html,/<span class="glyph-resources">▤<\/span>/);
  assert.match(html,/<svg class="glyph-spinner" viewBox="0 0 24 24" focusable="false">/);
  assert.match(html,/<path d="M21 12a9 9 0 1 1-6\.219-8\.56"\/>/);
  assert.match(css,/\.glyph-sync\{display:grid;place-items:center;width:26px;height:26px/);
  assert.match(css,/\.glyph-spinner\{display:none;[\s\S]*?transform-box:view-box;transform-origin:50% 50%/);
  assert.match(css,/\.action\.syncing \.glyph-spinner\{display:block;animation:cfvigil-poll/);
  assert.match(css,/@media\(prefers-reduced-motion:reduce\)\{\.action\.syncing \.glyph-spinner\{animation:none\}\}/);
  assert.match(app,/button\.classList\.toggle\("syncing", syncPending\)/);
  assert.doesNotMatch(app,/\.textContent = syncPending \? "⟳"/);
});
