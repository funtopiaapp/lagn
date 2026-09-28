import { defineConfig, devices } from "@playwright/test";

// Runs the real Rust server over the built app (npm run build and
// npm run build:xorigin first).
//   8799  lagn-server + dist, and the API for the cross-origin build
//   8800  dist-xorigin from another origin, listed in --allow-origin (a wrapped app)
//   8801  the same build from an origin NOT listed
export default defineConfig({
  testDir: "e2e",
  timeout: 60_000,
  fullyParallel: false,
  reporter: "list",
  use: { baseURL: "http://127.0.0.1:8799" },
  webServer: [
    {
      command: "../target/release/lagn-server --addr 127.0.0.1:8799 --static dist --ephe ../ephe --corpus ../corpus --places ../data/places.tsv --tzdb ../data/zoneinfo --review-token-file e2e/fixtures/review-token.txt --allow-origin http://127.0.0.1:8800",
      url: "http://127.0.0.1:8799/api/health",
      reuseExistingServer: false,
      timeout: 30_000,
    },
    { command: "python3 -m http.server 8800 --bind 127.0.0.1 --directory dist-xorigin", url: "http://127.0.0.1:8800/", reuseExistingServer: false },
    { command: "python3 -m http.server 8801 --bind 127.0.0.1 --directory dist-xorigin", url: "http://127.0.0.1:8801/", reuseExistingServer: false },
  ],
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "phone", use: { ...devices["Pixel 7"] } },
  ],
});
