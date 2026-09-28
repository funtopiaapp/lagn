import { cellOf } from "../lib/southIndian";
import type { SignView } from "../types";

export interface ChartEntry {
  signIndex: number;
  /** e.g. "Su 06°16'" or "Su" */
  label: string;
  retrograde?: boolean;
}

interface Props {
  signs: SignView[];
  lagnaIndex: number;
  entries: ChartEntry[];
  caption: string[];
  ariaLabel: string;
}

const CELL = 100;

/** South Indian square chart. Signs are fixed; the lagna is marked with a diagonal. */
export function SouthIndianChart({ signs, lagnaIndex, entries, caption, ariaLabel }: Props) {
  const bySign = new Map<number, ChartEntry[]>();
  for (const e of entries) bySign.set(e.signIndex, [...(bySign.get(e.signIndex) ?? []), e]);

  return (
    <svg className="si-chart" viewBox={`0 0 ${CELL * 4} ${CELL * 4}`} role="img" aria-label={ariaLabel}>
      <rect x="0.5" y="0.5" width={CELL * 4 - 1} height={CELL * 4 - 1} className="frame" />
      {signs.map((s) => {
        const [row, col] = cellOf(s.index);
        const x = col * CELL, y = row * CELL;
        const here = bySign.get(s.index) ?? [];
        const isLagna = s.index === lagnaIndex;
        return (
          <g key={s.index} data-sign={s.name} data-lagna={isLagna || undefined}>
            <rect x={x} y={y} width={CELL} height={CELL} className={isLagna ? "cell lagna" : "cell"} />
            {isLagna && <line x1={x} y1={y + 22} x2={x + 22} y2={y} className="lagna-mark" />}
            <text x={x + CELL - 6} y={y + 14} className="sign-name" textAnchor="end">{s.name}</text>
            {isLagna && <text x={x + 6} y={y + 34} className="graha asc">Asc</text>}
            {here.map((e, i) => (
              <text key={e.label} x={x + 6} y={y + (isLagna ? 50 : 34) + i * 15} className={e.retrograde ? "graha retro" : "graha"}>
                {e.label}{e.retrograde ? " (R)" : ""}
              </text>
            ))}
          </g>
        );
      })}
      <rect x={CELL} y={CELL} width={CELL * 2} height={CELL * 2} className="centre" />
      {caption.map((line, i) => (
        <text key={i} x={CELL * 2} y={CELL * 2 - (caption.length - 1) * 9 + i * 18} className={i === 0 ? "caption title" : "caption"} textAnchor="middle">{line}</text>
      ))}
    </svg>
  );
}
