import { useState } from "react";

import type { TimingRank, TimingWindow } from "../types";

/** Best first. Every stretch in range gets one, so the list has no holes. */
const ORDER: TimingRank[] = ["best", "good", "mixed", "caution", "not judged"];

/**
 * What each rank means, in words a reader who knows no astrology can act on.
 *
 * `short` is the hover hint on the chip. `plain` is shown under every stretch,
 * because a hint that only appears on hover is no use on a phone, and a word
 * like "mixed" means nothing on its own.
 */
const MEANING: Record<TimingRank, { short: string; plain: (area: string) => string }> = {
  best: {
    short: "One to plan around: the classical rules point here, and the period supports it",
    plain: (area) =>
      `One to plan around. The classical timing rules point at ${area} in this stretch, and the ` +
      `period itself is supportive. If you have any choice about when to act, a stretch like this ` +
      `is the one to choose.`,
  },
  good: {
    short: "A supportive period, though the rules do not single out this area in it",
    plain: (area) =>
      `A supportive stretch in general. What you begin here tends to go well — but the timing ` +
      `rules do not pick out ${area} in particular, so this is a good period to act in rather ` +
      `than a period when ${area} comes forward by itself.`,
  },
  mixed: {
    short: "Support and difficulty both apply; read the reasons and weigh them",
    plain: (area) =>
      `Support and difficulty both apply here. Not a stretch to avoid, and not one to lean on ` +
      `either. Read the reasons below and judge which side matters more for what you are ` +
      `actually planning around ${area}.`,
  },
  caution: {
    short: "Expect more effort, delay or rework; do not stake everything on this window",
    plain: (area) =>
      `This stretch asks for care. It does not say ${area} will go wrong — it says expect more ` +
      `effort, waiting or rework than usual, so keep a second option open and do not stake ` +
      `everything on this window.`,
  },
  "not judged": {
    short: "No reviewed rule applies, so nothing is claimed either way",
    plain: () =>
      `No reviewed period rule applies to this stretch, so the engine says nothing about it ` +
      `either way. An absence of judgement is not a judgement.`,
  },
};

function chipKind(rank: TimingRank): string {
  if (rank === "best" || rank === "good") return "ok";
  if (rank === "caution") return "warn";
  return "";
}

/**
 * Every dasha stretch in range, ranked for this topic.
 *
 * It used to list only the stretches a topic's timing rules named, which left
 * holes - on some charts a whole mahadasha vanished, twelve years at a time -
 * and a reader could not tell a quiet period from a missing one. Now every
 * stretch is here and ranked, with the ones the timing rules single out marked
 * as such.
 *
 * The best stretches lead, because "when is a good time for this" is the
 * question people arrive with. Each line names the area it is about: before
 * this the reading said "Supports" and left the reader to guess what of.
 */
export function TopicTiming({ windows, topic, ages }: { windows: TimingWindow[]; topic: string; ages: [number, number] }) {
  const [showAll, setShowAll] = useState(false);
  if (windows.length === 0) return null;

  const area = topic.toLowerCase();
  const byRank = (a: TimingWindow, b: TimingWindow) =>
    ORDER.indexOf(a.rank) - ORDER.indexOf(b.rank) || a.start.localeCompare(b.start);

  const best = windows.filter((w) => w.rank === "best" || w.rank === "good");
  // Ranked when leading with recommendations, chronological when showing the
  // whole span - a timeline that jumps around is not a timeline.
  const shown = showAll ? [...windows].sort((a, b) => a.start.localeCompare(b.start)) : [...best].sort(byRank);
  // Only the ranks actually present, so the legend never explains a word that
  // does not appear on the page.
  const present = ORDER.filter((r) => windows.some((w) => w.rank === r));

  return (
    <section className="topic-timing">
      <h4>When this comes forward</h4>
      <p className="hint">
        Every dasha stretch between ages {ages[0]} and {ages[1]}, ranked for {area}. A rank
        describes the conditions of the period, never an outcome.
      </p>

      <details className="rank-legend">
        <summary>What {present.filter((r) => r !== "not judged").join(", ")} mean</summary>
        <dl>
          {present.map((r) => (
            <div key={r} className="rank-legend-row">
              <dt><span className={`chip ${chipKind(r)}`}>{r}</span></dt>
              <dd>{MEANING[r].plain(area)}</dd>
            </div>
          ))}
        </dl>
      </details>

      {best.length === 0 ? (
        <p className="notice" role="note">
          Of the {windows.length} stretches in this range, none comes out favourable on the period
          factors. That is not a refusal: it means the stretches where this area is live also ask
          for effort, so the reasons under each one are worth reading before you plan.
        </p>
      ) : (
        <p className="summary">
          <strong>{best.length} of {windows.length}</strong>{" "}
          {best.length === 1 ? "stretch is" : "stretches are"} recommended for {area}.
        </p>
      )}

      <ul className="timing-list">
        {shown.map((w) => (
          <li key={`${w.start}-${w.maha}-${w.antar}`} className={`timing-window v-${w.rank.replace(/ /g, "-")}`}>
            <div className="timing-head">
              <strong className="timing-dates">{w.start} → {w.end}</strong>
              <span className="tag">{w.maha} / {w.antar}</span>
              {/* The hint on the chip for a pointer; the sentence below for
                  everyone else, since a phone has no hover. */}
              <span className={`chip ${chipKind(w.rank)}`} title={MEANING[w.rank].short}>{w.rank}</span>
              {w.live && <span className="tag" title={`The classical timing rules for ${area} point at this stretch`}>{area} is live here</span>}
            </div>
            <p className="timing-blurb">{MEANING[w.rank].plain(area)}</p>
            {(w.supports.length > 0 || w.cautions.length > 0) && (
              <ul className="timing-why">
                {w.supports.map((l, i) => <li key={`s${i}`} className="ok">{l}</li>)}
                {w.cautions.map((l, i) => <li key={`c${i}`} className="warn">{l}</li>)}
              </ul>
            )}
          </li>
        ))}
      </ul>

      {windows.length > best.length && (
        <button type="button" className="link" onClick={() => setShowAll(!showAll)}>
          {showAll
            ? "Show only the recommended stretches"
            : `Show the whole span: all ${windows.length} stretches in order`}
        </button>
      )}
    </section>
  );
}
