// Run the WebAssembly build outside a browser, to prove the module works
// before any of the web plumbing exists.
//
//   node scripts/wasm-smoke.mjs
//
// The engine reads its reference data as files, so the host gives it a
// filesystem: here Node's WASI with the project directory preopened. In a
// browser the same module gets a virtual filesystem instead.

import { readFile } from "node:fs/promises";
import { WASI } from "node:wasi";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const wasmPath = join(root, "target/wasm32-wasip1/release/lagn_ffi.wasm");

const wasi = new WASI({
  version: "preview1",
  args: [],
  env: {},
  // The module sees the project as /lagn.
  preopens: { "/lagn": root },
});

const module = await WebAssembly.compile(await readFile(wasmPath));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
wasi.initialize(instance);

const e = instance.exports;
const memory = e.memory;

/** Put a JS string into the module's memory as NUL-terminated UTF-8. */
function put(text) {
  const bytes = new TextEncoder().encode(text);
  const ptr = e.lagn_buffer_alloc(bytes.length + 1);
  const view = new Uint8Array(memory.buffer, ptr, bytes.length + 1);
  view.set(bytes);
  view[bytes.length] = 0;
  return { ptr, len: bytes.length + 1 };
}

/** Read a string the engine returned, then release it. */
function take(ptr) {
  if (ptr === 0) throw new Error("the engine returned null");
  const bytes = new Uint8Array(memory.buffer, ptr);
  const end = bytes.indexOf(0);
  const text = new TextDecoder().decode(bytes.subarray(0, end));
  e.lagn_string_free(ptr);
  return text;
}

/** Call an export that takes one JSON string and returns one. */
function call(name, json) {
  const arg = put(json);
  try {
    return JSON.parse(take(e[name](arg.ptr)));
  } finally {
    e.lagn_buffer_free(arg.ptr, arg.len);
  }
}

console.log("\nlagn, running as WebAssembly\n");
console.log("  exports:", Object.keys(e).filter((k) => k.startsWith("lagn_")).length, "functions");

const t0 = performance.now();
const init = call("lagn_init", JSON.stringify({
  ephemeris: "/lagn/ephe",
  corpus: "/lagn/corpus",
  places: "/lagn/data/places.tsv",
  // No tzdb path: the copy compiled into the module is used. Pointing jiff at
  // a zoneinfo directory sends it into a runaway walk under WASI, and the
  // bundled copy saves shipping 1.4 MB to a browser anyway.
}));
if (init.error) {
  console.error("  init failed:", init.error);
  process.exit(1);
}
console.log(`  loaded in ${(performance.now() - t0).toFixed(0)} ms, ready = ${!!e.lagn_ready()}`);
console.log(`  engine ${init.version.engine}, Swiss Ephemeris ${init.version.swiss_ephemeris}`);
console.log(`  tz database: ${init.version.tzdb}`);
console.log(`  corpus: ${init.version.corpus.rules} rules, ${init.version.places} places`);

const birth = {
  date: "1985-06-21", time: "14:30:00",
  latitude: 13.08, longitude: 80.27, utc_offset_hours: 5.5,
};

const c0 = performance.now();
const chart = call("lagn_chart", JSON.stringify({ birth }));
console.log(`\n  chart: lagna ${chart.lagna.rasi}, ${chart.positions.length} grahas, in ${(performance.now() - c0).toFixed(0)} ms`);

const topics = JSON.parse(take(e.lagn_topics()));
console.log(`\n  readings (${topics.topics.length} topics)`);
let total = 0;
for (const t of topics.topics) {
  const name = put(t.id);
  const body = put(JSON.stringify({ birth }));
  const t1 = performance.now();
  const reading = JSON.parse(take(e.lagn_topic(name.ptr, body.ptr)));
  total += performance.now() - t1;
  e.lagn_buffer_free(name.ptr, name.len);
  e.lagn_buffer_free(body.ptr, body.len);
  const summary = reading.writeup?.summary?.[0] ?? reading.error ?? "";
  console.log(`    ${t.id.padEnd(12)} ${summary.slice(0, 96)}${summary.length > 96 ? "..." : ""}`);
}
console.log(`\n    ${topics.topics.length} readings in ${total.toFixed(0)} ms`);

const p0 = performance.now();
const periods = call("lagn_periods", JSON.stringify({ birth, from_age: 20, to_age: 45 }));
console.log(`\n  sensitive periods: ${periods.windows.length} windows with transits, in ${(performance.now() - p0).toFixed(0)} ms`);

