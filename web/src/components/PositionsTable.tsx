import type { ChartResponse } from "../types";

export function PositionsTable({ chart }: { chart: ChartResponse }) {
  return (
    <div className="table-wrap">
      <table className="positions">
        <caption>Graha positions (sidereal, Lahiri)</caption>
        <thead>
          <tr><th scope="col">Graha</th><th scope="col">Rasi</th><th scope="col">Degree</th><th scope="col">House</th>
            <th scope="col">Nakshatra</th><th scope="col">Pada</th><th scope="col">Dignity</th><th scope="col">State</th></tr>
        </thead>
        <tbody>
          <tr className="lagna-row">
            <th scope="row">Lagna</th>
            <td>{chart.lagna.rasi} <span className="ta">{chart.lagna.rasi_tamil}</span></td>
            <td className="num">{chart.lagna.degrees}</td><td className="num">1</td>
            <td>{chart.lagna.nakshatra} <span className="ta">{chart.lagna.nakshatra_tamil}</span></td>
            <td className="num">{chart.lagna.pada}</td><td></td><td></td>
          </tr>
          {chart.positions.map((p) => (
            <tr key={p.key} data-graha={p.key}>
              <th scope="row">{p.name}</th>
              <td>{p.rasi} <span className="ta">{p.rasi_tamil}</span></td>
              <td className="num">{p.degrees}</td>
              <td className="num">{p.house}</td>
              <td>{p.nakshatra} <span className="ta">{p.nakshatra_tamil}</span></td>
              <td className="num">{p.pada}</td>
              <td>{p.dignity}</td>
              <td>{[p.retrograde && "retrograde", p.combust && "combust", p.baladi, p.jagradadi].filter(Boolean).join(", ")}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
