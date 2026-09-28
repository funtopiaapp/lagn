// Read a date or time back in words, so a DD/MM vs MM/DD misreading is visible
// before a chart is computed. The browser's date field shows the date in the
// OS locale (06/07/1985 is 7 June in the US, 6 July in India); this readback
// is unambiguous. Pure string handling - no Date object, no calendar maths.

const MONTHS = ["January", "February", "March", "April", "May", "June", "July",
  "August", "September", "October", "November", "December"];

/** "1985-06-21" -> "21 June 1985"; null if not YYYY-MM-DD. */
export function readDate(iso: string): string | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  if (!m) return null;
  const month = MONTHS[Number(m[2]) - 1];
  if (!month) return null;
  return `${Number(m[3])} ${month} ${m[1]}`;
}

/** "14:30" or "14:30:05" -> "14:30:05 (24-hour clock)"; null if malformed. */
export function readTime(t: string): string | null {
  const m = /^(\d{2}):(\d{2})(?::(\d{2}))?$/.exec(t);
  if (!m) return null;
  return `${m[1]}:${m[2]}:${m[3] ?? "00"} (24-hour clock)`;
}
