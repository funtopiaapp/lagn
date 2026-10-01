import { cspFor } from "../pwa-plugin";

describe("the Content-Security-Policy", () => {
  it("allows no external anything by default", () => {
    const csp = cspFor("");
    expect(csp).toContain("img-src 'self' data:");
    expect(csp).toContain("connect-src 'self'");
    expect(csp).not.toMatch(/https?:\/\//);
  });

  it("allows the counter as an image only, never as a script or a connection", () => {
    const csp = cspFor("", "https://hits.sh");
    expect(csp).toContain("img-src 'self' data: https://hits.sh");
    // The counting service may not run code or be connected to.
    expect(csp).toContain("script-src 'self' 'wasm-unsafe-eval'");
    expect(csp.match(/script-src[^;]*/)![0]).not.toContain("hits.sh");
    expect(csp.match(/connect-src[^;]*/)![0]).not.toContain("hits.sh");
  });

  it("still permits WebAssembly but never eval", () => {
    const csp = cspFor("");
    expect(csp).toContain("'wasm-unsafe-eval'");
    // "'wasm-unsafe-eval'" does not contain "'unsafe-eval'": the quote matters.
    expect(csp).not.toContain("'unsafe-eval'");
  });
});
