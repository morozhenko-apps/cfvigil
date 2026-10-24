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

const read = path => readFileSync(new URL("../" + path, import.meta.url), "utf8");

test("only complete English, Russian and European Portuguese can be enabled", () => {
  assert.deepEqual(ACTIVE_LOCALES, ["en","ru","pt"]);
  assert.deepEqual(validateCatalog(russian, english, "ru"), []);
  assert.deepEqual(validateCatalog(portuguese, english, "pt"), []);
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
  assert.match(main,/write_json\(&app, "language.json", &next\)/);
  assert.match(main,/app.emit\("cfvigil-language-changed", language\)/);
  const start=main.indexOf("fn set_language(");
  const end=main.indexOf("fn update_native_language(",start);
  const setter=main.slice(start,end);
  assert.doesNotMatch(setter,/poll_refresh|refresh_once|invalidate_resources|last_notified/);
  assert.match(native,/pub const ACTIVE: &\[&str\] = &\["en", "ru", "pt"\]/);
  assert.match(native,/fn unknown_languages_fail_closed\(/);
  assert.match(read("apps/desktop/src-tauri/src/linux_tray.rs"),/language::menu\(&locale, "open"\)/);
  assert.match(main,/tray\.set_menu\(Some\(menu\)\)/);
  const settings=read("apps/desktop/ui/settings.mjs");
  assert.match(settings,/invoke\("set_language", \{language:selected\}\)/);
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
