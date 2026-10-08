// The professional surface has to show its working, because an arudha is the
// one Jaimini quantity a practitioner recomputes by hand. These pin the three
// things that make it checkable: the karaka ranking, the arudha exception
// being visible as an exception, and the variants being named rather than
// buried.

import { render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { JaiminiView } from "./JaiminiView";
import type { JaiminiResponse } from "../types";

const birth = { date: "1981-12-21", time: "14:10:00", latitude: 8.8932, longitude: 76.6141, utc_offset_hours: 5.5, place: "Kollam" };

afterEach(() => vi.unstubAllGlobals());

/**
 * The owner's own chart, taken verbatim from `/api/jaimini` rather than
 * written by hand. Hand-writing it is how this fixture drifted from the
 * engine once already: it carried properly cased names while the endpoint
 * was still emitting raw enum identifiers, so the tab showed "rahu karka"
 * and the tests were happy.
 */
const data: JaiminiResponse = {
  "karakas": [
    {
      "id": "atma",
      "abbrev": "AK",
      "name": "Atmakaraka",
      "signifies": "the self",
      "graha": "Rahu",
      "rasi": "Karka",
      "advancement": 29.85275916706557
    },
    {
      "id": "amatya",
      "abbrev": "AmK",
      "name": "Amatyakaraka",
      "signifies": "career, the minister",
      "graha": "Shani",
      "rasi": "Kanya",
      "advancement": 27.206084323315764
    },
    {
      "id": "bhratri",
      "abbrev": "BK",
      "name": "Bhratrikaraka",
      "signifies": "siblings, the guru",
      "graha": "Shukra",
      "rasi": "Makara",
      "advancement": 13.216938915111143
    },
    {
      "id": "matri",
      "abbrev": "MK",
      "name": "Matrikaraka",
      "signifies": "mother",
      "graha": "Budha",
      "rasi": "Dhanus",
      "advancement": 11.850312537123841
    },
    {
      "id": "pitri",
      "abbrev": "PiK",
      "name": "Pitrikaraka",
      "signifies": "father",
      "graha": "Chandra",
      "rasi": "Tula",
      "advancement": 10.907007560751396
    },
    {
      "id": "putra",
      "abbrev": "PuK",
      "name": "Putrakaraka",
      "signifies": "children",
      "graha": "Guru",
      "rasi": "Tula",
      "advancement": 10.818660140697432
    },
    {
      "id": "gnati",
      "abbrev": "GK",
      "name": "Gnatikaraka",
      "signifies": "obstacles, illness, cousins",
      "graha": "Kuja",
      "rasi": "Kanya",
      "advancement": 8.868851963492602
    },
    {
      "id": "dara",
      "abbrev": "DK",
      "name": "Darakaraka",
      "signifies": "spouse",
      "graha": "Surya",
      "rasi": "Dhanus",
      "advancement": 5.7975404713676255
    }
  ],
  "padas": [
    {
      "bhava": 1,
      "label": "AL",
      "bhava_rasi": "Mesha",
      "lord": "Kuja",
      "lord_rasi": "Kanya",
      "count": 6,
      "raw": "Kumbha",
      "rasi": "Kumbha",
      "adjusted": false
    },
    {
      "bhava": 4,
      "label": "A4",
      "bhava_rasi": "Karka",
      "lord": "Chandra",
      "lord_rasi": "Tula",
      "count": 4,
      "raw": "Makara",
      "rasi": "Tula",
      "adjusted": true
    },
    {
      "bhava": 12,
      "label": "UL",
      "bhava_rasi": "Meena",
      "lord": "Guru",
      "lord_rasi": "Tula",
      "count": 8,
      "raw": "Vrishabha",
      "rasi": "Vrishabha",
      "adjusted": false
    }
  ],
  "argala": [
    {
      "rasi": "Mesha",
      "pairs": [
        {
          "kind": "wealth",
          "argala_house": 2,
          "counter_house": 12,
          "argala_rasi": "Vrishabha",
          "counter_rasi": "Meena",
          "argala_grahas": [],
          "counter_grahas": [],
          "verdict": "none"
        },
        {
          "kind": "home",
          "argala_house": 4,
          "counter_house": 10,
          "argala_rasi": "Karka",
          "counter_rasi": "Makara",
          "argala_grahas": [
            "Rahu"
          ],
          "counter_grahas": [
            "Shukra",
            "Ketu"
          ],
          "verdict": "overcome"
        },
        {
          "kind": "gain",
          "argala_house": 11,
          "counter_house": 3,
          "argala_rasi": "Kumbha",
          "counter_rasi": "Mithuna",
          "argala_grahas": [],
          "counter_grahas": [],
          "verdict": "none"
        }
      ]
    }
  ],
  "variants": [
    {
      "id": "V-13-1",
      "question": "number of chara karakas",
      "chosen": "8, Rahu included"
    },
    {
      "id": "V-13-2",
      "question": "Rahu's advancement",
      "chosen": "reversed (30 - degrees)"
    }
  ]
} as JaiminiResponse;

function stub(body: unknown, status = 200) {
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(new Response(JSON.stringify(body), { status }))));
}

