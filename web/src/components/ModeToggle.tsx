import { MODES, type Mode } from "../lib/mode";

interface Props { mode: Mode; onChange: (m: Mode) => void }

/**
 * Lite or Pro, in the header beside the theme toggle.
 *
 * It lives at the top because it changes what the whole app offers, and a
 * setting of that weight should not be something a reader has to scroll to
 * the footer and tick a box for. Same segmented control as the theme, so
 * there is one idiom for "pick one of these" rather than two.
 */
export function ModeToggle({ mode, onChange }: Props) {
  return (
    <div className="theme-toggle mode-toggle" role="group" aria-label="Mode">
      {MODES.map(([m, label, hint]) => (
        <button
          key={m}
          type="button"
          aria-pressed={mode === m}
          title={hint}
          onClick={() => onChange(m)}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
