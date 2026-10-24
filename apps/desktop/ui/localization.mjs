import english from "./locales/en.mjs";
import russian from "./locales/ru.mjs";
import portuguese from "./locales/pt.mjs";
import spanish from "./locales/es.mjs";
import german from "./locales/de.mjs";
import french from "./locales/fr.mjs";

// The exact product locale set. Only reviewed dictionaries can be activated.
export const PLANNED_LOCALES = Object.freeze([
  "en", "ru", "es", "pt", "de", "fr", "it", "pl", "hi", "zh-Hant",
  "ja", "nl", "sv", "tr", "ko", "bn", "th", "id", "fil", "ar"
]);
export const ACTIVE_LOCALES = Object.freeze(["en", "ru", "pt", "es", "de", "fr"]);
export const catalogs = Object.freeze({en: english, ru: russian, pt: portuguese,
  es: spanish, de: german, fr: french});
export const LANGUAGE_NAMES = Object.freeze({en:"English", ru:"Русский", pt:"Português (Portugal)",
  es:"Español", de:"Deutsch", fr:"Français"});
export let currentLocale = "en";

export function setLocale(locale) {
  catalog(locale);
  currentLocale = locale;
  return locale;
}

export function intlLocale(locale) {
  return locale === "pt" ? "pt-PT" : locale;
}

export function directionFor(locale) {
  return locale === "ar" ? "rtl" : "ltr";
}

export function tokens(value) {
  const matches = typeof value === "string"
    ? [...value.matchAll(/\{([a-z][a-zA-Z0-9_]*)\}/g)].map(match => match[1]) : [];
  return [...new Set(matches)].sort();
}

export function validateCatalog(candidate, baseline = english, locale = "en") {
  const errors = [];
  if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
    return ["Catalog must be a plain object"];
  }
  for (const key of new Set([...Object.keys(baseline), ...Object.keys(candidate)])) {
    if (!(key in candidate)) { errors.push(`Missing key: ${key}`); continue; }
    if (!(key in baseline)) { errors.push(`Unexpected key: ${key}`); continue; }
    const ref = baseline[key], val = candidate[key];
    if (typeof ref === "string") {
      if (typeof val !== "string" || !val.trim()) {
        errors.push(`Invalid message: ${key}`); continue;
      }
      if (JSON.stringify(tokens(ref)) !== JSON.stringify(tokens(val))) {
        errors.push(`Interpolation mismatch: ${key}`);
      }
    } else if (ref && typeof ref === "object" && !Array.isArray(ref)) {
      if (!val || typeof val !== "object" || Array.isArray(val) ||
          typeof val.other !== "string" || !val.other.trim()) {
        errors.push(`Invalid plural: ${key}`); continue;
      }
      const categories = new Intl.PluralRules(intlLocale(locale)).resolvedOptions().pluralCategories;
      for (const category of categories) {
        if (typeof val[category] !== "string" || !val[category].trim()) {
          errors.push(`Missing plural form ${category}: ${key}`);
        }
      }
      for (const category of Object.keys(val)) {
        if (!categories.includes(category)) errors.push(`Unexpected plural form ${category}: ${key}`);
      }
      const expectedTokens = tokens(ref.other);
      for (const [category, value] of Object.entries(val)) {
        if (typeof value !== "string" ||
            JSON.stringify(tokens(value)) !== JSON.stringify(expectedTokens)) {
          errors.push(`Plural interpolation mismatch: ${key}.${category}`);
        }
      }
    } else {
      errors.push(`Invalid baseline: ${key}`);
    }
  }
  return errors;
}

function catalog(locale) {
  if (!ACTIVE_LOCALES.includes(locale) || !catalogs[locale]) {
    throw new Error(`Unsupported or unreviewed locale: ${locale}`);
  }
  return catalogs[locale];
}

function substitute(pattern, params) {
  for (const token of tokens(pattern)) {
    if (!Object.hasOwn(params, token) || params[token] === null ||
        params[token] === undefined) {
      throw new Error(`Missing localization parameter: ${token}`);
    }
  }
  return pattern.replace(/\{([a-z][a-zA-Z0-9_]*)\}/g, (_, key) => String(params[key]));
}

export function t(key, params = {}, locale = currentLocale) {
  const value = catalog(locale)[key];
  if (typeof value !== "string") throw new Error(`Unknown text key: ${key}`);
  return substitute(value, params);
}

