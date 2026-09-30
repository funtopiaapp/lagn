import { api, ApiError } from "./api";
import { isNative } from "./lib/native";

const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13, longitude: 80, utc_offset_hours: 5.5, place: "" };

/** A stand-in for the engine embedded in the native app. */
function installBridge(overrides: Record<string, unknown> = {}) {
  const calls: string[] = [];
  const reply = (name: string, body: unknown) => async (...args: string[]) => {
    calls.push(`${name}(${args.join("|")})`);
    return JSON.stringify(body);
  };
  const b = {
    version: reply("version", { engine: "0.1.0", corpus: { rules: 201 } }),
    topics: reply("topics", { topics: [{ id: "marriage", title: "Marriage", summary: "", ages: [18, 45] }], bhavas: [] }),
    chart: reply("chart", { input: birth, positions: [] }),
    topic: reply("topic", { report: { topic: "marriage", results: [] }, windows: [] }),
    periods: reply("periods", { windows: [], mode: "production" }),
    family: reply("family", { relation: "child", comparison: [] }),
    match: reply("match", { results: [] }),
    places: reply("places", [{ id: 1, name: "Chennai" }]),
    offset: reply("offset", { timezone: "Asia/Kolkata", candidates: [] }),
    ...overrides,
  };
  vi.stubGlobal("window", { ...globalThis.window, Lagn: b });
  return calls;
}

afterEach(() => vi.unstubAllGlobals());

describe("native transport", () => {
  it("is not used in a browser: everything goes over HTTP", async () => {
    expect(isNative()).toBe(false);
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ engine: "0.1.0" })));
    vi.stubGlobal("fetch", fetchMock);
    await api.version();
    expect(fetchMock).toHaveBeenCalledWith("/api/version", undefined);
  });

  it("uses the embedded engine when the app provides one, and never touches the network", async () => {
    const calls = installBridge();
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    expect(isNative()).toBe(true);

    expect(await api.version()).toMatchObject({ corpus: { rules: 201 } });
    expect(await api.topics()).toMatchObject({ topics: [{ id: "marriage" }] });
    await api.chart(birth);
    await api.topic("marriage", birth, {});
    await api.periods(birth, { from_age: 18, to_age: 70 });
    await api.family({ birth }, { birth }, "child");
    await api.match(birth, birth);
    await api.places("Chennai", 5);
    await api.offset("Asia/Kolkata", "1985-06-21", "14:30:00");

    expect(fetchMock).not.toHaveBeenCalled();
    expect(calls.map((c) => c.split("(")[0])).toEqual([
      "version", "topics", "chart", "topic", "periods", "family", "match", "places", "offset",
    ]);
    // The topic name is passed separately, as the bridge expects.
    expect(calls.find((c) => c.startsWith("topic("))).toContain("marriage|{");
  });

  it("passes the same payloads the HTTP API would receive", async () => {
    const calls = installBridge();
    vi.stubGlobal("fetch", vi.fn());
    await api.periods(birth, { sex: "female", from_age: 20, to_age: 40 });
    const call = calls[0];
    expect(call).toBeDefined();
    const sent = JSON.parse(call!.slice("periods(".length, -1));
    expect(sent).toMatchObject({ birth, sex: "female", from_age: 20, to_age: 40, mode: "production" });
  });

  it("reports an engine error the same way an HTTP error is reported", async () => {
    installBridge({ chart: async () => JSON.stringify({ error: "invalid date 1985-13-45" }) });
    vi.stubGlobal("fetch", vi.fn());
    await expect(api.chart(birth)).rejects.toThrow(ApiError);
    await expect(api.chart(birth)).rejects.toThrow(/invalid date/);
  });

  it("survives a bridge that throws or answers with nonsense", async () => {
    installBridge({ chart: async () => { throw new Error("JNI boom"); } });
    vi.stubGlobal("fetch", vi.fn());
    await expect(api.chart(birth)).rejects.toThrow(/JNI boom/);

    installBridge({ chart: async () => "not json at all" });
    await expect(api.chart(birth)).rejects.toThrow(/unreadable/);
  });

  it("falls back to HTTP rather than failing when the bridge is incomplete", async () => {
    // A wrapper that forgot to implement one method must not break the app.
    const partial = { version: async () => JSON.stringify({ engine: "0.1.0" }) };
    vi.stubGlobal("window", { ...globalThis.window, Lagn: partial });
    expect(isNative()).toBe(false);
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ engine: "0.1.0" })));
    vi.stubGlobal("fetch", fetchMock);
    await api.version();
    expect(fetchMock).toHaveBeenCalled();
  });
});
