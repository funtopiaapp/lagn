import type { Question, TimingWindow, TopicMeta, TopicResponse } from "../types";

interface Props {
  question: Question;
  topic: TopicMeta;
  data: TopicResponse;
}

/**
 * The windows worth naming: recommended, still to come, soonest first.
 *
 * Someone asking when to do a thing cannot act on a stretch that ended twenty
 * years ago. Those are kept only as a fallback, and said to be past, because
 * a chart whose good windows have all gone by is itself the answer.
 */
function pick(windows: TimingWindow[]): { list: TimingWindow[]; past: boolean } {
  const good = windows.filter((w) => w.rank === "best" || w.rank === "good");
  const rank = (w: TimingWindow) => (w.rank === "best" ? 0 : 1);
  const soonest = (a: TimingWindow, b: TimingWindow) => a.start.localeCompare(b.start);

  // Running now first, then what is ahead, each soonest first.
  const ahead = [
    ...good.filter((w) => w.when === "now").sort(soonest),
    ...good.filter((w) => w.when === "ahead").sort(soonest),
  ];
  if (ahead.length > 0) return { list: ahead.slice(0, 4), past: false };
  return {
    list: [...good].sort((a, b) => rank(a) - rank(b) || soonest(b, a)).slice(0, 3),
    past: good.length > 0,
  };
}

/**
 * A direct answer to the question that was asked.
 *
 * The reading underneath says everything, at length. Someone who asked "when
 * should I buy a house" wants the dates first and the reasoning after, so this
 * says the standing of the matter in one line and then names the stretches.
 *
 * Nothing here is new astrology: it is the topic's own verdict and its own
 * ranked windows, said shortly. Which is why it can never disagree with the
 * reading below it.
 */
export function QuestionAnswer({ question, topic, data }: Props) {
  const rep = data.report;
  const { list: best, past } = pick(data.windows);
  const area = topic.title.toLowerCase();

  // The standing of the matter in this chart, in the plainest words available.
  const standing =
    rep.results.length === 0
      ? `No reviewed rule for ${area} applies to this chart yet, so there is nothing to answer from.`
      : rep.score > 0
        ? `The chart supports ${area} overall.`
        : rep.score < 0
          ? `The chart asks for care over ${area}.`
          : `The chart is evenly balanced on ${area}.`;

  return (
    <section className="card question-answer">
      <h3>{question.question}</h3>
      <p className="summary">
        <strong>{standing}</strong>{" "}
        {rep.results.length > 0 && (
          best.length > 0
            ? past
            ? `The stretches this chart recommends for it have already passed; the most recent ${best.length === 1 ? "one is" : `${best.length} are`} below.`
            : `${best.length === 1 ? "One stretch is" : `${best.length} stretches are`} recommended, soonest first.`
            : "None of the stretches read for this comes out recommended, so there is no window to point you at."
        )}
      </p>

      {topic.disclaimer && <p className="notice disclaimer" role="note">{topic.disclaimer}</p>}

      {best.length > 0 && (
        <>
          <h4>{past ? "The recommended stretches, all now past" : "The timings to consider"}</h4>
          <ol className="answer-windows">
            {best.map((w) => (
              <li key={`${w.start}-${w.maha}-${w.antar}`}>
                <strong>{w.start} → {w.end}</strong>
                <span className={`chip ${w.rank === "best" ? "ok" : ""}`}>{w.rank}</span>
                {w.when === "now" && <span className="tag now">running now</span>}
                {w.live && <span className="tag">{area} is live here</span>}
                {w.supports[0] && <p className="hint">{w.supports[0]}</p>}
              </li>
            ))}
          </ol>
          <p className="hint">
            These describe the conditions of a period, not an outcome. Nothing here says the thing
            will happen, or that it cannot happen outside these stretches — only that the tradition
            reads these as the better-supported ones for {area}.
          </p>
          {past && (
            <p className="notice" role="note">
              Nothing recommended for {area} remains ahead in the range this reading covers. That
              is worth knowing plainly rather than being shown an old date as though it were
              advice.
            </p>
          )}
        </>
      )}

      {best.length === 0 && rep.results.length > 0 && (
        <p className="notice" role="note">
          That is not a refusal. It means the stretches when {area} is live are also ones that ask
          for effort, so read <em>When this comes forward</em> below and weigh the reasons rather
          than waiting for a window that the chart does not offer.
        </p>
      )}

      <p className="hint">
        For a day to act on once you have chosen a stretch, see <strong>Day timings</strong>:
        Rahu kalam and the rest are the hour-by-hour layer, and this is the years-long one.
      </p>
    </section>
  );
}