export function plural(key, count, params = {}, locale = currentLocale) {
  if (!Number.isFinite(count) || count < 0) throw new Error("Invalid plural count");
  const value = catalog(locale)[key];
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Unknown plural key: ${key}`);
  }
  const form = new Intl.PluralRules(intlLocale(locale)).select(count);
  return substitute(value[form] ?? value.other, {...params, count});
}

export function formatDate(when, locale = currentLocale) {
  const timestamp = Date.parse(when ?? "");
  if (!Number.isFinite(timestamp)) return t("history.unknownTime", {}, locale);
  return new Intl.DateTimeFormat(intlLocale(locale), {dateStyle: "medium", timeStyle: "short"}).format(timestamp);
}

export function relativeTime(when, now = Date.now(), locale = currentLocale) {
  const timestamp = Date.parse(when ?? "");
  if (!Number.isFinite(timestamp)) return "—";
  const minutes = Math.max(0, Math.floor((now - timestamp) / 60000));
  if (minutes < 1) return t("age.now", {}, locale);
  if (minutes < 60) return plural("age.minute", minutes, {}, locale);
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return plural("age.hour", hours, {}, locale);
  return plural("age.day", Math.floor(hours / 24), {}, locale);
}

export function verboseRelativeTime(when, now = Date.now(), locale = currentLocale) {
  const timestamp = Date.parse(when ?? "");
  if (!Number.isFinite(timestamp)) return t("history.unknownTime", {}, locale);
  const minutes = Math.max(0, Math.floor((now - timestamp) / 60000));
  const formatter = new Intl.RelativeTimeFormat(intlLocale(locale), {numeric: "auto", style: "long"});
  if (minutes < 60) return formatter.format(-minutes, "minute");
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return formatter.format(-hours, "hour");
  return formatter.format(-Math.floor(hours / 24), "day");
}

// History stores legacy English event text; localize known stable values and
// leave unknown/provider-specific messages intact, never invent event types.
const legacyEvents = new Map(Object.entries(english)
  .filter(([key, value]) => key.startsWith("event.") && typeof value === "string")
  .map(([key, value]) => [value, key]));
export function eventText(message, locale = currentLocale) {
  const key = legacyEvents.get(message);
  return key ? t(key, {}, locale) : String(message ?? "");
}

// The persisted notice combines an API resource name with the stable event
// message. Only the known suffix is translated; the name is never interpreted
// as a localization key, even if it contains punctuation.
export function incidentText(message, locale = currentLocale) {
  if (typeof message !== "string") return "";
  for (const [literal, key] of legacyEvents) {
    if (message === literal) return t(key, {}, locale);
    const suffix = ": " + literal;
    if (message.endsWith(suffix)) {
      return message.slice(0, -suffix.length) + ": " + t(key, {}, locale);
    }
  }
  return message;
}

export function resourceDetail(message, locale = currentLocale) {
  if (typeof message !== "string") return "";
  const workers = /^Workers errors spiking: (\d+) errors \/ (\d+) requests \((\d+\.\d+%)\)$/.exec(message);
  if (workers) {
    return t("detail.workerSpike", {errors:workers[1],requests:workers[2],rate:workers[3].slice(0,-1)},locale);
  }
  const waf = /^WAF blocks spiking: (\d+) in 10m; previous 30m (\d+)$/.exec(message);
  if (waf) return t("detail.wafSpike", {recent:waf[1],baseline:waf[2]},locale);
  return eventText(message, locale);
}

export function localizeDocument(doc, locale = currentLocale) {
  catalog(locale); // An incomplete/unreviewed translation must never render.
  doc.documentElement.lang = intlLocale(locale);
  doc.documentElement.dir = directionFor(locale);
  const attributes = [
    ["data-i18n", "textContent"], ["data-i18n-title", "title"],
    ["data-i18n-placeholder", "placeholder"],
    ["data-i18n-aria-label", "aria-label"]
  ];
  for (const [attribute, target] of attributes) {
    for (const node of doc.querySelectorAll(`[${attribute}]`)) {
      const value = t(node.getAttribute(attribute), {}, locale);
      if (target === "aria-label") node.setAttribute(target, value);
      else node[target] = value;
    }
  }
}
