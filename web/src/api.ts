import type {
  BirthInput, ChartResponse, FamilyReading, MatchResponse, OffsetSuggestion, PeriodsResponse, Place, TopicResponse, TopicsResponse, VersionInfo,
} from "./types";
import { bridge, type NativeBridge } from "./lib/native";

/** An API failure, carrying the server's own error message. */
export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

/**
 * Where the API lives. Empty for the web app, which is served by the API
 * server itself. A wrapped iOS/Android app is served from its own origin
 * (capacitor://localhost), so it is built with VITE_API_BASE set to the
 * server's origin, and that server must list the app's origin in
 * --allow-origin.
 */
export const API_BASE = (import.meta.env.VITE_API_BASE ?? "").replace(/\/+$/, "");

/**
 * One reading, two transports. On a device the engine is embedded and answers
 * through `window.Lagn`; in a browser the same JSON comes over HTTP. Errors
 * have the same `{error}` shape either way, so there is one error path.
 */
async function viaNative<T>(call: (b: NativeBridge) => Promise<string>): Promise<T | typeof NOT_NATIVE> {
  const b = bridge();
  if (!b) return NOT_NATIVE;
  let text: string;
  try {
    text = await call(b);
  } catch (e) {
    throw new ApiError(0, e instanceof Error ? e.message : "The engine on this device failed to answer.");
  }
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    throw new ApiError(0, "The engine on this device returned something unreadable.");
  }
  if (body && typeof body === "object" && "error" in body) {
    throw new ApiError(400, String((body as { error: unknown }).error));
  }
  return body as T;
}

const NOT_NATIVE = Symbol("not native");

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(API_BASE + path, init);
  } catch {
    throw new ApiError(0, navigator.onLine === false
      ? "You're offline. Computing a chart needs a connection."
      : "Could not reach the server. Check your connection and try again.");
  }
  const text = await res.text();
  let body: unknown;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    throw new ApiError(res.status, text || res.statusText || "Unexpected response from the server");
  }
  if (!res.ok) {
    const msg = body && typeof body === "object" && "error" in body ? String((body as { error: unknown }).error) : res.statusText;
    throw new ApiError(res.status, msg);
  }
  return body as T;
}

function post<T>(path: string, payload: unknown, reviewToken?: string): Promise<T> {
  const headers: Record<string, string> = { "content-type": "application/json" };
  if (reviewToken) headers["x-review-token"] = reviewToken;
  return request<T>(path, { method: "POST", headers, body: JSON.stringify(payload) });
}

export const api = {
  version: async () => {
    const n = await viaNative<VersionInfo>((b) => b.version());
    return n !== NOT_NATIVE ? n : request<VersionInfo>("/api/version");
  },

  places: async (q: string, limit = 8) => {
    const n = await viaNative<Place[]>((b) => b.places(JSON.stringify({ q, limit })));
    return n !== NOT_NATIVE ? n : request<Place[]>(`/api/places?${new URLSearchParams({ q, limit: String(limit) })}`);
  },

  offset: async (tz: string, date: string, time: string, longitude?: number) => {
    const n = await viaNative<OffsetSuggestion>((b) => b.offset(JSON.stringify({ tz, date, time, longitude })));
    if (n !== NOT_NATIVE) return n;
    const p = new URLSearchParams({ tz, date, time });
    if (longitude !== undefined) p.set("longitude", String(longitude));
    return request<OffsetSuggestion>(`/api/offset?${p}`);
  },

  chart: async (birth: BirthInput) => {
    const n = await viaNative<ChartResponse>((b) => b.chart(JSON.stringify({ birth })));
    return n !== NOT_NATIVE ? n : post<ChartResponse>("/api/chart", { birth });
  },

  topics: async () => {
    const n = await viaNative<TopicsResponse>((b) => b.topics());
    return n !== NOT_NATIVE ? n : request<TopicsResponse>("/api/topics");
  },

  /** Review mode is requested only when a reviewer token is supplied. Ages
   *  default to the topic's own range on the server. */
  topic: async (name: string, birth: BirthInput, opts: { sex?: "female" | "male"; from_age?: number; to_age?: number }, reviewToken?: string) => {
    const payload = { birth, ...opts, mode: reviewToken ? "review" : "production" };
    const n = await viaNative<TopicResponse>((b) => b.topic(name, JSON.stringify(payload)));
    return n !== NOT_NATIVE ? n : post<TopicResponse>(`/api/topic/${encodeURIComponent(name)}`, payload, reviewToken);
  },

  periods: async (birth: BirthInput, opts: { sex?: "female" | "male"; from_age: number; to_age: number }, reviewToken?: string) => {
    const payload = { birth, ...opts, mode: reviewToken ? "review" : "production" };
    const n = await viaNative<PeriodsResponse>((b) => b.periods(JSON.stringify(payload)));
    return n !== NOT_NATIVE ? n : post<PeriodsResponse>("/api/periods", payload, reviewToken);
  },

  family: async (native: { birth: BirthInput; sex?: "female" | "male" }, member: { birth: BirthInput; sex?: "female" | "male" },
    relation: string, reviewToken?: string) => {
    const payload = { native, member, relation, mode: reviewToken ? "review" : "production" };
    const n = await viaNative<FamilyReading>((b) => b.family(JSON.stringify(payload)));
    return n !== NOT_NATIVE ? n : post<FamilyReading>("/api/family", payload, reviewToken);
  },

  match: async (bride: BirthInput, groom: BirthInput, reviewToken?: string) => {
    const payload = { bride, groom, mode: reviewToken ? "review" : "production" };
    const n = await viaNative<MatchResponse>((b) => b.match(JSON.stringify(payload)));
    return n !== NOT_NATIVE ? n : post<MatchResponse>("/api/match", payload, reviewToken);
  },
};
