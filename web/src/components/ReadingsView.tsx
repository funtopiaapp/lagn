import { useEffect, useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, ChartResponse, Sex, TopicMeta } from "../types";
import { LagnaMargin } from "./LagnaMargin";
import { TopicView } from "./TopicView";

interface Props { birth: BirthInput; sex?: Sex; reviewToken: string; lagna: ChartResponse["lagna"] }

/** Every topic the server can answer, from its catalogue. */
export function ReadingsView({ birth, sex, reviewToken, lagna }: Props) {
  const [topics, setTopics] = useState<TopicMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string>("marriage");

  useEffect(() => {
    api.topics()
      .then((r) => setTopics(r.topics))
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  }, []);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!topics) return <p className="hint" role="status">Loading topics…</p>;
  const topic = topics.find((t) => t.id === selected) ?? topics[0];
  if (!topic) return <p className="notice">No reviewed topics are available yet.</p>;

  return (
    <>
      <nav className="topics" aria-label="Topics">
        {topics.map((t) => (
          <button key={t.id} type="button" aria-pressed={t.id === topic.id} onClick={() => setSelected(t.id)}>{t.title}</button>
        ))}
      </nav>
      {/* Above the verdicts, not only on the chart tab: every house below is
          counted from the lagna, so this is where the margin matters. */}
      <LagnaMargin hold={lagna.holds_for} rasi={lagna.rasi} />
      <TopicView key={topic.id} birth={birth} sex={sex} reviewToken={reviewToken} topic={topic} />
    </>
  );
}
