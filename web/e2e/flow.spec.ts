import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";


const TOKEN = readFileSync(new URL("./fixtures/review-token.txt", import.meta.url), "utf8").trim();

async function enter(page: Page, date: string, time: string, place: string, pick: RegExp) {
  await page.fill('input[type="date"]', date);
  await page.fill('input[type="time"]', time);
  await page.getByPlaceholder(/Town/).fill(place);
  await page.getByRole("button", { name: pick }).first().click();
}

/** The birth the UI will send, read back from the API's echo after computing. */
async function chartFromApi(page: Page, birth: unknown) {
  const res = await page.request.post("/api/chart", { data: { birth } });
  expect(res.ok()).toBeTruthy();
  return res.json();
}


// Most of these tests exercise the HTTP transport. When the local engine is
// deployed beside the app it answers instead, and no request is made - so the
// engine is withheld here, and the tests that cover it opt back in.
test.beforeEach(async ({ page }) => {
  await page.route("**/engine/**", (route) => route.abort());
});

test("the rendered chart equals the API's output, cell for cell", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Madras", /Chennai/);
  await expect(page.getByText("Reading this as: 21 June 1985, 14:30:00 (24-hour clock)")).toBeVisible();
  await expect(page.getByRole("radio", { name: /UTC\+05:30/ })).toBeChecked();

  const req = page.waitForRequest("/api/chart");
  await page.getByRole("button", { name: "Compute chart" }).click();
  const sent = JSON.parse((await req).postData()!).birth;
  expect(sent).toMatchObject({ date: "1985-06-21", time: "14:30:00", utc_offset_hours: 5.5 });
  const api = await chartFromApi(page, sent);

  // Positions table: every graha row equals the API's display strings.
  for (const p of api.positions) {
    const row = page.locator(`tr[data-graha="${p.key}"]`);
    await expect(row.locator("td").nth(0)).toContainText(p.rasi);
    await expect(row.locator("td").nth(1)).toHaveText(p.degrees);
    await expect(row.locator("td").nth(2)).toHaveText(String(p.house));
    await expect(row.locator("td").nth(3)).toContainText(p.nakshatra);
    await expect(row.locator("td").nth(4)).toHaveText(String(p.pada));
  }
  await expect(page.locator("tr.lagna-row td").nth(1)).toHaveText(api.lagna.degrees);

  // D-1 square chart: each graha drawn in the sign the API says.
  for (const cell of api.vargas[0].grahas) {
    await expect(page.locator(`svg.si-chart g[data-sign="${cell.sign}"]`)).toContainText(cell.abbrev);
  }
  await expect(page.locator(`svg.si-chart g[data-lagna="true"]`)).toHaveAttribute("data-sign", api.vargas[0].lagna);

  // Every divisional chart when selected.
  for (const v of api.vargas.slice(1)) {
    await page.getByRole("combobox", { name: "Chart" }).selectOption(v.varga);
    await expect(page.locator(`svg.si-chart g[data-lagna="true"]`)).toHaveAttribute("data-sign", v.lagna);
    for (const cell of v.grahas) {
      await expect(page.locator(`svg.si-chart g[data-sign="${cell.sign}"]`)).toContainText(cell.abbrev);
    }
  }

  // Dasha: mahadasha dates as the engine formatted them.
  const lines = page.locator("ul.dasha > li > .period-line");
  await expect(lines).toHaveCount(api.dasha.mahadashas.length);
  for (const [i, m] of api.dasha.mahadashas.entries()) {
    await expect(lines.nth(i)).toContainText(`${m.lord}`);
    await expect(lines.nth(i)).toContainText(`${m.start} → ${m.end}`);
  }
  // Ashtakavarga totals.
  await expect(page.locator("table.av tr.sav td").nth(12)).toHaveText(String(api.ashtakavarga.sav_total));
  expect(errors).toEqual([]);
});

