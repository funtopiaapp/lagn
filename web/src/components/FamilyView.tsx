import { useRef, useState } from "react";
import { api, ApiError } from "../api";
import { exportJson, importJson, loadMembers, newId, RELATIONS, saveMembers, type Member, type Relation } from "../lib/family";
import type { BirthInput, FamilyReading, Lean, Sex } from "../types";
import { BirthForm } from "./BirthForm";
import { WriteUpView } from "./WriteUpView";

interface Props { birth: BirthInput; sex?: Sex; reviewToken: string }

const LEAN: Record<Lean, string> = { favourable: "Favourable", balanced: "Evenly balanced", calls_for_care: "Calls for care" };
const READ_FOR: Record<Relation, string> = {
  spouse: "marriage and the spouse (7th house)",
  child: "children (5th house)",
  mother: "the mother (4th house and the Moon)",
  father: "the father (9th house and the Sun)",
};
const AGREEMENT: Record<FamilyReading["agreement"], string> = {
  agree: "Both readings point the same way.",
  differ: "The two readings point different ways. Both are shown; neither overrides the other.",
  inconclusive: "No clear comparison: at least one reading is evenly balanced or has no reviewed indication.",
};
const leanText = (l: Lean | null) => (l ? LEAN[l] : "No reviewed indication");
const relLabel = (r: Relation) => RELATIONS.find(([k]) => k === r)![1];
/** Family names stay on the device, so the engine's text carries a placeholder. */
const withName = (text: string, name: string) => text.split("{member}").join(name);

type Pane = "yours" | "theirs" | "together";

function Reading({ m, f }: { m: Member; f: FamilyReading }) {
  const [pane, setPane] = useState<Pane>("yours");
  const who = m.name || relLabel(m.relation);
  const first = f.own.find((o) => o.report.topic === f.compared_with) ?? f.own[0];
  const panes: [Pane, string][] = [["yours", "Your chart"], ["theirs", `${who}'s chart`], ["together", "Together"]];

  return (
    <div className="family-reading" data-agreement={f.agreement}>
      <div className="verdict" data-agreement={f.agreement}>
        <h5>{who} · {relLabel(m.relation).toLowerCase()}</h5>
        <p>{AGREEMENT[f.agreement]}</p>
        <div className="leans">
          <div className="lean-card">
            <div className="whose">Your chart</div>
            <div className="lean">{leanText(f.relational.lean)}</div>
            <div className="read-for">read for {READ_FOR[m.relation]}</div>
          </div>
          <div className="lean-card">
            <div className="whose">{who}'s own chart</div>
            <div className="lean">{leanText(first?.lean ?? null)}</div>
            <div className="read-for">read for {first?.meta?.title.toLowerCase() ?? f.compared_with}, on their own birth details</div>
          </div>
        </div>
      </div>

      <nav className="subtabs" aria-label={`Readings for ${who}`}>
        {panes.map(([p, label]) => (
          <button key={p} type="button" aria-current={pane === p ? "page" : undefined} onClick={() => setPane(p)}>{label}</button>
        ))}
      </nav>

      {pane === "yours" && (
        <section aria-label="Your chart">
          <p className="hint">Your own chart, read for {READ_FOR[m.relation]}. This is what this relationship tends to bring into your life.</p>
          {f.relational.writeup ? <WriteUpView w={f.relational.writeup} level={5} /> : <p className="notice">No reviewed interpretations are available yet.</p>}
        </section>
      )}

      {pane === "theirs" && (
        <section aria-label={`${who}'s chart`}>
          <p className="hint">{who}'s own chart, computed from their birth details. This is about their life, not yours.</p>
          {f.own.map((o) => (
            <details key={o.report.topic} className="own-topic" open={o.report.topic === f.compared_with}>
              <summary>{o.meta?.title ?? o.report.topic} — {leanText(o.lean)}</summary>
              {o.meta?.disclaimer && <p className="notice disclaimer" role="note">{o.meta.disclaimer}</p>}
              {o.writeup ? <WriteUpView w={o.writeup} level={5} omit={o.meta?.disclaimer} /> : <p className="hint">No reviewed interpretations are available for this topic yet.</p>}
            </details>
          ))}
        </section>
      )}

      {pane === "together" && (
        <section className="together" aria-label="Together">
          {f.comparison.map((line, i) => <p key={i} className={i === 0 ? "hint" : undefined}>{withName(line, who)}</p>)}
        </section>
      )}
    </div>
  );
}

