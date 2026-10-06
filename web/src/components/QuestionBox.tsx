import { useMemo, useState } from "react";

import { suggest } from "../lib/questions";
import type { Question, TopicMeta } from "../types";

interface Props {
  questions: Question[];
  topics: TopicMeta[];
  /** Open the reading that answers the chosen question. */
  onChoose: (topicId: string, question: Question) => void;
}

/**
 * Ask about a matter, and be taken to the reading that answers it.
 *
 * Typing is for finding the question; the answer always comes from the
 * question chosen, never from the text typed. That is what keeps the reading
 * reproducible - the same chart and the same question always give the same
 * answer - and it is why nothing typed here leaves the device.
 *
 * A question with no match is told so. Guessing would produce an answer that
 * looked like the others and could not be traced to anything.
 */
export function QuestionBox({ questions, topics, onChoose }: Props) {
  const [typed, setTyped] = useState("");
  const [browsing, setBrowsing] = useState(false);

  const hits = useMemo(() => suggest(questions, typed), [questions, typed]);
  const title = (id: string) => topics.find((t) => t.id === id)?.title ?? id;
  const asked = typed.trim().length >= 2;

  if (questions.length === 0) return null;

  return (
    <section className="card question-box">
      <h3>Ask about a particular matter</h3>
      <p className="hint">
        Type what it is about — a house, a job, a visa, a wedding — and pick the question that
        fits. The answer comes from the question you pick, so the same chart and the same question
        always give the same reading.
      </p>

      <label>What is it about?
        <input
          type="text"
          autoComplete="off"
          placeholder="e.g. buying a house, changing jobs, green card"
          value={typed}
          onChange={(e) => { setTyped(e.target.value); setBrowsing(false); }}
        />
      </label>

      {asked && hits.length > 0 && (
        <ul className="question-hits">
          {hits.map(({ question }) => (
            <li key={question.id}>
              <button type="button" onClick={() => onChoose(question.topic, question)}>
                <strong>{question.question}</strong>
                <span className="hint">Answered by the {title(question.topic)} reading</span>
              </button>
            </li>
          ))}
        </ul>
      )}

      {asked && hits.length === 0 && (
        <div className="notice" role="status">
          <strong>No reviewed question matches that.</strong>
          <p>
            Rather than guess what you meant and answer something else, the app says so. Try a
            plainer word — marriage, child, house, loan, job, business, study, visa, surgery,
            travel, court — or{" "}
            <button type="button" className="link" onClick={() => setBrowsing(true)}>
              see every question it can answer
            </button>
            .
          </p>
        </div>
      )}

      {(browsing || (!asked && questions.length > 0)) && (
        <details className="question-all" open={browsing}>
          <summary>Every matter it can answer ({questions.length})</summary>
          <ul className="question-hits">
            {questions.map((question) => (
              <li key={question.id}>
                <button type="button" onClick={() => onChoose(question.topic, question)}>
                  <strong>{question.question}</strong>
                  <span className="hint">Answered by the {title(question.topic)} reading</span>
                </button>
              </li>
            ))}
          </ul>
        </details>
      )}

      <p className="hint">
        A reading describes the conditions of a period, never an outcome. It cannot tell you that a
        thing will happen, only what the tradition reads in the chart for that matter and when it
        comes forward.
      </p>
    </section>
  );
}