test("a historical birth must have its offset chosen explicitly", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1943-05-05", "14:00", "Chennai", /Chennai/);
  await expect(page.getByText(/Historical: please confirm/)).toBeVisible();
  const submit = page.getByRole("button", { name: "Compute chart" });
  await expect(submit).toBeDisabled();
  await page.getByRole("radio", { name: /UTC\+06:30/ }).check();
  await expect(submit).toBeEnabled();
  const req = page.waitForRequest("/api/chart");
  await submit.click();
  expect(JSON.parse((await req).postData()!).birth.utc_offset_hours).toBe(6.5);
  await expect(page.locator("svg.si-chart")).toBeVisible();
});

test("production shows only reviewed interpretations, with their provenance", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  // The write-up comes first; the rule cards are under Details.
  await expect(page.locator(".writeup .lead")).toBeVisible();
  await page.getByText(/Details: every rule checked/).click();
  await expect(page.locator(".rule").first()).toBeVisible();
  await expect(page.locator(".rule .tag.draft")).toHaveCount(0);
  await expect(page.getByText(/Rules reviewed by: Claude \(AI review/)).toBeVisible();
  await expect(page.getByText(/practising astrologer/)).toHaveCount(0);
  await expect(page.getByText("Noted (not scored)")).toBeVisible();

  // Reviewer mode still works, and says so.
  await page.getByText("Reviewer sign-in").click();
  await page.locator('input[type="password"]').fill(TOKEN);
  await page.getByRole("button", { name: "Enter" }).click();
  // The reading refreshes by itself in review mode; no extra click.
  await expect(page.getByText(/Review mode: drafts are shown/)).toBeVisible();
});

test("a wrong reviewer token is refused, and nothing draft is shown", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await page.getByText("Reviewer sign-in").click();
  await page.locator('input[type="password"]').fill("not-the-token-at-all-000000");
  await page.getByRole("button", { name: "Enter" }).click();
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("X-Review-Token");
  await expect(page.locator(".rule")).toHaveCount(0);
});

test("porutham in review mode equals the API", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1992-03-15", "06:20", "Chennai", /Chennai/);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await page.getByText("Reviewer sign-in").click();
  await page.locator('input[type="password"]').fill(TOKEN);
  await page.getByRole("button", { name: "Enter" }).click();
  await page.getByRole("button", { name: "Match", exact: true }).click();
  await enter(page, "1988-11-02", "21:45", "Madurai", /Madurai/);
  const req = page.waitForRequest("/api/match");
  await page.getByRole("button", { name: "Match", exact: true }).last().click();
  const body = JSON.parse((await req).postData()!);
  const api = await (await page.request.post("/api/match", { data: body, headers: { "x-review-token": TOKEN } })).json();
  for (const [r] of api.results) {
    const row = page.locator(`tr[data-porutham="${r.kind}"]`);
    await expect(row).toContainText(r.detail);
  }
  await expect(page.getByText(`${api.matched} of ${api.evaluated} evaluated poruthams match`)).toBeVisible();
});

test("no horizontal page scroll at any width", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(0);
});

test("every topic reads, and sensitive periods equal the API", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  const req = page.waitForRequest("/api/chart");
  await page.getByRole("button", { name: "Compute chart" }).click();
  const birth = JSON.parse((await req).postData()!).birth;

  await page.getByRole("button", { name: "Readings", exact: true }).click();
  const topics = (await (await page.request.get("/api/topics")).json()).topics as { id: string; title: string; disclaimer?: string }[];
  expect(topics.length).toBeGreaterThanOrEqual(9);
  for (const t of topics) {
    await page.getByRole("navigation", { name: "Topics" }).getByRole("button", { name: t.title, exact: true }).click();
    await expect(page.getByRole("heading", { name: t.title, exact: true })).toBeVisible();
    if (t.disclaimer) await expect(page.locator(".notice.disclaimer")).toHaveText(t.disclaimer);
    // The write-up's conclusion, read on open, is the engine's own text.
    const api = await (await page.request.post(`/api/topic/${t.id}`, { data: { birth } })).json();
    await expect(page.locator(".writeup .lead")).toHaveText(api.writeup.summary[0]);
    await expect(page.locator(".writeup-section h4").first()).toHaveText(api.writeup.sections[0].heading);
    // Every finding shown carries its plain-language line.
    const points = api.writeup.sections.flatMap((s: { points?: { meaning: string }[] }) => s.points ?? []);
    await expect(page.locator(".writeup .meaning")).toHaveCount(points.length);
    if (points.length > 0) await expect(page.locator(".writeup .meaning").first()).toContainText(points[0].meaning);
  }

  await page.getByRole("button", { name: "Sensitive periods", exact: true }).click();
  await page.getByRole("button", { name: "Show periods" }).click();
  const api = await (await page.request.post("/api/periods", { data: { birth, from_age: 18, to_age: 70 } })).json();
  const cards = page.locator("li.period-window");
  await expect(cards).toHaveCount(api.windows.length);
  const want = api.windows.map((w: { maha_name: string; antar_name: string }) => `${w.maha_name}/${w.antar_name}`);
  expect(await cards.evaluateAll((els) => els.map((e) => e.getAttribute("data-window")))).toEqual(want);
  await expect(page.locator("li.period-window.sensitive")).toHaveCount(api.windows.filter((w: { sensitive: boolean }) => w.sensitive).length);
  // The first line of each card is the engine's own explanation, verbatim.
  await expect(cards.first().locator("p").first()).toHaveText(api.windows[0].explanation[0]);
  expect(errors).toEqual([]);
});

