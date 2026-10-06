import type { Question, TimingWindow, TopicMeta, TopicResponse } from "../types";

interface Props {
  question: Question;
  topic: TopicMeta;
  data: TopicResponse;
}

/** How the windows were chosen, which decides how plainly they are offered. */
type Basis = "recommended" | "best-available" | "all-past" | "none";

interface Choice {
  list: TimingWindow[];
  basis: Basis;
  /** Supported stretches that fall before the soonest clean one. A clean
   *  window twenty years out is a poor answer on its own when something
   *  positive sits in between. */
  sooner: TimingWindow[];
}

/**
 * The windows worth naming, and on what basis.
 *
 * Three tiers, because a chart does not always offer a clean answer and
 * pretending otherwise produces a useless one:
 *
 *  - stretches still ahead that come out recommended outright;
 *  - failing that, the best-supported stretches still ahead, offered as such.
 *    A seventeen-year mahadasha whose lord rules the 3rd and 6th puts a
 *    caution on every sub-period inside it, so nothing in it can ever be
 *    "recommended" - and skipping past it to the next clean window twenty
 *    years out is not an answer to "when should I";
 *  - failing that, what has already passed, said to have passed.
 */
function choose(windows: TimingWindow[]): Choice {
  const soonest = (a: TimingWindow, b: TimingWindow) => a.start.localeCompare(b.start);
  const ahead = windows.filter((w) => w.when === "now" || w.when === "ahead");
  const nowFirst = (a: TimingWindow, b: TimingWindow) =>
    (a.when === "now" ? 0 : 1) - (b.when === "now" ? 0 : 1) || soonest(a, b);

  const positive = ahead.filter((w) => w.score > 0);
  const recommended = ahead.filter((w) => w.rank === "best" || w.rank === "good");

  if (recommended.length > 0) {
    const list = [...recommended].sort(nowFirst).slice(0, 4);
    // Anything supported that falls before the soonest clean one. Comparing
    // the dates as strings, not as dates: the browser does no calendar
    // arithmetic, and ISO dates sort correctly as text.
    const soonestClean = [...recommended].sort(soonest)[0]?.start ?? "";
    const earlier = positive.filter((w) => w.start < soonestClean && !recommended.includes(w));
    const best = Math.max(...earlier.map((w) => w.score), 0);
    return {
      list,
      basis: "recommended",
      sooner: earlier.filter((w) => w.score === best).sort(soonest).slice(0, 2),
    };
  }

  // Nothing clean ahead. The best-supported ones still count for something,
  // and their cautions are shown alongside so the offer is not oversold.
  if (positive.length > 0) {
    const best = Math.max(...positive.map((w) => w.score));
    return {
      list: positive.filter((w) => w.score === best).sort(nowFirst).slice(0, 3),
      basis: "best-available",
      sooner: [],
    };
  }

  const past = windows.filter((w) => w.rank === "best" || w.rank === "good");
  if (past.length > 0) {
    // Most recent first: the nearest thing to useful about a window gone by.
    return { list: [...past].sort((a, b) => soonest(b, a)).slice(0, 3), basis: "all-past", sooner: [] };
  }
  return { list: [], basis: "none", sooner: [] };
}

/** "December 2026" from "2026-12-21", without any date arithmetic. */
const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];
function monthYear(iso: string): string {
  const [y, m] = iso.split("-");
  const name = MONTHS[Number(m) - 1];
  return name && y ? `${name} ${y}` : iso;
}

/**
 * A direct answer to the question that was asked, in the words someone would
 * use saying it.
 *
 * The full reading follows underneath and says everything. Someone who asked
 * "when should I buy a house" wants to be answered first: how the chart
 * stands, then the dates, then the reasoning. Nothing here is new astrology -
 * it is the topic's own verdict and its own ranked windows, said shortly -
 * which is why it can never disagree with the reading below it.
 */
