import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {planSettingsApply} from "../apps/desktop/ui/settings-transaction.mjs";
import {ACTIVE_LOCALES, t} from "../apps/desktop/ui/localization.mjs";

const read = p => readFileSync(new URL("../" + p, import.meta.url), "utf8");

const previous = {
  config:{account_id:"a".repeat(32),refresh_seconds:180},
  language:"ru",
  autostart:false
};
const unchanged = {
  accountId:"a".repeat(32),
  refreshSeconds:180,
  token:"",
  language:"ru",
  autostart:false
};

test("Apply unchanged values is a genuine no-op with no keyring, autostart or Cloudflare side effects",()=>{
  const plan=planSettingsApply(previous,unchanged);
  assert.equal(plan.changed,false);
  assert.equal(plan.args.token,null);
  assert.equal(plan.args.autostart,null);
});

test("Locale-only Apply does not rewrite any other setting or mutate OS auth",()=>{
  const plan=planSettingsApply(previous,{...unchanged,language:"pt"});
  assert.equal(plan.changed,true);
  assert.deepEqual(plan.args,{
    accountId:"a".repeat(32),refreshSeconds:180,token:null,
    language:"pt",autostart:null
  });
  const native=read("apps/desktop/src-tauri/src/main.rs");
  assert.match(native,/previous\.account_id != account_id/);
  assert.match(native,/fn needs_refresh\(/);
  assert.match(native,/fn commit_language\(/);
  assert.match(native,/if current\.locale == language \{ return Ok\(\(\)\) \}/);
});

test("Autostart is applied only when the submitted draft changed",()=>{
  assert.equal(planSettingsApply(previous,{...unchanged,autostart:true}).args.autostart,true);
  assert.equal(planSettingsApply(previous,{...unchanged,autostart:false}).args.autostart,null);
  assert.equal(planSettingsApply({...previous,autostart:null},
    {...unchanged,autostart:true}).args.autostart,null);
});

test("Only changed token causes a new replacement request",()=>{
  assert.equal(planSettingsApply(previous,{...unchanged,token:"  replaced-token  "}).args.token,
    "replaced-token");
  assert.equal(planSettingsApply(previous,{...unchanged,token:"   "}).changed,false);
});

test("Successful Settings Apply closes and Cancel discards drafts on both platforms",()=>{
  const settings=read("apps/desktop/ui/settings.mjs");
  assert.match(settings,/if \(changed\) await invoke\("save_settings", args\)/);
  assert.match(settings,/if \(saved\)[\s\S]*?getCurrentWindow\(\)\.hide\(\)/);
  assert.match(settings,/function restoreDraft\(/);
  assert.match(settings,/document\.getElementById\("cancel"\)\.onclick/);
  assert.match(settings,/if \(currentLocale !== nextLanguage\)/);
  assert.match(settings,/localizeDocument\(document\)/);
});

test("Read-only startup roster restores grey account-scoped resource keys",()=>{
  const native=read("apps/desktop/src-tauri/src/main.rs");
  const cache=read("apps/desktop/src-tauri/src/resource_cache.rs");
  assert.match(native,/catalog\.value\.show_stale\(&settings\.value\.account_id\)/);
  assert.match(native,/read_json::<ResourceCatalogCache>/);
  assert.match(native,/write_json\(&app, "resource-catalog\.json"/);
  assert.match(cache,/status: "unknown"\.into\(\)/);
  assert.match(cache,/account_id != account/);
  assert.match(cache,/format!\("https:\/\/dash\.cloudflare\.com\/\{account\}\/\{route\}"\)/);
  assert.doesNotMatch(cache,/get_password|bearer_auth|status: "healthy"/);
});

test("Cloudflare bounded fanout never adds a second poller or read-write mutation",()=>{
  const api=read("apps/desktop/src-tauri/src/cloudflare.rs");
  assert.match(api,/tokio::join!\(\s*self\.list_workers/);
  assert.match(api,/script_list\.chunks\(4\)/);
  assert.match(api,/projects\.chunks\(4\)/);
  assert.match(api,/self\.get\(token/);
  assert.doesNotMatch(api,/\.post\(|\.put\(|\.delete\(/);
  const native=read("apps/desktop/src-tauri/src/main.rs");
  assert.equal((native.match(/fn run_polling\(/g)||[]).length,1);
});

test("Native tray locale switches explicitly refresh KDE KSNI and Windows Tauri",()=>{
  const tray=read("apps/desktop/src-tauri/src/linux_tray.rs");
  assert.match(tray,/pub fn set_language\(/);
  assert.match(tray,/handler\.update\(move \|tray\|/);
  const native=read("apps/desktop/src-tauri/src/main.rs");
  assert.match(native,/linux_tray::set_language\(app, locale\)/);
  assert.match(native,/tray\.set_menu\(Some\(menu\)\)/);
});

test("Resource scrollbar owns pointer interactions and never overlays action buttons",()=>{
  const app=read("apps/desktop/ui/app.mjs");
  const css=read("apps/desktop/ui/style.css");
  assert.match(app,/event\.target\.closest\("#drawer,#search-row/);
  assert.match(css,/scrollbar-gutter:stable/);
  assert.match(css,/padding-inline-end:7px/);
  assert.match(css,/\.rows::-webkit-scrollbar-thumb:active/);
  assert.match(css,/#drawer,#search-row\{cursor:default\}/);
});

test("About mirrors RepoTicker layout, uses native version, safe fixed links and localized labels",()=>{
  const markup=read("apps/desktop/ui/about.html");
  const source=read("apps/desktop/ui/about.mjs");
  const native=read("apps/desktop/src-tauri/src/main.rs");
  assert.match(markup,/class="about"/);
  assert.match(markup,/src="\.\/icon\.png"/);
  for(const key of ["about.name","about.copyright","about.license","about.source"])
    assert.ok(markup.includes('data-i18n="'+key+'"'),key);
  assert.ok(!markup.includes('data-i18n="about.description"'),
    "owner removed the About description; retain only visible content");
  assert.match(source,/invoke\("get_app_version"\)/);
  assert.match(source,/initializeLanguage/);
  assert.match(native,/fn open_about_website\(/);
  assert.match(native,/fn open_about_source\(/);
  assert.match(native,/native\.set_icon\(Some\(&pixbuf\)\)/);
  for (const locale of ACTIVE_LOCALES) {
    for(const key of ["about.title","about.description","about.license","about.copyright","about.source"])
      assert.ok(t(key,{},locale).trim().length>0,`${locale}/${key}`);
  }
});

test("configured first poll announces pending instead of disconnected",()=>{
  const app=read("apps/desktop/ui/app.mjs");
  const native=read("apps/desktop/src-tauri/src/main.rs");
  assert.match(app,/syncPending = snapshot.pending/);
  assert.match(app,/emptyResourceMessageKey/);
  assert.match(native,/pending: state.pending.load/);
  assert.match(native,/state.pending.store\(!next.account_id.is_empty\(\), Ordering::Release\)/);
});

test("card reopening reads native cached state without an extra provider poll",()=>{
  const app=read("apps/desktop/ui/app.mjs");
  assert.match(app,/cfvigil-card-shown/);
  assert.match(app,/void refreshFromNative\(\)/);
  assert.match(app,/snapshotEventSerial \+= 1/);
  assert.match(app,/if \(snapshotEventSerial !== eventSerial\) return/);
  assert.match(app,/invoke\("get_snapshot"\)/);
});
