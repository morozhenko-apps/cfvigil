import test from "node:test";
import assert from "node:assert/strict";
import {planSettingsApply} from "../apps/desktop/ui/settings-transaction.mjs";

const OLD = Object.freeze({
  config:Object.freeze({account_id:"a".repeat(32),refresh_seconds:180}),
  language:"ru",autostart:false
});
const DRAFT = Object.freeze({
  accountId:"a".repeat(32),refreshSeconds:180,token:"",
  language:"ru",autostart:false
});
const submit = (changes={}, prior=OLD) =>
  planSettingsApply(prior,{...DRAFT,...changes});

test("Given all unchanged Settings When calculating Apply Then no native write or OS side effect is requested",()=>{
  const before=JSON.stringify(OLD);
  const draftBefore=JSON.stringify(DRAFT);
  assert.deepEqual(submit(),{
    changed:false,
    args:{accountId:"a".repeat(32),refreshSeconds:180,token:null,language:"ru",autostart:null}
  });
  assert.equal(JSON.stringify(OLD),before);
  assert.equal(JSON.stringify(DRAFT),draftBefore);
});

test("Given each independent Settings field When it changes Then only that value is committed",()=>{
  const cases=[
    ["account", {accountId:"b".repeat(32)}, {accountId:"b".repeat(32)}],
    ["refresh", {refreshSeconds:"300"}, {refreshSeconds:300}],
    ["locale", {language:"pt"}, {language:"pt"}],
    ["token", {token:"real-replacement-token"}, {token:"real-replacement-token"}],
    ["autostart on", {autostart:true}, {autostart:true}],
  ];
  const baseline=submit().args;
  for(const [name,draft,delta] of cases){
    const result=submit(draft);
    assert.equal(result.changed,true,name);
    assert.deepEqual(result.args,{...baseline,...delta},name);
  }
});

test("Given no new token When applying locale/interval/account-neutral changes Then keyring is never re-written",()=>{
  for (const token of ["", " ", "\n", "\t  "]){
    for(const change of [{language:"ar"},{refreshSeconds:600},{language:"ja",refreshSeconds:60}]){
      const result=submit({...change,token});
      assert.equal(result.changed,true);
      assert.equal(result.args.token,null);
      assert.equal(result.args.autostart,null);
    }
  }
});

test("Given token draft with surrounding whitespace When applying Then normalization is consistent with native token input",()=>{
  const output=submit({token:"  valid-token-1234567890  "});
  assert.equal(output.args.token,"valid-token-1234567890");
  assert.equal(output.changed,true);
});

test("Given unchanged token as whitespace When pressing OK Then no credential read or poll is requested",()=>{
  for (const token of ["", "  ", "\n\t"]){
    assert.deepEqual(submit({token}),submit(),JSON.stringify(token));
  }
});

test("Given autostart true or false When unchanged Then native OS is not asked to reapply it",()=>{
  const on={...OLD,autostart:true};
  assert.equal(submit({autostart:true},on).args.autostart,null);
  assert.equal(submit({autostart:false},OLD).args.autostart,null);
  assert.equal(submit({autostart:false},on).args.autostart,false);
  assert.equal(submit({autostart:true},OLD).args.autostart,true);
});

test("Given unavailable autostart state When applying another field Then registration is left untouched",()=>{
  for(const unavailable of [null,undefined]){
    for(const wanted of [false,true]){
      const prior={...OLD,autostart:unavailable};
      const result=submit({autostart:wanted,language:"nl"},prior);
      assert.equal(result.changed,true);
      assert.equal(result.args.autostart,null);
      assert.equal(result.args.token,null);
    }
  }
});

test("Given account text with leading whitespace When applying Then canonical identity remains unchanged",()=>{
  const result=submit({accountId:"  "+"a".repeat(32)+"  "});
  assert.equal(result.changed,false);
  assert.equal(result.args.accountId,"a".repeat(32));
});

test("Given refresh interval as select string When Apply runs Then equal numeric interval is a no-op",()=>{
  for(const value of [180,"180",180.0]){
    assert.equal(submit({refreshSeconds:value}).changed,false);
  }
});

test("Given invalid interval draft When calculating Apply Then invalid input cannot be silently treated as unchanged",()=>{
  for(const value of ["not-a-number","Infinity",NaN,Infinity,-1,0,601,null,undefined]){
    const result=submit({refreshSeconds:value});
    assert.equal(result.changed,true,String(value));
    assert.equal(result.args.token,null,String(value));
    assert.equal(result.args.autostart,null,String(value));
  }
});

test("Given previous config unavailable When first settings load has not completed Then defaults are not silently interpreted as prior user settings",()=>{
  const fallback=planSettingsApply({language:"en",autostart:null},{
    accountId:"",refreshSeconds:180,token:"",language:"en",autostart:false
  });
  assert.equal(fallback.changed,false);
  assert.equal(fallback.args.autostart,null);
  // UI separately disables OK until the actual persisted config loads.
});

test("Given two independent locale edits When computed Then the prior native state stays immutable",()=>{
  const original=structuredClone(OLD);
  const french=submit({language:"fr"});
  const korean=submit({language:"ko"});
  assert.equal(french.args.language,"fr");
  assert.equal(korean.args.language,"ko");
  assert.equal(OLD.language,"ru");
  assert.deepEqual(OLD,original);
});

test("Given a full multi-field change When planned Then it is one coherent settings commit",()=>{
  const result=submit({
    accountId:"b".repeat(32), refreshSeconds:"600", token:"replacement-token",
    language:"pt",autostart:true
  });
  assert.deepEqual(result,{
    changed:true,args:{
      accountId:"b".repeat(32),refreshSeconds:600,token:"replacement-token",
      language:"pt",autostart:true
    }
  });
});

test("Given disabled autostart toggle and locale-only change When planned Then neither token nor autostart state is reset",()=>{
  const prior={...OLD,autostart:null};
  const result=submit({language:"de",autostart:false},prior);
  assert.equal(result.args.language,"de");
  assert.equal(result.args.autostart,null);
  assert.equal(result.args.token,null);
});
