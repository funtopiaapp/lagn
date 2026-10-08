// The native bridge, when the app is running on a device with the engine
// embedded. The wrapper (a Capacitor plugin on iOS and Android) puts an object
// on `window.Lagn` whose methods take and return the same JSON the HTTP API
// uses, because both call the same Rust functions.
//
// When it is absent - a browser, or the dev server - everything falls back to
// fetch(), and nothing else in the app knows the difference.

/** What the wrapper must provide. Every method resolves to a JSON string. */
export interface NativeBridge {
  version(): Promise<string>;
  topics(): Promise<string>;
  chart(requestJson: string): Promise<string>;
  topic(name: string, requestJson: string): Promise<string>;
  periods(requestJson: string): Promise<string>;
  days(requestJson: string): Promise<string>;
  jaimini(requestJson: string): Promise<string>;
  chara(requestJson: string): Promise<string>;
  family(requestJson: string): Promise<string>;
  match(requestJson: string): Promise<string>;
  places(requestJson: string): Promise<string>;
  offset(requestJson: string): Promise<string>;
}

declare global {
  interface Window {
    Lagn?: Partial<NativeBridge>;
  }
}

/** The bridge, if this build is running inside the native app. */
export function bridge(): NativeBridge | null {
  const w = typeof window === "undefined" ? undefined : window.Lagn;
  if (!w) return null;
  // A half-implemented bridge is worse than none: fall back rather than fail
  // on the one call that happens to be missing.
  const needed: (keyof NativeBridge)[] = [
    "version", "topics", "chart", "topic", "periods", "family", "match", "places", "offset",
  ];
  return needed.every((m) => typeof w[m] === "function") ? (w as NativeBridge) : null;
}

export function isNative(): boolean {
  return bridge() !== null;
}
