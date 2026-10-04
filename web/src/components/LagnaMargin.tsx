import type { LagnaHold } from "../types";

/** Below this, a plausible error in the stated birth time would move every
 *  house. Half an hour is the granularity people actually remember a birth
 *  time to - "about half past seven" - so it is the honest line. */
const TIGHT_MINUTES = 30;

function minutes(n: number): string {
  const m = Math.round(n);
  if (m < 60) return `${m} minute${m === 1 ? "" : "s"}`;
  const h = Math.floor(m / 60);
  const rest = m % 60;
  const hours = `${h} hour${h === 1 ? "" : "s"}`;
  return rest === 0 ? hours : `${hours} ${rest} minute${rest === 1 ? "" : "s"}`;
}

/**
 * How much the stated birth time matters for this particular chart.
 *
 * Every house in every reading is counted from the lagna, so a birth time
 * that is wrong by more than the margin below does not shade the readings -
 * it renumbers all twelve houses and changes the verdicts wholesale. The app
 * used to present a chart cast from a remembered or defaulted time with the
 * same authority as one from a birth record. This says, per chart, what is
 * actually at stake, using the engine's own recomputation of when the lagna
 * enters and leaves its rasi.
 */
export function LagnaMargin({ hold, rasi }: { hold?: LagnaHold; rasi: string }) {
  // An engine build older than this field - a service worker still serving the
  // previous WebAssembly bundle, or a native host that has not caught up -
  // sends no margin at all. Say nothing rather than taking the readings down
  // with us: this is an aside, and no aside is worth a blank page.
  if (!hold || typeof hold.margin_minutes !== "number") return null;
  // Only at extreme latitudes, where the search found no boundary within
  // twelve hours. There is nothing useful to warn about.
  if (hold.capped) return null;

  const tight = hold.margin_minutes < TIGHT_MINUTES;
  // The nearer edge is the one that matters, and saying which side it is on
  // lets someone check it against what they actually know.
  const earlier = hold.minutes_in <= hold.minutes_left;
  const edge = earlier
    ? `${minutes(hold.minutes_in)} earlier`
    : `${minutes(hold.minutes_left)} later`;

  return (
    <p className={tight ? "notice warn lagna-margin" : "hint lagna-margin"} role="note">
      <strong>{rasi} lagna holds for {minutes(hold.margin_minutes)}.</strong>{" "}
      A birth time {edge} than the one entered would put the lagna in the next
      sign, and every house in these readings is counted from the lagna.
      {tight ? (
        <>
          {" "}
          This chart is unusually close to that edge, so if the time is
          remembered, rounded or unknown rather than taken from a birth record,
          treat the house-based readings below as one of two possibilities and
          confirm the time before relying on them.
        </>
      ) : (
        <>
          {" "}
          A time known to the nearest half hour is enough to settle it for this
          chart.
        </>
      )}
    </p>
  );
}
