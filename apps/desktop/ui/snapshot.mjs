// Initial native snapshot and poll events use the same contract.
export function normalizeSnapshot(payload) {
  return {
    resources: Array.isArray(payload?.resources) ? payload.resources : [],
    ready: payload?.ready === true,
    pending: payload?.pending === true,
    configured: payload?.configured === true,
    provider_warnings: Array.isArray(payload?.provider_warnings)
      ? payload.provider_warnings.filter(value => typeof value === "string").slice(0, 6)
      : [],
  };
}
