// Loads the engine as WebAssembly and installs it as `window.Lagn`, so the
// app computes every reading in the browser with no server at all.
//
// The module and its reference data are fetched once and cached by the service
// worker, after which the app works offline. If anything here fails - the
// files are not deployed, the browser is ancient - nothing is installed and
// the app falls back to HTTP, which is why every step returns rather than
// throwing.
//
// Specification: docs/phase12/DESIGN.md.

import type { NativeBridge } from "./native";
import { Wasi, type Files } from "./wasi";

/** Where the build script puts the engine, relative to the app's base URL. */
const ENGINE = "engine/";

/**
 * Compiled in by vite.config.ts: a content hash of the staged engine.
 *
 * Every engine URL carries it as `?v=`, because the service worker caches
 * `engine/*` cache-first and does not skipWaiting. After a release the old
 * worker is still active, and the engine's filenames are stable, so it would
 * answer with the engine from the previous build while the new JavaScript -
 * fetched fresh, because its filename is content-hashed - called an export
 * that build did not have. Changing the URL is what makes the old cache miss.
 */
declare const __ENGINE_VERSION__: string;
const VERSION = typeof __ENGINE_VERSION__ === "string" ? __ENGINE_VERSION__ : "dev";

/** An engine URL, carrying the engine's version. */
function url(root: string, path: string): string {
  return `${root}${path}?v=${encodeURIComponent(VERSION)}`;
}

/**
 * Exports the app calls. Checked after instantiation so a stale module fails
 * with something a reader can act on, instead of "not a function" from
 * whichever feature happens to be opened first.
 */
export const REQUIRED = [
  "lagn_init", "lagn_version", "lagn_topics", "lagn_chart", "lagn_topic",
  "lagn_periods", "lagn_days", "lagn_family", "lagn_match", "lagn_places",
  "lagn_offset", "lagn_jaimini", "lagn_chara", "lagn_upagraha", "lagn_kp", "lagn_varsha",
] as const;

/** The files the engine reads, and where the module sees them. */
const MANIFEST = "manifest.json";

interface Manifest {
  /** Guest paths, e.g. "ephe/semo_18.se1", to fetch from ENGINE. */
  files: string[];
  wasm: string;
  /** Guest mount point, e.g. "/lagn". */
  root: string;
}

interface Engine {
  exports: {
    memory: WebAssembly.Memory;
    lagn_init(ptr: number): number;
    lagn_ready(): number;
    lagn_version(): number;
    lagn_topics(): number;
    lagn_chart(ptr: number): number;
    lagn_topic(name: number, request: number): number;
    lagn_periods(ptr: number): number;
    lagn_days(ptr: number): number;
    lagn_jaimini(ptr: number): number;
    lagn_chara(ptr: number): number;
    lagn_upagraha(ptr: number): number;
    lagn_kp(ptr: number): number;
    lagn_varsha(ptr: number): number;
    lagn_family(ptr: number): number;
    lagn_match(ptr: number): number;
    lagn_places(ptr: number): number;
    lagn_offset(ptr: number): number;
    lagn_string_free(ptr: number): void;
    lagn_buffer_alloc(len: number): number;
    lagn_buffer_free(ptr: number, len: number): void;
  };
}

function base(): string {
  // Works under a path prefix, which GitHub Pages uses for project sites.
  return new URL(ENGINE, document.baseURI).href;
}

/** Put a string into the module's memory as NUL-terminated UTF-8. */
function put(e: Engine["exports"], text: string): { ptr: number; len: number } {
  const bytes = new TextEncoder().encode(text);
  const len = bytes.length + 1;
  const ptr = e.lagn_buffer_alloc(len);
  if (ptr === 0) throw new Error("the engine could not allocate memory");
  const view = new Uint8Array(e.memory.buffer, ptr, len);
  view.set(bytes);
  view[bytes.length] = 0;
  return { ptr, len };
}

/** Read a string the engine returned, and release it. */
function take(e: Engine["exports"], ptr: number): string {
  if (ptr === 0) throw new Error("the engine returned nothing");
  const bytes = new Uint8Array(e.memory.buffer, ptr);
  const end = bytes.indexOf(0);
  const text = new TextDecoder().decode(bytes.subarray(0, end));
  e.lagn_string_free(ptr);
  return text;
}

