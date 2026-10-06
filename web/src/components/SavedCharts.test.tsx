import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SavedCharts } from "./SavedCharts";
import { addSaved, loadSaved, type SavedBirth } from "../lib/savedBirths";
import type { BirthInput } from "../types";

const birth = (over: Partial<BirthInput> = {}): BirthInput => ({
  date: "1981-12-21",
  time: "14:10:00",
  latitude: 8.88113,
  longitude: 76.58469,
  utc_offset_hours: 5.5,
  place: "Kollam, Kerala",
  ...over,
});

beforeEach(() => {
  localStorage.clear();
  vi.unstubAllGlobals();
});

describe("SavedCharts", () => {
  it("says what to do when nothing is saved, rather than showing an empty panel", () => {
    render(<SavedCharts saved={[]} onChange={() => {}} onOpen={() => {}} />);
    expect(screen.getByText(/No profiles are saved on this device yet/)).toBeInTheDocument();
    expect(screen.getByText(/Save this profile on this\s+device/)).toBeInTheDocument();
  });

  it("lists a profile by name, and an unnamed one by its birth details", () => {
    const list: SavedBirth[] = [
      addSaved([], birth(), { name: "Wife" }).list[0]!,
      { id: "x2", birth: birth({ date: "1979-02-02", place: "" }) },
    ];
    render(<SavedCharts saved={list} onChange={() => {}} onOpen={() => {}} />);
    expect(screen.getByText("Wife")).toBeInTheDocument();
    expect(screen.getByText("Unnamed profile")).toBeInTheDocument();
    expect(screen.getByText(/1981-12-21 14:10 · Kollam, Kerala/)).toBeInTheDocument();
  });

  it("opens the profile it was asked to open, with its sex", async () => {
    const list = addSaved([], birth(), { name: "Wife", sex: "female" }).list;
    const onOpen = vi.fn();
    render(<SavedCharts saved={list} onChange={() => {}} onOpen={onOpen} />);
    await userEvent.setup().click(screen.getByRole("button", { name: /Wife/ }));
    expect(onOpen).toHaveBeenCalledWith(list[0]!.birth, "female");
  });

  it("removes a profile and reports the new list", async () => {
    const list = addSaved([], birth(), { name: "Wife" }).list;
    const onChange = vi.fn();
    render(<SavedCharts saved={list} onChange={onChange} onOpen={() => {}} />);
    await userEvent.setup().click(screen.getByRole("button", { name: "Remove" }));
    expect(onChange).toHaveBeenCalledWith([]);
    expect(loadSaved()).toEqual([]);
  });

  it("says plainly where the profiles are kept", () => {
    render(<SavedCharts saved={addSaved([], birth()).list} onChange={() => {}} onOpen={() => {}} />);
    expect(screen.getByText(/stay in this browser on this device/)).toBeInTheDocument();
    expect(screen.getByText(/never sent anywhere/)).toBeInTheDocument();
  });
});
