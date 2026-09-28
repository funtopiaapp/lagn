import { CELL, cellOf, isCentre } from "./southIndian";

describe("South Indian cell layout", () => {
  it("places the twelve signs on twelve distinct cells, none in the centre", () => {
    const seen = new Set(CELL.map(([r, c]) => `${r},${c}`));
    expect(seen.size).toBe(12);
    for (const [r, c] of CELL) expect(isCentre(r, c)).toBe(false);
  });

  it("matches the CLI layout: Meena top-left, running clockwise", () => {
    // Same anchors as crates/lagn-cli/src/square.rs signs_run_clockwise_from_meena.
    expect(cellOf(0)).toEqual([0, 1]);  // Mesha
    expect(cellOf(3)).toEqual([1, 3]);  // Karka
    expect(cellOf(6)).toEqual([3, 2]);  // Tula
    expect(cellOf(9)).toEqual([2, 0]);  // Makara
    expect(cellOf(11)).toEqual([0, 0]); // Meena
  });

  it("walks the outer ring one step at a time", () => {
    for (let i = 0; i < 12; i++) {
      const [r1, c1] = cellOf(i);
      const [r2, c2] = cellOf((i + 1) % 12);
      expect(Math.abs(r1 - r2) + Math.abs(c1 - c2)).toBe(1);
    }
  });

  it("rejects out-of-range indices", () => {
    expect(() => cellOf(12)).toThrow(RangeError);
    expect(() => cellOf(-1)).toThrow(RangeError);
  });
});
