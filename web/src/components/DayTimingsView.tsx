import { useCallback, useEffect, useState } from "react";

import { api, ApiError } from "../api";
import type { BirthInput, DaySegment, DayView } from "../types";

/** Avoided or auspicious, with what each one is for a reader who has only
 *  heard the name. */
const PARTS: { key: keyof Pick<DayView, "rahu_kalam" | "yamagandam" | "kuligai" | "abhijit">;
  name: string; good: boolean; what: string }[] = [
  { key: "rahu_kalam", name: "Rahu kalam", good: false,
    what: "The eighth of the day given to Rahu. Tradition avoids starting anything new in it - a journey, a purchase, a signature." },
  { key: "yamagandam", name: "Yamagandam", good: false,
    what: "Another eighth, avoided in the same way, though usually treated as less weighty than Rahu kalam." },
  { key: "kuligai", name: "Kuligai", good: false,
    what: "Also called Gulika. Avoided for new beginnings; some households treat it as the mildest of the three." },
  { key: "abhijit", name: "Abhijit muhurta", good: true,
    what: "The middle of the day, and the one segment here that is auspicious. Traditionally a good short window to begin something." },
];

function Segment({ seg, name, good, what }: { seg: DaySegment; name: string; good: boolean; what: string }) {
  return (
    <li className={good ? "day-part good" : "day-part avoid"}>
      <div className="day-part-head">
        <strong>{name}</strong>
        <span className="day-part-span">{seg.start} – {seg.end}</span>
        <span className={`chip ${good ? "ok" : "warn"}`}>{good ? "auspicious" : "avoided"}</span>
      </div>
      <p className="hint">{what}</p>
    </li>
  );
}

/**
 * A day's own timings: the panchanga, and the parts of the daylight tradition
 * marks out.
 *
 * This is the other time scale in the app. A dasha stretch is years long;
 * these are the hour-and-a-half segments a household checks before stepping
 * out, and they are measured from sunrise rather than midnight.
 *
 * Weeks are walked by sending an offset in days and letting the engine do the
 * arithmetic, because the browser does none (DESIGN.md section 1, rule 2).
 */
export function DayTimingsView({ birth }: { birth: BirthInput }) {
  const [offset, setOffset] = useState(0);
  const [days, setDays] = useState<DayView[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    setBusy(true); setError(null);
    try {
      const r = await api.days({
        latitude: birth.latitude,
        longitude: birth.longitude,
        utc_offset_hours: birth.utc_offset_hours,
        days: 7,
        offset_days: offset,
      });
      setDays(r.days);
      // Opening the first day of the week shown keeps the panel from being
      // empty, and on the current week that day is today.
      setOpen(r.days[0]?.date ?? null);
    } catch (e) {
      setDays(null);
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(false); }
  }, [birth.latitude, birth.longitude, birth.utc_offset_hours, offset]);

  useEffect(() => { void load(); }, [load]);

  const shown = days?.find((d) => d.date === open) ?? days?.[0];

  return (
    <section className="card day-timings">
      <div className="card-head">
        <h3>Day timings</h3>
        {offset !== 0 && <button type="button" className="link" onClick={() => setOffset(0)}>Back to this week</button>}
      </div>
      <p className="hint">
        Rahu kalam and the rest divide sunrise to sunset into eight parts, so they move with the
        day's length and with the place. These are computed for{" "}
        <strong>{birth.place || `${birth.latitude.toFixed(2)}, ${birth.longitude.toFixed(2)}`}</strong>{" "}
        at UTC{birth.utc_offset_hours >= 0 ? "+" : ""}{birth.utc_offset_hours}.
      </p>

      <div className="week-nav">
        <button type="button" onClick={() => setOffset(offset - 7)} disabled={busy}>← Previous week</button>
        <span className="spacer" />
        <button type="button" onClick={() => setOffset(offset + 7)} disabled={busy}>Next week →</button>
      </div>

      {error && <><p className="error" role="alert">{error}</p><button type="button" onClick={() => void load()}>Try again</button></>}
      {busy && !days && <p className="hint" role="status">Computing sunrise…</p>}

      {days && days.length === 0 && (
        <p className="notice" role="note">
          The Sun does not rise on these dates at this latitude, so a day cannot be divided from
          sunrise to sunset. Inside the polar circles these timings have no meaning.
        </p>
      )}

      {days && days.length > 0 && (
        <>
          {/* Scrollable strip: a week at a time, any week. */}
          <div className="week-strip" role="tablist" aria-label="Days of the week shown">
            {days.map((d) => (
              <button
                key={d.date}
                type="button"
                role="tab"
                aria-selected={shown?.date === d.date}
                className={shown?.date === d.date ? "day-chip current" : "day-chip"}
                onClick={() => setOpen(d.date)}
              >
                <span className="day-chip-vara">{d.vara.slice(0, 3)}</span>
                <span className="day-chip-date">{d.date.slice(5)}</span>
                <span className="day-chip-rahu">{d.rahu_kalam.start}</span>
              </button>
            ))}
          </div>

          {shown && (
            <div className="day-detail">
              <h4>{shown.vara} ({shown.vara_tamil}), {shown.date}</h4>
              <p className="summary">
                Sunrise <strong>{shown.sunrise}</strong> · sunset <strong>{shown.sunset}</strong>
              </p>
              <dl className="panchanga">
                <div><dt>Tithi</dt><dd>{shown.tithi_name} ({shown.tithi}) · {shown.paksha}</dd></div>
                <div><dt>Nakshatra</dt><dd>{shown.nakshatra} ({shown.nakshatra_tamil}), pada {shown.pada}</dd></div>
                <div><dt>Yoga</dt><dd>{shown.yoga}</dd></div>
                <div><dt>Karana</dt><dd>{shown.karana}</dd></div>
              </dl>

              <ul className="day-parts">
                {PARTS.map((p) => (
                  <Segment key={p.key} seg={shown[p.key]} name={p.name} good={p.good} what={p.what} />
                ))}
              </ul>

              <p className="hint">
                These are traditional timings, not predictions. Nothing here says an hour will
                go well or badly; it says what the tradition does with it.
              </p>
            </div>
          )}
        </>
      )}
    </section>
  );
}