export function QuestionAnswer({ question, topic, data }: Props) {
  const rep = data.report;
  const { list, basis, sooner } = choose(data.windows);
  const area = topic.title.toLowerCase();
  const [fromAge, toAge] = data.meta?.ages ?? topic.ages;

  // When the reading's own age range has run out, that is the reason there is
  // nothing ahead - not the chart. Saying "all past" without saying why would
  // put it on the chart.
  const rangeEnded =
    data.windows.length > 0 && data.windows.every((w) => w.when === "past");
  const lastEnd = data.windows[data.windows.length - 1]?.end;

  if (rep.results.length === 0) {
    return (
      <section className="card question-answer">
        <h3>{question.question}</h3>
        <p className="summary">
          There is no answer to give yet. No reviewed rule for {area} applies to this chart, so
          anything said here would be invented rather than derived.
        </p>
      </section>
    );
  }

  const standing =
    rep.score > 0
      ? `Taken as a whole, your chart supports ${area}.`
      : rep.score < 0
        ? `Taken as a whole, your chart asks for care over ${area}.`
        : `Taken as a whole, your chart is evenly balanced on ${area}.`;

  const lead = (() => {
    switch (basis) {
      case "recommended":
        return list.length === 1
          ? "One stretch ahead comes out recommended, and it is the one to aim at."
          : `${list.length} stretches ahead come out recommended. The soonest is first.`;
      case "best-available":
        return "No stretch ahead comes out recommended outright, so what follows is the "
          + "best-supported of what is ahead rather than a clean recommendation. "
          + "The reasons on both sides are shown so you can weigh them.";
      case "all-past":
        return rangeEnded
          ? `This reading covers ages ${fromAge} to ${toAge}, which for you ran out`
            + `${lastEnd ? ` in ${monthYear(lastEnd)}` : ""}. So the stretches it recommends are `
            + "behind you. That is the range of the reading talking, not a judgement about what is left."
          : "The stretches this chart recommends for it are behind you. The most recent are below.";
      default:
        return "Nothing in the range this reading covers comes out as a supported stretch for this.";
    }
  })();

  return (
    <section className="card question-answer">
      <h3>{question.question}</h3>
      <p className="summary"><strong>{standing}</strong> {lead}</p>

      {topic.disclaimer && <p className="notice disclaimer" role="note">{topic.disclaimer}</p>}

      {list.length > 0 && (
        <>
          <h4>
            {basis === "recommended"
              ? "When to aim for"
              : basis === "best-available"
                ? "The best-supported stretches ahead"
                : "The recommended stretches, now behind you"}
          </h4>
          <ol className="answer-windows">
            {list.map((w) => (
              <li key={`${w.start}-${w.maha}-${w.antar}`}>
                <strong>{monthYear(w.start)} to {monthYear(w.end)}</strong>
                <span className={`chip ${w.rank === "best" || w.rank === "good" ? "ok" : ""}`}>{w.rank}</span>
                {w.when === "now" && <span className="tag now">running now</span>}
                {w.live && <span className="tag">{area} is live here</span>}
                {w.supports.slice(0, 2).map((l, i) => <p key={`s${i}`} className="hint">{l}</p>)}
                {basis !== "recommended" && w.cautions.slice(0, 2).map((l, i) => (
                  <p key={`c${i}`} className="hint">{l}</p>
                ))}
              </li>
            ))}
          </ol>
        </>
      )}

      {sooner.length > 0 && (
        <section className="answer-sooner">
          <h4>Sooner, though not without caution</h4>
          <p className="hint">
            The stretch above is the next clean one, and it is a long way off. These come first and
            carry support of their own — they are simply not free of difficulty, so they are
            offered with their reasons rather than as a recommendation.
          </p>
          <ol className="answer-windows">
            {sooner.map((w) => (
              <li key={`${w.start}-${w.maha}-${w.antar}`}>
                <strong>{monthYear(w.start)} to {monthYear(w.end)}</strong>
                <span className="chip">{w.rank}</span>
                {w.supports.slice(0, 1).map((l, i) => <p key={`s${i}`} className="hint">{l}</p>)}
                {w.cautions.slice(0, 1).map((l, i) => <p key={`c${i}`} className="hint">{l}</p>)}
              </li>
            ))}
          </ol>
        </section>
      )}

      {basis === "none" && (
        <p className="notice" role="note">
          That is not a refusal. It means the stretches where {area} is live are also ones that ask
          for effort, so the reading below is worth weighing rather than waiting for a window the
          chart does not offer.
        </p>
      )}

      <p className="hint">
        A stretch being supported describes the conditions of a period, not an outcome. Nothing
        here says the thing will happen, or that it cannot happen outside these dates.
        {" "}For a day to act on once you have chosen a stretch, see <strong>Day timings</strong>:
        that is the hour-by-hour layer, and this is the years-long one.
      </p>
    </section>
  );
}
