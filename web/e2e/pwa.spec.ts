import { expect, test, type Page } from "@playwright/test";

async function enter(page: Page, place = "Chennai") {
  await page.fill('input[type="date"]', "1985-06-21");
  await page.fill('input[type="time"]', "14:30");
  await page.getByPlaceholder(/Town/).fill(place);
  await page.getByRole("button", { name: new RegExp(place) }).first().click();
  await expect(page.getByRole("radio", { name: /UTC\+05:30/ })).toBeChecked();
}

function pngSize(buf: Buffer) { return [buf.readUInt32BE(16), buf.readUInt32BE(20)]; }

test.describe("installable PWA", () => {
  test("Chromium reports the app installable, and every icon matches its manifest size", async ({ page, browserName }, info) => {
    test.skip(browserName !== "chromium" || info.project.name !== "desktop", "installability is a Chromium CDP check");
    await page.goto("/");
    const res = await page.request.get("/manifest.webmanifest");
    const m = await res.json();
    expect(m.display).toBe("standalone");
    expect(m.start_url).toBe("/");
    expect(m.icons.some((i: { purpose: string }) => i.purpose === "maskable")).toBe(true);
    for (const icon of m.icons) {
      const img = await page.request.get(icon.src);
      expect(img.ok(), icon.src).toBeTruthy();
      const [w, h] = pngSize(await img.body());
      expect(`${w}x${h}`).toBe(icon.sizes);
    }
    const apple = await page.request.get("/icons/apple-touch-icon.png");
    expect(pngSize(await apple.body())).toEqual([180, 180]);
    await page.waitForFunction(() => navigator.serviceWorker.controller !== null || navigator.serviceWorker.ready.then(() => true));
    const cdp = await page.context().newCDPSession(page);
    const { installabilityErrors } = await cdp.send("Page.getInstallabilityErrors");
    expect(installabilityErrors).toEqual([]);
  });

  test("the shell opens offline, the API is never answered from cache, and offline is explained", async ({ page, context }) => {
    page.on("console", (m) => { if (m.type() === "error") console.log("DIAG console", m.text()); });
    page.on("pageerror", (e) => console.log("DIAG pageerror", e.message));
    await page.goto("/");
    await page.evaluate(() => navigator.serviceWorker.ready);
    await page.reload();                       // now controlled by the worker
    expect(await page.evaluate(() => !!navigator.serviceWorker.controller)).toBe(true);
    // Compute once online, so an API response exists that a bad worker could cache.
    await enter(page);
    await page.getByRole("button", { name: "Compute chart" }).click();
    await expect(page.locator("svg.si-chart")).toBeVisible();
    const cached = await page.evaluate(async () => {
      const urls: string[] = [];
      for (const k of await caches.keys()) for (const r of await (await caches.open(k)).keys()) urls.push(new URL(r.url).pathname);
      return urls;
    });
    expect(cached.length).toBeGreaterThan(0);
    expect(cached.filter((u) => u.startsWith("/api/")), "API responses must never be cached").toEqual([]);

    await context.setOffline(true);
    const resp = await page.reload();
    if (!(await page.getByRole("heading", { name: "Lagn" }).isVisible())) {
      console.log("DIAG status", resp?.status(), "fromSW", resp?.fromServiceWorker(), "url", page.url());
      console.log("DIAG body", (await page.locator("body").innerText()).slice(0, 300));
      console.log("DIAG controller", await page.evaluate(() => !!navigator.serviceWorker.controller));
      console.log("DIAG head", (await page.content()).slice(0, 900));
      console.log("DIAG cache", JSON.stringify(await page.evaluate(async () => {
        const out: Record<string, string> = {};
        for (const s of Array.from(document.querySelectorAll<HTMLScriptElement>("script[src]"))) {
          const hit = await caches.match(s.src);
          out[s.src] = hit ? `hit ${hit.status}` : "MISS";
        }
        const keys: string[] = [];
        for (const k of await caches.keys()) keys.push(k);
        out["cache names"] = keys.join(",");
        return out;
      })));
    }
    await expect(page.getByRole("heading", { name: "Lagn" })).toBeVisible();
    await expect(page.locator(".offline")).toContainText("You're offline");
    // An API call offline must fail, not be served from a cache.
    const apiResult = await page.evaluate(() => fetch("/api/health").then(() => "answered", () => "failed"));
    expect(apiResult).toBe("failed");
    await context.setOffline(false);
  });
});

