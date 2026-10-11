import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BalaResponse } from "../types";
import type { BirthInput } from "../types";

const V = (n: number) => n.toFixed(2);

/**
 * Shadbala, five of six.
 *
 * This is the only view in the app whose numbers are not verified, and it is
 * written to say so rather than to look authoritative. The caveat sits above
 * the table, not below it; the missing component is named; and the customary
 * minimum strengths are shown greyed with an explicit note that they are
 * thresholds for a complete Shadbala and so cannot be compared against a
 * five-component total.
 *
 * No reading anywhere in the app consumes these numbers. That is enforced by
 * a test over the rule corpus, not by convention.
 */
export function BalaView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<BalaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<"none" | "sthana" | "kala">("none");

  useEffect(() => {
    let live = true;
    setData(null); setError(null);
    api.bala(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  return (
    <div className="jaimini bala">
      <section className="card">
        <h3>Shadbala — {data.components_computed} of 6 components</h3>
        <p className="notice warn" role="note">
          <strong>These numbers are not verified.</strong> {data.caveat}
        </p>
      </section>

      <section className="card">
        <h3>Totals</h3>
        <table className="grid" aria-label="Shadbala totals">
          <thead>
            <tr>
              <th>Graha</th><th>Sthana</th><th>Dig</th><th>Kala</th>
              <th>Cheshta</th><th>Naisargika</th><th>Virupas</th><th>Rupas</th>
              <th>Customary min</th>
            </tr>
          </thead>
          <tbody>
            {data.rows.map((r) => (
              <tr key={r.graha}>
                <th scope="row">{r.graha}</th>
                <td className="num">{V(r.sthana)}</td>
                <td className="num">{V(r.dig)}</td>
                <td className="num">{V(r.kala)}</td>
                <td className="num">
                  {V(r.cheshta)}
                  {r.cheshta_from_ayana && <span className="hint"> †</span>}
                </td>
                <td className="num">{V(r.naisargika)}</td>
                <td className="num"><strong>{V(r.total_virupas)}</strong></td>
                <td className="num">{V(r.total_rupas)}</td>
                <td className="num hint">
                  {r.customary_minimum_rupas?.toFixed(1) ?? "—"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="hint">
          † Surya and Chandra never retrograde, so their Cheshta is taken from Ayana.
        </p>
        <p className="hint">
          The customary minima are thresholds for a <em>complete</em> Shadbala. These totals
          contain five of the six components, so the two are not comparable — the column is here
          for reference, and nothing in the app tests against it.
        </p>
        <p className="hint">
          Sixty virupas make one rupa. Drik bala is absent: it needs sphuta drishti, which is
          degree-based and disputed, and a sign-based stand-in would produce a plausible number
          with no way to tell it was wrong.
        </p>
      </section>

      <section className="card">
        <div className="card-head">
          <h3>Component detail</h3>
          <div className="theme-toggle" role="group" aria-label="Detail">
            {([["none", "Hide"], ["sthana", "Sthana"], ["kala", "Kala"]] as const).map(([k, label]) => (
              <button key={k} type="button" aria-pressed={detail === k} onClick={() => setDetail(k)}>
                {label}
              </button>
            ))}
          </div>
        </div>

        {detail === "sthana" && (
          <table className="grid" aria-label="Sthana bala detail">
            <thead>
              <tr><th>Graha</th><th>Uchcha</th><th>Saptavargaja</th><th>Ojayugma</th><th>Kendra</th><th>Drekkana</th></tr>
            </thead>
            <tbody>
              {data.rows.map((r) => (
                <tr key={r.graha}>
                  <th scope="row">{r.graha}</th>
                  <td className="num">{V(r.uchcha)}</td>
                  <td className="num">{V(r.saptavargaja)}</td>
                  <td className="num">{V(r.ojayugma)}</td>
                  <td className="num">{V(r.kendra)}</td>
                  <td className="num">{V(r.drekkana)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {detail === "kala" && (
          <table className="grid" aria-label="Kala bala detail">
            <thead>
              <tr>
                <th>Graha</th><th>Nathonnatha</th><th>Paksha</th><th>Tribhaga</th>
                <th>Abda</th><th>Masa</th><th>Vara</th><th>Hora</th><th>Ayana</th>
              </tr>
            </thead>
            <tbody>
              {data.rows.map((r) => (
                <tr key={r.graha}>
                  <th scope="row">{r.graha}</th>
                  <td className="num">{V(r.nathonnatha)}</td>
                  <td className="num">{V(r.paksha)}</td>
                  <td className="num">{V(r.tribhaga)}</td>
                  <td className="num">{V(r.abda)}</td>
                  <td className="num">{V(r.masa)}</td>
                  <td className="num">{V(r.vara)}</td>
                  <td className="num">{V(r.hora)}</td>
                  <td className="num">{V(r.ayana)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {detail === "none" && (
          <p className="hint">Choose a component to see its parts.</p>
        )}
      </section>

      <section className="card">
        <h3>Variants in force — all unsigned</h3>
        <p className="notice" role="note">
          Shadbala was held back from this engine for a long time precisely because several of
          these have more than one published formula and no reference output was available to
          check against. Each choice below is this engine&apos;s, not a consensus.
        </p>
        <dl className="variants">
          {data.variants.map((v) => (
            <div key={v.id}>
              <dt>{v.id} · {v.question}</dt>
              <dd>{v.chosen}</dd>
            </div>
          ))}
        </dl>
      </section>
    </div>
  );
}
