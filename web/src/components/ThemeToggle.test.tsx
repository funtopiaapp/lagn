import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { ThemeToggle } from "./ThemeToggle";
import type { Theme } from "../lib/theme";

function Harness() {
  const [theme, setTheme] = useState<Theme>("system");
  return <ThemeToggle theme={theme} onChange={setTheme} />;
}

afterEach(() => { localStorage.clear(); document.documentElement.removeAttribute("data-theme"); vi.unstubAllGlobals(); });

it("offers the three settings and applies the one chosen", async () => {
  vi.stubGlobal("matchMedia", vi.fn().mockReturnValue({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() }));
  render(<Harness />);
  const group = screen.getByRole("group", { name: "Theme" });
  expect(group).toBeInTheDocument();
  const dark = screen.getByRole("button", { name: /Dark/ });
  expect(screen.getByRole("button", { name: /System/ })).toHaveAttribute("aria-pressed", "true");
  await userEvent.setup().click(dark);
  expect(dark).toHaveAttribute("aria-pressed", "true");
  expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  expect(localStorage.getItem("lagn.theme")).toBe("dark");
});
