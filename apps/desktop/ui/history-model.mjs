// Pure search/filter logic for the local history window.
import {formatDate, currentLocale, intlLocale, eventText} from "./localization.mjs";
export const HISTORY_FILTERS = Object.freeze(["all", "incident", "recovery", "update"]);

export function filterHistory(events, query = "", category = "all", locale = currentLocale) {
  const language = intlLocale(locale);
  const needle = String(query).trim().toLocaleLowerCase(language);
  if (!HISTORY_FILTERS.includes(category)) return [];
  return events.filter(event => {
    if (category !== "all" && event.category !== category) return false;
    if (!needle) return true;
    return [event.name, event.key, event.message, eventText(event.message, locale)]
      .some(value => String(value ?? "").toLocaleLowerCase(language).includes(needle));
  });
}

export function historyTimestamp(at, language = undefined) {
  return formatDate(at, language || currentLocale);
}
