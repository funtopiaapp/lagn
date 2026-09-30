import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { installNativeBridge } from "./lib/capacitor";
import { beginTransportDecision } from "./lib/transport";
import { installWasmEngine } from "./lib/wasm";
import "./styles.css";

// Three ways to reach the engine, in order of preference:
//   1. the native plugin, inside the iOS or Android wrapper;
//   2. WebAssembly, when the engine is deployed beside the app;
//   3. HTTP, when neither is present.
// The UI never learns which: they all appear as window.Lagn or a fetch.
if (!installNativeBridge()) {
  // The UI renders at once; readings wait for this to settle.
  const decided = beginTransportDecision();
  void installWasmEngine().finally(decided);
}

createRoot(document.getElementById("root")!).render(<StrictMode><App /></StrictMode>);

// The service worker exists only in production builds (see pwa-plugin.ts).
if (import.meta.env.PROD && "serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    // Relative to the base URL: the app may live under a path prefix.
    navigator.serviceWorker.register(new URL("sw.js", document.baseURI)).catch(() => { /* the app works without it */ });
  });
}
