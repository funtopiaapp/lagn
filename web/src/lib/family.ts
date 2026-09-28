// Family members live on this device only (localStorage), never on the server:
// each reading sends both birth records and the server keeps nothing. Storage
// can be unavailable (private mode, blocked site data), so every access is
// guarded and the app still works for the current page view.

import type { BirthInput } from "../types";

export type Relation = "spouse" | "child" | "mother" | "father";
export const RELATIONS: [Relation, string][] = [["spouse", "Spouse"], ["child", "Child"], ["mother", "Mother"], ["father", "Father"]];

export interface Member {
  id: string;
  name: string;
  relation: Relation;
  sex?: "female" | "male";
  birth: BirthInput;
}

const KEY = "lagn.family.v1";
export const EXPORT_FORMAT = "lagn-family/1";

function isBirth(b: unknown): b is BirthInput {
  if (!b || typeof b !== "object") return false;
  const x = b as Record<string, unknown>;
  return typeof x.date === "string" && /^\d{4}-\d{2}-\d{2}$/.test(x.date)
    && typeof x.time === "string" && /^\d{2}:\d{2}(:\d{2})?$/.test(x.time)
    && typeof x.latitude === "number" && Math.abs(x.latitude) <= 90
    && typeof x.longitude === "number" && Math.abs(x.longitude) <= 180
    && typeof x.utc_offset_hours === "number" && Math.abs(x.utc_offset_hours) <= 14
    && (x.place === undefined || typeof x.place === "string");
}

/** Validate one member; anything malformed is rejected rather than repaired. */
export function isMember(m: unknown): m is Member {
  if (!m || typeof m !== "object") return false;
  const x = m as Record<string, unknown>;
  return typeof x.id === "string" && x.id.length > 0
    && typeof x.name === "string" && x.name.length <= 80
    && RELATIONS.some(([r]) => r === x.relation)
    && (x.sex === undefined || x.sex === "female" || x.sex === "male")
    && isBirth(x.birth);
}

export function loadMembers(): Member[] {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(v) ? v.filter(isMember) : [];
  } catch { return []; }
}

export function saveMembers(members: Member[]): boolean {
  try { localStorage.setItem(KEY, JSON.stringify(members)); return true; } catch { return false; }
}

export function exportJson(members: Member[]): string {
  return JSON.stringify({ format: EXPORT_FORMAT, members }, null, 2);
}

/** Parse an exported file. Throws with a readable message if it is not one. */
export function importJson(text: string): Member[] {
  let v: unknown;
  try { v = JSON.parse(text); } catch { throw new Error("This file is not valid JSON."); }
  const o = v as { format?: unknown; members?: unknown };
  if (!o || o.format !== EXPORT_FORMAT || !Array.isArray(o.members)) throw new Error("This is not a Lagn family export.");
  const bad = o.members.findIndex((m) => !isMember(m));
  if (bad >= 0) throw new Error(`Member ${bad + 1} in the file is incomplete or invalid.`);
  return o.members as Member[];
}

export function newId(): string {
  // randomUUID needs a secure context; getRandomValues does not.
  try { return crypto.randomUUID(); } catch {
    return "m" + Array.from(crypto.getRandomValues(new Uint8Array(12)), (b) => b.toString(16).padStart(2, "0")).join("");
  }
}
