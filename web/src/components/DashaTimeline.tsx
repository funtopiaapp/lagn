import { useState } from "react";
import type { DashaView, Period } from "../types";

function Row({ p, depth }: { p: Period; depth: number }) {
  const [open, setOpen] = useState(p.current && depth < 2);
  const hasKids = !!p.children?.length;
  return (
    <li className={p.current ? "period current" : "period"} data-depth={depth}>
      <div className="period-line">
        {hasKids ? (
          <button type="button" className="toggle" aria-expanded={open} onClick={() => setOpen(!open)}>
            {open ? "−" : "+"}<span className="sr-only">{open ? " collapse" : " expand"} {p.lord}</span>
          </button>
        ) : <span className="toggle-spacer" />}
        <span className="lord">{p.lord}</span>
        <span className="dates">{p.start} → {p.end}</span>
        {p.from_birth && <span className="tag">from birth</span>}
        {p.current && <span className="tag now">running now</span>}
      </div>
      {hasKids && open && (
        <ul>{p.children!.map((c) => <Row key={`${c.lord}-${c.start}`} p={c} depth={depth + 1} />)}</ul>
      )}
    </li>
  );
}

export function DashaTimeline({ dasha }: { dasha: DashaView }) {
  return (
    <section className="card">
      <h3>Vimshottari dasha</h3>
      <p className="summary">
        Janma nakshatra <strong>{dasha.janma_nakshatra}</strong> ({dasha.janma_nakshatra_tamil}).
        Birth dasha {dasha.birth_lord}, running until {dasha.balance_until}.
        {dasha.running_now && <> Now: <strong>{dasha.running_now.maha} / {dasha.running_now.antar} / {dasha.running_now.pratyantar}</strong>.</>}
      </p>
      <ul className="dasha">{dasha.mahadashas.map((m) => <Row key={`${m.lord}-${m.start}`} p={m} depth={0} />)}</ul>
      <p className="hint">Dasha years of {dasha.year_length_days} days. Dates are in the birth's UTC offset.</p>
    </section>
  );
}
