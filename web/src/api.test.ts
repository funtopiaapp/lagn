import { api, ApiError } from "./api";

const birth = { date: "1985-06-21", time: "14:30:00", latitude: 13, longitude: 80, utc_offset_hours: 5.5, place: "" };

function mockFetch(status: number, body: string) {
  const fn = vi.fn().mockResolvedValue(new Response(body, { status }));
  vi.stubGlobal("fetch", fn);
  return fn;
}

afterEach(() => vi.unstubAllGlobals());

describe("api client", () => {
  it("surfaces the server's own error message", async () => {
    mockFetch(400, JSON.stringify({ error: "invalid calendar date: 1985-02-30" }));
    await expect(api.chart(birth)).rejects.toMatchObject({ status: 400, message: "invalid calendar date: 1985-02-30" });
  });

  it("handles a non-JSON error body", async () => {
    mockFetch(502, "Bad Gateway");
    await expect(api.version()).rejects.toBeInstanceOf(ApiError);
  });

  it("reports a network failure plainly", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("fetch failed")));
    await expect(api.version()).rejects.toMatchObject({ status: 0 });
  });

  it("requests production mode and sends no token by default", async () => {
    const f = mockFetch(200, JSON.stringify({ report: {}, windows: [] }));
    await api.topic("marriage", birth, { from_age: 18, to_age: 45 });
    const [, init] = f.mock.calls[0]!;
    expect(JSON.parse(init.body).mode).toBe("production");
    expect(init.headers["x-review-token"]).toBeUndefined();
  });

  it("requests review mode only with a token, and sends it as a header", async () => {
    const f = mockFetch(200, JSON.stringify({ report: {}, windows: [] }));
    await api.topic("marriage", birth, { from_age: 18, to_age: 45 }, "tok");
    const [, init] = f.mock.calls[0]!;
    expect(JSON.parse(init.body).mode).toBe("review");
    expect(init.headers["x-review-token"]).toBe("tok");
  });

  it("escapes query parameters", async () => {
    const f = mockFetch(200, "[]");
    await api.places("Saint Thomas & Mount");
    expect(f.mock.calls[0]![0]).toContain("q=Saint+Thomas+%26+Mount");
  });
});
