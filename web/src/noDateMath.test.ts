// DESIGN.md section 1, rule 2: the browser does no calendar arithmetic. Every
// date is formatted by the engine. This scan fails if Date handling creeps in.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? files(p) : /\.(ts|tsx)$/.test(n) && !/\.test\.tsx?$/.test(n) ? [p] : [];
  });
}

// One exemption, with a reason: the WASI host has to hand the engine a clock
// (clock_time_get). That is the opposite of the rule - it feeds the engine the
// time so that *it* does the arithmetic - and the file does no date handling
// of its own. Any other use of Date in it would still be caught by review.
const CLOCK_HOST = "lib/wasi.ts";

describe("no date or astrology arithmetic in the frontend", () => {
  const src = files(dirname(fileURLToPath(import.meta.url))).filter((f) => !f.endsWith(CLOCK_HOST));
  it("finds the source files", () => expect(src.length).toBeGreaterThan(5));

  it("the only exempt file is the WASI clock, and it uses Date for nothing else", () => {
    const text = readFileSync(join(dirname(fileURLToPath(import.meta.url)), CLOCK_HOST), "utf8");
    // Date is allowed only to answer the guest's clock_time_get.
    const uses = text.match(/\bDate\.\w+|new Date\b/g) ?? [];
    expect(uses).toEqual(["Date.now"]);
    expect(text).toMatch(/clock_time_get/);
  });
  for (const f of src) {
    it(`${f.split("/src/")[1]} uses no Date APIs`, () => {
      const text = readFileSync(f, "utf8");
      for (const pat of [/new Date\b/, /\bDate\.(now|parse|UTC)\b/, /toLocale(Date|Time)?String/, /getFullYear|getMonth|getDate\(/]) {
        expect(text, `${f} matches ${pat}`).not.toMatch(pat);
      }
    });
  }
});
