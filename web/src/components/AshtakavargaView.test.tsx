// The Ashtakavarga grid on the professional surface.
//
// The fixture is the real grid from /api/ashtakavarga, so the fixed totals in
// it are the ones the engine actually produces - which is the point: this view
// exists to let a reader watch those invariants hold.

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AshtakavargaView } from "./AshtakavargaView";
import type { AshtakavargaResponse } from "../types";

const birth = { date: "1981-12-21", time: "14:10:00", latitude: 8.8932, longitude: 76.6141, utc_offset_hours: 5.5, place: "Kollam" };

afterEach(() => vi.unstubAllGlobals());

const data: AshtakavargaResponse = {
  "lagna": "Mesha",
  "signs": [
    "Mesha",
    "Vrishabha",
    "Mithuna",
    "Karka",
    "Simha",
    "Kanya",
    "Tula",
    "Vrischika",
    "Dhanus",
    "Makara",
    "Kumbha",
    "Meena"
  ],
  "rows": [
    {
      "graha": "Surya",
      "bindus": [
        3,
        3,
        6,
        6,
        4,
        5,
        4,
        1,
        5,
        2,
        3,
        6
      ],
      "by_house": [
        3,
        3,
        6,
        6,
        4,
        5,
        4,
        1,
        5,
        2,
        3,
        6
      ],
      "total": 48,
      "expected_total": 48
    },
    {
      "graha": "Chandra",
      "bindus": [
        4,
        4,
        4,
        7,
        2,
        5,
        6,
        3,
        2,
        4,
        5,
        3
      ],
      "by_house": [
        4,
        4,
        4,
        7,
        2,
        5,
        6,
        3,
        2,
        4,
        5,
        3
      ],
      "total": 49,
      "expected_total": 49
    },
    {
      "graha": "Kuja",
      "bindus": [
        5,
        3,
        4,
        3,
        3,
        5,
        3,
        1,
        4,
        1,
        3,
        4
      ],
      "by_house": [
        5,
        3,
        4,
        3,
        3,
        5,
        3,
        1,
        4,
        1,
        3,
        4
      ],
      "total": 39,
      "expected_total": 39
    },
    {
      "graha": "Budha",
      "bindus": [
        6,
        8,
        2,
        4,
        5,
        6,
        4,
        5,
        3,
        3,
        3,
        5
      ],
      "by_house": [
        6,
        8,
        2,
        4,
        5,
        6,
        4,
        5,
        3,
        3,
        3,
        5
      ],
      "total": 54,
      "expected_total": 54
    },
    {
      "graha": "Guru",
      "bindus": [
        5,
        4,
        4,
        4,
        6,
        5,
        6,
        4,
        5,
        5,
        5,
        3
      ],
      "by_house": [
        5,
        4,
        4,
        4,
        6,
        5,
        6,
        4,
        5,
        5,
        5,
        3
      ],
      "total": 56,
      "expected_total": 56
    },
    {
      "graha": "Shukra",
      "bindus": [
        4,
        7,
        4,
        5,
        6,
        2,
        4,
        6,
        3,
        4,
        6,
        1
      ],
      "by_house": [
        4,
        7,
        4,
        5,
        6,
        2,
        4,
        6,
        3,
        4,
        6,
        1
      ],
      "total": 52,
      "expected_total": 52
    },
    {
      "graha": "Shani",
      "bindus": [
        1,
        1,
        4,
        5,
        4,
        4,
        2,
        4,
        3,
        4,
        4,
        3
      ],
      "by_house": [
        1,
        1,
        4,
        5,
        4,
        4,
        2,
        4,
        3,
        4,
        4,
        3
      ],
      "total": 39,
      "expected_total": 39
    }
  ],
  "sav": [
    28,
    30,
    28,
    34,
    30,
    32,
    29,
    24,
    25,
    23,
    29,
    25
  ],
  "sav_by_house": [
    28,
    30,
    28,
    34,
    30,
    32,
    29,
    24,
    25,
    23,
    29,
    25
  ],
  "sav_total": 337,
  "sav_expected_total": 337,
  "contributors": [
    [
      [
        "Kuja",
        "Budha",
        "Shani"
      ],
      [
        "Kuja",
        "Budha",
        "Shani"
      ],
      [
        "Surya",
        "Kuja",
        "Guru",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Budha",
        "Guru"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani"
      ],
      [
        "Budha"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Shukra",
        "Shani"
      ],
      [
        "Surya",
        "Lagna"
      ],
      [
        "Budha",
        "Guru",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Guru",
        "Shani",
        "Lagna"
      ]
    ],
    [
      [
        "Chandra",
        "Budha",
        "Guru",
        "Shukra"
      ],
      [
        "Surya",
        "Kuja",
        "Guru",
        "Shukra"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Shani"
      ],
      [
        "Chandra",
        "Guru"
      ],
      [
        "Surya",
        "Budha",
        "Guru",
        "Shukra",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra"
      ],
      [
        "Kuja",
        "Shukra",
        "Shani"
      ],
      [
        "Chandra",
        "Budha"
      ],
      [
        "Kuja",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Budha",
        "Shukra"
      ]
    ],
    [
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Budha",
        "Shani"
      ],
      [
        "Kuja",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Kuja",
        "Guru",
        "Shani"
      ],
      [
        "Chandra",
        "Guru",
        "Shukra"
      ],
      [
        "Surya",
        "Kuja",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha"
      ],
      [
        "Shukra"
      ],
      [
        "Chandra",
        "Kuja",
        "Shukra",
        "Shani"
      ],
      [
        "Lagna"
      ],
      [
        "Surya",
        "Budha",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Guru",
        "Shani"
      ]
    ],
    [
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Kuja",
        "Shani"
      ],
      [
        "Chandra",
        "Kuja",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Budha",
        "Guru",
        "Shukra"
      ],
      [
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani"
      ],
      [
        "Surya",
        "Chandra",
        "Budha",
        "Shukra",
        "Lagna"
      ],
      [
        "Kuja",
        "Budha",
        "Shani"
      ],
      [
        "Chandra",
        "Shukra",
        "Lagna"
      ],
      [
        "Budha",
        "Shukra",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Guru",
        "Shukra",
        "Shani"
      ]
    ],
    [
      [
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Lagna"
      ],
      [
        "Budha",
        "Guru",
        "Shukra",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Shukra"
      ],
      [
        "Surya",
        "Kuja",
        "Guru",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Budha",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shukra",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Lagna"
      ],
      [
        "Chandra",
        "Guru",
        "Shukra",
        "Shani"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Guru",
        "Lagna"
      ],
      [
        "Surya",
        "Budha",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha"
      ]
    ],
    [
      [
        "Budha",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Lagna"
      ],
      [
        "Chandra",
        "Shukra"
      ],
      [
        "Surya",
        "Chandra",
        "Budha",
        "Shukra"
      ],
      [
        "Surya",
        "Chandra",
        "Kuja",
        "Shukra",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Shukra",
        "Shani"
      ],
      [
        "Chandra",
        "Kuja",
        "Budha",
        "Guru",
        "Shukra",
        "Lagna"
      ],
      [
        "Shukra"
      ]
    ],
    [
      [
        "Lagna"
      ],
      [
        "Budha"
      ],
      [
        "Surya",
        "Kuja",
        "Shukra",
        "Lagna"
      ],
      [
        "Surya",
        "Kuja",
        "Budha",
        "Shani",
        "Lagna"
      ],
      [
        "Chandra",
        "Kuja",
        "Budha",
        "Guru"
      ],
      [
        "Surya",
        "Budha",
        "Guru",
        "Lagna"
      ],
      [
        "Surya",
        "Budha"
      ],
      [
        "Kuja",
        "Budha",
        "Shukra",
        "Shani"
      ],
      [
        "Surya",
        "Chandra",
        "Shukra"
      ],
      [
        "Surya",
        "Kuja",
        "Shani",
        "Lagna"
      ],
      [
        "Kuja",
        "Guru",
        "Shani",
        "Lagna"
      ],
      [
        "Surya",
        "Chandra",
        "Guru"
      ]
    ]
  ]
} as AshtakavargaResponse;

