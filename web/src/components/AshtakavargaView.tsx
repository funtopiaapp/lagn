import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { AshtakavargaResponse, BirthInput } from "../types";

/** A bindu count, shaded so the eye finds the strong and weak signs. */
function Cell({ n, title }: { n: number; title?: string }) {
  const band = n >= 5 ? "strong" : n <= 2 ? "weak" : "";
  return <td className={`num av-cell ${band}`} title={title}>{n}</td>;
}

/**
 * The full Ashtakavarga grid.
 *
 * Nothing here is newly computed: the engine has produced BAV, SAV and the
 * prastara since phase 2B, cross-validated against an independent
 * reimplementation. This surfaces it.
 *
 * The fixed totals are shown beside the computed ones on purpose. A graha's
 * BAV always comes to the same number whatever the chart, and the SAV always
 * to 337 - so a reader can watch the invariant hold instead of trusting the
 * grid. If one ever disagreed, the view says so rather than quietly printing
 * a wrong number.
 */
export function AshtakavargaView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<AshtakavargaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [frame, setFrame] = useState<"sign" | "house">("sign");

  useEffect(() => {
    let live = true;
    setData(null); setError(null);
    api.ashtakavarga(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  const byHouse = frame === "house";
  const headings = byHouse
    ? Array.from({ length: 12 }, (_, i) => String(i + 1))
    : data.signs.map((s) => s.slice(0, 3));
  const mismatched = data.rows.filter((r) => r.total !== r.expected_total);
  const savOff = data.sav_total !== data.sav_expected_total;

  return (
    <div className="jaimini ashtakavarga">
      <section className="card">
        <div className="card-head">
          <h3>Ashtakavarga</h3>
          <div className="theme-toggle" role="group" aria-label="Frame">
            {([["sign", "By sign"], ["house", "By house"]] as const).map(([k, label]) => (
              <button key={k} type="button" aria-pressed={frame === k} onClick={() => setFrame(k)}>
                {label}
              </button>
            ))}
          </div>
        </div>
        <p className="hint">
          Lagna in {data.lagna}. Each graha&apos;s Bhinnashtakavarga counts the bindus the eight
          contributors give it in every sign; the Sarvashtakavarga is their sum. Five or more is
          marked strong, two or fewer weak — a convention for reading, not a verdict.
        </p>
        {(mismatched.length > 0 || savOff) && (
          <p className="notice warn" role="alert">
            An invariant failed: {mismatched.map((r) => r.graha).join(", ")}
            {savOff && (mismatched.length > 0 ? " and the SAV" : "the SAV")} did not come to the
            fixed total. This should be impossible, and the numbers above should not be relied on.
          </p>
        )}
      </section>

      <section className="card">
        <table className="grid av" aria-label="Bhinnashtakavarga">
          <thead>
            <tr>
              <th>{byHouse ? "House" : "Graha"}</th>
              {headings.map((h) => <th key={h} className="num">{h}</th>)}
              <th className="num">Total</th>
              <th className="num">Always</th>
            </tr>
          </thead>
          <tbody>
            {data.rows.map((r, gi) => (
              <tr key={r.graha}>
                <th scope="row">{r.graha}</th>
                {(byHouse ? r.by_house : r.bindus).map((n, i) => (
                  <Cell
                    key={i}
                    n={n}
                    title={
                      byHouse
                        ? undefined
                        : `${data.signs[i]}: ${
                            (data.contributors[gi]?.[i] ?? []).join(", ") || "no contributor"
                          }`
                    }
                  />
                ))}
                <td className="num"><strong>{r.total}</strong></td>
                <td className="num hint">{r.expected_total}</td>
              </tr>
            ))}
            <tr className="sav-row">
              <th scope="row">SAV</th>
              {(byHouse ? data.sav_by_house : data.sav).map((n, i) => (
                <td key={i} className="num"><strong>{n}</strong></td>
              ))}
              <td className="num"><strong>{data.sav_total}</strong></td>
              <td className="num hint">{data.sav_expected_total}</td>
            </tr>
          </tbody>
        </table>
        <p className="hint">
          The last column is the total that graha&apos;s BAV always comes to, whatever the chart —
          and the SAV always comes to {data.sav_expected_total}. Hover a cell in the by-sign view
          to see which contributors gave its bindus.
        </p>
      </section>
    </div>
  );
}
