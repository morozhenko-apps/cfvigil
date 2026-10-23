// Initial native snapshot and poll events use the same contract.
export function normalizeSnapshot(payload) {
  return {
    resources: Array.isArray(payload?.resources) ? payload.resources : [],
    ready: payload?.ready === true,
    configured: payload?.configured === true,
  };
}
