import { useEffect, useState } from "react";
import { api, ApiError } from "../api";
import type { BirthInput, ChartResponse, Question, Sex, TopicMeta, TopicResponse } from "../types";
import { LagnaMargin } from "./LagnaMargin";
import { QuestionAnswer } from "./QuestionAnswer";
import { QuestionBox } from "./QuestionBox";
import { SharePdf } from "./SharePdf";
import { TopicView } from "./TopicView";

interface Props { birth: BirthInput; sex?: Sex; reviewToken: string; lagna: ChartResponse["lagna"] }

/** Every topic the server can answer, from its catalogue. */
export function ReadingsView({ birth, sex, reviewToken, lagna }: Props) {
  const [topics, setTopics] = useState<TopicMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string>("marriage");
  const [loaded, setLoaded] = useState<TopicResponse | null>(null);
  const [questions, setQuestions] = useState<Question[]>([]);
  // The question a reader picked, so the reading can say what it is answering.
  const [asked, setAsked] = useState<Question | null>(null);

  useEffect(() => {
    api.topics()
      .then((r) => { setTopics(r.topics); setQuestions(r.questions ?? []); })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  }, []);

  if (error) return <p className="error" role="alert">{error}</p>;
  if (!topics) return <p className="hint" role="status">Loading topics…</p>;
  const topic = topics.find((t) => t.id === selected) ?? topics[0];
  if (!topic) return <p className="notice">No reviewed topics are available yet.</p>;

  return (
    <>
      <QuestionBox
        questions={questions}
        topics={topics}
        // The reading already in hand is kept. Clearing it meant a question
        // about the topic already selected - marriage, which is the default -
        // never produced an answer: the topic view did not remount, so it
        // never refetched, so it never reported the reading back. The guard
        // below is what stops a previous topic's answer being shown.
        onChoose={(topicId, q) => { setSelected(topicId); setAsked(q); }}
      />
      {asked && loaded && loaded.report.topic === asked.topic && (
        <>
          <QuestionAnswer question={asked} topic={topic} data={loaded} />
          <p className="hint">
            The full {topic.title.toLowerCase()} reading follows, with every factor and its
            justification.{" "}
            <button type="button" className="link" onClick={() => setAsked(null)}>
              Clear the question
            </button>
          </p>
        </>
      )}
      {asked && !(loaded && loaded.report.topic === asked.topic) && (
        <p className="notice asked" role="status">
          Answering: <strong>{asked.question}</strong> — reading the chart…
        </p>
      )}
      <nav className="topics" aria-label="Topics">
        {topics.map((t) => (
          <button key={t.id} type="button" aria-pressed={t.id === topic.id} onClick={() => setSelected(t.id)}>{t.title}</button>
        ))}
      </nav>
      {/* Above the verdicts, not only on the chart tab: every house below is
          counted from the lagna, so this is where the margin matters. */}
      <LagnaMargin hold={lagna.holds_for} rasi={lagna.rasi} />
      <TopicView key={topic.id} birth={birth} sex={sex} reviewToken={reviewToken} topic={topic}
        onLoaded={setLoaded} />
      <SharePdf
        birth={birth}
        lagna={`${lagna.rasi} ${lagna.degrees}`}
        sex={sex}
        reviewToken={reviewToken}
        current={loaded && loaded.report.topic === topic.id ? { meta: topic, data: loaded } : undefined}
        all={topics}
      />
    </>
  );
}
