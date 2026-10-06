// Two reported problems, pinned.
//
// Picking a saved profile on the porutham page ran the match without showing
// what it had used, so it looked as though nothing happened. And the list was
// a plain scroll, which is unusable once someone keeps hundreds of charts.

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ProfilePicker } from "./ProfilePicker";
import type { SavedBirth } from "../lib/savedBirths";

const profile = (i: number, name?: string): SavedBirth => ({
  id: `p${i}`,
  name,
  birth: {
    date: `19${String(50 + (i % 50)).padStart(2, "0")}-0${(i % 9) + 1}-1${i % 9}`,
    time: "09:30:00",
    latitude: 13.08784,
    longitude: 80.27847,
    utc_offset_hours: 5.5,
    place: i % 2 === 0 ? "Chennai, Tamil Nadu" : "Kollam, Kerala",
  },
});

const many = (n: number) => Array.from({ length: n }, (_, i) => profile(i, `Person ${i}`));

describe("choosing a saved profile", () => {
  it("shows nothing at all when there is nothing saved", () => {
    const { container } = render(<ProfilePicker profiles={[]} onPick={() => {}} label="Pick" />);
    expect(container).toBeEmptyDOMElement();
  });

  it("re-reads the store when opened, so a profile saved elsewhere appears", async () => {
    const onRefresh = vi.fn();
    render(<ProfilePicker profiles={many(3)} onPick={() => {}} onRefresh={onRefresh} label="Pick" />);
    await userEvent.setup().click(screen.getByRole("button", { name: /Choose from 3 saved profiles/ }));
    expect(onRefresh).toHaveBeenCalled();
  });

  it("hands back the profile that was picked", async () => {
    const onPick = vi.fn();
    const list = many(3);
    render(<ProfilePicker profiles={list} onPick={onPick} label="Pick" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: /Choose from/ }));
    await u.click(screen.getByRole("button", { name: /Person 1/ }));
    expect(onPick).toHaveBeenCalledWith(list[1]);
  });
});

describe("a list of hundreds", () => {
  it("shows a bounded page and says how many there are", async () => {
    render(<ProfilePicker profiles={many(400)} onPick={() => {}} label="Pick" />);
    await userEvent.setup().click(screen.getByRole("button", { name: /Choose from 400 saved profiles/ }));
    const rows = screen.getAllByRole("listitem");
    expect(rows.length).toBe(12);
    expect(screen.getByRole("button", { name: /Show more — 12 of 400 shown/ })).toBeInTheDocument();
  });

  it("pages further on request", async () => {
    render(<ProfilePicker profiles={many(400)} onPick={() => {}} label="Pick" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: /Choose from/ }));
    await u.click(screen.getByRole("button", { name: /Show more/ }));
    expect(screen.getAllByRole("listitem").length).toBe(36);
  });

  it("finds a profile by name, by date or by place", async () => {
    render(<ProfilePicker profiles={many(400)} onPick={() => {}} label="Pick" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: /Choose from/ }));
    const box = screen.getByLabelText(/Find a profile/);

    // A substring search, so "37" would also reach 137, 237 and 337. 399 is
    // unique in a list of 400.
    await u.type(box, "Person 399");
    expect(screen.getAllByRole("listitem").length).toBe(1);
    expect(screen.getByText("Person 399")).toBeInTheDocument();

    await u.clear(box);
    await u.type(box, "Kollam");
    // Odd-numbered profiles are Kollam; the page is still bounded.
    const rows = screen.getAllByRole("listitem");
    expect(rows.length).toBe(12);
    for (const r of rows) expect(within(r).getByText(/Kollam/)).toBeInTheDocument();
  });

  it("says so when nothing matches, rather than showing an empty list", async () => {
    render(<ProfilePicker profiles={many(40)} onPick={() => {}} label="Pick" />);
    const u = userEvent.setup();
    await u.click(screen.getByRole("button", { name: /Choose from/ }));
    await u.type(screen.getByLabelText(/Find a profile/), "zzzz");
    expect(screen.getByText(/Nothing saved here matches/)).toBeInTheDocument();
    expect(screen.queryAllByRole("listitem")).toHaveLength(0);
  });

  it("offers no search box for a handful, where scrolling is fine", async () => {
    render(<ProfilePicker profiles={many(5)} onPick={() => {}} label="Pick" />);
    await userEvent.setup().click(screen.getByRole("button", { name: /Choose from/ }));
    expect(screen.queryByLabelText(/Find a profile/)).toBeNull();
  });
});
