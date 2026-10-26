import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {
  ACTIVE_LOCALES, PLANNED_LOCALES, directionFor, intlLocale,
  diagnosticText, resourceDetail, incidentText, eventText, formatDate, plural, t
} from "../apps/desktop/ui/localization.mjs";
import {
  classifyWorkerBuild, classifyPagesDeployment, counters
} from "../apps/desktop/ui/status.mjs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";

const visibleNativeDetails = [
  "Latest Cloudflare Workers build",
  "Latest Worker deployment",
  "Workers Builds unavailable",
  "No verified Worker deployment",
  "Latest Cloudflare Pages deployment",
  "No Pages deployments",
  "Malformed Pages deployment response",
  "Pages deployment status unavailable",
  "Resource catalog unavailable; previous state unverified",
  "WAF analytics unavailable; previous alert unverified",
  "Workers analytics unavailable; previous spike unverified"
];


test("Mode B: audited locale states describe success, in-progress attempts and unverified history", () => {
  assert.equal(t("card.healthy", {}, "es"), "Exitosos");
  assert.equal(t("card.healthy", {}, "it"), "Riusciti");
  assert.equal(t("card.statusUnknown", {}, "de"), "Nicht verifiziert");
  assert.equal(t("card.statusUnknown", {}, "th"), "ยังไม่ยืนยัน");
  assert.equal(t("card.deploying", {}, "zh-Hant"), "進行中");
  assert.equal(t("card.deploying", {}, "ja"), "進行中");
  assert.equal(t("card.deploying", {}, "ko"), "진행 중");
  assert.equal(t("card.deploying", {}, "ar"), "قيد التنفيذ");
});



test("Given legacy deployed-success bytes When a Worker build passes Then the display does not claim deployment", () => {
  const stored = "Latest Cloudflare deployment succeeded";
  assert.equal(t("event.buildSuccess", {}, "en"),
    "Latest Cloudflare build or deployment attempt succeeded");
  assert.equal(eventText(stored, "en"),
    "Latest Cloudflare build or deployment attempt succeeded");
  assert.equal(incidentText("worker-a: " + stored, "ru"),
    "worker-a: Последняя попытка сборки или развёртывания Cloudflare завершилась успешно");
  for (const locale of ACTIVE_LOCALES) {
    const rendered = eventText(stored, locale);
    assert.ok(rendered.length > 5);
    assert.equal(rendered, t("event.buildSuccess", {}, locale));
  }
  assert.equal(stored, "Latest Cloudflare deployment succeeded");
});


test("Given Arabic RTL settings When listing Cloudflare scopes Then each immutable identifier is directionally isolated", () => {
  const help = t("settings.tokenHelp", {}, "ar");
  assert.match(help, /^استخدم رمز API خاصًا بالمستخدم/);
  for (const scope of ["Workers Scripts Read","Workers CI Read","Pages Read","Account Analytics Read","Zone Read","Zone Analytics Read"]) {
    assert.ok(help.includes("\u2068" + scope + "\u2069"), scope);
  }
  assert.match(help, /Workers/);
});

test("Given latest attempt When success state renders Then English never claims uptime", () => {
  assert.equal(t("card.healthy", {}, "en"), "Successful");
  assert.equal(t("card.deploying", {}, "en"), "Deploying");
  assert.equal(t("card.statusUnknown", {}, "en"), "Unverified");
});

test("Given locked credential store When German token guidance renders Then it names credentials not system lock", () => {
  assert.match(t("settings.tokenUnavailable", {}, "de"), /Anmeldedatenspeicher gesperrt/);
  assert.doesNotMatch(t("settings.tokenUnavailable", {}, "de"), /Systemsperre/);
});

test("Given user API token When Turkish connection guidance renders Then it does not misname an API key", () => {
  const actual = t("settings.tokenHelp", {}, "tr");
  assert.match(actual, /kullanıcı API belirteci/);
  assert.doesNotMatch(actual, /API anahtarı/);
});

test("Given persisted build events When localized to Russian Then the attempt wording is grammatical", () => {
  assert.equal(t("event.buildFailure", {}, "ru"),
    "Последняя попытка сборки или развёртывания Cloudflare завершилась ошибкой");
  assert.equal(t("event.buildStarted", {}, "ru"),
    "Запущена новая сборка или новое развёртывание Cloudflare");
  assert.equal(incidentText("Latest Cloudflare build/deployment failed", "ru"),
    "Последняя попытка сборки или развёртывания Cloudflare завершилась ошибкой");
});

