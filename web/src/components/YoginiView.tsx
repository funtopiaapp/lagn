import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, YoginiPeriod, YoginiResponse } from "../types";

/** What each yogini is held to be about. Stated, never interpreted. */
const WHAT: Record<string, string> = {
  Mangala: "the auspicious one",
  Pingala: "the tawny one",
  Dhanya: "the fortunate one",
  Bhramari: "the wandering one",
  Bhadrika: "the gentle one",
  Ulka: "the meteor",
  Siddha: "the accomplished one",
  Sankata: "the straitened one",
};

/**
 * Yogini dasha: the 36-year cycle of eight yoginis.
 *
 * The periods are the consecutive years 1 to 8, which is why the cycle is 36
 * and why this dasha could be built from a specification while most of its
 * siblings could not. The view says so, because a practitioner comparing
 * against other software will want to know what is being relied on.
 *
 * Dates arrive formatted from the engine. Nothing here does any calendar
 * arithmetic.
 */
export function YoginiView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<YoginiResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setError(null); setOpen(null);
    api.yogini(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  const running = data.running_now;
  const key = (p: YoginiPeriod) => `${p.cycle}-${p.yogini}`;

  return (
    <div className="jaimini chara">
      <section className="card">
        <h3>Yogini dasha</h3>
        <p className="summary">
          Janma nakshatra <strong>{data.janma_nakshatra}</strong>, so the cycle opens in{" "}
          <strong>{data.birth_yogini}</strong> ({data.birth_lord}), with{" "}
          <strong>{data.balance_years.toFixed(3)} years</strong> of it left at birth — running to{" "}
          {data.balance_until}.
          {running && <> Running now: <strong>{running.maha}</strong> / {running.antar}.</>}
        </p>
        <p className="hint">
          The eight periods are 1, 2, 3, 4, 5, 6, 7 and 8 years, so a full cycle is{" "}
          {data.cycle_years} years and repeats. The first period opens <em>before</em> birth: the
          yogini was already running, and keeping its elapsed part is what makes the sub-period
          boundaries inside it correct.
        </p>
      </section>

      <section className="card">
        <h3>Periods</h3>
        <p className="hint">
          Select a period to see its eight antardashas, which divide it in proportion to each
          yogini&apos;s own years and open on the period&apos;s own yogini.
        </p>
        <ol className="chara-periods">
          {data.periods.map((p) => (
            <li key={key(p)}>
              <button
                type="button"
                className="chara-period"
                aria-expanded={open === key(p)}
                onClick={() => setOpen(open === key(p) ? null : key(p))}
              >
                <strong>{p.yogini}</strong>
                <span className="hint">{p.lord} · {p.years}y</span>
                <span className="chara-span">{p.start} – {p.end}</span>
                <span className="chip">cycle {p.cycle}</span>
                {running && running.maha === p.yogini
                  && p.start <= running.as_of_utc && running.as_of_utc < p.end && (
                  <span className="tag now">running now</span>
                )}
              </button>
              {open === key(p) && p.children && (
                <table className="grid" aria-label={`Antardashas of ${p.yogini}, cycle ${p.cycle}`}>
                  <thead><tr><th>Antardasha</th><th>Lord</th><th>From</th><th>To</th></tr></thead>
                  <tbody>
                    {p.children.map((c) => (
                      <tr key={c.yogini}>
                        <th scope="row">
                          {c.yogini}
                          {WHAT[c.yogini] && <span className="hint"> · {WHAT[c.yogini]}</span>}
                        </th>
                        <td>{c.lord}</td>
                        <td>{c.start}</td>
                        <td>{c.end}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </li>
          ))}
        </ol>
      </section>

      <section className="card">
        <h3>Variants in force</h3>
        <p className="notice" role="note">
          None of these has been signed off by an astrologer. V-13-33 is the one to check first:
          the cycle&apos;s starting yogini comes from the janma nakshatra&apos;s number plus three,
          and a remainder of zero is read as the eighth yogini rather than the first.
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