describe("the chara karakas", () => {
  it("are listed in rank order with the graha holding each", async () => {
    stub(data);
    render(<JaiminiView birth={birth} />);

    const karakas = within(await screen.findByRole("table", { name: "Chara karakas" }));
    const row = within(karakas.getByRole("row", { name: /Atmakaraka/ }));
    // Properly cased, and the South Indian name - the same words every other
    // tab uses, because they come from the engine rather than a table here.
    expect(row.getByText("Rahu")).toBeInTheDocument();
    // Rahu at 0.147 degrees of Karka reads as 29.853 travelled, which is what
    // puts it top. Showing the figure is what makes that checkable.
    expect(row.getByText("29.853°")).toBeInTheDocument();
    // Kuja, not "mars": the engine names it.
    expect(karakas.getByRole("row", { name: /Gnatikaraka/ })).toHaveTextContent("Kuja");

    expect(karakas.getByRole("row", { name: /Darakaraka/ })).toHaveTextContent("Surya");
    // Each karaka says what it stands for, unglossed but present.
    expect(screen.getByText(/the self/)).toBeInTheDocument();
    expect(screen.getByText(/spouse/)).toBeInTheDocument();
  });
});

describe("the arudha padas", () => {
  it("name A1 and A12 the way practice does", async () => {
    stub(data);
    render(<JaiminiView birth={birth} />);
    const padas = within(await screen.findByRole("table", { name: "Arudha padas" }));
    expect(padas.getByRole("rowheader", { name: "AL" })).toBeInTheDocument();
    expect(padas.getByRole("rowheader", { name: "UL" })).toBeInTheDocument();
  });

  it("says where a pada landed before the exception moved it", async () => {
    // A4 counted 4 to Makara, which is the 7th from Karka, so the 10th was
    // taken. A practitioner checking by hand needs to see the refused sign,
    // not just the answer. Tula appears twice in this row - the lord sits
    // there and the pada ends up there - so the row text is what is checked.
    stub(data);
    render(<JaiminiView birth={birth} />);
    const padas = within(await screen.findByRole("table", { name: "Arudha padas" }));
    const row = padas.getByRole("row", { name: /^A4/ });
    expect(row).toHaveTextContent("Karka");
    expect(row).toHaveTextContent("10th from Makara");
  });

  it("says nothing extra when the pada stood where it fell", async () => {
    stub(data);
    render(<JaiminiView birth={birth} />);
    const padas = within(await screen.findByRole("table", { name: "Arudha padas" }));
    expect(within(padas.getByRole("row", { name: /^AL/ })).queryByText(/10th from/)).toBeNull();
  });
});

describe("argala", () => {
  it("shows the verdict and the count on both sides", async () => {
    // Scoped to the argala table: Mesha is also a bhava in the pada table.
    stub(data);
    render(<JaiminiView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Argala and virodhargala" }));
    const row = within(table.getByRole("row", { name: /Mesha/ }));
    expect(row.getByText("overcome")).toBeInTheDocument();
    expect(row.getByText(/1 v 2/)).toBeInTheDocument();
  });
});

describe("the variants", () => {
  it("are named on the surface, not left to a footnote", async () => {
    // Criterion 7: a reader has to be able to tell which scheme produced the
    // numbers above, because none of these has been signed off yet.
    stub(data);
    render(<JaiminiView birth={birth} />);
    expect(await screen.findByText(/V-13-1/)).toBeInTheDocument();
    expect(screen.getByText("8, Rahu included")).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent(/none of them has been signed off/);
  });
});

describe("when the engine cannot answer", () => {
  it("says so instead of rendering an empty table", async () => {
    stub({ error: "birth time out of range" }, 400);
    render(<JaiminiView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("birth time out of range");
  });
});
