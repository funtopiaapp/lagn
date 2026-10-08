// Build-time PWA support: a Content-Security-Policy meta tag that includes the
// API origin, and a service worker whose precache list is the exact set of
// files in this build. See docs/phase5/DESIGN.md sections 4 and 5.
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import type { Plugin } from "vite";

function cspFor(apiBase: string, counter = ""): string {
  const api = apiBase ? ` ${new URL(apiBase).origin}` : "";
  // The visit counter is an image from one named host, and nothing else: no
  // script from it runs, and it is not allowed to be connected to.
  const img = counter ? ` ${counter}` : "";
  return [
    // 'wasm-unsafe-eval' lets the engine compile in the browser; eval() of
    // JavaScript stays forbidden.
    "default-src 'self'", "script-src 'self' 'wasm-unsafe-eval'", "style-src 'self'", `img-src 'self' data:${img}`,
    `connect-src 'self'${api}`, "manifest-src 'self'", "worker-src 'self'", "font-src 'self'",
    "object-src 'none'", "base-uri 'none'", "form-action 'self'",
  ].join("; ");
}

/**
 * A content hash of the staged engine, for appending to engine URLs.
 */
export function engineVersion(engineDir: string): string {
  // A content hash of the staged engine, computed before the bundle is built
  // so it can be compiled into the app. It is what the app appends to every
  // engine URL.
  //
  // This exists because the service worker deliberately does not skipWaiting:
  // after a release the *old* worker stays active for the rest of the session.
  // index.html is network-first and the JS bundle has a content-hashed name,
  // so both come back new - but the engine's filenames are stable, so the old
  // worker served them cache-first from its own cache. New JavaScript calling
  // a WebAssembly export that only exists in the new engine then failed with
  // "not a function". Versioning the URL means a new build asks for a URL no
  // old cache has ever seen.
  //
  // Missing engine (a dev build with no staged engine) hashes to "dev", which
  // is stable, so nothing is busted needlessly.
  let names: string[];
  try {
    names = files(engineDir).sort();
  } catch {
    return "dev";
  }
  if (names.length === 0) return "dev";
  const hash = createHash("sha256");
  for (const f of names) hash.update(f).update(readFileSync(join(engineDir, f)));
  return hash.digest("hex").slice(0, 16);
}

/**
 * What goes in the version hash, and what goes in the precache. They are not
 * the same list, and the difference is the whole point.
 *
 * Everything in the build feeds the hash, the engine and rule corpus
 * included: they are served cache-first under a cache named for that hash, so
 * if they were left out, a release that changed only the corpus or only the
 * WebAssembly would keep the old cache and go on serving the old engine to
 * every returning reader.
 *
 * The engine stays out of the precache, because installing 8 MB up front
 * would make a first visit slow and could fail outright.
 */
export function buildManifest(outDir: string, base: string): { version: string; precache: string[] } {
  const all = files(outDir).filter((f) => f !== "sw.js" && !f.endsWith(".map"));
  const precache = all.filter((f) => !f.startsWith("engine/")).map((f) => base + f).sort();
  const hash = createHash("sha256");
  for (const f of all.sort()) hash.update(f).update(readFileSync(join(outDir, f)));
  return { version: hash.digest("hex").slice(0, 16), precache };
}

function files(dir: string, root = dir): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? files(p, root) : [relative(root, p).split("\\").join("/")];
  });
}

const SW = (version: string, precache: string[], base: string) => `// Generated at build time. Do not edit.
const VERSION = ${JSON.stringify(version)};
const CACHE = "lagn-shell-" + VERSION;
const PRECACHE = ${JSON.stringify(precache)};
const BASE = ${JSON.stringify(base)};

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
  if (url.origin !== self.location.origin || url.pathname.startsWith(BASE + "api/") || e.request.method !== "GET") return;
  if (e.request.mode === "navigate") {
    // Network first for the page itself, falling back to the cached shell offline.
    e.respondWith(fetch(e.request).catch(() => caches.match(BASE + "index.html", { ignoreVary: true })));
    return;
  }
  // The engine and its reference data are large and never change under the
  // same build: cache them the first time they are fetched, so the next visit
  // - and every offline one - is served locally.
  if (url.pathname.startsWith(BASE + "engine/")) {
    e.respondWith(caches.open(CACHE).then(async (c) => {
      const hit = await c.match(e.request, { ignoreVary: true });
      if (hit) return hit;
      const response = await fetch(e.request);
      if (response.ok) c.put(e.request, response.clone());
      return response;
    }));
    return;
  }
  // Hashed assets never change content under the same name: cache first.
  // ignoreVary: a precached copy must match whatever headers (Origin, for
  // module scripts) the browser's own request carries (QA-P5-1).
  e.respondWith(caches.match(e.request, { ignoreVary: true }).then((hit) => hit || fetch(e.request)));
});
`;

export { cspFor };

export function pwa(apiBase: string, base = "/", counterOrigin = ""): Plugin {
  let outDir = "dist";
  return {
    name: "lagn-pwa",
    configResolved(c) { outDir = c.build.outDir; },
    transformIndexHtml: (html) => html.replace(
      "<!--lagn:csp-->",
      `<meta http-equiv="Content-Security-Policy" content="${cspFor(apiBase, counterOrigin)}" />`,
    ),
    closeBundle() {
      // start_url and the icon paths are absolute in the source manifest;
      // under a path prefix they must carry it, or the installed app opens
      // the wrong URL and shows no icon.
      if (base !== "/") {
        const file = join(outDir, "manifest.webmanifest");
        const manifest = JSON.parse(readFileSync(file, "utf8")) as {
          start_url: string;
          scope?: string;
          icons: Array<{ src: string }>;
        };
        manifest.start_url = base;
        manifest.scope = base;
        for (const icon of manifest.icons) icon.src = base + icon.src.replace(/^\//, "");
        writeFileSync(file, JSON.stringify(manifest, null, 2));
      }
      const { version, precache } = buildManifest(outDir, base);
      writeFileSync(join(outDir, "sw.js"), SW(version, precache, base));
    },
  };
}
