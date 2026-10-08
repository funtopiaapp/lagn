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

// The engine's own version, and why it is on every engine URL.
//
// The service worker caches engine/* cache-first and deliberately does not
// skipWaiting, so after a release the previous worker is still active for the
// rest of the session. index.html is network-first and the JS bundle's name is
// content-hashed, so both arrive new - but the engine's filenames are stable,
// so the old worker answered them from its own cache. New JavaScript then
// called a WebAssembly export that build did not have and failed with
// "not a function". This is what stops that.

import { engineVersion } from "../pwa-plugin";
import { REQUIRED } from "./lib/wasm";
// The interface as source, so the test reads the contract rather than a copy.
import nativeSrc from "./lib/native.ts?raw";

describe("the engine's version", () => {
  it("changes when the WebAssembly changes", () => {
    mkdirSync(join(dir, "engine"), { recursive: true });
    writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm one");
    const before = engineVersion(join(dir, "engine"));
    writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm two");
    expect(engineVersion(join(dir, "engine"))).not.toBe(before);
  });

  it("changes when only the rule corpus changes", () => {
    // A corpus-only release still changes what the engine answers, so a
    // reader holding the old copy would read the old rules.
    mkdirSync(join(dir, "engine", "corpus"), { recursive: true });
    writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm");
    writeFileSync(join(dir, "engine", "corpus", "karma.json"), '{"rules":1}');
    const before = engineVersion(join(dir, "engine"));
    writeFileSync(join(dir, "engine", "corpus", "karma.json"), '{"rules":2}');
    expect(engineVersion(join(dir, "engine"))).not.toBe(before);
  });

  it("is stable when the engine is untouched", () => {
    mkdirSync(join(dir, "engine"), { recursive: true });
    writeFileSync(join(dir, "engine", "lagn.wasm"), "\0asm");
    expect(engineVersion(join(dir, "engine"))).toBe(engineVersion(join(dir, "engine")));
  });

  it("is a stable placeholder when no engine is staged", () => {
    // A dev build with no engine must not bust its cache on every reload.
    expect(engineVersion(join(dir, "absent"))).toBe("dev");
    mkdirSync(join(dir, "empty"), { recursive: true });
    expect(engineVersion(join(dir, "empty"))).toBe("dev");
  });
});

describe("the loader's required-export list", () => {
  it("covers every method the bridge promises", () => {
    // The bug this pins: a new engine call was added to the bridge, and a
    // reader whose cached engine predated it got "not a function" from
    // whichever feature they opened. Checking the list against the interface
    // means the guard cannot fall behind the bridge again.
    const iface = nativeSrc.slice(nativeSrc.indexOf("export interface NativeBridge"));
    const methods = [...iface.slice(0, iface.indexOf("}")).matchAll(/^\s*(\w+)\(/gm)].map((m) => m[1]);

    expect(methods).toContain("jaimini");
    expect(methods).toContain("chara");
    for (const m of methods) {
      expect(REQUIRED).toContain(`lagn_${m}`);
    }
    // And the init call, which no bridge method exposes but every load needs.
    expect(REQUIRED).toContain("lagn_init");
  });
});
