// A configured first poll is distinct from missing credentials or failed data.
export function emptyResourceMessageKey({configured, pending, ready}) {
  if (!configured) return "card.noAccount";
  if (pending) return "diagnostic.pending";
  return ready ? "card.noResources" : "card.noData";
}
