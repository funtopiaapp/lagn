// A visible count of visits, in the footer.
//
// Counting visits needs a server, and this app deliberately has none, so the
// count comes from an external counting service. That is the only request the
// app makes to anyone once it has loaded, and the footer says so plainly.
//
// The service is configured at build time rather than hard-coded, because
// free counters disappear: countapi.xyz is gone and counterapi's v1 now
// answers 410. Changing provider is a build setting, not a code change, and
// with none set the component renders nothing and makes no request at all -
// which is what happens in development and in tests.

const BADGE = import.meta.env.VITE_COUNTER_BADGE ?? "";

/** The origin a configured badge is fetched from, for the CSP. "" if none. */
export function counterOrigin(badge = BADGE): string {
  try {
    return badge ? new URL(badge).origin : "";
  } catch {
    return "";
  }
}

export function VisitCounter() {
  if (!BADGE) return null;
  return (
    <p className="visits">
      {/* A plain image: no script from the counting service runs here, and if
          it fails to load the footer simply has no number in it. */}
      <img src={BADGE} alt="Visits to this site" width={56} height={20} loading="lazy" decoding="async" />
    </p>
  );
}
