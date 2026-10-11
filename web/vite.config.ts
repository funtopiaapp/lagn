/// <reference types="vitest/config" />
import { fileURLToPath } from "node:url";
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { engineVersion, pwa } from "./pwa-plugin";

/** The origin of the visit-counter badge, for the Content-Security-Policy. */
function originOf(url?: string): string {
  try {
    return url ? new URL(url).origin : "";
  } catch {
    return "";
  }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "VITE_");
  // A GitHub Pages project site is served from /<repo>/, not from the root.
  const base = process.env.BASE_URL ?? "/";
  // Compiled into the app and appended to every engine URL, so a release that
  // changes the engine cannot be served a stale copy by the previous service
  // worker. See engineVersion() for why that was happening.
  // Resolved against this file, not the working directory, so the version is
  // the same whoever runs the build and from wherever.
  const engine = engineVersion(fileURLToPath(new URL("public/engine", import.meta.url)));
  // The owner's kill switch, as a literal the bundler can fold. Unset means
  // on, so forgetting it can never remove features; only "off" (or 0/false/no)
  // turns the professional surface off, and then its chunk is not emitted at
  // all. See docs/INTEGRATION.md.
  const proRaw = (process.env.VITE_PRO ?? env.VITE_PRO ?? "").trim().toLowerCase();
  const proBuild = !["off", "0", "false", "no"].includes(proRaw);
  return {
    base,
    define: {
      __ENGINE_VERSION__: JSON.stringify(engine),
      PRO_BUILD: JSON.stringify(proBuild),
    },
    plugins: [react(), pwa(env.VITE_API_BASE ?? "", base, originOf(env.VITE_COUNTER_BADGE))],
    server: {
      // During development the API runs separately: `lagn-server --addr 127.0.0.1:8080`.
      proxy: { "/api": "http://127.0.0.1:8080" },
    },
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: ["./src/test/setup.ts"],
      include: ["src/**/*.test.{ts,tsx}"],
    },
  };
});