// The professional surface runs through the same engine, so it has to answer
// in the browser too and not only on the server.
const jaimini = call("lagn_jaimini", JSON.stringify({ birth }));
const ak = jaimini.karakas.find((a) => a.id === "atma");
const moved = jaimini.padas.filter((x) => x.adjusted).length;
console.log(`  jaimini: AK is ${ak.graha} in ${ak.rasi}; ${jaimini.padas.length} padas (${moved} took the 10th); ${jaimini.variants.length} variants named`);
// The shape the app actually reads, checked field by field. A stale engine
// that still returns the pre-view shape, or raw enum identifiers instead of
// names, has to fail here rather than in the browser.
if (!Array.isArray(jaimini.karakas) || jaimini.karakas.length !== 8
  || jaimini.padas.length !== 12 || jaimini.argala.length !== 12
  || jaimini.variants.length !== 8) {
  throw new Error("jaimini came back the wrong shape from the WebAssembly engine");
}
for (const f of ["id", "abbrev", "name", "signifies", "graha", "rasi", "advancement"]) {
  if (ak[f] === undefined) throw new Error(`jaimini karaka is missing ${f}`);
}
if (!/^[A-Z]/.test(ak.graha) || !/^[A-Z]/.test(ak.rasi)) {
  throw new Error(`jaimini returned identifiers rather than names: ${ak.graha} in ${ak.rasi}`);
}
if (jaimini.padas[0].label !== "AL" || jaimini.padas[11].label !== "UL") {
  throw new Error("jaimini padas are not labelled the way practice names them");
}

const chara = call("lagn_chara", JSON.stringify({ birth }));
const cycle = chara.lengths.reduce((a, l) => a + l.years, 0);
console.log(`  chara: lagna ${chara.lagna}, runs ${chara.direction}, ${cycle}-year cycle, ${chara.periods.length} periods`);
if (chara.lengths.length !== 12 || chara.variants.length !== 5 || chara.periods.length % 12 !== 0) {
  throw new Error("chara dasha came back the wrong shape from the WebAssembly engine");
}

const upa = call("lagn_upagraha", JSON.stringify({ birth }));
const gulika = upa.day_parts.find((p) => p.name === "Gulika");
console.log(`  upagrahas: ${upa.at_night ? "night" : "day"} birth, Gulika in ${gulika.rasi} (part ${gulika.part}), ${upa.time_lagnas.length} time lagnas`);
if (upa.sun_offsets.length !== 5 || upa.day_parts.length !== 5 || upa.time_lagnas.length !== 3
  || upa.variants.length !== 4 || !/^[A-Z]/.test(gulika.rasi)) {
  throw new Error("upagrahas came back the wrong shape from the WebAssembly engine");
}

const kp = call("lagn_kp", JSON.stringify({ birth }));
const subs = new Set(kp.cusps.map((c) => c.sub_lord));
console.log(`  kp: asc sub ${kp.ascendant.sub_lord}, ${kp.cusps.length} cusps (${subs.size} distinct sub lords), ${kp.ruling_planets.length} ruling planets`);
if (kp.cusps.length !== 12 || kp.grahas.length !== 9 || kp.ruling_planets.length !== 5
  || kp.significators.length !== 12 || kp.variants.length !== 4
  || !/^[A-Z]/.test(kp.ascendant.sub_lord)) {
  throw new Error("kp came back the wrong shape from the WebAssembly engine");
}
// Every house's significators are ranked strongest first.
for (const h of kp.significators) {
  const ranks = h.significators.map((s) => s.rank);
  if (ranks.join() !== [...ranks].sort().join()) {
    throw new Error(`kp house ${h.house} significators are not ranked`);
  }
}

const varsha = call("lagn_varsha", JSON.stringify({ birth, age: 45 }));
console.log(`  annual chart: year 45 begins ${varsha.begins}, Muntha in ${varsha.muntha} (house ${varsha.muntha_house}), Sun within ${varsha.sun_error.toExponential(1)} deg`);
if (varsha.positions.length !== 9 || varsha.kaksha.length !== 7
  || varsha.variants.length !== 4 || varsha.sun_error > 1e-6) {
  throw new Error("the annual chart came back wrong from the WebAssembly engine");
}

const places = call("lagn_places", JSON.stringify({ q: "Chennai", limit: 3 }));
console.log(`  place search: ${places.map((p) => p.name).join(", ")}`);

const bad = call("lagn_chart", JSON.stringify({ birth: { ...birth, latitude: 999 } }));
console.log(`  error handling: ${bad.error}`);

console.log(`\n  module: ${((await readFile(wasmPath)).length / 1024 / 1024).toFixed(1)} MB\n`);
