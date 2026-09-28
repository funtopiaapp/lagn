import type { Suggested } from "../types";

/** Pariharams as optional traditional practices, each with what brought it up. */
export function PariharamList({ items }: { items: Suggested[] }) {
  if (items.length === 0) return null;
  return (
    <div className="pariharams">
      <p className="hint">
        Optional traditional practices, as commonly followed in Tamil Nadu and Kerala. No gemstones or paid
        services are suggested, and none of this replaces medical, legal or financial advice.
      </p>
      <ul className="rules">
        {items.map(({ pariharam: p, because }) => (
          <li key={p.id} className="rule pariharam" data-pariharam={p.id}>
            <div className="rule-head"><strong>{p.title}</strong></div>
            <ul className="plain">
              {p.practices.map((x) => <li key={x}>{x}</li>)}
            </ul>
            {(p.deity || p.day) && <p className="facts">{[p.deity && `Deity: ${p.deity}`, p.day && `Day: ${p.day}`].filter(Boolean).join(" · ")}</p>}
            {p.places.length > 0 && <p className="facts">Temples: {p.places.join("; ")}</p>}
            {p.charity && <p className="facts">Charity: {p.charity}</p>}
            <p className="facts">Because: {because.join("; ")}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
