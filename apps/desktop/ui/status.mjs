// Pure, deterministic UI status rules. Latest attempt overrides previous failure.
export const STATES = Object.freeze(["healthy", "deploying", "failed", "unknown"]);

export function classifyWorkerBuild(build) {
  if (!build || typeof build !== "object") return "unknown";
  if (["queued", "initializing", "running"].includes(build.status)) return "deploying";
  if (build.status === "stopped") {
    if (build.build_outcome === "success") return "healthy";
    if (["fail", "cancelled", "terminated"].includes(build.build_outcome)) return "failed";
    return "unknown";
  }
  return "unknown";
}

export function classifyPagesDeployment(deployment) {
  if (!deployment || typeof deployment !== "object") return "unknown";
  const status = deployment.latest_stage?.status;
  if (["active", "idle"].includes(status)) return "deploying";
  if (status === "success") return "healthy";
  if (status === "failure") return "failed";
  return "unknown";
}

export function counters(resources) {
  const result = { healthy: 0, deploying: 0, failed: 0, unknown: 0 };
  for (const resource of resources) {
    if (Object.hasOwn(result, resource.status)) result[resource.status] += 1;
    else result.unknown += 1;
  }
  return result;
}

export function filterResources(resources, query) {
  const needle = query.trim().toLocaleLowerCase();
  return resources.filter(resource => resource.name.toLocaleLowerCase().includes(needle));
}

export function relativeTime(when, now = Date.now()) {
  const timestamp = Date.parse(when ?? "");
  if (!Number.isFinite(timestamp)) return "—";
  const minutes = Math.max(0, Math.floor((now - timestamp) / 60000));
  if (minutes < 1) return "now";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  return `${Math.floor(hours / 24)}d`;
}
