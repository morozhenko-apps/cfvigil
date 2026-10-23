// Pure search/filter logic for the local history window.
export const HISTORY_FILTERS = Object.freeze(["all", "incident", "recovery", "update"]);

export function filterHistory(events, query = "", category = "all") {
  const needle = String(query).trim().toLocaleLowerCase();
  if (!HISTORY_FILTERS.includes(category)) return [];
  return events.filter(event => {
    if (category !== "all" && event.category !== category) return false;
    if (!needle) return true;
    return [event.name, event.key, event.message]
      .some(value => String(value ?? "").toLocaleLowerCase().includes(needle));
  });
}

export function historyTimestamp(at, language = undefined) {
  const value = Date.parse(at ?? "");
  return Number.isFinite(value) ? new Date(value).toLocaleString(language) : "Unknown time";
}