test("a family member is saved on the device and read against the API", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  const req = page.waitForRequest("/api/chart");
  await page.getByRole("button", { name: "Compute chart" }).click();
  const native = JSON.parse((await req).postData()!).birth;

  await page.getByRole("button", { name: "Family", exact: true }).click();
  await page.getByRole("button", { name: "Add a member" }).click();
  await page.getByLabel("Name (optional)").fill("Meena");
  await page.getByLabel("Relation").selectOption("child");
  const form = page.locator(".add-member");
  await form.locator('input[type="date"]').fill("2012-04-02");
  await form.locator('input[type="time"]').fill("09:15");
  await form.getByPlaceholder(/Town/).fill("Chennai");
  await form.getByRole("button", { name: /Chennai/ }).first().click();
  await form.getByRole("button", { name: "Save member" }).click();
  await expect(page.locator("li.member")).toHaveCount(1);

  // Stored locally, and still there after a reload (after recomputing the chart).
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem("lagn.family.v1") ?? "[]"));
  expect(stored).toHaveLength(1);
  expect(stored[0]).toMatchObject({ name: "Meena", relation: "child", birth: { date: "2012-04-02" } });

  const freq = page.waitForRequest("/api/family");
  await page.getByRole("button", { name: "Read", exact: true }).click();
  const sent = JSON.parse((await freq).postData()!);
  expect(sent.native.birth).toMatchObject(native); // the chart echo adds "settings": null (defaults)
  const api = await (await page.request.post("/api/family", { data: sent })).json();
  await expect(page.locator(".family-reading")).toHaveAttribute("data-agreement", api.agreement);

  // Verdict first: both leans, each labelled with whose chart it is.
  await expect(page.locator(".verdict")).toBeVisible();
  await expect(page.locator(".lean-card .whose").first()).toHaveText("Your chart");
  await expect(page.locator(".lean-card .whose").nth(1)).toHaveText("Meena's own chart");

  // Then the three tabs.
  await expect(page.locator(".subtabs button")).toHaveCount(3);
  await page.getByRole("button", { name: "Meena's chart" }).click();
  for (const o of api.own) await expect(page.locator(".own-topic summary", { hasText: o.meta.title })).toBeVisible();
  await page.getByRole("button", { name: "Together" }).click();
  // The engine's own comparison, with the name filled in on the device only.
  const together = page.locator(".together p");
  await expect(together).toHaveCount(api.comparison.length);
  await expect(together.first()).toHaveText(api.comparison[0].split("{member}").join("Meena"));
  await expect(page.getByText("{member}")).toHaveCount(0);
});

