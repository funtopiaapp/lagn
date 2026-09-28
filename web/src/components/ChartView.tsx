import { useState } from "react";
import type { ChartResponse } from "../types";
import { AshtakavargaTable } from "./AshtakavargaTable";
import { DashaTimeline } from "./DashaTimeline";
import { PositionsTable } from "./PositionsTable";
import { SouthIndianChart, type ChartEntry } from "./SouthIndianChart";

export function ChartView({ chart }: { chart: ChartResponse }) {
  const [varga, setVarga] = useState("d1");
  const v = chart.vargas.find((x) => x.varga === varga) ?? chart.vargas[0]!;

  // D-1 shows degrees (the engine's strings, truncated to minutes for the cell);
  // divisional charts carry signs only.
  const entries: ChartEntry[] = varga === "d1"
    ? chart.positions.map((p) => {
        const cell = v.grahas.find((g) => g.key === p.key)!;
        return { signIndex: cell.sign_index, label: `${p.abbrev} ${p.degrees.slice(0, 6)}`, retrograde: p.retrograde };
      })
    : v.grahas.map((g) => ({ signIndex: g.sign_index, label: g.abbrev }));

  const i = chart.input;
  return (
    <div className="chart-view">
      <section className="card">
        <p className="summary">
          {i.place || `${i.latitude}, ${i.longitude}`} · {i.date} {i.time} · UTC{i.utc_offset_hours >= 0 ? "+" : ""}{i.utc_offset_hours}
        </p>
        <p className="summary">
          Lagna <strong>{chart.lagna.rasi}</strong> {chart.lagna.degrees} · Janma nakshatra <strong>{chart.dasha.janma_nakshatra}</strong> ({chart.dasha.janma_nakshatra_tamil}) · Ayanamsa (Lahiri) {chart.ayanamsa}
        </p>
        <label className="varga-select">Chart
          <select value={varga} onChange={(e) => setVarga(e.target.value)}>
            {chart.vargas.map((x) => <option key={x.varga} value={x.varga}>{x.label}</option>)}
          </select>
        </label>
        <SouthIndianChart
          signs={chart.signs}
          lagnaIndex={v.lagna_index}
          entries={entries}
          caption={[v.label, `Lagna ${v.lagna}`, i.date]}
          ariaLabel={`${v.label} South Indian chart. Lagna in ${v.lagna}. ${v.grahas.map((g) => `${g.abbrev} in ${g.sign}`).join(", ")}.`}
        />
      </section>
      <PositionsTable chart={chart} />
      <DashaTimeline dasha={chart.dasha} />
      <AshtakavargaTable chart={chart} />
    </div>
  );
}
