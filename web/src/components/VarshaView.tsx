import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, VarshaResponse } from "../types";

/**
 * The annual (solar return) chart, with Muntha and kaksha transit.
 *
 * The age is the one input this view takes, because an annual chart is
 * per-year by definition. It shows the solve's residual - how far the annual
 * Sun sits from the natal Sun - rather than hiding it: the whole claim of a
 * solar return is that the Sun has come back exactly, and a reader is
 * entitled to see that it did.
 *
 * The kaksha table reads the *annual* chart's positions against the *natal*
 * Ashtakavarga, which is what makes it a transit rather than a second chart's
 * own bindus.
 */
export function VarshaView({ birth }: { birth: BirthInput }) {
  const [age, setAge] = useState(45);
  const [data, setData] = useState<VarshaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setError(null);
    api.varsha(birth, age)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth, age]);

  return (
    <div className="jaimini varsha">
      <section className="card">
        <h3>Annual chart</h3>
        <label className="varsha-age">
          Year the native completes age
          <input
            type="number"
            min={0}
            max={120}
            value={age}
            onChange={(e) => setAge(Math.max(0, Math.min(120, Number(e.target.value) || 0)))}
          />
        </label>
        {error && <p className="error" role="alert">{error}</p>}
        {!error && !data && <p className="hint">Computing…</p>}
        {data && (
          <>
            <p className="summary">
              Year {data.age} begins <strong>{data.begins}</strong>, local to the birth place.
              Annual lagna <strong>{data.lagna}</strong>; Muntha in <strong>{data.muntha}</strong>,
              house {data.muntha_house}.
            </p>
            <p className="hint">
              The moment is solved, not stepped: the Sun at it sits{" "}
              {data.sun_error.toExponential(1)}° from its natal longitude. Muntha starts in the
              natal lagna and advances one sign for each completed year.
            </p>
          </>
        )}
      </section>

      {data && (
        <>
          <section className="card">
            <h3>Positions</h3>
            <table className="grid" aria-label="Annual chart positions">
              <thead>
                <tr><th>Graha</th><th>Rasi</th><th>Degrees</th><th>House</th></tr>
              </thead>
              <tbody>
                {data.positions.map((p) => (
                  <tr key={p.graha}>
                    <th scope="row">{p.graha}{p.retrograde && <span className="hint"> R</span>}</th>
                    <td>{p.rasi}<span className="hint"> / {p.rasi_tamil}</span></td>
                    <td className="num">{p.degrees}</td>
                    <td className="num">{p.house}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>

          <section className="card">
            <h3>Kaksha</h3>
            <p className="hint">
              Each sign divides into eight kakshas of 3°45′, one per Ashtakavarga contributor. A
              transit is supported when its kaksha&apos;s owner gave a bindu there in the{" "}
              <em>natal</em> chart. The nodes are absent: they have no Bhinnashtakavarga.
            </p>
            <table className="grid" aria-label="Kaksha transit">
              <thead>
                <tr><th>Graha</th><th>Rasi</th><th>Kaksha</th><th>Owner</th><th>Bindus</th><th></th></tr>
              </thead>
              <tbody>
                {data.kaksha.map((k) => (
                  <tr key={k.graha}>
                    <th scope="row">{k.graha}</th>
                    <td>{k.rasi}</td>
                    <td className="num">{k.kaksha}</td>
                    <td>{k.owner}</td>
                    <td className="num">{k.bindus}</td>
                    <td>
                      <span className={`chip ${k.supported ? "ok" : "warn"}`}>
                        {k.supported ? "supported" : "unsupported"}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>

          <section className="card">
            <h3>Variants in force</h3>
            <p className="notice" role="note">
              None of these has been signed off by an astrologer. Varshesha, the sahams and the
              Patyayini and Mudda dashas are deliberately absent: each is a published table rather
              than a derivation, and there is no identity to check a misremembering against.
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
        </>
      )}
    </div>
  );
}
