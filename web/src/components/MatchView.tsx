import { useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, MatchResponse } from "../types";
import { BirthForm } from "./BirthForm";

interface Props { birth: BirthInput; reviewToken: string }

const NAMES: Record<string, string> = {
  dina: "Dina", gana: "Gana", mahendra: "Mahendra", stree_deergha: "Stree Deergha", yoni: "Yoni",
  rasi: "Rasi", rasi_adhipati: "Rasi Adhipati", vasya: "Vasya", rajju: "Rajju", vedha: "Vedha",
};

export function MatchView({ birth, reviewToken }: Props) {
  const [thisIs, setThisIs] = useState<"bride" | "groom">("bride");
  const [data, setData] = useState<MatchResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const run = async (partner: BirthInput) => {
    setBusy(true); setError(null);
    try {
      const [bride, groom] = thisIs === "bride" ? [birth, partner] : [partner, birth];
      setData(await api.match(bride, groom, reviewToken || undefined));
    } catch (e) {
      setData(null);
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(false); }
  };

  return (
    <section>
      <div className="card">
        <h3>Porutham (marriage matching)</h3>
        <fieldset className="inline">
          <legend>The chart already entered is the</legend>
          <label className="radio"><input type="radio" checked={thisIs === "bride"} onChange={() => setThisIs("bride")} /> bride</label>
          <label className="radio"><input type="radio" checked={thisIs === "groom"} onChange={() => setThisIs("groom")} /> groom</label>
        </fieldset>
      </div>
      <BirthForm title={thisIs === "bride" ? "Groom's birth details" : "Bride's birth details"} submitLabel="Match" onSubmit={run} busy={busy} />
      {error && <p className="error" role="alert">{error}</p>}
      {data && (
        <div className="card">
          <p className="summary">Bride: {data.bride.nakshatra}, {data.bride.rasi}. Groom: {data.groom.nakshatra}, {data.groom.rasi}.</p>
          {data.results.length === 0 ? (
            <div className="notice" role="status">
              <strong>No reviewed poruthams are available yet.</strong>
              <p>Poruthams are shown only after review. {data.withheld} are not shown.</p>
            </div>
          ) : (
            <>
              {data.mode === "review" && <p className="notice review">Review mode: drafts are shown and marked.</p>}
              <div className="table-wrap"><table>
                <thead><tr><th scope="col">Porutham</th><th scope="col">Result</th><th scope="col">Detail</th></tr></thead>
                <tbody>{data.results.map(([r, status]) => (
                  <tr key={r.kind} data-porutham={r.kind} className={r.critical ? "critical" : undefined}>
                    <th scope="row">{NAMES[r.kind] ?? r.kind} {status === "draft" && <span className="tag draft">DRAFT</span>}</th>
                    <td>{r.verdict === "matching" ? "Matching" : r.verdict === "not_matching" ? (r.critical ? "Not matching (critical)" : "Not matching") : "Not evaluated"}</td>
                    <td>{r.detail}</td>
                  </tr>
                ))}</tbody>
              </table></div>
              {data.reviewers.length > 0 && <p className="provenance">Porutham method reviewed by: {data.reviewers.join("; ")}.</p>}
              <p className="summary">{data.matched} of {data.evaluated} evaluated poruthams match.{data.critical_failures.length > 0 && <> Critical: {data.critical_failures.map((k) => NAMES[k] ?? k).join(", ")}.</>}</p>
              <p className="hint">This is the porutham procedure only. The decision rests with the family's astrologer.</p>
            </>
          )}
        </div>
      )}
    </section>
  );
}
