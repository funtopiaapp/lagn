import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { installNativeBridge } from "./lib/capacitor";
import "./styles.css";

// Inside the iOS or Android wrapper the engine is embedded: install the bridge
// before the first render so no reading goes looking for a server. In a
// browser this does nothing.
installNativeBridge();

createRoot(document.getElementById("root")!).render(<StrictMode><App /></StrictMode>);

// The service worker exists only in production builds (see pwa-plugin.ts).
if (import.meta.env.PROD && "serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => { /* the app works without it */ });
  });
}
