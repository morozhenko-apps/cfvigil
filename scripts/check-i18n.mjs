#!/usr/bin/env node
// Explicit feature-freeze key/locale checks, independent from desktop WebView.
import {readFileSync, readdirSync} from "node:fs";
import {fileURLToPath, pathToFileURL} from "node:url";
import path from "node:path";
import {validateCatalog, PLANNED_LOCALES, ACTIVE_LOCALES, catalogs, tokens} from "../apps/desktop/ui/localization.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fromRoot = name => readFileSync(path.join(root, name), "utf8");
const pages = ["index", "settings", "history", "about"];
const uiScripts = ["app", "settings", "history", "about", "history-model", "status", "runtime-language"];
const htmlKinds = Object.freeze({
  "data-i18n": "text", "data-i18n-title": "title",
  "data-i18n-placeholder": "placeholder", "data-i18n-aria-label": "aria-label"
});

export function referencedKeys(htmlSources, jsSources) {
  const keys = new Set(), errors = [], usages = [];
  for (const [name, html] of Object.entries(htmlSources)) {
    for (const match of html.matchAll(/\b(data-i18n[a-z-]*)="([^"]+)"/g)) {
      if (!Object.hasOwn(htmlKinds, match[1])) {
        errors.push(`Unsupported localization attribute in ${name}: ${match[1]}`);
      } else {
        keys.add(match[2]);
        usages.push({key: match[2], kind:"html"});
      }
    }
    if (!/<html\s+lang="en"\s+dir="ltr">/.test(html)) {
      errors.push(`Missing explicit HTML baseline direction in ${name}`);
    }
  }
  for (const [name, source] of Object.entries(jsSources)) {
    // Dynamic inputs must never be used as translation keys. All ordinary
    // presentation calls use literal IDs, and tests exercise the renderer.
    for (const match of source.matchAll(/\b(t|plural)\(\s*"([^"]+)"/g)) {
      keys.add(match[2]);
      usages.push({key:match[2], kind:match[1]});
    }
    if (/\b(?:t|plural)\(\s*(?:resource|event|payload)\b/.test(source)) {
      errors.push(`Provider-supplied translation key in ${name}`);
    }
  }
  return {keys, errors, usages};
}

export function checkSourceKeys(source, dictionary) {
  const failures = [...source.errors];
  for (const key of source.keys) {
    if (!(key in dictionary)) failures.push(`Undeclared UI key: ${key}`);
    else if (typeof dictionary[key] !== "string" &&
             (!dictionary[key] || typeof dictionary[key].other !== "string")) {
      failures.push(`Invalid UI key type: ${key}`);
    }
  }
  for (const usage of source.usages || []) {
    if (!(usage.key in dictionary)) continue;
    const value = dictionary[usage.key];
    if (usage.kind === "plural" && (typeof value !== "object" || !value)) {
      failures.push(`Expected plural key: ${usage.key}`);
    }
    if ((usage.kind === "t" || usage.kind === "html") && typeof value !== "string") {
      failures.push(`Expected simple text key: ${usage.key}`);
    }
    if (usage.kind === "html" && typeof value === "string" && tokens(value).length) {
      failures.push(`HTML key requires parameters: ${usage.key}`);
    }
  }
  return failures;
}

export async function checkRepository() {
  const html = Object.fromEntries(pages.map(page =>
    [page, fromRoot(`apps/desktop/ui/${page}.html`)]));
  const javascript = Object.fromEntries(uiScripts.map(script =>
    [script, fromRoot(`apps/desktop/ui/${script}.mjs`)]));
  const english = catalogs.en;
  const errors = [
    ...checkSourceKeys(referencedKeys(html, javascript), english),
    ...validateCatalog(english, english, "en")
  ];
  const expected = [
    "en", "ru", "es", "pt", "de", "fr", "it", "pl", "hi", "zh-Hant",
    "ja", "nl", "sv", "tr", "ko", "bn", "th", "id", "fil", "ar"
  ];
  if (JSON.stringify(PLANNED_LOCALES) !== JSON.stringify(expected)) {
    errors.push("Required 20-locale contract changed");
  }
  const dir = path.join(root, "apps/desktop/ui/locales");
  const files = readdirSync(dir).filter(name => name.endsWith(".mjs"));
  const installed = files.map(name => name.slice(0, -4));
  for (const active of ACTIVE_LOCALES) {
    if (!installed.includes(active)) errors.push(`Activated locale missing dictionary: ${active}`);
  }
  for (const locale of installed) {
    if (!PLANNED_LOCALES.includes(locale)) {
      errors.push(`Unapproved locale file: ${locale}`); continue;
    }
    const url = pathToFileURL(path.join(dir, `${locale}.mjs`)).href;
    const candidate = (await import(url)).default;
    errors.push(...validateCatalog(candidate, english, locale).map(error => `${locale}: ${error}`));
    if (!ACTIVE_LOCALES.includes(locale)) {
      errors.push(`Locale not approved for runtime activation: ${locale}`);
    }
  }
  for (const [key, value] of Object.entries(english)) {
    for (const variant of typeof value === "string" ? [value] : Object.values(value)) {
      if (typeof variant !== "string") continue;
      const withoutTokens = variant.replace(/\{[a-z][a-zA-Z0-9_]*\}/g, "");
      if (/[{}]/.test(withoutTokens)) errors.push(`Unmatched interpolation brace: ${key}`);
      for (const token of tokens(variant)) {
        if (token.toLowerCase() === "html") errors.push(`Unsafe markup placeholder: ${key}`);
      }
    }
  }
  return errors;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  checkRepository().then(errors => {
    for (const error of errors) console.error(`i18n: ${error}`);
    if (errors.length) process.exitCode = 1;
    else console.log("CFVigil localization source checks passed (enabled dictionaries checked)");
  }).catch(error => {console.error(error); process.exitCode = 1;});
}