/** One call: JSON in, JSON out, freeing both sides. */
function call(e: Engine["exports"], fn: (...a: number[]) => number, args: string[]): string {
  const buffers = args.map((a) => put(e, a));
  try {
    return take(e, fn(...buffers.map((b) => b.ptr)));
  } finally {
    for (const b of buffers) e.lagn_buffer_free(b.ptr, b.len);
  }
}

async function fetchEngine(): Promise<{ engine: Engine["exports"]; bytes: number } | null> {
  const root = base();
  const manifestResponse = await fetch(url(root, MANIFEST));
  if (!manifestResponse.ok) return null;
  const manifest: Manifest = await manifestResponse.json();

  const downloads = await Promise.all(
    manifest.files.map(async (path) => {
      const r = await fetch(url(root, path));
      if (!r.ok) throw new Error(`missing engine file: ${path}`);
      return [path, new Uint8Array(await r.arrayBuffer())] as const;
    }),
  );
  const files: Files = new Map(downloads);
  let bytes = downloads.reduce((n, [, d]) => n + d.length, 0);

  const wasmResponse = await fetch(url(root, manifest.wasm));
  if (!wasmResponse.ok) return null;
  const wasmBytes = await wasmResponse.arrayBuffer();
  bytes += wasmBytes.byteLength;

  const wasi = new Wasi({ root: manifest.root, files });
  const { instance } = await WebAssembly.instantiate(wasmBytes, wasi.imports);
  const exports = instance.exports as unknown as Engine["exports"];

  // A module missing an export the app calls is a stale module, not a usable
  // one. Saying so here names the cause; letting it through would surface as
  // "not a function" inside whichever feature was opened.
  const missing = REQUIRED.filter((n) => typeof (exports as unknown as Record<string, unknown>)[n] !== "function");
  if (missing.length > 0) {
    throw new Error(
      `the engine on this device is out of date (missing ${missing.join(", ")}); reload the page to update it`,
    );
  }

  wasi.bind(exports.memory);

  // No tzdb path: the copy compiled into the module is used. Pointing the
  // timezone library at a directory makes it walk away under WASI, and the
  // built-in copy saves the browser 1.4 MB.
  const config = JSON.stringify({
    ephemeris: `${manifest.root}/ephe`,
    corpus: `${manifest.root}/corpus`,
    places: `${manifest.root}/data/places.tsv`,
  });
  const result = JSON.parse(call(exports, exports.lagn_init, [config]));
  if (result.error) throw new Error(String(result.error));
  return { engine: exports, bytes };
}

/**
 * Load the engine and install it as `window.Lagn`. Resolves to the number of
 * bytes fetched, or null when the engine is not deployed and HTTP should be
 * used instead.
 */
export async function installWasmEngine(): Promise<number | null> {
  try {
    const loaded = await fetchEngine();
    if (!loaded) return null;
    const e = loaded.engine;
    const bridge: NativeBridge = {
      version: async () => take(e, e.lagn_version()),
      topics: async () => take(e, e.lagn_topics()),
      chart: async (request) => call(e, e.lagn_chart, [request]),
      topic: async (name, request) => call(e, e.lagn_topic, [name, request]),
      periods: async (request) => call(e, e.lagn_periods, [request]),
      days: async (request) => call(e, e.lagn_days, [request]),
      jaimini: async (request) => call(e, e.lagn_jaimini, [request]),
      chara: async (request) => call(e, e.lagn_chara, [request]),
      upagraha: async (request) => call(e, e.lagn_upagraha, [request]),
      kp: async (request) => call(e, e.lagn_kp, [request]),
      varsha: async (request) => call(e, e.lagn_varsha, [request]),
      family: async (request) => call(e, e.lagn_family, [request]),
      match: async (request) => call(e, e.lagn_match, [request]),
      places: async (request) => call(e, e.lagn_places, [request]),
      offset: async (request) => call(e, e.lagn_offset, [request]),
    };
    window.Lagn = bridge;
    return loaded.bytes;
  } catch (error) {
    // A missing or broken engine must not take the app down with it.
    console.warn("the local engine could not be loaded; using the server instead", error);
    return null;
  }
}
