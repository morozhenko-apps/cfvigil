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
  for (const item of receipt.artifacts) {
    if (!Number.isSafeInteger(item.run_id) || item.run_id < 1 ||
        !Array.isArray(item.files) || item.files.length < 1) {
      throw Error("Missing verified R2 release files or source run");
    }
    for (const file of item.files) {
      if (typeof file.url !== "string" ||
          !file.url.startsWith(BASE + "releases/v" + receipt.version + "/") ||
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
  let removed = 0;
  for (const target of receipt.artifacts) {
    const artifacts = await listRunArtifacts(target.run_id, token, request);
    const matches = artifacts.filter(a => a.name === target.name);
    if (matches.length > 1) throw Error("Ambiguous release artifact; refusing deletion");
    if (!matches.length) continue; // idempotent if already cleaned
    const artifact = matches[0];
    if (!Number.isSafeInteger(artifact.id) || artifact.id < 1 ||
        artifact.workflow_run?.id !== target.run_id ||
        artifact.workflow_run?.head_sha !== receipt.commit) {
      throw Error("Artifact belongs to a different build; refusing deletion");
    }
    const response = await request(
      "https://api.github.com/repos/" + REPO + "/actions/artifacts/" + artifact.id,
      {method:"DELETE",headers:{Authorization:"Bearer " + token,Accept:"application/vnd.github+json"}}
    );
    if (response.status !== 204) throw Error("GitHub Actions artifact cleanup failed");
    removed++;
    const remaining = await listRunArtifacts(target.run_id, token, request);
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
