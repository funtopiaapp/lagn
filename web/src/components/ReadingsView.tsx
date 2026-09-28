import { useEffect, useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, Sex, TopicMeta } from "../types";
import { TopicView } from "./TopicView";

interface Props { birth: BirthInput; sex?: Sex; reviewToken: string }

/** Every topic the server can answer, from its catalogue. */
export function ReadingsView({ birth, sex, reviewToken }: Props) {
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
      <TopicView key={topic.id} birth={birth} sex={sex} reviewToken={reviewToken} topic={topic} />
    </>
  );
}
