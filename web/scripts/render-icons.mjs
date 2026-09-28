// Renders the app icons from SVG with Chromium, at exact pixel sizes.
// Run: node scripts/render-icons.mjs   (writes public/icons/*.png)
import { chromium } from "playwright";
import { writeFileSync } from "node:fs";

const MAROON = "#7a2e1f", CREAM = "#f6e7c8";
// South Indian chart: a 4x4 grid whose centre 2x2 is open.
function grid(x, y, s, stroke) {
  const c = s / 4, lines = [];
  for (let i = 0; i <= 4; i++) {
    const inner = i === 2;
    lines.push(inner
      ? `<path d="M${x} ${y + i * c}h${c}M${x + 3 * c} ${y + i * c}h${c}"/>`
      : `<path d="M${x} ${y + i * c}h${s}"/>`);
    lines.push(inner
      ? `<path d="M${x + i * c} ${y}v${c}M${x + i * c} ${y + 3 * c}v${c}"/>`
      : `<path d="M${x + i * c} ${y}v${s}"/>`);
  }
  return `<g fill="none" stroke="${CREAM}" stroke-width="${stroke}" stroke-linecap="square">${lines.join("")}
    <path d="M${x} ${y + c * 0.55}L${x + c * 0.55} ${y}" stroke-width="${stroke}" stroke-linecap="butt"/></g>`;
}
const standard = (n) => `<svg xmlns="http://www.w3.org/2000/svg" width="${n}" height="${n}" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="96" fill="${MAROON}"/>${grid(96, 96, 320, 18)}</svg>`;
// Maskable: full-bleed background; content inside the central 80% safe zone.
const maskable = (n) => `<svg xmlns="http://www.w3.org/2000/svg" width="${n}" height="${n}" viewBox="0 0 512 512">
  <rect width="512" height="512" fill="${MAROON}"/>${grid(136, 136, 240, 14)}</svg>`;

const b = await chromium.launch();
const page = await b.newPage();
for (const [name, svg, n] of [
  ["icon-192.png", standard, 192], ["icon-512.png", standard, 512],
  ["icon-maskable-512.png", maskable, 512], ["apple-touch-icon.png", maskable, 180],
]) {
  await page.setViewportSize({ width: n, height: n });
  await page.setContent(`<html><body style="margin:0;background:transparent">${svg(n)}</body></html>`);
  writeFileSync(`public/icons/${name}`, await page.locator("svg").screenshot({ omitBackground: true }));
  console.log("wrote", name);
}
writeFileSync("public/icon.svg", standard(32).replace('width="32" height="32" ', ""));
await b.close();
