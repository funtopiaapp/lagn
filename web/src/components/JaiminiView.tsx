import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { ArgalaVerdict, BirthInput, JaiminiResponse } from "../types";

const VERDICT: Record<ArgalaVerdict, { label: string; chip: string }> = {
  stands: { label: "stands", chip: "ok" },
  neutralised: { label: "neutralised", chip: "" },
  overcome: { label: "overcome", chip: "warn" },
  none: { label: "—", chip: "" },
};

/**
 * Jaimini core: the eight chara karakas, the twelve arudha padas, and the
 * argala acting on each sign.
 *
 * This is the professional surface, so it is written for someone who already
 * knows the words. It still shows its working - the count to the lord, where
 * the pada landed before the exception moved it, how many grahas stood on
 * each side of an argala - because an arudha is the one Jaimini quantity a
 * practitioner most often recomputes by hand, and the point is to let them
 * check this rather than take it on trust.
 *
 * Names - of grahas, signs and karakas - come from the engine rather than
 * from a table here, so this tab says "Kuja" where every other tab does.
 *
 * Every variant in force is named at the foot, by its ID from
 * docs/phase13/DESIGN.md section 7. None of them is signed off yet.
 */
export function JaiminiView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<JaiminiResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setError(null);
    api.jaimini(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  return (
    <div className="jaimini">
      <section className="card">
        <h3>Chara karakas</h3>
        <p className="hint">
          The eight ranked by how far each has travelled through its sign. Rahu is read
          backwards, so its figure is the part of the sign behind it.
        </p>
        <table className="grid" aria-label="Chara karakas">
          <thead>
            <tr><th>Rank</th><th>Karaka</th><th>Graha</th><th>Rasi</th><th>Advanced</th></tr>
          </thead>
          <tbody>
            {data.karakas.map((a) => (
              <tr key={a.id}>
                <th scope="row">{a.abbrev}</th>
                <td>{a.name}<span className="hint"> · {a.signifies}</span></td>
                <td>{a.graha}</td>
                <td>{a.rasi}</td>
                <td className="num">{a.advancement.toFixed(3)}°</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Arudha padas</h3>
        <p className="hint">
          Counted from each bhava to its lord, then the same count on from the lord. Where that
          would land on the bhava itself or opposite it, the tenth is taken instead — marked below.
        </p>
        <table className="grid" aria-label="Arudha padas">
          <thead>
            <tr><th>Pada</th><th>Bhava</th><th>Lord</th><th>Lord in</th><th>Count</th><th>Pada</th></tr>
          </thead>
          <tbody>
            {data.padas.map((p) => (
              <tr key={p.bhava}>
                <th scope="row">{p.label}</th>
                <td>{p.bhava_rasi}</td>
                <td>{p.lord}</td>
                <td>{p.lord_rasi}</td>
                <td className="num">{p.count}</td>
                <td>
                  {p.rasi}
                  {p.adjusted && (
                    <span className="hint"> · 10th from {p.raw}</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Argala and virodhargala</h3>
        <p className="hint">
          Intervention from the 2nd, 4th and 11th, each countered from the 12th, 10th and 3rd.
          The figures are grahas intervening against grahas countering.
        </p>
        <table className="grid argala" aria-label="Argala and virodhargala">
          <thead>
            <tr><th>Sign</th><th>Wealth (2 / 12)</th><th>Home (4 / 10)</th><th>Gain (11 / 3)</th></tr>
          </thead>
          <tbody>
            {data.argala.map((a) => (
              <tr key={a.rasi}>
                <th scope="row">{a.rasi}</th>
                {a.pairs.map((p) => (
                  <td key={p.kind}>
                    <span className={`chip ${VERDICT[p.verdict].chip}`}>{VERDICT[p.verdict].label}</span>
                    <span className="num"> {p.argala_grahas.length} v {p.counter_grahas.length}</span>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Variants in force</h3>
        <p className="notice" role="note">
          Jaimini has more live disagreement in it than Parashara does. These are the choices this
          engine makes, and none of them has been signed off by an astrologer yet. They are listed
          so you can tell which scheme produced what you are reading.
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
