import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {
  ACTIVE_LOCALES, currentLocale, setLocale, validateCatalog, t,
  intlLocale, plural, incidentText, resourceDetail
} from "../apps/desktop/ui/localization.mjs";
import {initializeLanguage} from "../apps/desktop/ui/runtime-language.mjs";
import english from "../apps/desktop/ui/locales/en.mjs";
import russian from "../apps/desktop/ui/locales/ru.mjs";
import portuguese from "../apps/desktop/ui/locales/pt.mjs";
import spanish from "../apps/desktop/ui/locales/es.mjs";
import german from "../apps/desktop/ui/locales/de.mjs";
import french from "../apps/desktop/ui/locales/fr.mjs";
import italian from "../apps/desktop/ui/locales/it.mjs";
import polish from "../apps/desktop/ui/locales/pl.mjs";
import dutch from "../apps/desktop/ui/locales/nl.mjs";
import swedish from "../apps/desktop/ui/locales/sv.mjs";

const read = path => readFileSync(new URL("../" + path, import.meta.url), "utf8");

test("only complete and explicitly reviewed-for-key-parity dictionaries can be enabled", () => {
  assert.equal(ACTIVE_LOCALES.length, 20);
  assert.deepEqual(validateCatalog(russian, english, "ru"), []);
  assert.deepEqual(validateCatalog(portuguese, english, "pt"), []);
  assert.deepEqual(validateCatalog(spanish, english, "es"), []);
  assert.deepEqual(validateCatalog(german, english, "de"), []);
  assert.deepEqual(validateCatalog(french, english, "fr"), []);
  assert.deepEqual(validateCatalog(italian, english, "it"), []);
  assert.deepEqual(validateCatalog(polish, english, "pl"), []);
  assert.deepEqual(validateCatalog(dutch, english, "nl"), []);
  assert.deepEqual(validateCatalog(swedish, english, "sv"), []);
  // European Portuguese uses CLDR "many" for multiples of one million.
  assert.ok(new Intl.PluralRules("pt-PT").resolvedOptions().pluralCategories.includes("many"));
  for (const key of ["age.minute", "age.hour", "age.day", "history.eventCount"]) {
    assert.equal(typeof portuguese[key].many, "string", key);
  }
  assert.equal(plural("history.eventCount", 1000000,
    {shown:2,total:1000000}, "pt"), "2 de 1000000 eventos");
  assert.equal(intlLocale("pt"), "pt-PT");
  assert.equal(t("card.issues", {}, "ru"), "Проблемы");
  assert.equal(t("card.resources", {}, "pt"), "Recursos");
  assert.equal(plural("history.eventCount", 22, {shown:5,total:22}, "ru"),
    "Показано 5 из 22 событий");
  assert.equal(incidentText("worker-name: WAF block spike detected", "ru"),
    "worker-name: Обнаружен рост блокировок WAF");
  assert.match(resourceDetail("Workers errors spiking: 25 errors / 100 requests (25.0%)", "pt"),
    /25 erros em 100 pedidos/);
});

test("language bootstrap listens before getting native preference and follows changes", async () => {
  let eventListener, activated=0, invoked=false;
  const doc = {
    documentElement:{lang:"",dir:""},
    querySelectorAll:()=>[]
  };
  const events = [];
  const listen = async (name,fn) => {
    assert.equal(name, "cfvigil-language-changed");
    eventListener=fn;
    events.push("listen");
  };
  const invoke = async name => {
    assert.equal(name, "get_language");
    invoked=true;
    events.push("invoke");
    return "ru";
  };
  try {
    await initializeLanguage({invoke,listen,doc,onChange:()=>{activated++;}});
    assert.deepEqual(events,["listen","invoke"]);
    assert.equal(currentLocale,"ru");
    assert.equal(doc.documentElement.lang,"ru");
    eventListener({payload:"pt"});
    assert.equal(currentLocale,"pt");
    assert.equal(doc.documentElement.lang,"pt-PT");
    assert.equal(activated,2);
    assert.equal(invoked,true);
    assert.throws(()=>eventListener({payload:"xx"}),/Unsupported or unreviewed/);
  } finally {setLocale("en");}
});

