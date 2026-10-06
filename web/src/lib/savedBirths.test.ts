import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  addSaved, describe as describeSaved, isSavedBirth, loadSaved, MAX_NAME, MAX_SAVED,
  removeSaved, renameSaved, sameBirth, writeSaved, type SavedBirth,
} from "./savedBirths";
import type { BirthInput } from "../types";

const birth = (over: Partial<BirthInput> = {}): BirthInput => ({
  date: "1954-11-07",
  time: "12:00:00",
  latitude: 9.37158,
  longitude: 78.83077,
  utc_offset_hours: 5.5,
  place: "Ramanathapuram, Tamil Nadu",
  ...over,
});

beforeEach(() => {
  localStorage.clear();
  vi.unstubAllGlobals();
});

describe("saved charts", () => {
  it("round-trips a chart with and without a name", () => {
    const a = addSaved([], birth(), { name: "Appa" });
    expect(a.stored).toBe(true);
    expect(loadSaved()).toHaveLength(1);
    expect(loadSaved()[0]!.name).toBe("Appa");

    const b = addSaved(a.list, birth({ date: "1990-01-01" }));
    expect(b.list).toHaveLength(2);
    expect(loadSaved()[0]!.name).toBeUndefined();
    // Newest first.
    expect(loadSaved()[0]!.birth.date).toBe("1990-01-01");
  });

  it("treats the same birth as one chart, not two", () => {
    const a = addSaved([], birth(), { name: "Appa" });
    const b = addSaved(a.list, birth());
    expect(b.replaced).toBe(true);
    expect(b.list).toHaveLength(1);
    // Re-saving without a name keeps the label already given.
    expect(b.list[0]!.name).toBe("Appa");
    expect(b.list[0]!.id).toBe(a.list[0]!.id);
  });

  it("knows which differences make a different chart", () => {
    expect(sameBirth(birth(), birth())).toBe(true);
    // The place label is not part of the chart; the offset certainly is.
    expect(sameBirth(birth(), birth({ place: "somewhere else" }))).toBe(true);
    expect(sameBirth(birth(), birth({ utc_offset_hours: 6.5 }))).toBe(false);
    expect(sameBirth(birth(), birth({ time: "12:01:00" }))).toBe(false);
    expect(sameBirth(birth(), birth({ latitude: 9.4 }))).toBe(false);
  });

  it("renames and removes", () => {
    const a = addSaved([], birth());
    const r = renameSaved(a.list, a.list[0]!.id, "  Amma  ");
    expect(r.list[0]!.name).toBe("Amma");
    // Clearing the name is allowed, and leaves the chart saved.
    const cleared = renameSaved(r.list, r.list[0]!.id, "   ");
    expect(cleared.list[0]!.name).toBeUndefined();
    expect(cleared.list).toHaveLength(1);

    const gone = removeSaved(cleared.list, cleared.list[0]!.id);
    expect(gone.list).toHaveLength(0);
    expect(loadSaved()).toHaveLength(0);
  });

  it("caps the list and trims long names", () => {
    let list: SavedBirth[] = [];
    for (let i = 0; i < MAX_SAVED + 5; i++) {
      list = addSaved(list, birth({ date: `19${String(50 + i).padStart(2, "0")}-01-01` })).list;
    }
    expect(list).toHaveLength(MAX_SAVED);

    const long = addSaved([], birth(), { name: "x".repeat(MAX_NAME + 40) });
    expect(long.list[0]!.name).toHaveLength(MAX_NAME);
  });

  it("rejects malformed records rather than repairing them", () => {
    expect(isSavedBirth({ id: "a", birth: birth() })).toBe(true);
    expect(isSavedBirth({ id: "", birth: birth() })).toBe(false);
    expect(isSavedBirth({ id: "a", birth: { ...birth(), latitude: 99 } })).toBe(false);
    expect(isSavedBirth({ id: "a", birth: birth(), sex: "other" })).toBe(false);
    expect(isSavedBirth({ id: "a" })).toBe(false);
    expect(isSavedBirth(null)).toBe(false);

    // A corrupt store reads as empty, not as a crash.
    localStorage.setItem("lagn.savedBirths.v1", "{not json");
    expect(loadSaved()).toEqual([]);
    localStorage.setItem("lagn.savedBirths.v1", JSON.stringify([{ id: "x" }, null, 7]));
    expect(loadSaved()).toEqual([]);
  });

  it("survives storage being unavailable", () => {
    // Private browsing, or site data blocked: setItem throws.
    vi.stubGlobal("localStorage", {
      getItem: () => { throw new Error("denied"); },
      setItem: () => { throw new Error("denied"); },
      removeItem: () => { throw new Error("denied"); },
      clear: () => {},
    });
    expect(loadSaved()).toEqual([]);
    expect(writeSaved([])).toBe(false);
    // The caller still gets the list it asked for, and is told it was not kept.
    const out = addSaved([], birth(), { name: "Appa" });
    expect(out.stored).toBe(false);
    expect(out.list).toHaveLength(1);
  });

  it("describes an unnamed chart well enough to recognise", () => {
    const withPlace = addSaved([], birth()).list[0]!;
    expect(describeSaved(withPlace)).toBe("1954-11-07 12:00 · Ramanathapuram, Tamil Nadu");
    const noPlace = addSaved([], birth({ place: "" })).list[0]!;
    expect(describeSaved(noPlace)).toBe("1954-11-07 12:00 · 9.37, 78.83");
  });
});
