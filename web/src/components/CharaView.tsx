import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, CharaPeriod, CharaResponse } from "../types";

/**
 * Chara dasha, the Jaimini rasi dasha.
 *
 * Written for someone who already knows the words, and showing its working
 * for the same reason the Jaimini view does: a practitioner will want to
 * check the length of a sign's dasha against their own count.
 *
 * The two directions in play are not the same one, and this says so plainly,
 * because it is the single most likely source of a difference from other
 * software. The order the signs are visited in comes from the lagna's parity
 * (V-13-9). The count that measures a sign's length runs in that sign's own
 * parity (V-13-10) - so an odd lagna still counts its even signs backwards.
 *
 * Dates come formatted from the engine. Nothing here does any calendar
 * arithmetic.
 */
export function CharaView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<CharaResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setError(null); setOpen(null);
    api.chara(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  const running = data.running_now;
  const key = (p: CharaPeriod) => `${p.cycle}-${p.rasi}`;

  return (
    <div className="jaimini chara">
      <section className="card">
        <h3>Chara dasha</h3>
        <p className="summary">
          Lagna in <strong>{data.lagna}</strong>, {data.lagna_is_odd ? "an odd sign" : "an even sign"},
          so the sequence runs {data.direction === "direct" ? "zodiacally" : "anti-zodiacally"}.
          One cycle is <strong>{data.cycle_years} years</strong>.
          {running && <> Running now: <strong>{running.maha}</strong> / {running.antar}.</>}
        </p>
        <p className="hint">
          Note the two directions are different. The order above comes from the lagna. The count
          that sets each sign's length below runs in <em>that sign's</em> own direction, so an odd
          lagna still counts its even signs backwards.
        </p>
      </section>

      <section className="card">
        <h3>How long each sign runs</h3>
        <table className="grid" aria-label="Chara dasha lengths">
          <thead>
            <tr>
              <th>Rasi</th><th>Lord</th><th>Lord in</th><th>Counted</th><th>Count</th><th>Years</th>
            </tr>
          </thead>
          <tbody>
            {data.lengths.map((l) => (
              <tr key={l.rasi}>
                <th scope="row">{l.rasi}</th>
                <td>{l.lord}</td>
                <td>{l.lord_rasi}</td>
                <td>{l.direction === "direct" ? "forwards" : "backwards"}</td>
                <td className="num">{l.count}</td>
                <td className="num">
                  {l.years}
                  {l.lord_at_home && <span className="hint"> · lord at home</span>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="hint">
          A sign's dasha is its count to its lord, less one. Where the lord sits in the sign itself
          the count is one, which would leave nothing, so the period is twelve years instead.
        </p>
      </section>

      <section className="card">
        <h3>Periods</h3>
        <p className="hint">
          A cycle can be as short as twelve years, so the sequence repeats. Select a period to see
          its twelve antardashas, which divide it equally and open on the period's own sign.
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
                <strong>{p.rasi}</strong>
                <span className="chara-span">{p.start} – {p.end}</span>
                {data.periods.length > 12 && <span className="chip">cycle {p.cycle}</span>}
                {running && running.maha === p.rasi && p.start <= running.as_of_utc && running.as_of_utc < p.end && (
                  <span className="tag now">running now</span>
                )}
              </button>
              {open === key(p) && p.children && (
                <table className="grid" aria-label={`Antardashas of ${p.rasi}, cycle ${p.cycle}`}>
                  <thead><tr><th>Antardasha</th><th>From</th><th>To</th></tr></thead>
                  <tbody>
                    {p.children.map((c) => (
                      <tr key={c.rasi}>
                        <th scope="row">{c.rasi}</th>
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
          None of these has been signed off by an astrologer. V-13-9 is the one most likely to
          explain a difference from other software: the sequence's direction is taken from the
          lagna's odd or even parity, where some implementations use its navamsa instead.
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
