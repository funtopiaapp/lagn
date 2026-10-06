import { useCallback, useEffect, useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, RuleResult, Sex, TopicMeta, TopicResponse } from "../types";
import { PariharamList } from "./PariharamList";
import { TopicTiming } from "./TopicTiming";
import { WriteUpView } from "./WriteUpView";

const MARRIAGE: TopicMeta = { id: "marriage", title: "Marriage", summary: "", ages: [18, 45] };

interface Props {
  birth: BirthInput; sex?: Sex; reviewToken: string; topic?: TopicMeta;
  /** Called with the loaded reading, so a parent can share it as a PDF
   *  without fetching it again. */
  onLoaded?: (data: TopicResponse) => void;
}

function RuleCard({ r }: { r: RuleResult }) {
  return (
    <li className="rule" data-rule={r.id}>
      <div className="rule-head">
        <span className="polarity">{r.polarity > 0 ? `+${r.polarity}` : r.polarity}</span>
        <strong>{r.title}</strong>
        {r.status === "draft" && <span className="tag draft">DRAFT</span>}
      </div>
      <p>{r.text}</p>
      {r.impact && <p className="meaning"><span className="meaning-label">What this means for you:</span> {r.impact}</p>}
      <details>
        <summary>Why this applies</summary>
        <ul className="trace">
          {r.trace.map((t, i) => <li key={i}><code>{t.value}</code> {t.condition} <span className="facts">({t.facts})</span></li>)}
        </ul>
      </details>
    </li>
  );
}

export function TopicView({ birth, sex, reviewToken, topic = MARRIAGE, onLoaded }: Props) {
  const [data, setData] = useState<TopicResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // A topic is read as soon as it is opened; no extra click.
  const run = useCallback(async () => {
    setBusy(true); setError(null);
    try {
      setData(await api.topic(topic.id, birth, { sex }, reviewToken || undefined));
    } catch (e) {
      setData(null);
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(false); }
  }, [topic.id, birth, sex, reviewToken]);
  useEffect(() => { void run(); }, [run]);

  // A reading belongs to the topic it was fetched for.
  const rep = data?.report.topic === topic.id ? data.report : undefined;
  // The parent needs the loaded reading for the PDF, so it does not have to
  // fetch it a second time.
  useEffect(() => { if (data && rep) onLoaded?.(data); }, [data, rep, onLoaded]);
  const [fromAge, toAge] = data?.meta?.ages ?? topic.ages;
  const byId = (id: string) => rep!.results.find((r) => r.id === id)!;
  const withheld = rep ? Object.values(rep.withheld).reduce((a, b) => a + b, 0) : 0;

  return (
    <section className="card">
      <div className="card-head">
        <h3>{topic.title}</h3>
        {rep && rep.results.length > 0 && (() => {
          const kind = rep.score > 0 ? "ok" : rep.score < 0 ? "warn" : "";
          const label = rep.score > 0 ? "Supportive" : rep.score < 0 ? "Calls for care" : "Evenly balanced";
          return <span className={`chip ${kind}`}>{label}{rep.score !== 0 && ` · ${rep.score > 0 ? "+" : ""}${rep.score}`}</span>;
        })()}
      </div>
      {topic.summary && <p className="hint">{topic.summary}</p>}
      {topic.disclaimer && <p className="notice disclaimer" role="note">{topic.disclaimer}</p>}
      {busy && !data && <p className="hint" role="status">Reading the chart…</p>}
      {error && <><p className="error" role="alert">{error}</p><button type="button" onClick={() => void run()}>Try again</button></>}

      {rep && rep.mode === "production" && rep.results.length === 0 && (
        <div className="notice" role="status">
          <strong>No reviewed interpretations are available for this topic yet.</strong>
          <p>Interpretive rules are shown only after review. {withheld} rules are not shown.</p>
        </div>
      )}

      {rep && data?.writeup && <WriteUpView w={data.writeup} omit={topic.disclaimer} />}
      {rep && data!.windows.length > 0 && (
        <TopicTiming windows={data!.windows} topic={topic.title} ages={[fromAge, toAge]} />
      )}
      {rep && data!.pariharams && data!.pariharams.length > 0 && <><h4>Pariharams</h4><PariharamList items={data!.pariharams} /></>}

      {rep && rep.results.length > 0 && (
        <details className="rule-details" open={rep.mode === "review"}>
          <summary>Details: every rule checked, with its full trace</summary>
          {rep.mode === "review" && <p className="notice review">Review mode: drafts are shown and marked. They are not approved for users.</p>}
          <p className="provenance">
            Rules reviewed by: {[...new Set(rep.results.map((r) => r.reviewer ?? "not recorded"))].join("; ")}.
          </p>
          <p className="summary">Overall: <strong>{rep.label}</strong> (score {rep.score > 0 ? `+${rep.score}` : rep.score}). Supporting and afflicting factors are listed separately; the engine never collapses a mixed picture into one verdict.</p>
          {rep.supporting.length > 0 && <><h4>Supporting</h4><ul className="rules">{rep.supporting.map((id) => <RuleCard key={id} r={byId(id)} />)}</ul></>}
          {rep.afflicting.length > 0 && <><h4>Afflicting</h4><ul className="rules">{rep.afflicting.map((id) => <RuleCard key={id} r={byId(id)} />)}</ul></>}
          {(() => {
            // Effective, unscored rules - same selection as the CLI: polarity 0,
            // excluding timing-only rules, which appear as periods below.
            const timed = new Set(data!.windows.map((w) => w.rule));
            const noted = rep.results.filter((r) => r.effective && r.polarity === 0 && !timed.has(r.id));
            return noted.length > 0 && <><h4>Noted (not scored)</h4><ul className="rules">{noted.map((r) => <RuleCard key={r.id} r={r} />)}</ul></>;
          })()}
          {rep.cancelled.length > 0 && (
            <><h4>Cancelled</h4><ul className="plain">{rep.cancelled.map(([id, by]) => <li key={id}>{byId(id).title}: cancelled by “{byId(by).title}”</li>)}</ul></>
          )}
          {rep.unknown.length > 0 && (
            <><h4>Could not evaluate</h4><ul className="plain">{rep.unknown.map((id) => <li key={id}>{byId(id).title}</li>)}</ul></>
          )}
          {data!.windows.length > 0 && (
            <><h4>Periods to examine (ages {fromAge} to {toAge})</h4>
              <div className="table-wrap"><table>
                <thead><tr><th scope="col">From</th><th scope="col">To</th><th scope="col">Dasha</th><th scope="col">Via</th><th scope="col">For this area</th></tr></thead>
                <tbody>{data!.windows.map((w, i) => <tr key={i}><td>{w.start}</td><td>{w.end}</td><td>{w.maha} / {w.antar}</td><td>{w.matched.join(", ")}</td><td>{w.verdict}</td></tr>)}</tbody>
              </table></div></>
          )}
        </details>
      )}
    </section>
  );
}
