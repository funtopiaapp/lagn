import type { ChartResponse } from "../types";

export function AshtakavargaTable({ chart }: { chart: ChartResponse }) {
  const av = chart.ashtakavarga;
  const short = (n: string) => n.slice(0, 3);
  return (
    <section className="card">
      <h3>Ashtakavarga</h3>
      <div className="table-wrap">
        <table className="av">
          <thead><tr><th scope="col"></th>{chart.signs.map((s) => <th key={s.index} scope="col" title={s.name}>{short(s.name)}</th>)}<th scope="col">Total</th></tr></thead>
          <tbody>
            {av.rows.map((r) => (
              <tr key={r.graha}><th scope="row">{r.graha}</th>{r.bindus.map((b, i) => <td key={i} className="num">{b}</td>)}<td className="num">{r.total}</td></tr>
            ))}
            <tr className="sav"><th scope="row">SAV</th>{av.sav.map((b, i) => <td key={i} className="num">{b}</td>)}<td className="num">{av.sav_total}</td></tr>
          </tbody>
        </table>
      </div>
      <div className="table-wrap">
        <table className="av">
          <thead><tr><th scope="col">House</th>{av.sav_by_house.map((_, i) => <th key={i} scope="col">{i + 1}</th>)}</tr></thead>
          <tbody><tr className="sav"><th scope="row">SAV</th>{av.sav_by_house.map((b, i) => <td key={i} className="num">{b}</td>)}</tr></tbody>
        </table>
      </div>
    </section>
  );
}
