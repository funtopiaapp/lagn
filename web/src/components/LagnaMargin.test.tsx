import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LagnaMargin } from "./LagnaMargin";

const hold = (margin: number, before = margin, after = 600) => ({
  minutes_in: before,
  minutes_left: after,
  margin_minutes: margin,
  capped: false,
});

describe("LagnaMargin", () => {
  it("warns when the lagna is minutes from the next sign", () => {
    // Subject-06 of the validation pass: Tula 29°48', one minute from
    // Vrischika, read with the same authority as any other chart.
    render(<LagnaMargin hold={{ minutes_in: 139, minutes_left: 1, margin_minutes: 1, capped: false }} rasi="Tula" />);
    const note = screen.getByRole("note");
    expect(note).toHaveTextContent("Tula lagna holds for 1 minute");
    expect(note).toHaveTextContent("1 minute later");
    expect(note).toHaveTextContent(/unusually close to that edge/);
    expect(note.className).toContain("warn");
  });

  it("names the nearer edge and which side it is on", () => {
    render(<LagnaMargin hold={hold(16, 16, 37)} rasi="Mesha" />);
    expect(screen.getByRole("note")).toHaveTextContent("16 minutes earlier");

    render(<LagnaMargin hold={{ minutes_in: 94, minutes_left: 40, margin_minutes: 40, capped: false }} rasi="Karka" />);
    expect(screen.getAllByRole("note")[1]).toHaveTextContent("40 minutes later");
  });

  it("reassures rather than warns when the margin is comfortable", () => {
    render(<LagnaMargin hold={hold(68, 72, 68)} rasi="Tula" />);
    const note = screen.getByRole("note");
    expect(note).toHaveTextContent(/nearest half hour is enough/);
    expect(note).not.toHaveTextContent(/unusually close/);
    expect(note.className).not.toContain("warn");
  });

  it("writes hours and minutes the way a person would say them", () => {
    render(<LagnaMargin hold={hold(60, 60, 600)} rasi="Kanya" />);
    expect(screen.getByRole("note")).toHaveTextContent("holds for 1 hour.");

    render(<LagnaMargin hold={hold(131, 131, 600)} rasi="Kanya" />);
    expect(screen.getAllByRole("note")[1]).toHaveTextContent("holds for 2 hours 11 minutes.");
  });

  it("says nothing, and does not throw, when the engine sends no margin", () => {
    // A service worker still serving the previous WebAssembly bundle sends a
    // chart with no holds_for at all. This took the whole readings page down
    // with "Cannot read properties of undefined (reading 'capped')".
    expect(() => render(<LagnaMargin hold={undefined} rasi="Tula" />)).not.toThrow();
    expect(screen.queryByRole("note")).toBeNull();
  });

  it("says nothing when no boundary was found at all", () => {
    // Extreme latitudes only. There is no useful warning to give.
    render(<LagnaMargin hold={{ minutes_in: 720, minutes_left: 720, margin_minutes: 720, capped: true }} rasi="Makara" />);
    expect(screen.queryByRole("note")).toBeNull();
  });
});
