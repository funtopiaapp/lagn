// The service worker serves /engine/* cache-first, under a cache named for the
// build's version hash. So what feeds that hash decides whether a release
// reaches anyone who has visited before.
//
// This existed as a bug: the hash excluded engine/ entirely, so a release that
// changed only the rule corpus or only the WebAssembly produced the same
// version, kept the same cache, and went on serving the old engine for ever.
// Every content-only release was invisible to returning readers.

import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { buildManifest } from "../pwa-plugin";

let dir: string;

/** A build tree with the shape the real one has. */
function build(engineCorpus: string, js = "console.log(1)") {
  mkdirSync(join(dir, "assets"), { recursive: true });
  mkdirSync(join(dir, "engine", "corpus"), { recursive: true });
  writeFileSync(join(dir, "index.html"), "<!doctype html>");
  writeFileSync(join(dir, "assets", "index-abc.js"), js);
  writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm fake");
  writeFileSync(join(dir, "engine", "corpus", "karma.json"), engineCorpus);
}

beforeEach(() => { dir = mkdtempSync(join(tmpdir(), "lagn-sw-")); });
afterEach(() => rmSync(dir, { recursive: true, force: true }));

describe("the service worker's version hash", () => {
  it("changes when only the rule corpus changes", () => {
    build('{"rules":1}');
    const before = buildManifest(dir, "/").version;
    build('{"rules":2}');
    const after = buildManifest(dir, "/").version;
    expect(after).not.toBe(before);
  });

  it("changes when only the WebAssembly changes", () => {
    build('{"rules":1}');
    const before = buildManifest(dir, "/").version;
    writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm different");
    const after = buildManifest(dir, "/").version;
    expect(after).not.toBe(before);
  });

  it("changes when the app's own code changes", () => {
    build('{"rules":1}', "console.log(1)");
    const before = buildManifest(dir, "/").version;
    build('{"rules":1}', "console.log(2)");
    expect(buildManifest(dir, "/").version).not.toBe(before);
  });

  it("is stable when nothing changes", () => {
    build('{"rules":1}');
    expect(buildManifest(dir, "/").version).toBe(buildManifest(dir, "/").version);
  });

  it("still keeps the engine out of the precache list", () => {
    build('{"rules":1}');
    const { precache } = buildManifest(dir, "/");
    // 8 MB at install would make a first visit slow and could fail outright,
    // so the engine is fetched on demand and cached then.
    expect(precache.some((p) => p.includes("engine/"))).toBe(false);
    expect(precache).toContain("/index.html");
    expect(precache).toContain("/assets/index-abc.js");
  });

  it("honours a base path, for a project site served from a subdirectory", () => {
    build('{"rules":1}');
    const { precache } = buildManifest(dir, "/lagn/");
    expect(precache).toContain("/lagn/index.html");
  });
});
