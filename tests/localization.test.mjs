import test from "node:test";
import assert from "node:assert/strict";
import english from "../apps/desktop/ui/locales/en.mjs";
import {
  ACTIVE_LOCALES, PLANNED_LOCALES, directionFor, validateCatalog,
  setLocale, currentLocale,
  t, plural, formatDate, relativeTime, verboseRelativeTime, eventText,
  incidentText, resourceDetail, intlLocale,
  localizeDocument
} from "../apps/desktop/ui/localization.mjs";
import {referencedKeys, checkSourceKeys, checkRepository} from "../scripts/check-i18n.mjs";

test("feature freeze only activates reviewed English and retains exactly 20 target locales", () => {
  assert.deepEqual(ACTIVE_LOCALES, ["en", "ru", "pt", "es", "de", "fr"]);
  assert.equal(PLANNED_LOCALES.length, 20);
  assert.equal(new Set(PLANNED_LOCALES).size, 20);
  assert.ok(PLANNED_LOCALES.includes("zh-Hant"));
  assert.ok(PLANNED_LOCALES.includes("pt"));
  assert.equal(directionFor("ar"), "rtl");
  assert.equal(intlLocale("pt"), "pt-PT");
  assert.equal(directionFor("en"), "ltr");
  assert.throws(() => t("card.healthy", {}, "ar"), /Unsupported or unreviewed/);
  assert.equal(t("card.issues", {}, "ru"), "Проблемы");
  assert.equal(t("settings.title", {}, "pt"), "Definições");
});

test("English messages and their interpolation, plurals, and invalid keys", () => {
  assert.deepEqual(validateCatalog(english), []);
  assert.equal(t("card.menuLabel", {name: "worker-a"}), "Actions for worker-a");
  assert.equal(plural("history.eventCount", 1, {shown:1, total:1}), "1 of 1 event");
  assert.equal(plural("history.eventCount", 3, {shown:2, total:3}), "2 of 3 events");
  assert.equal(plural("age.minute", 2), "2m");
  assert.throws(() => t("card.menuLabel"), /Missing localization parameter/);
  assert.throws(() => t("missing.key"), /Unknown text key/);
  assert.throws(() => t("history.eventCount"), /Unknown text key/);
  assert.throws(() => plural("card.healthy", 1), /Unknown plural key/);
  assert.throws(() => plural("age.day", -1), /Invalid plural count/);
  assert.equal(t("card.menuLabel", {name:"<img src=x onerror=alert(1)>"}),
    "Actions for <img src=x onerror=alert(1)>");
});

test("catalog diagnostics reject mismatches instead of silently falling back", () => {
  const missing = {...english};
  delete missing["settings.token"];
  assert.ok(validateCatalog(missing).some(x => x.includes("Missing key: settings.token")));
  const unexpected = {...english, "unknown.key":"anything"};
  assert.ok(validateCatalog(unexpected).some(x => x.includes("Unexpected key: unknown.key")));
  const interpolation = {...english, "card.menuLabel":"Actions for {resource}"};
  assert.ok(validateCatalog(interpolation).some(x => x.includes("Interpolation mismatch")));
  const pluralMissing = {...english, "history.eventCount": {other: "{shown} of {total} events"}};
  assert.ok(validateCatalog(pluralMissing).some(x => x.includes("Missing plural form one")));
  const pluralBroken = {...english, "history.eventCount": {
    one: "{shown} of {total} event", other: "{shown} of {other} events"
  }};
  assert.ok(validateCatalog(pluralBroken).some(x => x.includes("Plural interpolation mismatch")));
  assert.ok(checkSourceKeys({keys:new Set(["card.healthy","missing"]), errors:[]}, english)
    .includes("Undeclared UI key: missing"));
  assert.ok(referencedKeys({main:'<html lang="en" dir="ltr"><p data-i18n="card.healthy"></p>'},{})
    .keys.has("card.healthy"));
  assert.ok(checkSourceKeys({
    keys:new Set(["card.menuLabel"]),errors:[],
    usages:[{kind:"html",key:"card.menuLabel"}]
  }, english).some(x => x.includes("HTML key requires parameters")));
  assert.ok(referencedKeys({main:'<html lang="en" dir="ltr"><p data-i18n-unknown="foo"></p>'},{})
    .errors.some(x => x.includes("Unsupported localization attribute")));
  assert.ok(checkSourceKeys({
    keys:new Set(["history.eventCount"]),errors:[],
    usages:[{kind:"t",key:"history.eventCount"}]
  }, english).some(x => x.includes("Expected simple text key")));
});

test("dates and relative timestamps remain deterministic and locale-aware", () => {
  assert.equal(formatDate(null), "Unknown time");
  assert.equal(formatDate("not a date"), "Unknown time");
  assert.match(formatDate("2026-10-23T12:00:00Z"), /2026/);
  const now=Date.parse("2026-10-23T12:00:00Z");
  assert.equal(relativeTime(null,now), "—");
  assert.equal(relativeTime("2026-10-23T11:58:00Z",now), "2m");
  assert.equal(relativeTime("2026-10-23T10:00:00Z",now), "2h");
  assert.equal(relativeTime("2026-10-21T12:00:00Z",now), "2d");
  assert.match(verboseRelativeTime("2026-10-23T11:58:00Z",now), /2 minutes ago/);
  assert.equal(eventText("Workers runtime error spike detected"),
    "Workers runtime error spike detected");
  assert.equal(eventText("Provider-specific diagnostic"), "Provider-specific diagnostic");
  assert.equal(incidentText("worker:name: WAF block spike detected"),
    "worker:name: WAF block spike detected");
  assert.equal(resourceDetail("Workers errors spiking: 30 errors / 200 requests (15.0%)"),
    "Workers errors spiking: 30 errors / 200 requests (15.0%)");
  assert.equal(resourceDetail("WAF blocks spiking: 150 in 10m; previous 30m 25"),
    "WAF blocks spiking: 150 in 10m; previous 30m 25");

});

test("DOM localization uses textContent and explicit direction, never innerHTML", () => {
  const values = [];
  const node = {
    getAttribute: attr => attr === "data-i18n" ? "card.healthy" : null,
    set textContent(value) {values.push(value);}
  };
  const doc = {
    documentElement: {lang: "", dir: ""},
    querySelectorAll: selector => selector === "[data-i18n]" ? [node] : []
  };
  localizeDocument(doc);
  assert.deepEqual(values, ["Healthy"]);
  assert.equal(doc.documentElement.lang, "en");
  assert.equal(doc.documentElement.dir, "ltr");
});

test("all checked desktop source references resolve to declared catalog keys", async () => {
  assert.deepEqual(await checkRepository(), []);
});

test("activated language changes all presentation helpers, not saved event text", () => {
  setLocale("ru");
  try {
    assert.equal(currentLocale, "ru");
    assert.equal(t("card.resources"), "Ресурсы");
    assert.equal(eventText("WAF block spike detected"), "Обнаружен рост блокировок WAF");
    assert.match(formatDate("2026-10-23T10:00:00Z"), /2026/);
    assert.equal(plural("history.eventCount", 5, {shown:1,total:5}), "Показано 1 из 5 событий");
    assert.throws(() => setLocale("ar"), /Unsupported or unreviewed/);
  } finally { setLocale("en"); }
  assert.equal(currentLocale, "en");
});
