// Installs `window.Lagn` from the native plugin when the app is running
// inside the iOS or Android wrapper.
//
// The wrapper exposes a Capacitor plugin called LagnNative whose methods take
// and return JSON strings. Capacitor wraps every result in an object, so a
// call resolves to `{ value: "<json>" }`; this adapts that to the plain
// string-returning contract in `native.ts`.
//
// In a browser there is no `window.Capacitor`, nothing is installed, and the
// app uses HTTP exactly as before. Nothing here imports @capacitor/core, so
// the browser bundle carries no extra dependency.

import type { NativeBridge } from "./native";

interface PluginResult {
  value?: string;
}

interface CapacitorPlugin {
  /** Optional warm-up; every method loads the data anyway. */
  prepare?(): Promise<PluginResult>;
  version(): Promise<PluginResult>;
  topics(): Promise<PluginResult>;
  chart(options: { request: string }): Promise<PluginResult>;
  topic(options: { name: string; request: string }): Promise<PluginResult>;
  periods(options: { request: string }): Promise<PluginResult>;
  days(options: { request: string }): Promise<PluginResult>;
  jaimini(options: { request: string }): Promise<PluginResult>;
  chara(options: { request: string }): Promise<PluginResult>;
  upagraha(options: { request: string }): Promise<PluginResult>;
  family(options: { request: string }): Promise<PluginResult>;
  match(options: { request: string }): Promise<PluginResult>;
  places(options: { request: string }): Promise<PluginResult>;
  offset(options: { request: string }): Promise<PluginResult>;
}

declare global {
  interface Window {
    Capacitor?: { Plugins?: Record<string, unknown>; isNativePlatform?: () => boolean };
  }
}

function plugin(): CapacitorPlugin | null {
  const p = typeof window === "undefined" ? undefined : window.Capacitor?.Plugins?.LagnNative;
  return p && typeof (p as CapacitorPlugin).chart === "function" ? (p as CapacitorPlugin) : null;
}

/** Unwrap `{ value }`, and fail loudly rather than returning undefined. */
async function text(call: Promise<PluginResult>): Promise<string> {
  const r = await call;
  if (typeof r?.value !== "string") {
    throw new Error("The native plugin returned no value.");
  }
  return r.value;
}

/**
 * Called once at start-up. Returns true when the native engine is in use.
 *
 * The plugin loads the reference data on the native side, lazily, so no paths
 * cross this boundary and no call can arrive before it is ready. The warm-up
 * below only moves that cost off the first reading.
 */
export function installNativeBridge(): boolean {
  const p = plugin();
  if (!p) return false;
  const bridge: NativeBridge = {
    version: () => text(p.version()),
    topics: () => text(p.topics()),
    chart: (request) => text(p.chart({ request })),
    topic: (name, request) => text(p.topic({ name, request })),
    periods: (request) => text(p.periods({ request })),
    days: (request) => text(p.days({ request })),
    jaimini: (request) => text(p.jaimini({ request })),
    chara: (request) => text(p.chara({ request })),
    upagraha: (request) => text(p.upagraha({ request })),
    family: (request) => text(p.family({ request })),
    match: (request) => text(p.match({ request })),
    places: (request) => text(p.places({ request })),
    offset: (request) => text(p.offset({ request })),
  };
  window.Lagn = bridge;
  // Load the data now rather than on the first reading. Failures surface on
  // the call itself, so nothing is thrown here.
  void p.prepare?.().catch(() => {});
  return true;
}
