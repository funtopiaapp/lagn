import { useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, PeriodsResponse, PeriodWindow, Sex } from "../types";

interface Props { birth: BirthInput; sex?: Sex; reviewToken: string }

function WindowCard({ w }: { w: PeriodWindow }) {
  const cls = ["period-window", w.sensitive ? "sensitive" : "", w.current ? "current" : ""].join(" ").trim();
  return (
    <li className={cls} data-window={`${w.maha_name}/${w.antar_name}`}>
      <div className="rule-head">
        <span className="dates">{w.start} → {w.end}</span>
        <strong>{w.maha_name} / {w.antar_name}</strong>
        {w.current && <span className="tag now">NOW</span>}
        {w.sensitive && <span className="tag sensitive">SENSITIVE</span>}
        <span className="polarity" title="Sum of the period rules' weights">{w.score > 0 ? `+${w.score}` : w.score}</span>
      </div>
      <p>{w.explanation[0]}</p>
      <details>
        <summary>What amplifies, what eases</summary>
        <ul className="plain explanation">{w.explanation.slice(1).map((line, i) => <li key={i}>{line}</li>)}</ul>
      </details>
    </li>
  );
}

/** Every antardasha in an age range, with its period rules and transits. */
export function PeriodsView({ birth, sex, reviewToken }: Props) {
  const [from, setFrom] = useState(18);
  const [to, setTo] = useState(70);
  const [onlySensitive, setOnlySensitive] = useState(false);
  const [data, setData] = useState<PeriodsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const valid = Number.isFinite(from) && Number.isFinite(to) && from >= 0 && to > from && to <= 120;
  const run = async () => {
    setBusy(true); setError(null);
    try {
      setData(await api.periods(birth, { sex, from_age: from, to_age: to }, reviewToken || undefined));
    } catch (e) {
      setData(null);
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(false); }
  };

  const withheld = data ? Object.values(data.withheld).reduce((a, b) => a + b, 0) : 0;
  const shown = data ? data.windows.filter((w) => !onlySensitive || w.sensitive || w.current) : [];
  const sensitiveCount = data ? data.windows.filter((w) => w.sensitive).length : 0;

  return (
    <section className="card">
      <div className="card-head">
        <h3>Sensitive periods</h3>
        {data && <span className={`chip ${sensitiveCount > 0 ? "warn" : "ok"}`}>{sensitiveCount} sensitive of {data.windows.length}</span>}
      </div>
      <p className="hint">
        Each dasha bhukti with what it brings into focus, the factors that amplify or ease it, the Saturn, Jupiter and
        Rahu/Ketu transits running through it. A sensitive period calls for care and
        patience; it is not a prediction of any particular event.
      </p>
      <div className="row">
        <label>From age<input type="number" inputMode="numeric" min={0} max={119} value={from} onChange={(e) => setFrom(Number(e.target.value))} /></label>
        <label>To age<input type="number" inputMode="numeric" min={1} max={120} value={to} onChange={(e) => setTo(Number(e.target.value))} /></label>
        <button type="button" className="primary" onClick={run} disabled={busy || !valid}>{busy ? "Working…" : "Show periods"}</button>
      </div>
      {!valid && <p className="warn">Ages must satisfy 0 ≤ from &lt; to ≤ 120.</p>}
      {error && <p className="error" role="alert">{error}</p>}

      {data && data.mode === "review" && <p className="notice review">Review mode: draft period rules are included.</p>}
      {data && data.mode === "production" && withheld > 0 && <p className="hint">{withheld} unreviewed period rules are not shown.</p>}
      {data && (
        <>
          <p className="summary">
            {data.windows.length} periods from the Moon sign {data.moon_sign}; {sensitiveCount} marked sensitive.
          </p>
          <label className="radio"><input type="checkbox" checked={onlySensitive} onChange={(e) => setOnlySensitive(e.target.checked)} /> Show only sensitive and current periods</label>
          <ul className="rules periods">{shown.map((w) => <WindowCard key={`${w.start}-${w.maha_name}-${w.antar_name}`} w={w} />)}</ul>
        </>
      )}
    </section>
  );
}