test("the past-life reading appears, carries its frame, and bridges to the other readings", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  const req = page.waitForRequest("/api/chart");
  await page.getByRole("button", { name: "Compute chart" }).click();
  const birth = JSON.parse((await req).postData()!).birth;

  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await page.getByRole("navigation", { name: "Topics" }).getByRole("button", { name: "Past life", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Past life", exact: true })).toBeVisible();

  // The frame is stated before anything is read.
  await expect(page.locator(".notice.disclaimer")).toContainText("not a claim about events");
  await expect(page.locator(".notice.disclaimer")).toContainText("never names a past identity");

  const api = await (await page.request.post("/api/topic/past_life", { data: { birth } })).json();
  const axis = api.writeup.sections.find((s: { kind: string }) => s.kind === "karmic_axis");
  const bridge = api.writeup.sections.find((s: { kind: string }) => s.kind === "bridge");
  expect(axis).toBeTruthy();
  expect(bridge).toBeTruthy();
  await expect(page.locator('.writeup-section[data-kind="karmic_axis"] h4')).toHaveText(axis.heading);
  await expect(page.locator('.writeup-section[data-kind="karmic_axis"] p').first()).toHaveText(axis.paragraphs[0]);
  await expect(page.locator('.writeup-section[data-kind="bridge"] p').first()).toHaveText(bridge.paragraphs[0]);
  // The bridge names other readings, not this one.
  const bridgeText = (await page.locator('.writeup-section[data-kind="bridge"]').innerText()).toLowerCase();
  expect(bridgeText).not.toContain("your past life reading");
  expect(bridgeText).toMatch(/reading, which (comes out|calls for|has no)/);
});

test("theme: the choice overrides the system setting and survives a reload", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/");
  // Following the system: no explicit mark, dark colours in force.
  await expect(page.locator("html")).not.toHaveAttribute("data-theme", /.*/);
  const bgOf = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  const darkBg = await bgOf();
  await page.getByRole("button", { name: /Light/ }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  const lightBg = await bgOf();
  expect(lightBg).not.toBe(darkBg);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute("content", "#ffffff");

  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  expect(await bgOf()).toBe(lightBg); // the choice still beats the dark system setting

  await page.getByRole("button", { name: /Dark/ }).click();
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute("content", "#101216");
  await page.getByRole("button", { name: /System/ }).click();
  await expect(page.locator("html")).not.toHaveAttribute("data-theme", /.*/);
  expect(await bgOf()).toBe(darkBg);
});

/** Contrast ratio of two CSS colours, per WCAG. Handles rgb(), rgba() and the
 *  color(srgb ...) form Chromium reports for color-mix(). */
const CONTRAST = `(() => {
  const parse = (c) => {
    const n = (c.match(/-?[\\d.]+(e-?\\d+)?/g) || []).map(Number);
    if (n.length < 3) return null;
    const scale = c.trim().startsWith("color(") ? 255 : 1;
    return { r: n[0] * scale, g: n[1] * scale, b: n[2] * scale, a: n.length > 3 ? n[3] : 1 };
  };
  const lum = (c) => {
    const p = parse(c);
    if (!p) return null;
    const f = (v) => { const s = v / 255; return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4); };
    return 0.2126 * f(p.r) + 0.7152 * f(p.g) + 0.0722 * f(p.b);
  };
  return { parse, ratio: (a, b) => {
    const [x, y] = [lum(a), lum(b)];
    if (x === null || y === null) return null;
    const [hi, lo] = x >= y ? [x, y] : [y, x];
    return (hi + 0.05) / (lo + 0.05);
  } };
})()`;

for (const scheme of ["light", "dark"] as const) {
  test(`text on the rendered page meets WCAG AA in ${scheme} mode`, async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: new RegExp(scheme, "i") }).click();
    await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
    await page.getByRole("button", { name: "Compute chart" }).click();
    await page.getByRole("button", { name: "Readings", exact: true }).click();
    await expect(page.locator(".writeup .lead")).toBeVisible();

    const bad = await page.evaluate((helper: string) => {
      const { parse, ratio } = eval(helper) as {
        parse: (c: string) => { r: number; g: number; b: number; a: number } | null;
        ratio: (a: string, b: string) => number | null;
      };
      // The opaque background actually painted behind an element.
      const behind = (el: Element): string => {
        for (let n: Element | null = el; n; n = n.parentElement) {
          const c = getComputedStyle(n).backgroundColor;
          const p = parse(c);
          if (p && p.a >= 0.95) return c;
        }
        return getComputedStyle(document.body).backgroundColor;
      };
      const out: string[] = [];
      const nodes = Array.from(document.querySelectorAll<HTMLElement>("p, h1, h2, h3, h4, h5, li, td, th, button, label, span, div, summary, strong, code"));
      for (const el of nodes) {
        const text = Array.from(el.childNodes).filter((n) => n.nodeType === 3).map((n) => n.textContent ?? "").join("").trim();
        if (!text) continue;
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        const cs = getComputedStyle(el);
        if (cs.visibility === "hidden" || cs.opacity === "0") continue;
        const size = parseFloat(cs.fontSize);
        const large = size >= 24 || (size >= 18.66 && Number(cs.fontWeight) >= 700);
        const need = large ? 3 : 4.5;
        const got = ratio(cs.color, behind(el));
        if (got !== null && got < need) out.push(`${got.toFixed(2)} < ${need} on <${el.tagName.toLowerCase()} class="${el.className}"> ${text.slice(0, 40)}`);
      }
      return out;
    }, CONTRAST);
    expect(bad).toEqual([]);
  });
}

