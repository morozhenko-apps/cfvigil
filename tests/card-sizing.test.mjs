import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {expandedCardHeight,measureExpandedCardHeight} from "../apps/desktop/ui/card-sizing.mjs";
const read=p=>readFileSync(new URL("../"+p,import.meta.url),"utf8");

test("empty expanded card measures only message, search and actions",()=>{
  const height=expandedCardHeight({rowsHeight:40,searchHeight:43,actionsHeight:70});
  assert.equal(height,190);
  assert.ok(height<250,"no empty 510px list");
});

test("card grows for actual resources and clips excessive lists",()=>{
  const empty=expandedCardHeight({rowsHeight:38,searchHeight:42,actionsHeight:70});
  const two=expandedCardHeight({rowsHeight:2*36,searchHeight:42,actionsHeight:70});
  const warning=expandedCardHeight({rowsHeight:2*36,warningHeight:48,searchHeight:42,actionsHeight:70});
  assert.ok(two>empty);
  assert.equal(warning-two,48);
  assert.equal(expandedCardHeight({rowsHeight:40*36,searchHeight:42,actionsHeight:70}),510);
  assert.throws(()=>expandedCardHeight({rowsHeight:NaN,searchHeight:42,actionsHeight:70}),/Invalid/);
});

test("DOM-based size uses child row heights, not stretched viewport height",()=>{
  const rect=h=>({getBoundingClientRect:()=>({height:h})});
  const entries={
    card:{querySelector:()=>rect(70)},
    "resource-list":{children:[rect(36),rect(36)]},
    "provider-warning":{hidden:true,getBoundingClientRect:()=>({height:100})},
    "search-row":rect(42),
    drawer:{}
  };
  const doc={getElementById:id=>entries[id],
    defaultView:{getComputedStyle:el=>el===entries.card?{
      paddingTop:"10px",paddingBottom:"10px",borderTopWidth:"1px",borderBottomWidth:"1px",rowGap:"7px"
    }:{borderBottomWidth:"1px"}}};
  const got=measureExpandedCardHeight(doc);
  assert.equal(got,221);
});

test("native consumes measured height while preserving an anchored compact fallback",()=>{
  const main=read("apps/desktop/src-tauri/src/main.rs");
  const app=read("apps/desktop/ui/app.mjs");
  assert.match(main,/set_card_expanded\(app: AppHandle, expanded: bool, height: Option<f64>\)/);
  assert.match(main,/!requested\.is_finite\(\)/);
  assert.match(main,/requested\.clamp\(COMPACT_HEIGHT \+ 50\.0, EXPANDED_HEIGHT\)/);
  assert.match(main,/windows_card::resize_anchored\(&app, &window, height\)/);
  assert.match(app,/measureExpandedCardHeight\(document\)/);
  assert.match(app,/lastNativeHeight = height/);
  assert.match(app,/scheduleCardFit\(\);/);
  assert.match(app,/byId\("search"\)\.oninput = render/);
});

test("browser context menu is suppressed across all native webviews",()=>{
  const guard=read("apps/desktop/ui/suppress-context-menu.mjs");
  assert.match(guard,/addEventListener\("contextmenu",event=>event\.preventDefault\(\),\{capture:true\}\)/);
  for(const p of ["app.mjs","settings.mjs","history.mjs","about.mjs"]){
    assert.match(read("apps/desktop/ui/"+p),/import "\.\/suppress-context-menu\.mjs";/);
  }
  // In-app resource options stay available via their own accessible button.
  assert.match(read("apps/desktop/ui/app.mjs"),/openResourceMenu\(resource, button\)/);
});

test("settings read failures retry on reopen while avoiding fallback overwrite",()=>{
  const s=read("apps/desktop/ui/settings.mjs");
  assert.match(s,/settingsReady = false/);
  assert.match(s,/submit\.disabled = saving \|\| !ready/);
  assert.match(s,/if \(!settingsReady\) void loadStoredSettings\(\)/);
  assert.match(s,/settingsLoading \|\| settingsReady/);
  assert.match(s,/const config = await Promise\.race\(/);
  assert.match(s,/8_000/);
  assert.match(s,/setSettingsReady\(false\)/);
  assert.match(s,/setSettingsReady\(true\)/);
  assert.match(s,/if \(saving\) return/);
  assert.match(s,/planSettingsApply/);
  assert.doesNotMatch(s,/storedSettings = \{account_id:"", refresh_seconds:180\}/);
});
