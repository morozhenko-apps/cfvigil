import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {ACTIVE_LOCALES,t} from "../apps/desktop/ui/localization.mjs";

const read = file => readFileSync(new URL("../"+file, import.meta.url),"utf8");
const html=read("apps/desktop/ui/about.html");
const js=read("apps/desktop/ui/about.mjs");
const css=read("apps/desktop/ui/about.css");
const native=read("apps/desktop/src-tauri/src/main.rs");
const config=JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
const win=label=>config.app.windows.find(window=>window.label===label);

test("Given About window When opened on Linux or Windows Then native Minimize is absent",()=>{
  assert.equal(win("about").decorations,false);
  assert.equal(win("about").minimizable,false);
  assert.equal(win("about").maximizable,false);
  assert.equal(win("about").resizable,false);
  assert.equal(win("about").visible,false);
});

test("Given frameless About When user clicks Close Then existing window is hidden, not destroyed",()=>{
  assert.match(html,/<button id="about-close"[^>]+type="button"/);
  assert.match(html,/data-i18n-aria-label="about\.close"/);
  assert.match(html,/data-i18n-title="about\.close"/);
  assert.match(js,/getElementById\("about-close"\)\.addEventListener\("click", closeAbout\)/);
  assert.match(js,/getCurrentWindow\(\)\.hide\(\)/);
  assert.doesNotMatch(js,/getCurrentWindow\(\)\.destroy\(/);
  assert.match(css,/\.about-close:focus-visible/);
});

test("Given frameless About When keyboard user presses Escape Then the same Close action runs",()=>{
  assert.match(js,/event\.key === "Escape"/);
  assert.match(js,/event\.preventDefault\(\); closeAbout\(\)/);
});

test("Given About design When rendered Then original application description is not shown",()=>{
  assert.doesNotMatch(html,/data-i18n="about\.description"/);
  assert.match(html,/<h1 data-i18n="about\.name">CFVigil<\/h1>/);
  assert.match(html,/id="about-version"/);
  assert.match(html,/data-i18n="about\.copyright"/);
  assert.match(html,/data-i18n="about\.license"/);
  assert.match(html,/id="about-source"/);
  assert.match(html,/src="\.\/icon\.png"/);
});

test("Given twenty locales When localizing About Close Then there is no English fallback or missing label",()=>{
  const expected = {
    en:"Close",ru:"Закрыть",es:"Cerrar",pt:"Fechar",de:"Schließen",
    fr:"Fermer",it:"Chiudi",pl:"Zamknij",hi:"बंद करें","zh-Hant":"關閉",
    ja:"閉じる",nl:"Sluiten",sv:"Stäng",tr:"Kapat",ko:"닫기",
    bn:"বন্ধ করুন",th:"ปิด",id:"Tutup",fil:"Isara",ar:"إغلاق"
  };
  assert.deepEqual(Object.keys(expected),ACTIVE_LOCALES);
  for(const [locale,label] of Object.entries(expected))
    assert.equal(t("about.close",{},locale),label,locale);
});

test("Given installed DEB theme icon When history opens Then GTK requests matching icon name",()=>{
  const installedIcon = "cfvigil-desktop";
  const occurrences=native.match(/native\.set_icon_name\(Some\("cfvigil-desktop"\)\)/g) ?? [];
  assert.equal(occurrences.length,2,"startup icon and show-time icon");
  assert.match(native,/for label in \["main", "settings", "history", "about"\]/);
  assert.match(native,/window\.gtk_window\(\)/);
  assert.equal(installedIcon,"cfvigil-desktop");
});

test("Given Windows and Linux History When opening Then native icon is asserted and window remains decorated",()=>{
  assert.equal(win("history").decorations,undefined);
  assert.equal(win("history").resizable,true);
  assert.match(native,/if let Some\(icon\) = app\.default_window_icon\(\)\.cloned\(\)/);
  assert.match(native,/window\.set_icon\(icon\)/);
  assert.match(native,/window\.show\(\)/);
});

test("Given About project buttons When clicked Then only fixed destinations are allowed",()=>{
  assert.match(js,/invoke\("open_about_website"\)/);
  assert.match(js,/invoke\("open_about_source"\)/);
  assert.match(native,/open::that\("https:\/\/morozhenko\.com\/"\)/);
  assert.match(native,/open::that\("https:\/\/github\.com\/morozhenko-apps\/cfvigil"\)/);
});