test("sex is asked once, on the birth form, and used by every reading", async ({ page }) => {
  await page.goto("/");
  await enter(page, "1990-02-11", "04:10", "Kochi", /Kochi/);
  await page.getByLabel(/Sex \(optional\)/).selectOption("female");
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();

  const topicReq = page.waitForRequest((r) => r.url().includes("/api/topic/"));
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  expect(JSON.parse((await topicReq).postData()!).sex).toBe("female");
  // The marriage write-up includes the woman's-chart karaka, which needs the sex.
  await expect(page.getByText(/the natural significator of the husband in a woman's chart/)).toBeVisible();

  const periodsReq = page.waitForRequest((r) => r.url().includes("/api/periods"));
  await page.getByRole("button", { name: "Sensitive periods", exact: true }).click();
  await page.getByRole("button", { name: "Show periods" }).click();
  expect(JSON.parse((await periodsReq).postData()!).sex).toBe("female");

  await page.getByRole("button", { name: "Family", exact: true }).click();
  // No view after the birth form asks for the native's sex again.
  for (const tab of ["Readings", "Sensitive periods", "Family"]) {
    await page.getByRole("button", { name: tab, exact: true }).click();
    await expect(page.getByLabel(/sex/i)).toHaveCount(0);
  }
});

test("the source is offered to anyone using the hosted app (AGPL section 13)", async ({ page }) => {
  await page.goto("/");
  const link = page.getByRole("link", { name: "Source code" });
  await expect(link).toBeVisible();
  await expect(link).toHaveAttribute("href", "https://github.com/funtopiaapp/lagn");
  await expect(page.getByText(/Free software, AGPL-3.0/)).toBeVisible();
});

test("the engine runs in the browser, and agrees with the server exactly", async ({ page }) => {
  await page.unroute("**/engine/**"); // this test is about the local engine
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");

  // The engine installs itself at start-up; wait for it rather than assuming.
  await page.waitForFunction(() => typeof window.Lagn?.chart === "function", null, { timeout: 60_000 });

  const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13.08, longitude: 80.27, utc_offset_hours: 5.5 };

  // Every reading, computed locally, must equal what the server computes.
  const local = await page.evaluate(async (b) => {
    interface Engine {
      topics(): Promise<string>;
      chart(request: string): Promise<string>;
      periods(request: string): Promise<string>;
      match(request: string): Promise<string>;
      places(request: string): Promise<string>;
      topic(name: string, request: string): Promise<string>;
    }
    const L = window.Lagn as unknown as Engine;
    const topics = JSON.parse(await L.topics());
    const out: Record<string, unknown> = {
      topics,
      chart: JSON.parse(await L.chart(JSON.stringify({ birth: b }))),
      periods: JSON.parse(await L.periods(JSON.stringify({ birth: b, from_age: 20, to_age: 45 }))),
      match: JSON.parse(await L.match(JSON.stringify({ bride: b, groom: b }))),
      places: JSON.parse(await L.places(JSON.stringify({ q: "Chennai", limit: 3 }))),
    };
    for (const t of topics.topics as { id: string }[]) {
      out[`topic:${t.id}`] = JSON.parse(await L.topic(t.id, JSON.stringify({ birth: b })));
    }
    return out;
  }, birth);

  // The comparison is the one scripts/qa_cross_platform.py already applies
  // between macOS and Linux: anything discrete - a sign, a house, a verdict,
  // a score, a sentence - must match exactly, and floating-point values are
  // measured rather than assumed identical. wasi-libc's sin and cos differ
  // from the host's in the last bits, which is why this is not byte equality.
  const LIMIT = 1e-9;
  function compare(local: unknown, server: unknown, path: string, out: { discrete: string[]; worst: number; worstAt: string }) {
    const volatile = ["current", "running_now", "as_of_utc", "platform"];
    if (local && server && typeof local === "object" && typeof server === "object" && !Array.isArray(local)) {
      const a = local as Record<string, unknown>, b = server as Record<string, unknown>;
      const keys = new Set([...Object.keys(a), ...Object.keys(b)].filter((k) => !volatile.includes(k)));
      for (const k of keys) {
        if (!(k in a) || !(k in b)) { out.discrete.push(`${path}.${k}: present on one side only`); continue; }
        compare(a[k], b[k], `${path}.${k}`, out);
      }
    } else if (Array.isArray(local) && Array.isArray(server)) {
      if (local.length !== server.length) { out.discrete.push(`${path}: length ${local.length} vs ${server.length}`); return; }
      local.forEach((x, i) => compare(x, server[i], `${path}[${i}]`, out));
    } else if (typeof local === "number" && typeof server === "number") {
      const d = Math.abs(local - server);
      if (d > out.worst) { out.worst = d; out.worstAt = path; }
    } else if (local !== server) {
      out.discrete.push(`${path}: ${JSON.stringify(local)} vs ${JSON.stringify(server)}`);
    }
  }

  const post = async (path: string, body: unknown) =>
    (await page.request.post(path, { data: body })).json();

  const out = { discrete: [] as string[], worst: 0, worstAt: "" };
  compare(local.topics, await (await page.request.get("/api/topics")).json(), "topics", out);
  compare(local.chart, await post("/api/chart", { birth }), "chart", out);
  compare(local.periods, await post("/api/periods", { birth, from_age: 20, to_age: 45 }), "periods", out);
  compare(local.match, await post("/api/match", { bride: birth, groom: birth }), "match", out);

  const topics = (local.topics as { topics: { id: string }[] }).topics;
  expect(topics.length).toBeGreaterThanOrEqual(10);
  for (const t of topics) {
    compare(local[`topic:${t.id}`], await post(`/api/topic/${t.id}`, { birth }), `topic.${t.id}`, out);
  }

  // Nothing discrete may differ: every reading, verdict and sentence is identical.
  expect(out.discrete).toEqual([]);
  // And the numeric drift stays far below anything astrologically meaningful.
  expect(out.worst, `largest numeric difference at ${out.worstAt}`).toBeLessThan(LIMIT);
  console.log(`  local vs server: 0 discrete differences, largest numeric ${out.worst.toExponential(2)} at ${out.worstAt}`);

  expect(errors).toEqual([]);
});

test("a reading is computed locally, with no network request for it", async ({ page }) => {
  await page.unroute("**/engine/**");
  const apiCalls: string[] = [];
  page.on("request", (r) => { if (r.url().includes("/api/")) apiCalls.push(r.url()); });
  await page.goto("/");
  await page.waitForFunction(() => typeof window.Lagn?.chart === "function", null, { timeout: 60_000 });

  await enter(page, "1985-06-21", "14:30", "Chennai", /Chennai/);
  await page.getByRole("button", { name: "Compute chart" }).click();
  await expect(page.locator("svg.si-chart")).toBeVisible();
  await page.getByRole("button", { name: "Readings", exact: true }).click();
  await expect(page.locator(".writeup .lead")).toBeVisible();

  // Place search, the chart and the reading all happened in the browser.
  expect(apiCalls).toEqual([]);
});
