import { useEffect, useId, useRef, useState } from "react";
import { api, ApiError } from "../api";
import { readDate, readTime } from "../lib/readback";
import { addSaved, loadSaved } from "../lib/savedBirths";
import type { BirthInput, OffsetSuggestion, Place, Sex } from "../types";

interface Props {
  title: string;
  submitLabel: string;
  onSubmit: (birth: BirthInput, sex?: Sex) => void;
  busy?: boolean;
  /** Ask the person's sex here, once, for the rules that depend on it. */
  askSex?: boolean;
  /** Fill the form from a saved profile. Its offset was already seen and
   *  confirmed when the profile was first entered, so it is reused rather
   *  than asked again - but it is shown, because an offset applied without
   *  the person seeing it is how a chart goes silently wrong. */
  prefill?: { id: string; birth: BirthInput; sex?: Sex; label: string } | null;
}

/** Choice of UTC offset: one of the suggested candidates, or a typed value. */
type OffsetChoice = { kind: "candidate"; index: number } | { kind: "manual" } | null;

/**
 * Collects date, time and place, then asks the user to confirm the UTC offset.
 *
 * The offset is the single easiest place for a chart to go silently wrong, so
 * it is never applied without the user seeing it. A historical, ambiguous or
 * local-mean-time suggestion is never preselected: the user must choose.
 */
