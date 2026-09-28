// South Indian chart geometry: the rasis sit in fixed cells of a 4x4 grid,
// Meena at top-left, running clockwise. Must match crates/lagn-cli/src/square.rs.
//
//   Meena  | Mesha  | Vrisha | Mithuna
//   Kumbha |        |        | Karka
//   Makara |        |        | Simha
//   Dhanus | Vrisch | Tula   | Kanya

/** [row, col] for each sign index, Mesha = 0. */
export const CELL: readonly (readonly [number, number])[] = [
  [0, 1], [0, 2], [0, 3], [1, 3], [2, 3], [3, 3],
  [3, 2], [3, 1], [3, 0], [2, 0], [1, 0], [0, 0],
];

export function cellOf(signIndex: number): readonly [number, number] {
  const c = CELL[signIndex];
  if (!c) throw new RangeError(`sign index ${signIndex} is not 0-11`);
  return c;
}

export function isCentre(row: number, col: number): boolean {
  return row >= 1 && row <= 2 && col >= 1 && col <= 2;
}
