import { installNativeBridge } from "./capacitor";
import { isNative } from "./native";

afterEach(() => {
  vi.unstubAllGlobals();
  delete (window as { Lagn?: unknown }).Lagn;
});

function installPlugin(overrides: Record<string, unknown> = {}) {
  const calls: Array<{ method: string; options?: unknown }> = [];
  const reply = (method: string, value: string) => async (options?: unknown) => {
    calls.push({ method, options });
    return { value };
  };
  const LagnNative = {
    init: reply("init", '{"ok":true}'),
    version: reply("version", '{"engine":"0.1.0"}'),
    topics: reply("topics", '{"topics":[],"bhavas":[]}'),
    chart: reply("chart", '{"positions":[]}'),
    topic: reply("topic", '{"report":{}}'),
    periods: reply("periods", '{"windows":[]}'),
    family: reply("family", '{"relation":"child"}'),
    match: reply("match", '{"results":[]}'),
    places: reply("places", "[]"),
    offset: reply("offset", '{"timezone":"Asia/Kolkata"}'),
    ...overrides,
  };
  vi.stubGlobal("window", Object.assign(window, { Capacitor: { Plugins: { LagnNative } } }));
  return calls;
}

describe("the Capacitor bridge", () => {
  it("installs nothing in a browser", () => {
    expect(installNativeBridge()).toBe(false);
    expect(isNative()).toBe(false);
  });

  it("installs a complete bridge inside the app", async () => {
    const calls = installPlugin();
    expect(installNativeBridge()).toBe(true);
    expect(isNative()).toBe(true);

    const b = window.Lagn!;
    expect(await b.version!()).toBe('{"engine":"0.1.0"}');
    expect(await b.chart!('{"birth":{}}')).toBe('{"positions":[]}');
    await b.topic!("marriage", '{"birth":{}}');

    // Capacitor takes named options, and unwraps to a plain JSON string.
    expect(calls.map((c) => c.method)).toEqual(["version", "chart", "topic"]);
    expect(calls[1]!.options).toEqual({ request: '{"birth":{}}' });
    expect(calls[2]!.options).toEqual({ name: "marriage", request: '{"birth":{}}' });
  });

  it("is not installed when the plugin is missing or incomplete", () => {
    vi.stubGlobal("window", Object.assign(window, { Capacitor: { Plugins: {} } }));
    expect(installNativeBridge()).toBe(false);
    vi.stubGlobal("window", Object.assign(window, { Capacitor: { Plugins: { LagnNative: { version: () => {} } } } }));
    expect(installNativeBridge()).toBe(false);
    expect(isNative()).toBe(false);
  });

  it("fails loudly if the plugin answers without a value", async () => {
    installPlugin({ chart: async () => ({}) });
    installNativeBridge();
    await expect(window.Lagn!.chart!('{"birth":{}}')).rejects.toThrow(/no value/);
  });
});
