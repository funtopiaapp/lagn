/// <reference types="vitest/config" />
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { pwa } from "./pwa-plugin";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "VITE_");
  // A GitHub Pages project site is served from /<repo>/, not from the root.
  const base = process.env.BASE_URL ?? "/";
  return {
    base,
    plugins: [react(), pwa(env.VITE_API_BASE ?? "", base)],
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
