import { useEffect, useState } from "react";

/** Shown while the device is offline. The app opens offline; computing does not. */
export function OfflineBanner() {
  const [online, setOnline] = useState(() => navigator.onLine);
  useEffect(() => {
    const on = () => setOnline(true), off = () => setOnline(false);
    window.addEventListener("online", on);
    window.addEventListener("offline", off);
    return () => { window.removeEventListener("online", on); window.removeEventListener("offline", off); };
  }, []);
  if (online) return null;
  return <div className="offline" role="status">You're offline. Charts are computed on our server, so a connection is needed to compute one.</div>;
}
