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

describe("no date or astrology arithmetic in the frontend", () => {
  const src = files(dirname(fileURLToPath(import.meta.url)));
  it("finds the source files", () => expect(src.length).toBeGreaterThan(5));
  for (const f of src) {
    it(`${f.split("/src/")[1]} uses no Date APIs`, () => {
      const text = readFileSync(f, "utf8");
      for (const pat of [/new Date\b/, /\bDate\.(now|parse|UTC)\b/, /toLocale(Date|Time)?String/, /getFullYear|getMonth|getDate\(/]) {
        expect(text, `${f} matches ${pat}`).not.toMatch(pat);
      }
    });
  }
});