function stub(body: unknown, status = 200) {
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(new Response(JSON.stringify(body), { status }))));
}

describe("the grid", () => {
  it("shows every graha's BAV and the SAV beneath", async () => {
    stub(data);
    render(<AshtakavargaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Bhinnashtakavarga" }));
    expect(table.getAllByRole("row")).toHaveLength(9); // header + 7 grahas + SAV
    expect(table.getByRole("rowheader", { name: "Surya" })).toBeInTheDocument();
    expect(table.getByRole("rowheader", { name: "SAV" })).toBeInTheDocument();
  });

  it("shows the fixed total beside the computed one, so the invariant is visible", async () => {
    // A graha's BAV always comes to the same number whatever the chart, and
    // the SAV always to 337. Printing both is what lets a reader check the
    // grid instead of trusting it.
    stub(data);
    render(<AshtakavargaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Bhinnashtakavarga" }));
    const sav = within(table.getByRole("row", { name: /^SAV/ }));
    expect(sav.getAllByText("337").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText(/always comes to/)).toBeInTheDocument();
  });

  it("raises an alarm if an invariant ever fails, rather than printing the number quietly", async () => {
    // This should be impossible. If it happens the view must say so, because
    // a wrong bindu grid is not something a reader can spot by eye.
    const broken = { ...data, sav_total: data.sav_total + 1 };
    stub(broken);
    render(<AshtakavargaView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent(/An invariant failed/);
    expect(screen.getByRole("alert")).toHaveTextContent(/should not be relied on/);
  });
});

describe("the frame", () => {
  it("switches between reading by sign and by house", async () => {
    stub(data);
    render(<AshtakavargaView birth={birth} />);
    const table = await screen.findByRole("table", { name: "Bhinnashtakavarga" });
    // By sign to begin with: the first column heading is a sign abbreviation.
    expect(within(table).getByRole("columnheader", { name: "Graha" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "By house" }));
    expect(within(table).getByRole("columnheader", { name: "House" })).toBeInTheDocument();
  });
});

describe("when the engine cannot answer", () => {
  it("says so rather than rendering an empty grid", async () => {
    stub({ error: "latitude 91 out of range" }, 400);
    render(<AshtakavargaView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("latitude 91");
  });
});