test("Mode B: every activated locale supplies actual visible native descriptions, not English fallback", () => {
  assert.deepEqual(ACTIVE_LOCALES, PLANNED_LOCALES);
  assert.equal(ACTIVE_LOCALES.length, 20);
  for (const locale of ACTIVE_LOCALES) {
    for (const english of visibleNativeDetails) {
      const value = resourceDetail(english, locale);
      assert.ok(value.trim().length > 0, `${locale}: empty ${english}`);
      if (locale !== "en") {
        assert.notEqual(value, english, `${locale}: untranslated ${english}`);
      }
    }
    assert.equal(typeof t("card.statusUnknown", {}, locale), "string");
    assert.equal(typeof t("card.deploying", {}, locale), "string");
    assert.equal(typeof t("card.healthy", {}, locale), "string");
    assert.ok(plural("history.eventCount", 0, {shown:0,total:0}, locale).length > 0);
    assert.ok(plural("history.eventCount", 1, {shown:1,total:1}, locale).length > 0);
    assert.ok(plural("history.eventCount", 22, {shown:2,total:22}, locale).length > 0);
    assert.ok(formatDate("2026-10-26T14:05:00Z", locale).length > 5, locale);
  }
  assert.equal(intlLocale("pt"), "pt-PT");
  assert.equal(intlLocale("zh-Hant"), "zh-Hant");
  assert.equal(directionFor("ar"), "rtl");
});

test("Mode B: an HTTP 400 is never reclassified as Cloudflare permission denial", () => {
  for (const locale of ACTIVE_LOCALES) {
    const badRequest = diagnosticText("Pages: Cloudflare HTTP 400 (code 8000024)", locale);
    assert.match(badRequest, /^Pages: Cloudflare HTTP 400 /);
    assert.match(badRequest, /8000024/);
    assert.ok(!badRequest.includes("Cloudflare Pages Read"), locale);
    const permissionDenied = diagnosticText(
      "Pages: Cloudflare HTTP 403 (code 10000) (check Cloudflare Pages Read)", locale);
    assert.match(permissionDenied, /^Pages: Cloudflare HTTP 403 /);
    assert.match(permissionDenied, /10000/);
    assert.match(permissionDenied, /Cloudflare Pages Read/);
    const throttled = diagnosticText("Workers Builds: Cloudflare HTTP 429 (code 1015)", locale);
    assert.match(throttled, /^Workers Builds: Cloudflare HTTP 429 /);
    assert.ok(!throttled.includes("Workers CI Read"));
  }
  assert.match(diagnosticText(
    "Pages: Cloudflare HTTP 403 (check Cloudflare Pages Read)", "ar"),
    /\u2068Cloudflare Pages Read\u2069/);
});

test("Mode B: unknown native/provider strings and hostile resource names stay opaque", () => {
  for (const locale of ACTIVE_LOCALES) {
    for (const raw of [
      'Pages: Cloudflare HTTP 403 (check Administrator)',
      'Workers: <svg onload=alert(1)>',
      'Cloudflare HTTP 400 (code token-secret)',
      'Workers Analytics: token=credential',
      'WAF <script>alert(1)</script>'
    ]) {
      assert.equal(diagnosticText(raw, locale), raw);
      assert.equal(resourceDetail(raw, locale), raw);
    }
    const privateName = 'worker:<img src=x onerror=alert(1)>';
    const storedEvent = privateName + ": Latest Cloudflare build/deployment failed";
    assert.ok(incidentText(storedEvent, locale).startsWith(privateName + ": "));
  }
  const card = readFileSync(new URL("../apps/desktop/ui/app.mjs", import.meta.url), "utf8");
  assert.match(card, /entry\.textContent = diagnosticText\(item\)/);
  assert.match(card, /name\.textContent = resource\.name/);
  assert.match(card, /name\.dir = "auto"/);
});

test("Mode B: opaque source failures never count as deployment success", () => {
  const partial = normalizeSnapshot({ready:false,configured:true,
    provider_warnings:["Pages: Cloudflare HTTP 400 (code 8000024)"],
    resources:[{key:"worker:verified",status:"healthy"},{key:"pages:unknown",status:"unknown"}]});
  assert.equal(partial.ready, false);
  assert.deepEqual(counters(partial.resources),{healthy:1,deploying:0,failed:0,unknown:1});
  assert.equal(partial.provider_warnings.length,1);
  assert.equal(classifyWorkerBuild({status:"queued",build_outcome:"fail"}), "deploying");
  assert.equal(classifyWorkerBuild({status:"stopped",build_outcome:"cancelled"}), "failed");
  assert.equal(classifyWorkerBuild({status:"stopped",build_outcome:"skipped"}), "unknown");
  assert.equal(classifyPagesDeployment({is_skipped:true,
    latest_stage:{status:"success"}}), "unknown");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"active"}}), "deploying");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"failure"}}), "failed");
  assert.deepEqual(counters([{status:"unknown"},{status:"absent"}]),
    {healthy:0,deploying:0,failed:0,unknown:2});
});
