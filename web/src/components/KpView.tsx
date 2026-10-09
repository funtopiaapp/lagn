import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, KpLords, KpResponse } from "../types";

/** The four lords, as the columns practice reads them in. */
function LordCells({ l }: { l: KpLords }) {
  return (
    <>
      <td>{l.rasi}</td>
      <td className="num">{l.degrees}</td>
      <td>{l.nakshatra}<span className="hint"> · {l.pada}</span></td>
      <td>{l.sign_lord}</td>
      <td>{l.star_lord}</td>
      <td><strong>{l.sub_lord}</strong></td>
      <td>{l.sub_sub_lord}</td>
    </>
  );
}

const HEAD = (
  <tr>
    <th>Point</th><th>Rasi</th><th>Degrees</th><th>Nakshatra · pada</th>
    <th>Sign</th><th>Star</th><th>Sub</th><th>Sub-sub</th>
  </tr>
);

/**
 * A Krishnamurti Paddhati reading.
 *
 * KP is the one school in this app that reads houses as Placidus cusps rather
 * than as whole signs, and the sub lord rather than the sign is what it
 * weighs - so the sub column is emphasised and the cusps get a table of their
 * own. The significators carry the group number that earned them their place,
 * because a KP list is read as a ranking and a reader needs to see why
 * something is near the top.
 */
export function KpView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<KpResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setError(null);
    api.kp(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  return (
    <div className="jaimini kp">
      <section className="card">
        <h3>The four lords</h3>
        <p className="hint">
          Each nakshatra is divided into nine subs in Vimshottari order from its own lord, each
          proportional to that lord&apos;s years out of 120. The sub is what KP weighs, so it is the
          column to read first.
        </p>
        <table className="grid" aria-label="The four lords">
          <thead>{HEAD}</thead>
          <tbody>
            <tr>
              <th scope="row">Ascendant</th>
              <LordCells l={data.ascendant} />
            </tr>
            {data.grahas.map((g) => (
              <tr key={g.graha}>
                <th scope="row">{g.graha}</th>
                <LordCells l={g} />
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Placidus cusps</h3>
        <p className="hint">
          KP reads houses as cusps, not as whole signs — so these differ from the Chart tab, which
          uses the South Indian whole-sign bhavas throughout.
        </p>
        <table className="grid" aria-label="Placidus cusps">
          <thead>
            <tr>
              <th>House</th><th>Rasi</th><th>Degrees</th><th>Nakshatra · pada</th>
              <th>Sign</th><th>Star</th><th>Sub</th><th>Sub-sub</th>
            </tr>
          </thead>
          <tbody>
            {data.cusps.map((c) => (
              <tr key={c.house}>
                <th scope="row">{c.house}</th>
                <LordCells l={c} />
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Ruling planets</h3>
        <p className="hint">Taken at the moment the chart is cast, each with the sub practice weighs it by.</p>
        <table className="grid" aria-label="Ruling planets">
          <thead><tr><th>Role</th><th>Graha</th><th>Sub</th></tr></thead>
          <tbody>
            {data.ruling_planets.map((r) => (
              <tr key={r.role}>
                <th scope="row">{r.role}</th>
                <td>{r.graha}</td>
                <td>{r.sub_lord ?? <span className="hint">—</span>}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Significators</h3>
        <p className="hint">
          Strongest first. The number is the group that earned the place: 1 in the star of an
          occupant, 2 occupies the house, 3 in the star of the house lord, 4 the lord itself.
        </p>
        <table className="grid" aria-label="House significators">
          <thead><tr><th>House</th><th>Significators, strongest first</th></tr></thead>
          <tbody>
            {data.significators.map((h) => (
              <tr key={h.house}>
                <th scope="row">{h.house}</th>
                <td>
                  {h.significators.map((s) => (
                    <span key={s.graha} className="kp-sig" title={s.because}>
                      {s.graha}<span className="kp-rank">{s.rank}</span>
                    </span>
                  ))}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Variants in force</h3>
        <p className="notice" role="note">
          None of these has been signed off by an astrologer. V-13-18 is the one to check first: a
          KP reading here uses the chart&apos;s own ayanamsa, so to read it the way KP practice
          does, select Krishnamurti on the birth form and every tab will agree.
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
