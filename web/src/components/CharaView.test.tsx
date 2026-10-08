// Chara dasha on the professional surface. These pin the three things a
// practitioner checks first: that the sequence's direction is explained, that
// each sign's length shows the count behind it, and that the two directions
// are not conflated.

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { CharaView } from "./CharaView";
import type { CharaResponse } from "../types";

const birth = { date: "1981-12-21", time: "14:10:00", latitude: 8.8932, longitude: 76.6141, utc_offset_hours: 5.5, place: "Kollam" };

afterEach(() => vi.unstubAllGlobals());

const antar = (rasi: string, start: string, end: string) =>
  ({ rasi, start, end, cycle: 1, start_jd: 0, end_jd: 0 });

/** The owner's own chart, as the engine computes it: Mesha lagna, 68-year cycle. */
const data: CharaResponse = {
  lagna: "Mesha",
  lagna_is_odd: true,
  direction: "direct",
  cycle_years: 68,
  year_length_days: 365.25,
  lengths: [
    { rasi: "Mesha", lord: "Kuja", lord_rasi: "Kanya", direction: "direct", count: 6, years: 5, lord_at_home: false },
    // Even sign: counted backwards, which is why this is 4 and not 8.
    { rasi: "Vrishabha", lord: "Shukra", lord_rasi: "Makara", direction: "reverse", count: 5, years: 4, lord_at_home: false },
    { rasi: "Simha", lord: "Surya", lord_rasi: "Simha", direction: "direct", count: 1, years: 12, lord_at_home: true },
  ],
  periods: [
    {
      rasi: "Mesha", start: "1981-12-21", end: "1986-12-21", cycle: 1, start_jd: 0, end_jd: 0,
      children: [antar("Mesha", "1981-12-21", "1982-05-07"), antar("Vrishabha", "1982-05-07", "1982-09-21")],
    },
    { rasi: "Vrishabha", start: "1986-12-21", end: "1990-12-21", cycle: 1, start_jd: 0, end_jd: 0, children: [] },
  ],
  running_now: { as_of_utc: "2026-10-08", maha: "Dhanus", antar: "Simha" },
  variants: [
    { id: "V-13-9", question: "direction of the sequence", chosen: "by the lagna's odd/even parity" },
    { id: "V-13-13", question: "period when the lord is in its own sign", chosen: "12 years" },
  ],
};

function stub(body: unknown, status = 200) {
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(new Response(JSON.stringify(body), { status }))));
}

describe("the sequence", () => {
  it("says why it runs the way it does, and what is running now", async () => {
    stub(data);
    render(<CharaView birth={birth} />);
    expect(await screen.findByText(/an odd sign/)).toBeInTheDocument();
    expect(screen.getByText(/runs zodiacally/)).toBeInTheDocument();
    expect(screen.getByText(/68 years/)).toBeInTheDocument();
    expect(screen.getByText(/Dhanus/)).toBeInTheDocument();
  });

  it("warns that the length count uses a different direction", async () => {
    // V-13-10. Conflating the two is the single easiest mistake here, and the
    // one most likely to make a reader think the engine is wrong.
    stub(data);
    render(<CharaView birth={birth} />);
    expect(await screen.findByText(/still counts its even signs backwards/)).toBeInTheDocument();
  });
});

describe("each sign's length", () => {
  it("shows the count and which way it ran", async () => {
    stub(data);
    render(<CharaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Chara dasha lengths" }));

    // An even sign in an odd-lagna chart is counted backwards: 5 to Makara,
    // so 4 years. Counted forwards it would have been 9 and 8 years.
    const row = within(table.getByRole("row", { name: /Vrishabha/ }));
    expect(row.getByText("backwards")).toBeInTheDocument();
    expect(row.getByText("5")).toBeInTheDocument();
    expect(row.getByText("4")).toBeInTheDocument();
  });

  it("marks the twelve-year case for what it is", async () => {
    stub(data);
    render(<CharaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Chara dasha lengths" }));
    const row = within(table.getByRole("row", { name: /Simha/ }));
    expect(row.getByText("12")).toBeInTheDocument();
    expect(row.getByText(/lord at home/)).toBeInTheDocument();
  });
});

describe("the periods", () => {
  it("keeps antardashas behind a disclosure rather than on the page", async () => {
    stub(data);
    render(<CharaView birth={birth} />);
    const open = await screen.findByRole("button", { name: /Mesha/ });
    expect(screen.queryByRole("table", { name: /Antardashas of Mesha/ })).toBeNull();

    await userEvent.click(open);
    const table = within(screen.getByRole("table", { name: /Antardashas of Mesha/ }));
    // Opens on the mahadasha's own sign.
    expect(table.getAllByRole("rowheader")[0]).toHaveTextContent("Mesha");
  });
});

describe("the variants", () => {
  it("call out the one that explains differences from other software", async () => {
    stub(data);
    render(<CharaView birth={birth} />);
    // V-13-9 appears twice by design: once in the register below, and once in
    // the note that tells a reader comparing against other software where to
    // look first.
    expect((await screen.findAllByText(/V-13-9/)).length).toBe(2);
    expect(screen.getByText("by the lagna's odd/even parity")).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent(/navamsa/);
  });
});

describe("when the engine cannot answer", () => {
  it("says so rather than rendering an empty table", async () => {
    stub({ error: "latitude 999 out of range" }, 400);
    render(<CharaView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("latitude 999 out of range");
  });
});
