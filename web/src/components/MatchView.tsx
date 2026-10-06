import { useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, MatchResponse } from "../types";
import { BirthForm } from "./BirthForm";
import { ProfilePicker } from "./ProfilePicker";
import { describe as describeSaved, loadSaved, type SavedBirth } from "../lib/savedBirths";

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
  // Read once on mount: the list only changes through this page's own form,
  // which re-renders anyway.
  // Re-read on mount and whenever the picker is opened: a profile may have
  // been saved from another page since this one mounted.
  const [profiles, setProfiles] = useState<SavedBirth[]>(loadSaved);
  const [picked, setPicked] = useState<SavedBirth | null>(null);

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
      {/* The partner's chart is usually one you already have. Picking a saved
          profile fills the form below rather than matching straight away, so
          you can see which chart is about to be used. */}
      <ProfilePicker
        profiles={profiles}
        onRefresh={() => setProfiles(loadSaved())}
        onPick={setPicked}
        label="Use a saved profile for the partner"
      />
      <BirthForm
        title={thisIs === "bride" ? "Groom's birth details" : "Bride's birth details"}
        submitLabel="Match"
        onSubmit={run}
        busy={busy}
        prefill={picked ? { id: picked.id, birth: picked.birth, sex: picked.sex, label: picked.name || describeSaved(picked) } : null}
      />
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
              {data.reading.length > 0 && (
                <section className="match-reading">
                  <h4>How married life is read from this</h4>
                  {data.reading.map((p, k) => (
                    <p key={k} className={k === 0 ? "summary" : undefined}>{p}</p>
                  ))}
                </section>
              )}
              {data.areas.length > 0 && (
                <section className="match-areas">
                  <h4>By area</h4>
                  <ul className="areas">
                    {data.areas.map((a) => (
                      <li key={a.touches} className={a.not_matching.length > 0 ? (a.critical ? "area critical" : "area warn") : "area ok"}>
                        <strong>{a.touches}</strong>
                        {a.matching.length > 0 && <span className="chip ok">{a.matching.join(", ")}</span>}
                        {a.not_matching.length > 0 && <span className={`chip ${a.critical ? "" : "warn"}`}>{a.not_matching.join(", ")} not matching</span>}
                      </li>
                    ))}
                  </ul>
                </section>
              )}
              {data.papa && (
                <section className="match-papa">
                  <h4>Papasamyam</h4>
                  <p className="hint">
                    The papa (malefic) placements each chart carries, counted from the lagna, the
                    Moon and Venus. The check asks that the groom's count not be the lower of the
                    two. Weighted variants of this count exist and differ between sources, so a
                    close call belongs with an astrologer.
                  </p>
                  <div className="table-wrap"><table>
                    <thead><tr><th scope="col" /><th scope="col">From the lagna</th><th scope="col">From the Moon</th><th scope="col">From Venus</th><th scope="col">Total</th></tr></thead>
                    <tbody>
                      {([["Bride", data.papa.bride], ["Groom", data.papa.groom]] as const).map(([who, side]) => (
                        <tr key={who}>
                          <th scope="row">{who}</th>
                          {side.from.map((f) => (
                            <td key={f.from} className="num">
                              {f.count}
                              {f.placements.length > 0 && (
                                <span className="ta"> ({f.placements.map(([g, h]) => `${g} in ${h}`).join(", ")})</span>
                              )}
                            </td>
                          ))}
                          <td className="num"><strong>{side.total}</strong></td>
                        </tr>
                      ))}
                    </tbody>
                  </table></div>
                  <p className={data.papa.balanced ? "summary" : "warn"}>
                    {data.papa.balanced
                      ? "The groom's count is not the lower of the two, which is what the check asks for."
                      : `The groom's count is lower by ${-data.papa.difference}, which the check asks against.`}
                  </p>
                </section>
              )}

              {data.chevvai && (
                <section className="match-chevvai">
                  <h4>Chevvai dosha</h4>
                  <ul className="plain">
                    <li><strong>Bride:</strong> {data.chevvai.bride ? `present — ${data.chevvai.bride_counts.join("; ")}` : "not present"}</li>
                    <li><strong>Groom:</strong> {data.chevvai.groom ? `present — ${data.chevvai.groom_counts.join("; ")}` : "not present"}</li>
                  </ul>
                  {data.chevvai.mutual && (
                    <p className="summary">
                      Both charts carry it, which classical practice reads as the two cancelling
                      each other.
                    </p>
                  )}
                </section>
              )}

              {data.dasa_sandhi && data.dasa_sandhi.length > 0 && (
                <section className="match-sandhi">
                  <h4>Dasa sandhi</h4>
                  <p className="hint">
                    Points where both charts change major period within eighteen months of each
                    other. The texts read two turnings at once as a strain on a new household, so
                    these are worth weighing against when a marriage is actually planned.
                  </p>
                  <ul className="plain">
                    {data.dasa_sandhi.map((d) => (
                      <li key={`${d.bride_jd}-${d.groom_jd}`}>
                        Bride {d.bride_from} → {d.bride_to}, groom {d.groom_from} → {d.groom_to};{" "}
                        <strong>{Math.round(d.months_apart)} months apart</strong>
                      </li>
                    ))}
                  </ul>
                </section>
              )}

              {data.explained.length > 0 && (
                <details className="porutham-detail">
                  <summary>What each porutham measures, one by one</summary>
                  <ul className="rules">
                    {data.explained.filter((x) => x.means).map((x) => (
                      <li key={x.kind} className="rule" data-porutham={x.kind}>
                        <div className="rule-head">
                          <strong>{x.name}</strong>
                          <span className={`chip ${x.verdict === "matching" ? "ok" : "warn"}`}>
                            {x.verdict === "matching" ? "matching" : x.critical ? "not matching (serious)" : "not matching"}
                          </span>
                          <span className="tag">{x.touches}</span>
                        </div>
                        <p>It tests {x.measures}.</p>
                        <p className="meaning">
                          <span className="meaning-label">
                            {x.verdict === "matching" ? "Matching here means:" : "Not matching here means:"}
                          </span>{" "}
                          {x.means}.
                        </p>
                        <p className="because">From the charts: {x.detail}</p>
                      </li>
                    ))}
                  </ul>
                </details>
              )}
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
