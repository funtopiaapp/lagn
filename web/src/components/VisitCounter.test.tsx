import { render, screen } from "@testing-library/react";
import { counterOrigin } from "./VisitCounter";

describe("the visit counter", () => {
  afterEach(() => vi.resetModules());

  it("renders nothing, and requests nothing, when no counter is configured", async () => {
    // This is the state in development and in every test: the app makes no
    // request to anyone after loading.
    const { VisitCounter } = await import("./VisitCounter");
    const { container } = render(<VisitCounter />);
    expect(container.innerHTML).toBe("");
  });

  it("shows the badge when one is configured", async () => {
    vi.stubEnv("VITE_COUNTER_BADGE", "https://hits.sh/example.com.svg?label=visits");
    vi.resetModules();
    const { VisitCounter } = await import("./VisitCounter");
    render(<VisitCounter />);
    const img = screen.getByRole("img", { name: /visits to this site/i });
    expect(img).toHaveAttribute("src", "https://hits.sh/example.com.svg?label=visits");
    expect(img).toHaveAttribute("loading", "lazy");
    vi.unstubAllEnvs();
  });

  it("reports the origin the CSP must allow, and nothing wider", () => {
    expect(counterOrigin("https://hits.sh/a/b.svg?x=1")).toBe("https://hits.sh");
    expect(counterOrigin("")).toBe("");
    expect(counterOrigin("not a url")).toBe(""); // never widens the policy on bad input
  });
});
