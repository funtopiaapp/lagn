import { useEffect, useState } from "react";
import { api, ApiError } from "./api";
import { BirthForm } from "./components/BirthForm";
import { ChartView } from "./components/ChartView";
import { FamilyView } from "./components/FamilyView";
import { MatchView } from "./components/MatchView";
import { OfflineBanner } from "./components/OfflineBanner";
import { PeriodsView } from "./components/PeriodsView";
import { ReadingsView } from "./components/ReadingsView";
import { loadToken, saveToken } from "./lib/session";
import { loadTheme, type Theme } from "./lib/theme";
import { ThemeToggle } from "./components/ThemeToggle";
import { VisitCounter } from "./components/VisitCounter";
import type { BirthInput, ChartResponse, Sex, VersionInfo } from "./types";

type Tab = "chart" | "readings" | "periods" | "family" | "match";
const TABS: [Tab, string][] = [["chart", "Chart"], ["readings", "Readings"], ["periods", "Sensitive periods"], ["family", "Family"], ["match", "Match"]];

export function App() {
  const [chart, setChart] = useState<ChartResponse | null>(null);
  // Asked once, on the birth form, and used by every reading of this chart.
  const [sex, setSex] = useState<Sex | undefined>(undefined);
  const [tab, setTabState] = useState<Tab>("chart");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [version, setVersion] = useState<VersionInfo | null>(null);
  const [token, setToken] = useState(loadToken);
  const [tokenDraft, setTokenDraft] = useState("");
  const [theme, setTheme] = useState<Theme>(loadTheme);

  useEffect(() => { api.version().then(setVersion).catch(() => setVersion(null)); }, []);

  // Views are history entries, so the browser's Back - and Android's hardware
  // Back in a wrapped app - moves within the app instead of leaving it.
  useEffect(() => {
    if (!history.state?.lagn) history.replaceState({ lagn: true, view: "form" }, "");
    const onPop = (e: PopStateEvent) => {
      const view = e.state?.view as Tab | "form" | undefined;
      if (!view || view === "form") { setChart(null); return; }
      setTabState(view);
    };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  const setTab = (t: Tab) => {
    if (t !== tab) history.pushState({ lagn: true, view: t }, "");
    setTabState(t);
  };

  const compute = async (birth: BirthInput, sex?: Sex) => {
    setBusy(true); setError(null);
    try {
      setChart(await api.chart(birth));
      setSex(sex);
      history.pushState({ lagn: true, view: "chart" }, "");
      setTabState("chart");
    }
    catch (e) { setError(e instanceof ApiError ? e.message : String(e)); }
    finally { setBusy(false); }
  };

  const applyToken = (t: string) => { saveToken(t); setToken(t); setTokenDraft(""); };

  return (
    <div className="app">
      <OfflineBanner />
      <header className="top">
        <h1>lagn<span className="dot">.</span></h1>
        <span className="spacer" />
        <ThemeToggle theme={theme} onChange={setTheme} />
        {chart && (
          <button type="button" onClick={() => { history.pushState({ lagn: true, view: "form" }, ""); setChart(null); }}>New chart</button>
        )}
      </header>

      <main>
        {!chart ? (
          <><p className="tagline">Vedic astrology, computed deterministically, in the South Indian tradition</p>
          <BirthForm title="Birth details" submitLabel="Compute chart" onSubmit={compute} busy={busy} askSex /></>
        ) : (
          <>
            <nav className="tabs" aria-label="Sections">
              {TABS.map(([t, label]) => (
                <button key={t} type="button" aria-current={tab === t ? "page" : undefined} onClick={() => setTab(t)}>{label}</button>
              ))}
            </nav>
            {tab === "chart" && <ChartView chart={chart} />}
            {tab === "readings" && <ReadingsView birth={chart.input} sex={sex} reviewToken={token} lagna={chart.lagna} />}
            {tab === "periods" && <PeriodsView birth={chart.input} sex={sex} reviewToken={token} />}
            {tab === "family" && <FamilyView birth={chart.input} sex={sex} reviewToken={token} />}
            {tab === "match" && <MatchView birth={chart.input} reviewToken={token} />}
          </>
        )}
        {error && <p className="error" role="alert">{error}</p>}
      </main>

      <footer>
        {version?.review_enabled && (
          <details className="reviewer">
            <summary>{token ? "Reviewer mode on" : "Reviewer sign-in"}</summary>
            {token ? (
              <button type="button" onClick={() => applyToken("")}>Leave reviewer mode</button>
            ) : (
              <form onSubmit={(e) => { e.preventDefault(); applyToken(tokenDraft.trim()); }}>
                <label>Review token<input type="password" autoComplete="off" value={tokenDraft} onChange={(e) => setTokenDraft(e.target.value)} /></label>
                <button type="submit" disabled={!tokenDraft.trim()}>Enter</button>
              </form>
            )}
          </details>
        )}
        <p>For guidance and study. Not a substitute for professional advice on health, legal or financial matters.</p>
        {/* AGPL section 13: anyone using this over a network is entitled to
            the source of the version they are using. */}
        <p>
          Free software, AGPL-3.0.{" "}
          <a href="https://github.com/funtopiaapp/lagn" target="_blank" rel="noopener noreferrer">Source code</a>.
        </p>
        <VisitCounter />
        {import.meta.env.VITE_COUNTER_BADGE && (
          <p>
            Visits are counted by a third party, which sees your address and browser. No cookies, no accounts.
            Your birth details and family members never leave this device.
          </p>
        )}
        {version && (
          <p className="meta">
            Engine {version.engine} · Swiss Ephemeris {version.swiss_ephemeris} · tz database {version.tzdb} · {version.attribution}
          </p>
        )}
      </footer>
    </div>
  );
}
