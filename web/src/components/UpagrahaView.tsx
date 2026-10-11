import { useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, UpaPoint, UpagrahaResponse, YogiResponse } from "../types";

/** What each point is, for a practitioner who wants the gloss and not a lesson. */
const WHAT: Record<string, string> = {
  Dhuma: "smoke; read with the Sun's harsher side",
  Vyatipata: "calamity; the reflection of Dhuma",
  Parivesha: "the halo; opposite Vyatipata",
  Indrachapa: "the rainbow; also called Kodanda",
  Upaketu: "the comet; also Sikhi, and always 30° behind the Sun",
  Kaala: "Surya's eighth of the period",
  Mrityu: "Kuja's eighth",
  Ardhaprahara: "Budha's eighth",
  Yamaghantaka: "Guru's eighth",
  Gulika: "Shani's eighth; the point South Indian practice calls Mandi",
  "Bhava lagna": "one sign every two hours from sunrise",
  "Hora lagna": "one sign every hour from sunrise",
  "Ghati lagna": "one sign every 24 minutes from sunrise",
};

function Rows({ points, label }: { points: UpaPoint[]; label: string }) {
  return (
    <table className="grid" aria-label={label}>
      <thead>
        <tr><th>Point</th><th>Rasi</th><th>Degrees</th><th>House</th></tr>
      </thead>
      <tbody>
        {points.map((p) => (
          <tr key={p.name}>
            <th scope="row">
              {p.name}
              {WHAT[p.name] && <span className="hint"> · {WHAT[p.name]}</span>}
            </th>
            <td>{p.rasi}<span className="hint"> / {p.rasi_tamil}</span></td>
            <td className="num">{p.degrees}</td>
            <td className="num">{p.house}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/**
 * Upagrahas and the time lagnas.
 *
 * Three groups that practice keeps together and that are computed quite
 * differently: five fixed offsets from the Sun, five ascendants taken at the
 * start of a particular eighth of the day, and three points that march from
 * sunrise at a fixed rate. The headings say which is which, because a
 * practitioner checking one of them needs to know what to check it against.
 */
export function UpagrahaView({ birth }: { birth: BirthInput }) {
  const [data, setData] = useState<UpagrahaResponse | null>(null);
  const [yogi, setYogi] = useState<YogiResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setData(null); setYogi(null); setError(null);
    api.upagraha(birth)
      .then((d) => { if (live) setData(d); })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    // Yogi and Avayogi are derived points too, so they belong on this tab -
    // but they are not upagrahas, and the heading below says so. Fetched
    // separately so a failure in one does not blank the other.
    api.yogi(birth)
      .then((d) => { if (live) setYogi(d); })
      .catch(() => { /* its own card simply does not appear */ });
    return () => { live = false; };
  }, [birth]);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!data) return <p className="hint">Computing…</p>;

  return (
    <div className="jaimini">
      <section className="card">
        <h3>Upagrahas</h3>
        <p className="summary">
          A <strong>{data.at_night ? "night" : "day"}</strong> birth on a <strong>{data.vara}</strong>
          {" "}({data.vara_lord}), {data.hours_since_sunrise.toFixed(2)} hours after sunrise.
        </p>
        <p className="hint">
          {data.at_night
            ? "A night birth divides the night into eight, and the sequence starts from the lord fifth from the weekday lord."
            : "A day birth divides sunrise to sunset into eight, and the first part belongs to the lord of the weekday."}
        </p>
      </section>

      <section className="card">
        <h3>From the Sun</h3>
        <p className="hint">
          Fixed offsets from the Sun's own longitude, so these move only as the Sun does. The five
          compose into one another, which is why Upaketu always lands exactly 30° behind the Sun.
        </p>
        <Rows points={data.sun_offsets} label="Upagrahas from the Sun" />
      </section>

      <section className="card">
        <h3>From the eight-part division of the {data.at_night ? "night" : "day"}</h3>
        <p className="hint">
          Each is the ascendant at the moment its ruler's eighth begins. Gulika is the one practice
          reaches for most.
        </p>
        <table className="grid" aria-label="Upagrahas from the day division">
          <thead>
            <tr><th>Point</th><th>Ruler</th><th>Part</th><th>Rasi</th><th>Degrees</th><th>House</th></tr>
          </thead>
          <tbody>
            {data.day_parts.map((p) => (
              <tr key={p.name}>
                <th scope="row">
                  {p.name}
                  {WHAT[p.name] && <span className="hint"> · {WHAT[p.name]}</span>}
                </th>
                <td>{p.ruler}</td>
                <td className="num">{p.part}</td>
                <td>{p.rasi}<span className="hint"> / {p.rasi_tamil}</span></td>
                <td className="num">{p.degrees}</td>
                <td className="num">{p.house}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h3>Time lagnas</h3>
        <p className="hint">
          All three start from the Sun's position at that day's sunrise and advance at a fixed rate,
          so at sunrise itself each one sits on the Sun.
        </p>
        <Rows points={data.time_lagnas} label="Time lagnas" />
      </section>

      {yogi && (
        <section className="card">
          <h3>Yogi and Avayogi</h3>
          <p className="hint">
            Not upagrahas, but derived points of the same kind. The Yoga sphuta is the Sun plus the
            Moon plus 93°20′ — which is exactly seven nakshatras, so the Avayogi&apos;s lord always
            sits five places past the Yogi&apos;s in the Vimshottari cycle.
          </p>
          <table className="grid" aria-label="Yogi and Avayogi">
            <tbody>
              <tr>
                <th scope="row">Yoga sphuta</th>
                <td>{yogi.rasi}<span className="hint"> / {yogi.rasi_tamil}</span></td>
                <td className="num">{yogi.degrees}</td>
                <td>{yogi.nakshatra}<span className="hint"> pada {yogi.pada}</span></td>
              </tr>
              <tr>
                <th scope="row">Yogi</th>
                <td><strong>{yogi.yogi}</strong></td>
                <td colSpan={2} className="hint">lord of {yogi.nakshatra}</td>
              </tr>
              <tr>
                <th scope="row">Avayogi</th>
                <td><strong>{yogi.avayogi}</strong></td>
                <td colSpan={2} className="hint">
                  lord of {yogi.avayogi_nakshatra}, the 6th from the Yogi&apos;s
                </td>
              </tr>
            </tbody>
          </table>
          <p className="hint">
            The Duplicate Yogi is not computed: the published accounts disagree on whether it is
            the graha conjoining the sphuta, the lord of its sign or of its navamsa, and no
            identity settles it.
          </p>
        </section>
      )}

      <section className="card">
        <h3>Variants in force</h3>
        <p className="notice" role="note">
          None of these has been signed off by an astrologer. V-13-14 is the one most likely to
          explain a difference from other software: an upagraha is taken at the <em>start</em> of
          its part here, where some implementations take the end or the midpoint.
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