export function BirthForm({ title, submitLabel, onSubmit, busy, askSex, prefill }: Props) {
  const id = useId();
  const [date, setDate] = useState("");
  const [time, setTime] = useState("");
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Place[]>([]);
  const [place, setPlace] = useState<Place | null>(null);
  const [manual, setManual] = useState(false);
  const [lat, setLat] = useState("");
  const [lon, setLon] = useState("");
  const [tz, setTz] = useState("");
  const [suggestion, setSuggestion] = useState<OffsetSuggestion | null>(null);
  const [choice, setChoice] = useState<OffsetChoice>(null);
  const [manualOffset, setManualOffset] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [sex, setSex] = useState<"" | Sex>("");
  // Saving is opt-in: birth details are the most personal thing here, so
  // nothing is kept unless it is asked for.
  const [save, setSave] = useState(false);
  const [saveName, setSaveName] = useState("");
  const [saveFailed, setSaveFailed] = useState(false);
  // The profile the fields currently come from. A ref as well as state: both
  // effects below run in the same pass, so the offset effect would otherwise
  // read the previous render's value and reset the offset this one just
  // filled in. The ref updates immediately; the state is for rendering.
  const fromRef = useRef<string | null>(null);
  const [fromProfile, setFromProfileState] = useState<string | null>(null);
  const setFromProfile = (id: string | null) => {
    fromRef.current = id;
    setFromProfileState(id);
  };
  const searchSeq = useRef(0);

  // Fill from a saved profile when one is handed in.
  useEffect(() => {
    if (!prefill) return;
    const b = prefill.birth;
    setDate(b.date);
    setTime(b.time.slice(0, 8));
    // Coordinates rather than a place search: they are exact, and re-running
    // the search would ask the offset question again for an answer already
    // given.
    setManual(true);
    setPlace(null);
    setQuery("");
    setLat(String(b.latitude));
    setLon(String(b.longitude));
    setTz("");
    setManualOffset(String(b.utc_offset_hours));
    setChoice({ kind: "manual" });
    setSuggestion(null);
    setSex(prefill.sex ?? "");
    setFromProfile(prefill.id);
    setError(null);
  }, [prefill]);

  // Place search, debounced; stale responses are discarded.
  useEffect(() => {
    if (manual || place || query.trim().length < 2) { setResults([]); return; }
    const seq = ++searchSeq.current;
    const t = setTimeout(() => {
      api.places(query.trim())
        .then((r) => { if (seq === searchSeq.current) setResults(r); })
        .catch(() => { if (seq === searchSeq.current) setResults([]); });
    }, 200);
    return () => clearTimeout(t);
  }, [query, manual, place]);

  const latNum = manual ? Number(lat) : place?.latitude;
  const lonNum = manual ? Number(lon) : place?.longitude;
  const zone = manual ? tz.trim() : place?.timezone ?? "";
  // Number("") is 0, so an empty manual field must be rejected explicitly.
  const coordsOk = latNum !== undefined && lonNum !== undefined &&
    Number.isFinite(latNum) && Number.isFinite(lonNum) && (!manual || (lat.trim() !== "" && lon.trim() !== ""));

  // Offset suggestion whenever date, time and place are all known.
  useEffect(() => {
    // A prefilled profile already carries an offset its owner confirmed, so
    // this must not reset it. Editing any field clears fromProfile and the
    // ordinary flow resumes.
    if (fromRef.current) return;
    setSuggestion(null);
    setChoice(null);
    setError(null);
    if (!date || !time || !coordsOk || !zone) return;
    let live = true;
    api.offset(zone, date, time, lonNum)
      .then((s) => {
        if (!live) return;
        setSuggestion(s);
        // Preselect only the one case with nothing to decide.
        const safe = s.kind === "unique" && s.confidence === "reliable" && s.candidates.length === 1;
        setChoice(safe ? { kind: "candidate", index: 0 } : null);
      })
      .catch((e) => { if (live) setError(e instanceof ApiError ? e.message : String(e)); });
    return () => { live = false; };
  }, [date, time, zone, coordsOk, lonNum, fromProfile]);


  const offsetValue = (): number | null => {
    if (!choice) return null;
    if (choice.kind === "manual") {
      const v = Number(manualOffset);
      return manualOffset.trim() !== "" && Number.isFinite(v) && v >= -14 && v <= 14 ? v : null;
    }
    return suggestion?.candidates[choice.index]?.offset_hours ?? null;
  };

  const offset = offsetValue();
  const ready = !!date && !!time && coordsOk && offset !== null && !busy;

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!ready || offset === null || latNum === undefined || lonNum === undefined) return;
    const birth: BirthInput = {
      date,
      time: time.length === 5 ? `${time}:00` : time,
      latitude: latNum,
      longitude: lonNum,
      utc_offset_hours: offset,
      place: manual ? "" : place ? `${place.name}, ${place.admin1}` : "",
    };
    // The form saves, not the caller. This form is used for the main chart,
    // for a porutham partner and for a family member, and when only the first
    // of those did the saving the checkbox silently did nothing in the other
    // two - a control that lied.
    if (save) {
      const out = addSaved(loadSaved(), birth, { name: saveName, sex: sex || undefined });
      setSaveFailed(!out.stored);
    }
    onSubmit(birth, sex || undefined);
  };

  const pick = (p: Place) => { setPlace(p); setQuery(`${p.name}, ${p.admin1}, ${p.country}`); setResults([]); };

  return (
    <form className="card birth-form" onSubmit={submit} aria-labelledby={`${id}-title`}>
      <h2 id={`${id}-title`}>{title}</h2>

      {fromProfile && prefill && (
        <p className="notice asked" role="status">
          Filled from your saved profile <strong>{prefill.label}</strong>, including the UTC offset
          you confirmed for it ({prefill.birth.utc_offset_hours >= 0 ? "+" : ""}
          {prefill.birth.utc_offset_hours}). Change any field and the offset is asked again.
        </p>
      )}

      <div className="row">
        <label>Date of birth
          <input type="date" required min="1200-01-01" max="3000-12-31" value={date} onChange={(e) => { setDate(e.target.value); setFromProfile(null); }} />
        </label>
        <label>Time of birth
          <input type="time" required step={1} value={time} onChange={(e) => { setTime(e.target.value); setFromProfile(null); }} />
        </label>
      </div>
      {(readDate(date) || readTime(time)) && (
        <p className="readback" aria-live="polite">
          Reading this as: <strong>{[readDate(date), readTime(time)].filter(Boolean).join(", ")}</strong>
        </p>
      )}
      <p className="hint">As written on the birth record, local clock time. Dates before 15 October 1582 are read in the Julian calendar.</p>

      {!manual ? (
        <div className="place">
          <label>Place of birth
            <input type="text" autoComplete="off" placeholder="Town or city, e.g. Madurai" value={query}
              aria-expanded={results.length > 0} aria-controls={`${id}-places`}
              onChange={(e) => { setQuery(e.target.value); setPlace(null); setFromProfile(null); }} />
          </label>
          {results.length > 0 && (
            <ul id={`${id}-places`} className="place-results" role="listbox">
              {results.map((p) => (
                <li key={p.id} role="option" aria-selected={false}>
                  <button type="button" onClick={() => pick(p)}>
                    <strong>{p.name}</strong>, {p.admin1}, {p.country}
                    {p.matched_alias && <span className="alias"> (matched “{p.matched_alias}”)</span>}
                  </button>
                </li>
              ))}
            </ul>
          )}
          {place && <p className="hint">{place.latitude.toFixed(4)}°, {place.longitude.toFixed(4)}° · {place.timezone}</p>}
          <button type="button" className="link" onClick={() => { setManual(true); setPlace(null); }}>Enter coordinates instead</button>
        </div>
      ) : (
        <div className="place">
          <div className="row">
            <label>Latitude (north +)<input inputMode="decimal" value={lat} onChange={(e) => setLat(e.target.value)} placeholder="13.0827" /></label>
            <label>Longitude (east +)<input inputMode="decimal" value={lon} onChange={(e) => setLon(e.target.value)} placeholder="80.2707" /></label>
          </div>
          <label>Time zone (IANA name, optional)<input value={tz} onChange={(e) => setTz(e.target.value)} placeholder="Asia/Kolkata" /></label>
          <button type="button" className="link" onClick={() => setManual(false)}>Search for a place instead</button>
        </div>
      )}

      <fieldset className="offset" disabled={!date || !time || !coordsOk}>
        <legend>UTC offset at the birth</legend>
        {suggestion && (
          <>
            <p className={`badge ${suggestion.confidence}`}>
              {suggestion.confidence === "reliable" ? "From the time zone database" : "Historical: please confirm against the birth record"}
            </p>
            {suggestion.kind === "nonexistent" && <p className="warn">This local time did not exist at this place (clocks were moved forward). Please check the recorded time.</p>}
            {suggestion.candidates.map((c, i) => (
              <label key={i} className="radio">
                <input type="radio" name={`${id}-offset`} checked={choice?.kind === "candidate" && choice.index === i}
                  onChange={() => setChoice({ kind: "candidate", index: i })} />
                <span><strong>UTC{c.offset_text}</strong> · {c.label}</span>
              </label>
            ))}
            {suggestion.notes.map((n, i) => <p key={i} className="note">{n}</p>)}
          </>
        )}
        <label className="radio">
          <input type="radio" name={`${id}-offset`} checked={choice?.kind === "manual"} onChange={() => setChoice({ kind: "manual" })} />
          <span>Another offset (hours, e.g. 5.5)</span>
        </label>
        {choice?.kind === "manual" && (
          <input aria-label="UTC offset in hours" inputMode="decimal" value={manualOffset} onChange={(e) => setManualOffset(e.target.value)} placeholder="5.5" />
        )}
      </fieldset>

      {askSex && (
        <label className="narrow">Sex (optional)
          <select value={sex} onChange={(e) => setSex(e.target.value as typeof sex)}>
            <option value="">Not stated</option><option value="female">Female</option><option value="male">Male</option>
          </select>
          <span className="hint">A few classical rules depend on it. If not stated, those rules are shown as "could not be judged", never guessed.</span>
        </label>
      )}
      <fieldset className="inline save-here">
        <label className="radio">
          <input type="checkbox" checked={save} onChange={(e) => setSave(e.target.checked)} />
          <span>Save this profile on this device</span>
        </label>
        {save && (
          <label className="narrow">Name (optional)
            <input type="text" maxLength={60} value={saveName} placeholder="e.g. Appa"
              onChange={(e) => setSaveName(e.target.value)} />
            <span className="hint">
              Only so you can recognise this profile in the list later. Leave it blank and
              the date and place are shown instead. Nothing is sent anywhere.
            </span>
          </label>
        )}
      </fieldset>
      {saveFailed && (
        <p className="error" role="alert">
          This device would not let the profile be saved, so it has not been kept. Private
          browsing and blocked site data both do this.
        </p>
      )}

      {error && <p className="error" role="alert">{error}</p>}
      <button type="submit" className="primary" disabled={!ready}>{busy ? "Working…" : submitLabel}</button>
      {!ready && date && time && coordsOk && offset === null && <p className="hint">Choose the UTC offset to continue.</p>}
    </form>
  );
}