test("the app runs under its Content-Security-Policy with no violations", async ({ page }) => {
  const violations: string[] = [];
  page.on("console", (m) => { if (/Content Security Policy|CSP/i.test(m.text())) violations.push(m.text()); });
  await page.addInitScript(() => {
    document.addEventListener("securitypolicyviolation", (e) => console.error("CSP violation " + e.violatedDirective + " " + e.blockedURI));
  });
  const res = await page.goto("/");
  expect(res!.headers()["content-security-policy"]).toContain("default-src 'self'");
  await enter(page);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await expect(page.locator(".writeup .lead")).toBeVisible();
  await page.getByRole("button", { name: "Match", exact: true }).click();
  expect(violations).toEqual([]);
});

test("Back moves within the app: tab to tab, chart to form", async ({ page }) => {
  await page.goto("/");
  await enter(page);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Marriage" })).toBeVisible();
  await page.goBack();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Birth details" })).toBeVisible();
  await page.goForward();
  await expect(page.getByRole("heading", { name: "Birth details" })).toBeVisible(); // the chart is not resurrected from memory
});

test("phone ergonomics: 16px inputs and 44px touch targets", async ({ page }, info) => {
  test.skip(info.project.name !== "phone", "phone-width check");
  await page.goto("/");
  const vp = await page.evaluate(() => document.querySelector('meta[name="viewport"]')!.getAttribute("content"));
  expect(vp).toContain("viewport-fit=cover");
  await enter(page);
  const check = async (label: string) => {
    const bad = await page.evaluate(() => {
      const out: string[] = [];
      for (const el of Array.from(document.querySelectorAll<HTMLElement>("input, select, button, summary, .radio"))) {
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        const cs = getComputedStyle(el);
        const isRadio = el instanceof HTMLInputElement && (el.type === "radio" || el.type === "checkbox");
        if ((el.tagName === "INPUT" || el.tagName === "SELECT") && !isRadio && parseFloat(cs.fontSize) < 16) out.push(`font ${cs.fontSize} on ${el.outerHTML.slice(0, 60)}`);
        if (!isRadio && r.height < 44) out.push(`height ${r.height.toFixed(0)} on ${el.outerHTML.slice(0, 60)}`);
      }
      return out;
    });
    expect(bad, label).toEqual([]);
  };
  await check("birth form");
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  await check("chart view");
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await expect(page.locator(".writeup .lead")).toBeVisible();
  await check("marriage view");
  await page.getByRole("button", { name: "Sensitive periods", exact: true }).click();
  await page.getByRole("button", { name: "Show periods" }).click();
  await expect(page.locator("li.period-window").first()).toBeVisible();
  await check("periods view");
  const pad = await page.evaluate(() => getComputedStyle(document.body).paddingTop);
  expect(pad).toBeDefined();
});

test.describe("wrapped-app simulation: app and API on different origins", () => {
  test("an allowed origin computes a chart through the API's CORS", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto("http://127.0.0.1:8800/");
    const csp = await page.evaluate(() => document.querySelector('meta[http-equiv="Content-Security-Policy"]')!.getAttribute("content"));
    expect(csp).toContain("connect-src 'self' http://127.0.0.1:8799");
    await enter(page);
    const req = page.waitForRequest("http://127.0.0.1:8799/api/chart");
    await page.getByRole("button", { name: "Compute chart" }).click();
    await req;
    await expect(page.locator("tr.lagna-row td").nth(1)).toHaveText(/^10°20'/);
    expect(errors).toEqual([]);
  });

  test("an origin not on the list is refused by the browser", async ({ page }) => {
    await page.goto("http://127.0.0.1:8801/");
    await page.fill('input[type="date"]', "1985-06-21");
    await page.fill('input[type="time"]', "14:30");
    await page.getByPlaceholder(/Town/).fill("Chennai");
    // Place search is the first cross-origin call; CORS blocks it, so no results appear.
    await page.waitForTimeout(1500);
    await expect(page.getByRole("button", { name: /Chennai/ })).toHaveCount(0);
  });
});
