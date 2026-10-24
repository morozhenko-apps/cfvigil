// Dedicated read-only Cloudflare Worker for CFVigil artifacts. Never mutates R2.
const PREFIX = "/dist/CFVigil/";
const VERSION = "(?:0|[1-9][0-9]*)\\.(?:0|[1-9][0-9]*)\\.(?:0|[1-9][0-9]*)";
const aptPatterns = [
  new RegExp("^apt/pool/main/c/cf-vigil/cfvigil_" + VERSION + "_amd64\\.deb$"),
  /^apt\/dists\/stable\/InRelease$/,
  /^apt\/dists\/stable\/main\/binary-amd64\/Packages(?:\.gz)?$/,
  /^apt\/dists\/stable\/main\/binary-amd64\/by-hash\/SHA256\/[a-f0-9]{64}$/,
  /^apt\/cfvigil-archive-keyring\.gpg$/
];

export function keyFor(rawUrl) {
  let url;
  try { url = new URL(rawUrl); } catch { return null; }
  if (!url.pathname.startsWith(PREFIX) || url.search || url.hash) return null;
  const relative = url.pathname.slice(PREFIX.length);
  if (!relative || /%|\\|\.\.|\/\//.test(relative)) return null;
  if (relative.startsWith("releases/")) {
    const match = /^releases\/v((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*))\/(linux|windows)\/([^/]+)$/.exec(relative);
    if (!match) return null;
    const [, version, platform, name] = match;
    const expected = platform === "linux"
      ? ["cfvigil_" + version + "_amd64.deb",
         "CFVigil-" + version + "-linux-x64.AppImage"]
      : ["CFVigil-Setup-" + version + "-win-x64.exe"];
    if (!expected.some(filename => name === filename ||
        name === filename + ".sha256")) return null;
    return "dist/CFVigil/" + relative;
  }
  return aptPatterns.some(pattern => pattern.test(relative))
    ? "dist/CFVigil/" + relative : null;
}

function contentType(key) {
  if (key.endsWith(".deb")) return "application/vnd.debian.binary-package";
  if (key.endsWith(".gpg")) return "application/pgp-keys";
  if (key.endsWith(".gz")) return "application/gzip";
  if (key.endsWith(".sha256") || key.endsWith("/InRelease") ||
      key.endsWith("/Packages")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

export default {
  async fetch(request, env) {
    if (request.method !== "GET" && request.method !== "HEAD") {
      return new Response("Method Not Allowed", {
        status:405,headers:{Allow:"GET, HEAD","Cache-Control":"no-store"}
      });
    }
    const key = keyFor(request.url);
    if (!key) return new Response("Not Found", {status:404});
    const object = request.method === "HEAD"
      ? await env.ARCHIVE.head(key) : await env.ARCHIVE.get(key);
    if (!object) return new Response("Not Found", {status:404});
    const mutable = /\/(?:InRelease|Packages(?:\.gz)?)$/.test(key);
    const headers = new Headers({
      "Content-Type":contentType(key),
      "Cache-Control":mutable ? "no-store" : "public, max-age=31536000, immutable",
      "X-Content-Type-Options":"nosniff"
    });
    if (object.size != null) headers.set("Content-Length",String(object.size));
    if (object.httpEtag) headers.set("ETag",object.httpEtag);
    return new Response(request.method === "HEAD" ? null : object.body,{
      status:200,headers
    });
  }
};
