#!/usr/bin/env node
// Release-only finalizer. The R2 publisher emits the receipt ONLY after
// public GET readback verifies the exact bytes. Candidates never invoke it.
import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import path from "node:path";

const REPO = "morozhenko-apps/cfvigil";
const BASE = "https://download.morozhenko.com/dist/CFVigil/";
const NAMES = new Set(["cfvigil-release-linux", "cfvigil-release-windows"]);
const isSha = value => typeof value === "string" && /^[0-9a-f]{64}$/.test(value);

export function validateR2Receipt(receipt) {
  if (!receipt || receipt.schema !== 1 || receipt.product !== "CFVigil" ||
      receipt.base_url !== BASE ||
      !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(receipt.version ?? "") ||
      !/^[0-9a-f]{40}$/.test(receipt.commit ?? "") ||
      !Array.isArray(receipt.artifacts) || receipt.artifacts.length !== 2 ||
      new Set(receipt.artifacts.map(item => item.name)).size !== 2 ||
      receipt.artifacts.some(item => !NAMES.has(item.name))) {
    throw Error("Invalid CFVigil publication receipt or artifact scope");
  }
  // The mutable latest pointer is the final publication commit point.
  // Without verified GET body/size/checksum, no temporary source archives
  // can be deleted, even if platform binaries were already uploaded.
  const latest=receipt.latest;
  if (!latest || latest.url !== BASE + "latest.json" ||
      !isSha(latest.sha256) || latest.readback_sha256 !== latest.sha256 ||
      !Number.isSafeInteger(latest.size) || latest.size < 1 ||
      latest.readback_size !== latest.size || latest.readback_status !== 200) {
    throw Error("Missing verified final latest.json publication readback");
  }
  // r2_release.py emits one GitHub release run for BOTH handoff
  // archives; a receipt cannot splice archives from unrelated workflows.
  if (new Set(receipt.artifacts.map(item => item.run_id)).size !== 1) {
    throw Error("CFVigil receipt archives must share the same source run");
  }
  // Only the exact artifacts emitted by r2_release.py can authorize
  // destruction of their temporary GitHub handoff archives. The public
  // R2 URL prefix alone does not prove the full platform payload exists.
  const version = receipt.version;
  const approved = {
    "cfvigil-release-linux": [
      "linux/cfvigil_" + version + "_amd64.deb",
      "linux/cfvigil_" + version + "_amd64.deb.sha256",
      "linux/CFVigil-" + version + "-linux-x64.AppImage",
      "linux/CFVigil-" + version + "-linux-x64.AppImage.sha256"
    ],
    "cfvigil-release-windows": [
      "windows/CFVigil-Setup-" + version + "-win-x64.exe",
      "windows/CFVigil-Setup-" + version + "-win-x64.exe.sha256"
    ]
  };
  for (const item of receipt.artifacts) {
    if (!Number.isSafeInteger(item.run_id) || item.run_id < 1 ||
        !Array.isArray(item.files) || item.files.length < 1) {
      throw Error("Missing verified R2 release files or source run");
    }
    const expectedUrls = approved[item.name].map(relative => BASE + relative);
    const actualUrls = item.files.map(file => file?.url);
    if (actualUrls.length !== expectedUrls.length ||
        new Set(actualUrls).size !== expectedUrls.length ||
        expectedUrls.some(url => !actualUrls.includes(url))) {
      throw Error("Incomplete or mismatched CFVigil R2 platform receipt");
    }
    for (const file of item.files) {
      if (typeof file.url !== "string" ||
          !file.url.startsWith(BASE + (item.name === "cfvigil-release-linux" ? "linux/" : "windows/")) ||
          !isSha(file.sha256) || file.readback_sha256 !== file.sha256 ||
          !Number.isSafeInteger(file.size) || file.size < 1 ||
          file.readback_size !== file.size || file.readback_status !== 200) {
        throw Error("Missing verified public R2 readback for a release file");
      }
    }
  }
  return receipt;
}

async function listRunArtifacts(run, token, request) {
  const result = [];
  for (let page = 1; page <= 10; page++) {
    const url = "https://api.github.com/repos/" + REPO +
      "/actions/runs/" + run + "/artifacts?per_page=100&page=" + page;
    const response = await request(url, {
      headers: {Authorization: "Bearer " + token, Accept:"application/vnd.github+json"}
    });
    if (response.status !== 200) throw Error("Could not list GitHub Actions artifacts");
    const payload = await response.json();
    if (!Array.isArray(payload.artifacts) || payload.artifacts.length > 100)
      throw Error("Malformed GitHub artifact listing");
    result.push(...payload.artifacts);
    if (payload.artifacts.length < 100) return result;
  }
  throw Error("Too many artifacts for safe cleanup");
}

export async function deletePublishedArtifacts(receipt, token, request = fetch) {
  validateR2Receipt(receipt); // no DELETE without verified readback proof
  if (!token) throw Error("Missing Actions-write cleanup token");
  const runId = receipt.artifacts[0].run_id;
  // Inspect BOTH archives before the first destructive request.
  // A foreign second archive, duplicate or bad listing must never
  // allow deletion of an otherwise valid first archive.
  const artifacts = await listRunArtifacts(runId, token, request);
  const approved = [];
  for (const target of receipt.artifacts) {
    const matches = artifacts.filter(a => a.name === target.name);
    if (matches.length > 1) throw Error("Ambiguous release artifact; refusing deletion");
    if (!matches.length) continue; // idempotent after a successful prior cleanup
    const artifact = matches[0];
    if (!Number.isSafeInteger(artifact.id) || artifact.id < 1 ||
        artifact.workflow_run?.id !== runId ||
        artifact.workflow_run?.head_sha !== receipt.commit) {
      throw Error("Artifact belongs to a different build; refusing deletion");
    }
    approved.push({target, artifact});
  }
  if (new Set(approved.map(item => item.artifact.id)).size !== approved.length) {
    throw Error("Ambiguous release artifact; refusing deletion");
  }

  let removed = 0;
  for (const {target, artifact} of approved) {
    const response = await request(
      "https://api.github.com/repos/" + REPO + "/actions/artifacts/" + artifact.id,
      {method:"DELETE",headers:{Authorization:"Bearer " + token,Accept:"application/vnd.github+json"}}
    );
    if (response.status !== 204) throw Error("GitHub Actions artifact cleanup failed");
    removed++;
    const remaining = await listRunArtifacts(runId, token, request);
    if (remaining.some(a => a.id === artifact.id || a.name === target.name)) {
      throw Error("Deleted artifact remains in GitHub Actions listing");
    }
  }
  return removed;
}

async function main() {
  if (process.argv.length !== 4 || process.argv[2] !== "--verified-receipt") {
    throw Error("Usage: node scripts/cleanup-published-artifacts.mjs --verified-receipt PATH");
  }
  const receipt = JSON.parse(readFileSync(process.argv[3], "utf8"));
  const removed = await deletePublishedArtifacts(receipt, process.env.GH_TOKEN);
  console.log("CFVigil: removed " + removed + " temporary GitHub archives after R2 readback");
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(err => {console.error(err.message);process.exitCode=1;});
}