test("independent native language storage cannot wake Cloudflare polling", () => {
  const main=read("apps/desktop/src-tauri/src/main.rs");
  const native=read("apps/desktop/src-tauri/src/language.rs");
  assert.match(main,/read_json::<LanguagePreference>\(&handle, "language.json"\)/);
  assert.match(main,/language: Mutex<LanguagePreference>/);
  assert.match(main,/fn get_language\(/);
  assert.match(main,/fn set_language\(/);
  assert.match(main,/write_json\(app, "language.json", &next\)/);
  assert.match(main,/app.emit\("cfvigil-language-changed", language\)/);
  const start=main.indexOf("fn set_language(");
  const end=main.indexOf("fn update_native_language(",start);
  const setter=main.slice(start,end);
  assert.doesNotMatch(setter,/poll_refresh|refresh_once|invalidate_resources|last_notified/);
  assert.match(native,/pub const ACTIVE: &\[&str\] = &\[/);
  assert.match(native,/fn unknown_languages_fail_closed\(/);
  assert.match(read("apps/desktop/src-tauri/src/linux_tray.rs"),/language::menu\(&locale, "open"\)/);
  assert.match(main,/tray\.set_menu\(Some\(menu\)\)/);
  const settings=read("apps/desktop/ui/settings.mjs");
  assert.ok(!settings.includes('invoke("set_language"'));
  assert.match(settings, /language: language.value/);
  assert.ok(settings.includes('invoke("save_settings"'));
  assert.match(settings,/language\.value = confirmedLanguage/);
});

test("all compiled language options are valid and HTML remains free of injected names", () => {
  const settings=read("apps/desktop/ui/settings.html");
  assert.match(settings,/id="language" disabled/);
  assert.match(settings,/data-i18n="settings.language"/);
  assert.doesNotMatch(read("apps/desktop/ui/app.mjs"),/innerHTML/);
  assert.doesNotMatch(read("apps/desktop/ui/history.mjs"),/innerHTML/);
});

test("newer native change wins if initial language IPC responds out of order", async () => {
  const documentStub = {documentElement:{lang:"",dir:""},querySelectorAll:()=>[]};
  let receive, resolveInitial, applyCount = 0;
  const deferred = new Promise(resolve => {resolveInitial = resolve;});
  const pending = initializeLanguage({
    invoke: async () => deferred,
    listen: async (_, callback) => {receive=callback;},
    doc:documentStub,
    onChange:()=>{applyCount++;}
  });
  try {
    await Promise.resolve();
    receive({payload:"pt"});
    resolveInitial("ru"); // stale snapshot must NOT override pt
    await pending;
    assert.equal(currentLocale, "pt");
    assert.equal(documentStub.documentElement.lang, "pt-PT");
    assert.equal(applyCount, 1);
  } finally {setLocale("en");}
});

test("failed language bootstrap preserves a newer native event", async () => {
  const documentStub = {documentElement:{lang:"",dir:""},querySelectorAll:()=>[]};
  let receive, rejectInitial;
  const deferred = new Promise((_, reject) => {rejectInitial=reject;});
  const pending = initializeLanguage({
    invoke: async () => deferred,
    listen: async (_, callback)=>{receive=callback;},
    doc:documentStub
  });
  try {
    await Promise.resolve();
    receive({payload:"ru"});
    rejectInitial(new Error("IPC unavailable"));
    await pending;
    assert.equal(currentLocale, "ru");
    assert.equal(documentStub.documentElement.lang, "ru");
  } finally {setLocale("en");}
});

test("accessibility status names follow locale while status IDs remain stable", () => {
  const card = read("apps/desktop/ui/app.mjs");
  assert.match(card, /t\(statusLabelKey\[resource\.status\]/);
  assert.equal(t("card.statusUnknown", {}, "ru"), "Не проверено");
  assert.equal(t("card.statusUnknown", {}, "pt"), "Por verificar");
});

test("Spanish, German and French use correct CLDR categories and native menu labels", () => {
  const native = read("apps/desktop/src-tauri/src/language.rs");
  const languages = {es: spanish, de: german, fr: french};
  for (const [locale, dictionary] of Object.entries(languages)) {
    const expected = new Intl.PluralRules(locale).resolvedOptions().pluralCategories;
    for (const key of ["age.minute", "age.hour", "age.day", "history.eventCount"]) {
      assert.deepEqual(Object.keys(dictionary[key]).sort(), [...expected].sort(),
        `${locale}:${key}`);
    }
  }
  assert.equal(t("card.issues", {}, "es"), "Problemas");
  assert.equal(t("card.resources", {}, "de"), "Ressourcen");
  assert.equal(t("card.resources", {}, "fr"), "Ressources");
  assert.match(native, /\("es", "settings"\) => "Configuración"/);
  assert.match(native, /\("de", "settings"\) => "Einstellungen"/);
  assert.match(native, /\("fr", "settings"\) => "Paramètres"/);
  assert.equal(plural("history.eventCount", 1000000, {shown:2,total:1000000}, "fr"),
    "2 sur 1000000 événements");
});

test("newly enabled EU locales preserve every key and CLDR category", () => {
  const dictionaries = {it: italian, pl: polish, nl: dutch, sv: swedish};
  const native = read("apps/desktop/src-tauri/src/language.rs");
  for (const [locale, dictionary] of Object.entries(dictionaries)) {
    assert.deepEqual(validateCatalog(dictionary, english, locale), [], locale);
    const required = [...new Intl.PluralRules(locale).resolvedOptions().pluralCategories].sort();
    for (const key of ["age.minute", "age.hour", "age.day", "history.eventCount"]) {
      assert.deepEqual(Object.keys(dictionary[key]).sort(), required, locale + ":" + key);
    }
    assert.ok(native.includes('("'+locale+'", "settings")'), locale);
  }
  assert.equal(t("card.issues", {}, "it"), "Problemi");
  assert.equal(t("card.issues", {}, "pl"), "Problemy");
  assert.equal(t("card.resources", {}, "nl"), "Bronnen");
  assert.equal(t("card.resources", {}, "sv"), "Resurser");
  assert.equal(plural("history.eventCount", 22, {shown:3,total:22}, "pl"), "Wyświetlono 3 z 22 zdarzenia");
  assert.equal(plural("history.eventCount", 5, {shown:3,total:5}, "pl"), "Wyświetlono 3 z 5 zdarzeń");
  assert.equal(plural("history.eventCount", 1000000, {shown:1,total:1000000}, "it"), "1 di 1000000 eventi");
});

test("enabled locales and native supported locales remain in exact sync", () => {
  const native=read("apps/desktop/src-tauri/src/language.rs");
  const line=native.split("\n").find(text => text.startsWith("pub const ACTIVE:"));
  assert.ok(line);
  const actual=[...line.matchAll(/"([^"]+)"/g)].map(x=>x[1]);
  assert.deepEqual(actual, ACTIVE_LOCALES);
});


test("all twenty dictionaries match baseline keys, placeholders and CLDR forms", async () => {
  const languages = ["en","ru","es","pt","de","fr","it","pl","hi","zh-Hant","ja","nl","sv","tr","ko","bn","th","id","fil","ar"];
  assert.deepEqual(ACTIVE_LOCALES, languages);
  for (const locale of languages) {
    const mod = await import("../apps/desktop/ui/locales/" + locale + ".mjs");
    const dictionary = mod.default;
    assert.deepEqual(validateCatalog(dictionary, english, locale), [], locale);
    assert.equal(typeof dictionary["card.healthy"], "string", locale);
    assert.equal(typeof dictionary["event.buildFailure"], "string", locale);
    assert.notEqual(dictionary["card.healthy"].length, 0, locale);
  }
  assert.equal(new Intl.PluralRules("ar").resolvedOptions().pluralCategories.length, 6);
  assert.equal(plural("age.minute", 0, {}, "ar").includes("دقيقة"), true);
  assert.equal(t("card.resources", {}, "zh-Hant"), "資源");
  assert.equal(t("card.resources", {}, "ja"), "リソース");
  assert.equal(t("card.resources", {}, "ko"), "리소스");
  assert.equal(t("card.resources", {}, "hi"), "संसाधन");
  assert.equal(t("card.resources", {}, "bn"), "রিসোর্স");
  assert.equal(t("card.resources", {}, "th"), "ทรัพยากร");
  assert.equal(t("card.resources", {}, "id"), "Sumber daya");
  assert.equal(t("card.resources", {}, "tr"), "Kaynaklar");
  assert.equal(t("card.resources", {}, "fil"), "Mga resource");
});

test("Arabic layout applies RTL while resource identifiers are not translated", async () => {
  const doc = {documentElement:{lang:"en",dir:"ltr"},querySelectorAll:()=>[]};
  const original=currentLocale;
  try {
    setLocale("ar");
    const {localizeDocument}=await import("../apps/desktop/ui/localization.mjs");
    localizeDocument(doc);
    assert.equal(doc.documentElement.dir, "rtl");
    assert.equal(doc.documentElement.lang, "ar");
    assert.equal(t("card.menuLabel", {name:"worker:alpha"}),
      "إجراءات المورد worker:alpha");
  } finally {setLocale(original);}
});

test("user-reported machine fixes have minimum source-level protection", () => {
  const app=read("apps/desktop/ui/app.mjs");
  const surface=read("apps/desktop/src-tauri/src/linux_surface.rs");
  const native=read("apps/desktop/src-tauri/src/main.rs");
  const settings=read("apps/desktop/ui/settings.mjs");
  assert.match(app,/AUTO_HIDE_MS = 15_000/);
  assert.match(app,/cardVisible \|\| expanded \|\| resizing/);
  assert.match(app,/invoke\("start_card_drag"\)/);
  assert.match(app,/invoke\("stop_card_drag"\)/);
  assert.match(surface,/card-position.json/);
  assert.match(surface,/cfvigil_relative_begin/);
  assert.match(surface,/cfvigil_relative_end/);
  assert.match(surface,/MARGINS.with/);
  assert.match(native,/fn fit_settings\(/);
  assert.match(native,/account_id.is_empty\(\) && token.is_some\(\)/);
  assert.match(settings,/function restoreDraft\(/);
  assert.doesNotMatch(settings,/addEventListener\("change", async/);
});

test("KDE backend selection and Settings surface are native before machine retest", () => {
  const main = read("apps/desktop/src-tauri/src/main.rs");
  const settingsNative = read("apps/desktop/src-tauri/src/linux_settings.rs");
  const config = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
  const shell = read("apps/desktop/ui/settings.html");
  const ui = read("apps/desktop/ui/settings.mjs");
  const mainStart = main.indexOf("fn main() {");
  assert.ok(mainStart > 0);
  assert.ok(main.indexOf("configure_gdk_for_native_wayland();", mainStart) >
    mainStart && main.indexOf("configure_gdk_for_native_wayland();", mainStart) <
    main.indexOf("tauri::Builder::default()", mainStart));
  assert.match(main, /CFVigil surface mode=/);
  assert.match(main, /linux_settings::install\(app\)\?/);
  assert.match(main, /linux_settings::fit\(&app, height\.ceil\(\) as i32\)/);
  assert.match(settingsNative, /set_propagate_natural_height\(false\)/);
  assert.match(settingsNative, /viewport\.add\(&content\)/);
  assert.match(settingsNative, /window\.set_decorated\(false\)/);
  assert.ok(shell.includes('id="close-settings"'));
  assert.ok(shell.includes('id="settings-drag-area"'));
  assert.ok(ui.includes('invoke("start_settings_drag")'));
  const settingsConfig = config.app.windows.find(item => item.label === "settings");
  assert.equal(settingsConfig.decorations, false);
  assert.equal(settingsConfig.maximizable, false);
  assert.equal(settingsConfig.minimizable, false);
});
