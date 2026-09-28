// Build-time PWA support: a Content-Security-Policy meta tag that includes the
// API origin, and a service worker whose precache list is the exact set of
// files in this build. See docs/phase5/DESIGN.md sections 4 and 5.
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import type { Plugin } from "vite";

function cspFor(apiBase: string): string {
  const api = apiBase ? ` ${new URL(apiBase).origin}` : "";
  return [
    "default-src 'self'", "script-src 'self'", "style-src 'self'", "img-src 'self' data:",
    `connect-src 'self'${api}`, "manifest-src 'self'", "worker-src 'self'", "font-src 'self'",
    "object-src 'none'", "base-uri 'none'", "form-action 'self'",
  ].join("; ");
}

function files(dir: string, root = dir): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? files(p, root) : [relative(root, p).split("\\").join("/")];
  });
}

const SW = (version: string, precache: string[]) => `// Generated at build time. Do not edit.
const VERSION = ${JSON.stringify(version)};
const CACHE = "lagn-shell-" + VERSION;
const PRECACHE = ${JSON.stringify(precache)};

self.addEventListener("install", (e) => {
  // No skipWaiting: a new version takes over on the next launch, never mid-session.
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(PRECACHE)));
});

self.addEventListener("activate", (e) => {
  e.waitUntil(caches.keys().then((keys) =>
    Promise.all(keys.filter((k) => k.startsWith("lagn-shell-") && k !== CACHE).map((k) => caches.delete(k)))));
});

self.addEventListener("fetch", (e) => {
  const url = new URL(e.request.url);
  // The API is never intercepted: every chart comes from the engine, never a cache.
  if (url.origin !== self.location.origin || url.pathname.startsWith("/api/") || e.request.method !== "GET") return;
  if (e.request.mode === "navigate") {
    // Network first for the page itself, falling back to the cached shell offline.
    e.respondWith(fetch(e.request).catch(() => caches.match("/index.html", { ignoreVary: true })));
    return;
  }
  // Hashed assets never change content under the same name: cache first.
  // ignoreVary: a precached copy must match whatever headers (Origin, for
  // module scripts) the browser's own request carries (QA-P5-1).
  e.respondWith(caches.match(e.request, { ignoreVary: true }).then((hit) => hit || fetch(e.request)));
});
`;

export function pwa(apiBase: string): Plugin {
  let outDir = "dist";
  return {
    name: "lagn-pwa",
    configResolved(c) { outDir = c.build.outDir; },
    transformIndexHtml: (html) => html.replace(
      "<!--lagn:csp-->",
      `<meta http-equiv="Content-Security-Policy" content="${cspFor(apiBase)}" />`,
    ),
    closeBundle() {
      const list = files(outDir).filter((f) => f !== "sw.js" && !f.endsWith(".map"));
      const precache = list.map((f) => "/" + f).sort();
      const hash = createHash("sha256");
      for (const f of list.sort()) hash.update(f).update(readFileSync(join(outDir, f)));
      writeFileSync(join(outDir, "sw.js"), SW(hash.digest("hex").slice(0, 16), precache));
    },
  };
}
