// The toggle lives at the top, not as a checkbox in the footer.
//
// This was feedback, not a guess: the footer checkbox meant scrolling the
// whole page to change what the app offers. These tests pin the shape that
// replaced it, so it does not drift back.

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ModeToggle } from "./ModeToggle";

describe("the mode toggle", () => {
  it("offers Lite and Pro as one labelled group", () => {
    render(<ModeToggle mode="lite" onChange={() => {}} />);
    const group = within(screen.getByRole("group", { name: "Mode" }));
    expect(group.getByRole("button", { name: "Lite" })).toBeInTheDocument();
    expect(group.getByRole("button", { name: "Pro" })).toBeInTheDocument();
  });

  it("shows which one is in force", () => {
    render(<ModeToggle mode="pro" onChange={() => {}} />);
    expect(screen.getByRole("button", { name: "Pro" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "Lite" })).toHaveAttribute("aria-pressed", "false");
  });

  it("reports a change in one click, with no checkbox to find", async () => {
    const onChange = vi.fn();
    render(<ModeToggle mode="lite" onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Pro" }));
    expect(onChange).toHaveBeenCalledWith("pro");
    expect(screen.queryByRole("checkbox")).toBeNull();
  });

  it("says what Pro is for, without needing the reader to try it", () => {
    render(<ModeToggle mode="lite" onChange={() => {}} />);
    expect(screen.getByRole("button", { name: "Pro" })).toHaveAttribute(
      "title",
      expect.stringContaining("astrologer"),
    );
    // And that some of it is not signed off, which a reader deserves up front.
    expect(screen.getByRole("button", { name: "Pro" }).getAttribute("title")).toMatch(/signed off/);
  });
});
