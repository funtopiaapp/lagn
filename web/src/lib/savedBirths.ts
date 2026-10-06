// Saved charts live on this device only (localStorage), never on the server.
// Birth details are the most personal thing this app handles, so saving is
// opt-in, the list says plainly where it is kept, and nothing is written
// unless the person asks for it.
//
// Storage can be unavailable (private mode, blocked site data) or full, so
// every access is guarded and the app still works without it.

import type { BirthInput, Sex } from "../types";
import { isBirth } from "./family";

export interface SavedBirth {
  id: string;
  /** Optional, and only so a person can recognise this chart in the list.
   *  Nothing in the engine reads it. */
  name?: string;
  sex?: Sex;
  birth: BirthInput;
}

// No timestamp is stored, and none is needed: the list is newest-first by
// position, and DESIGN.md section 1 rule 2 keeps calendar arithmetic out of
// the browser entirely. A chart is recognised by its birth date, which the
// engine formatted.

const KEY = "lagn.savedBirths.v1";

/** A family keeps a handful; an astrologer keeps hundreds. The cap is only
 *  here so one device's storage cannot be filled without limit - the list is
 *  searched and paged rather than scrolled, so its length is not the problem
 *  it would be in a plain list. */
export const MAX_SAVED = 500;
export const MAX_NAME = 60;

/** Validate one record; anything malformed is rejected rather than repaired. */
export function isSavedBirth(v: unknown): v is SavedBirth {
  if (!v || typeof v !== "object") return false;
  const x = v as Record<string, unknown>;
  return typeof x.id === "string" && x.id.length > 0
    && (x.name === undefined || (typeof x.name === "string" && x.name.length <= MAX_NAME))
    && (x.sex === undefined || x.sex === "female" || x.sex === "male")
    && isBirth(x.birth);
}

export function loadSaved(): SavedBirth[] {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(v) ? v.filter(isSavedBirth) : [];
  } catch {
    return [];
  }
}

/** False when storage refused the write, so the caller can say so rather than
 *  letting the person believe a chart was kept. */
export function writeSaved(list: SavedBirth[]): boolean {
  try {
    localStorage.setItem(KEY, JSON.stringify(list));
    return true;
  } catch {
    return false;
  }
}

/** Same birth, by the values that define a chart. The name is not part of it:
 *  renaming is not a new chart, and saving the same birth twice is not two. */
export function sameBirth(a: BirthInput, b: BirthInput): boolean {
  return a.date === b.date
    && a.time === b.time
    && a.latitude === b.latitude
    && a.longitude === b.longitude
    && a.utc_offset_hours === b.utc_offset_hours;
}

let seq = 0;

function newId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    // Older WebViews have no randomUUID. The id only has to be unique within
    // one device's list, so a counter and some randomness are enough - and no
    // clock, which the frontend is not allowed to read.
    seq += 1;
    return `s${seq}-${Math.floor(Math.random() * 1e9).toString(36)}`;
  }
}

export interface SaveOutcome {
  list: SavedBirth[];
  /** False when storage refused; the list returned is still the intended one. */
  stored: boolean;
  /** True when this birth was already saved and was updated in place. */
  replaced: boolean;
}

/**
 * Add a chart, newest first. Saving a birth that is already in the list
 * updates that entry instead of adding a second copy, so repeated readings of
 * the same chart cannot fill the list.
 */
export function addSaved(
  existing: SavedBirth[],
  birth: BirthInput,
  opts: { name?: string; sex?: Sex } = {},
): SaveOutcome {
  const name = opts.name?.trim().slice(0, MAX_NAME) || undefined;
  const prior = existing.find((s) => sameBirth(s.birth, birth));
  const record: SavedBirth = {
    id: prior?.id ?? newId(),
    // Re-saving without a name keeps the name it already had, so casting the
    // same chart again does not quietly erase the label.
    name: name ?? prior?.name,
    sex: opts.sex ?? prior?.sex,
    birth,
  };
  const rest = existing.filter((s) => s.id !== record.id);
  const list = [record, ...rest].slice(0, MAX_SAVED);
  return { list, stored: writeSaved(list), replaced: !!prior };
}

export function removeSaved(existing: SavedBirth[], id: string): SaveOutcome {
  const list = existing.filter((s) => s.id !== id);
  return { list, stored: writeSaved(list), replaced: false };
}

export function renameSaved(existing: SavedBirth[], id: string, name: string): SaveOutcome {
  const trimmed = name.trim().slice(0, MAX_NAME);
  const list = existing.map((s) => (s.id === id ? { ...s, name: trimmed || undefined } : s));
  return { list, stored: writeSaved(list), replaced: false };
}

/** What to show for a chart with no name: enough to recognise it by. */
export function describe(s: SavedBirth): string {
  const where = s.birth.place?.trim();
  const when = `${s.birth.date} ${s.birth.time.slice(0, 5)}`;
  return where ? `${when} · ${where}` : `${when} · ${s.birth.latitude.toFixed(2)}, ${s.birth.longitude.toFixed(2)}`;
}