/** Spouse, children and parents: each read on their own chart and in yours. */
export function FamilyView({ birth, sex, reviewToken }: Props) {
  const [members, setMembers] = useState<Member[]>(loadMembers);
  const [stored, setStored] = useState(true);
  const [adding, setAdding] = useState(false);
  const [draft, setDraft] = useState<{ name: string; relation: Relation }>({ name: "", relation: "spouse" });
  const [readings, setReadings] = useState<Record<string, FamilyReading>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const update = (next: Member[]) => { setMembers(next); setStored(saveMembers(next)); };

  const add = (b: BirthInput, memberSex?: Sex) => {
    update([...members, { id: newId(), name: draft.name.trim().slice(0, 80), relation: draft.relation, sex: memberSex, birth: b }]);
    setAdding(false);
    setDraft({ name: "", relation: "spouse" });
  };

  const remove = (m: Member) => {
    if (!confirm(`Remove ${m.name || relLabel(m.relation)} from this device?`)) return;
    update(members.filter((x) => x.id !== m.id));
    setReadings(({ [m.id]: _, ...rest }) => rest);
  };

  const read = async (m: Member) => {
    setBusy(m.id); setError(null);
    try {
      const r = await api.family({ birth, sex }, { birth: m.birth, sex: m.sex }, m.relation, reviewToken || undefined);
      setReadings((x) => ({ ...x, [m.id]: r }));
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(null); }
  };

  const doExport = () => {
    const url = URL.createObjectURL(new Blob([exportJson(members)], { type: "application/json" }));
    const a = document.createElement("a");
    a.href = url; a.download = "lagn-family.json"; a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };

  const doImport = async (file: File) => {
    setError(null);
    try {
      const incoming = importJson(await file.text());
      const known = new Set(members.map((m) => m.id));
      update([...members, ...incoming.filter((m) => !known.has(m.id))]);
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
  };

  return (
    <section className="card">
      <h3>Family</h3>
      <p className="hint">
        Each member is read on their own chart and in yours: the 7th house for a spouse, the 5th for children, the
        4th and Moon for the mother, and the 9th and Sun for the father. Birth details are kept only on this device
        and sent with each reading; the server stores nothing.
      </p>
      {!stored && <p className="warn" role="alert">This browser is not letting the app save data, so members last only until the page closes.</p>}
      {error && <p className="error" role="alert">{error}</p>}

      {members.length === 0 && !adding && <p className="notice">No family members yet.</p>}
      <ul className="rules members">
        {members.map((m) => (
          <li key={m.id} className="rule member" data-member={m.relation}>
            <div className="rule-head">
              <strong>{m.name || relLabel(m.relation)}</strong>
              <span className="tag">{relLabel(m.relation)}</span>
              <span className="dates">{m.birth.date} {m.birth.time.slice(0, 5)}{m.birth.place ? `, ${m.birth.place}` : ""}</span>
            </div>
            <div className="row">
              <button type="button" className="primary" onClick={() => read(m)} disabled={busy !== null}>{busy === m.id ? "Working…" : "Read"}</button>
              <button type="button" onClick={() => remove(m)}>Remove</button>
            </div>
            {readings[m.id] && <Reading m={m} f={readings[m.id]!} />}
          </li>
        ))}
      </ul>

      {adding ? (
        <div className="add-member">
          <div className="row">
            <label>Name (optional)<input value={draft.name} maxLength={80} onChange={(e) => setDraft({ ...draft, name: e.target.value })} /></label>
            <label>Relation
              <select value={draft.relation} onChange={(e) => setDraft({ ...draft, relation: e.target.value as Relation })}>
                {RELATIONS.map(([k, v]) => <option key={k} value={k}>{v}</option>)}
              </select>
            </label>
          </div>
          <BirthForm title="Member's birth details" submitLabel="Save member" onSubmit={add} askSex />
          <button type="button" className="link" onClick={() => setAdding(false)}>Cancel</button>
        </div>
      ) : (
        <div className="row">
          <button type="button" onClick={() => setAdding(true)}>Add a member</button>
          <button type="button" onClick={doExport} disabled={members.length === 0}>Export</button>
          <button type="button" onClick={() => fileRef.current?.click()}>Import</button>
          <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={(e) => { const f = e.target.files?.[0]; if (f) void doImport(f); e.target.value = ""; }} />
        </div>
      )}
    </section>
  );
}
