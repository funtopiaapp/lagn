import { useMemo, useState } from "react";

import { describe, removeSaved, renameSaved, type SavedBirth } from "../lib/savedBirths";
import type { BirthInput, Sex } from "../types";

/** Beyond this the list stops being a list and becomes a wall. */
const PAGE = 12;

interface Props {
  saved: SavedBirth[];
  onChange: (list: SavedBirth[]) => void;
  /** Loading a saved chart reads it straight away: its UTC offset was already
   *  confirmed when it was first entered, so there is nothing left to ask. */
  onOpen: (birth: BirthInput, sex?: Sex) => void;
}

/**
 * Charts kept on this device, with what each one is.
 *
 * Birth details are the most personal thing the app holds, so the list says
 * where they are kept and offers to remove them, in plain sight rather than
 * in a settings page.
 */
export function SavedCharts({ saved, onChange, onOpen }: Props) {
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [failed, setFailed] = useState(false);
  // Searched and paged, not scrolled: this list is meant to hold hundreds.
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(PAGE);

  const hits = useMemo(() => {
    const q = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (q.length === 0) return saved;
    return saved.filter((s) => {
      const hay = `${s.name ?? ""} ${s.birth.date} ${s.birth.place ?? ""}`.toLowerCase();
      return q.every((w) => hay.includes(w));
    });
  }, [saved, query]);
  const shown = hits.slice(0, limit);

  // In a tab of its own an empty panel is a dead end, so say what to do. On
  // the landing page the caller only mounts this when there is something.
  if (saved.length === 0) {
    return (
      <section className="card saved-charts">
        <h3>Saved profiles</h3>
        <p className="hint">
          No profiles are saved on this device yet. Tick <strong>Save this profile on this
          device</strong> on any birth form - your own chart, a porutham partner, or a family
          member - and it will appear here to reuse.
        </p>
      </section>
    );
  }

  const apply = (out: { list: SavedBirth[]; stored: boolean }) => {
    onChange(out.list);
    setFailed(!out.stored);
  };

  return (
    <section className="card saved-charts" aria-labelledby="saved-charts-title">
      <h3 id="saved-charts-title">Saved profiles</h3>
      <p className="hint">
        These stay in this browser on this device. They are never sent anywhere, and
        clearing your site data removes them.
      </p>
      {saved.length > PAGE && (
        <label>Find a profile
          <input type="search" autoComplete="off" placeholder="A name, a date, or a place"
            value={query} onChange={(e) => { setQuery(e.target.value); setLimit(PAGE); }} />
        </label>
      )}
      {failed && (
        <p className="error" role="alert">
          This device would not let the change be saved, so the list above is what is
          actually kept.
        </p>
      )}
      {hits.length === 0 && (
        <p className="hint">Nothing saved here matches “{query}”.</p>
      )}
      <ul className="members saved-list">
        {shown.map((s) => (
          <li key={s.id} className="member">
            {editing === s.id ? (
              <div className="row">
                <label>Name
                  <input
                    autoFocus
                    value={draft}
                    maxLength={60}
                    onChange={(e) => setDraft(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") { apply(renameSaved(saved, s.id, draft)); setEditing(null); }
                      if (e.key === "Escape") setEditing(null);
                    }}
                  />
                </label>
                <button type="button" onClick={() => { apply(renameSaved(saved, s.id, draft)); setEditing(null); }}>Save name</button>
                <button type="button" className="link" onClick={() => setEditing(null)}>Cancel</button>
              </div>
            ) : (
              <div className="row saved-row">
                <button type="button" className="link saved-open" onClick={() => onOpen(s.birth, s.sex)}>
                  <strong>{s.name || "Unnamed profile"}</strong>
                  <span className="hint saved-when">{describe(s)}</span>
                </button>
                <span className="spacer" />
                <button type="button" onClick={() => { setEditing(s.id); setDraft(s.name ?? ""); }}>
                  {s.name ? "Rename" : "Add a name"}
                </button>
                <button type="button" onClick={() => apply(removeSaved(saved, s.id))}>Remove</button>
              </div>
            )}
          </li>
        ))}
      </ul>
      {hits.length > shown.length && (
        <button type="button" className="link" onClick={() => setLimit(limit + PAGE * 2)}>
          Show more — {shown.length} of {hits.length}{query ? " matching" : ""} shown
        </button>
      )}
    </section>
  );
}
