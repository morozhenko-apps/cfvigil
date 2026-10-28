import test from "node:test";
import assert from "node:assert/strict";
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
