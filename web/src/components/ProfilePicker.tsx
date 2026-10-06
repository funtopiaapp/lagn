import { useMemo, useState } from "react";

import { describe, type SavedBirth } from "../lib/savedBirths";

interface Props {
  profiles: SavedBirth[];
  onPick: (p: SavedBirth) => void;
  /** Re-read the store, in case another page saved something since mount. */
  onRefresh?: () => void;
  label: string;
}

/** Beyond this the list stops being a list and becomes a wall. */
const PAGE = 12;

/** Matches a name, a date or a place. Nothing clever: a profile is found by
 *  what the person can remember about it. */
function matches(p: SavedBirth, q: string): boolean {
  if (!q) return true;
  const hay = `${p.name ?? ""} ${p.birth.date} ${p.birth.place ?? ""}`.toLowerCase();
  return q
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((w) => hay.includes(w));
}

/**
 * Choose a saved profile.
 *
 * Built for a list that grows: a search box, a bounded number of rows, and a
 * count of what is not shown. A family keeps a handful of charts; an
 * astrologer keeps hundreds, and scrolling past four hundred names to reach
 * one is not a way to find anything.
 */
export function ProfilePicker({ profiles, onPick, onRefresh, label }: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(PAGE);

  const hits = useMemo(() => profiles.filter((p) => matches(p, query)), [profiles, query]);
  const shown = hits.slice(0, limit);

  if (profiles.length === 0) return null;

  return (
    <section className="card profile-picker">
      <div className="card-head">
        <h4>{label}</h4>
        {!open && (
          <button
            type="button"
            onClick={() => { onRefresh?.(); setOpen(true); }}
          >
            Choose from {profiles.length} saved {profiles.length === 1 ? "profile" : "profiles"}
          </button>
        )}
        {open && <button type="button" className="link" onClick={() => setOpen(false)}>Close</button>}
      </div>

      {open && (
        <>
          {profiles.length > PAGE && (
            <label>Find a profile
              <input
                type="search"
                autoComplete="off"
                placeholder="A name, a date, or a place"
                value={query}
                onChange={(e) => { setQuery(e.target.value); setLimit(PAGE); }}
              />
            </label>
          )}

          {hits.length === 0 ? (
            <p className="hint">Nothing saved here matches “{query}”.</p>
          ) : (
            <>
              <ul className="members saved-list">
                {shown.map((p) => (
                  <li key={p.id} className="member">
                    <button type="button" className="link saved-open" onClick={() => { onPick(p); setOpen(false); }}>
                      <strong>{p.name || "Unnamed profile"}</strong>
                      <span className="hint saved-when">{describe(p)}</span>
                    </button>
                  </li>
                ))}
              </ul>
              {hits.length > shown.length && (
                <button type="button" className="link" onClick={() => setLimit(limit + PAGE * 2)}>
                  Show more — {shown.length} of {hits.length}
                  {query ? " matching" : ""} shown
                </button>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
