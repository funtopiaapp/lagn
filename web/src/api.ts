import type {
  BirthInput, ChartResponse, FamilyReading, MatchResponse, OffsetSuggestion, PeriodsResponse, Place, TopicResponse, TopicsResponse, VersionInfo,
} from "./types";

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
  version: () => request<VersionInfo>("/api/version"),

  places: (q: string, limit = 8) =>
    request<Place[]>(`/api/places?${new URLSearchParams({ q, limit: String(limit) })}`),

  offset: (tz: string, date: string, time: string, longitude?: number) => {
    const p = new URLSearchParams({ tz, date, time });
    if (longitude !== undefined) p.set("longitude", String(longitude));
    return request<OffsetSuggestion>(`/api/offset?${p}`);
  },

  chart: (birth: BirthInput) => post<ChartResponse>("/api/chart", { birth }),

  topics: () => request<TopicsResponse>("/api/topics"),

  /** Review mode is requested only when a reviewer token is supplied. Ages
   *  default to the topic's own range on the server. */
  topic: (name: string, birth: BirthInput, opts: { sex?: "female" | "male"; from_age?: number; to_age?: number }, reviewToken?: string) =>
    post<TopicResponse>(`/api/topic/${encodeURIComponent(name)}`,
      { birth, ...opts, mode: reviewToken ? "review" : "production" }, reviewToken),

  periods: (birth: BirthInput, opts: { sex?: "female" | "male"; from_age: number; to_age: number }, reviewToken?: string) =>
    post<PeriodsResponse>("/api/periods", { birth, ...opts, mode: reviewToken ? "review" : "production" }, reviewToken),

  family: (native: { birth: BirthInput; sex?: "female" | "male" }, member: { birth: BirthInput; sex?: "female" | "male" },
    relation: string, reviewToken?: string) =>
    post<FamilyReading>("/api/family", { native, member, relation, mode: reviewToken ? "review" : "production" }, reviewToken),

  match: (bride: BirthInput, groom: BirthInput, reviewToken?: string) =>
    post<MatchResponse>("/api/match", { bride, groom, mode: reviewToken ? "review" : "production" }, reviewToken),
};
