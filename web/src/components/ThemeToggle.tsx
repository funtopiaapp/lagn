import { useEffect } from "react";
import { applyTheme, saveTheme, THEMES, watchSystem, type Theme } from "../lib/theme";

interface Props { theme: Theme; onChange: (t: Theme) => void }

const ICON: Record<Theme, string> = { system: "◐", light: "☀", dark: "☾" };

/** System / Light / Dark. The choice always wins over the OS setting. */
export function ThemeToggle({ theme, onChange }: Props) {
  useEffect(() => applyTheme(theme), [theme]);
  // While following the system, react to it changing under us.
  useEffect(() => (theme === "system" ? watchSystem(() => applyTheme("system")) : undefined), [theme]);

  const pick = (t: Theme) => { saveTheme(t); onChange(t); };
  return (
    <div className="theme-toggle" role="group" aria-label="Theme">
      {THEMES.map(([t, label]) => (
        <button key={t} type="button" aria-pressed={theme === t} title={label} onClick={() => pick(t)}>
          <span aria-hidden="true">{ICON[t]}</span><span className="theme-label">{label}</span>
        </button>
      ))}
    </div>
  );
}
