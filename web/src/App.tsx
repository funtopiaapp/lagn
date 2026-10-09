import { useEffect, useState } from "react";
import { api, ApiError } from "./api";
import { BirthForm } from "./components/BirthForm";
import { ChartView } from "./components/ChartView";
import { FamilyView } from "./components/FamilyView";
import { MatchView } from "./components/MatchView";
import { OfflineBanner } from "./components/OfflineBanner";
import { PeriodsView } from "./components/PeriodsView";
import { DayTimingsView } from "./components/DayTimingsView";
import { JaiminiView } from "./components/JaiminiView";
import { CharaView } from "./components/CharaView";
import { UpagrahaView } from "./components/UpagrahaView";
import { KpView } from "./components/KpView";
import { ReadingsView } from "./components/ReadingsView";
import { SavedCharts } from "./components/SavedCharts";
import { loadSaved, type SavedBirth } from "./lib/savedBirths";
import { loadToken, saveToken } from "./lib/session";
import { loadMode, saveMode, type Mode } from "./lib/mode";
import { loadTheme, type Theme } from "./lib/theme";
import { ThemeToggle } from "./components/ThemeToggle";
import { ModeToggle } from "./components/ModeToggle";
import { VisitCounter } from "./components/VisitCounter";
import type { BirthInput, ChartResponse, Sex, VersionInfo } from "./types";

type Tab = "chart" | "readings" | "periods" | "day" | "family" | "match" | "saved" | "jaimini" | "chara" | "upagraha" | "kp";

/** The general public's surface. Phase 13 adds nothing to this list, and a
 *  test pins that: turning Pro on must add tabs, never change Lite's. */
const LITE_TABS: [Tab, string][] = [["chart", "Chart"], ["readings", "Readings"], ["periods", "Sensitive periods"], ["day", "Day timings"], ["family", "Family"], ["match", "Match"], ["saved", "Saved profiles"]];

/** The professional surface, appended to the Lite tabs rather than replacing
 *  them. See docs/phase13/DESIGN.md section 2. */
const PRO_TABS: [Tab, string][] = [["jaimini", "Jaimini"], ["chara", "Chara dasha"], ["upagraha", "Upagrahas"], ["kp", "KP"]];

export function tabsFor(mode: Mode): [Tab, string][] {
  return mode === "pro" ? [...LITE_TABS, ...PRO_TABS] : LITE_TABS;
}

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
  // Lite or Pro. Device-local, never in the URL, so a shared link cannot drop
  // a lay reader onto the professional surface.
  const [mode, setModeState] = useState<Mode>(loadMode);
  // Charts kept on this device. Read once; every change writes through.
  const [saved, setSaved] = useState<SavedBirth[]>(loadSaved);
  const [showSaved, setShowSaved] = useState(false);

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

  const setMode = (m: Mode) => {
    saveMode(m);
    setModeState(m);
    // Leaving Pro while standing on a Pro tab would show an empty section.
    if (m === "lite" && !LITE_TABS.some(([t]) => t === tab)) setTab("chart");
  };

  const setTab = (t: Tab) => {
    if (t !== tab) history.pushState({ lagn: true, view: t }, "");
    // Any birth form may have saved a profile, so re-read rather than trust a
    // copy taken when the app started.
    if (t === "saved") setSaved(loadSaved());
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
        <ModeToggle mode={mode} onChange={setMode} />
        <ThemeToggle theme={theme} onChange={setTheme} />
        {chart && (
          <button type="button" onClick={() => { history.pushState({ lagn: true, view: "form" }, ""); setChart(null); }}>New chart</button>
        )}
      </header>

      <main>
        {!chart ? (
          <><p className="tagline">Vedic astrology, computed deterministically, in the South Indian tradition</p>
          <BirthForm title="Birth details" submitLabel="Compute chart" onSubmit={compute} busy={busy} askSex />
          {/* Before any chart exists there is no tab bar, so the saved
              profiles need their own way in. */}
          {saved.length > 0 && !showSaved && (
            <button type="button" className="link" onClick={() => { setSaved(loadSaved()); setShowSaved(true); }}>
              Open a saved profile ({saved.length})
            </button>
          )}
          {showSaved && (
            <SavedCharts saved={saved} onChange={setSaved} onOpen={(b, sx) => void compute(b, sx)} />
          )}</>
        ) : (
          <>
            <nav className="tabs" aria-label="Sections">
              {tabsFor(mode).map(([t, label]) => (
                <button key={t} type="button" aria-current={tab === t ? "page" : undefined} onClick={() => setTab(t)}>{label}</button>
              ))}
            </nav>
            {tab === "chart" && <ChartView chart={chart} />}
            {tab === "readings" && <ReadingsView birth={chart.input} sex={sex} reviewToken={token} lagna={chart.lagna} />}
            {tab === "periods" && <PeriodsView birth={chart.input} sex={sex} reviewToken={token} />}
            {tab === "day" && <DayTimingsView birth={chart.input} />}
            {tab === "family" && <FamilyView birth={chart.input} sex={sex} reviewToken={token} />}
            {tab === "match" && <MatchView birth={chart.input} reviewToken={token} />}
            {tab === "jaimini" && mode === "pro" && <JaiminiView birth={chart.input} />}
            {tab === "chara" && mode === "pro" && <CharaView birth={chart.input} />}
            {tab === "upagraha" && mode === "pro" && <UpagrahaView birth={chart.input} />}
            {tab === "kp" && mode === "pro" && <KpView birth={chart.input} />}
            {tab === "saved" && (
              <SavedCharts saved={saved} onChange={setSaved}
                onOpen={(b, sx) => { setTab("chart"); void compute(b, sx); }} />
            )}
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
