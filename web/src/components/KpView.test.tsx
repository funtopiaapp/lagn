// KP on the professional surface.
//
// The fixture is taken verbatim from /api/kp. Hand-writing one is how the
// Jaimini fixture drifted from the engine once already, carrying properly
// cased names while the endpoint still emitted raw identifiers.

import { render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { KpView } from "./KpView";
import type { KpResponse } from "../types";

const birth = { date: "1981-12-21", time: "14:10:00", latitude: 8.8932, longitude: 76.6141, utc_offset_hours: 5.5, place: "Kollam" };

afterEach(() => vi.unstubAllGlobals());

const data: KpResponse = {
  "ascendant": {
    "longitude": 6.794702084026838,
    "degrees": "06\u00b047'40.92\"",
    "rasi": "Mesha",
    "nakshatra": "Ashwini",
    "pada": 3,
    "sign_lord": "Kuja",
    "star_lord": "Ketu",
    "sub_lord": "Rahu",
    "sub_sub_lord": "Ketu"
  },
  "grahas": [
    {
      "graha": "Surya",
      "longitude": 245.79754047136763,
      "degrees": "05\u00b047'51.14\"",
      "rasi": "Dhanus",
      "nakshatra": "Mula",
      "pada": 2,
      "sign_lord": "Guru",
      "star_lord": "Ketu",
      "sub_lord": "Rahu",
      "sub_sub_lord": "Rahu"
    },
    {
      "graha": "Chandra",
      "longitude": 190.9070075607514,
      "degrees": "10\u00b054'25.22\"",
      "rasi": "Tula",
      "nakshatra": "Swati",
      "pada": 2,
      "sign_lord": "Shukra",
      "star_lord": "Rahu",
      "sub_lord": "Shani",
      "sub_sub_lord": "Budha"
    },
    {
      "graha": "Kuja",
      "longitude": 158.8688519634926,
      "degrees": "08\u00b052'07.86\"",
      "rasi": "Kanya",
      "nakshatra": "Uttara Phalguni",
      "pada": 4,
      "sign_lord": "Budha",
      "star_lord": "Surya",
      "sub_lord": "Shukra",
      "sub_sub_lord": "Rahu"
    }
  ],
  "cusps": [
    {
      "house": 1,
      "longitude": 6.794702084026838,
      "degrees": "06\u00b047'40.92\"",
      "rasi": "Mesha",
      "nakshatra": "Ashwini",
      "pada": 3,
      "sign_lord": "Kuja",
      "star_lord": "Ketu",
      "sub_lord": "Rahu",
      "sub_sub_lord": "Ketu"
    },
    {
      "house": 2,
      "longitude": 37.20707569157011,
      "degrees": "07\u00b012'25.47\"",
      "rasi": "Vrishabha",
      "nakshatra": "Krittika",
      "pada": 4,
      "sign_lord": "Shukra",
      "star_lord": "Surya",
      "sub_lord": "Ketu",
      "sub_sub_lord": "Surya"
    },
    {
      "house": 3,
      "longitude": 64.3269764020761,
      "degrees": "04\u00b019'37.11\"",
      "rasi": "Mithuna",
      "nakshatra": "Mrigashira",
      "pada": 4,
      "sign_lord": "Budha",
      "star_lord": "Kuja",
      "sub_lord": "Shukra",
      "sub_sub_lord": "Shani"
    }
  ],
  "ruling_planets": [
    {
      "role": "lord of the day",
      "graha": "Chandra",
      "sub_lord": null
    },
    {
      "role": "Moon's sign lord",
      "graha": "Shukra",
      "sub_lord": "Shani"
    },
    {
      "role": "Moon's star lord",
      "graha": "Rahu",
      "sub_lord": "Shani"
    },
    {
      "role": "ascendant's sign lord",
      "graha": "Kuja",
      "sub_lord": "Rahu"
    },
    {
      "role": "ascendant's star lord",
      "graha": "Ketu",
      "sub_lord": "Rahu"
    }
  ],
  "significators": [
    {
      "house": 1,
      "significators": [
        {
          "graha": "Shani",
          "rank": 3,
          "because": "in the star of the house lord"
        },
        {
          "graha": "Kuja",
          "rank": 4,
          "because": "lord of the house"
        }
      ]
    },
    {
      "house": 2,
      "significators": [
        {
          "graha": "Shukra",
          "rank": 4,
          "because": "lord of the house"
        }
      ]
    }
  ],
  "variants": [
    {
      "id": "V-13-18",
      "question": "ayanamsa for a KP reading",
      "chosen": "the chart's own"
    },
    {
      "id": "V-13-19",
      "question": "a graha qualifying in two significator groups",
      "chosen": "listed once, in the strongest"
    }
  ]
} as KpResponse;

function stub(body: unknown, status = 200) {
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(new Response(JSON.stringify(body), { status }))));
}

describe("the four lords", () => {
  it("are shown for the ascendant, with the sub emphasised", async () => {
    // The sub is what KP weighs, so it is the column a practitioner reads
    // first and the one this view marks.
    stub(data);
    render(<KpView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "The four lords" }));
    const row = within(table.getByRole("row", { name: /Ascendant/ }));
    expect(row.getByText("Ashwini")).toBeInTheDocument();
    // Rahu is the sub lord, and it is the emphasised cell.
    expect(row.getByText("Rahu").tagName).toBe("STRONG");
  });

  it("explains how a sub is arrived at, rather than just printing it", async () => {
    stub(data);
    render(<KpView birth={birth} />);
    expect(await screen.findByText(/nine subs in Vimshottari order/)).toBeInTheDocument();
    expect(screen.getByText(/years out of 120/)).toBeInTheDocument();
  });
});

describe("the cusps", () => {
  it("say plainly that they differ from the Chart tab", async () => {
    // A practitioner comparing the two tabs would otherwise think one is
    // wrong: KP reads Placidus, the rest of the app reads whole signs.
    stub(data);
    render(<KpView birth={birth} />);
    expect(await screen.findByRole("table", { name: "Placidus cusps" })).toBeInTheDocument();
    expect(screen.getByText(/differ from the Chart tab/)).toBeInTheDocument();
  });
});

describe("the ruling planets", () => {
  it("are listed in the order practice gives, each with its sub", async () => {
    stub(data);
    render(<KpView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Ruling planets" }));
    const rows = table.getAllByRole("row").slice(1);
    expect(rows).toHaveLength(5);
    expect(rows[0]).toHaveTextContent("lord of the day");
    expect(rows[4]).toHaveTextContent("ascendant\'s star lord");
  });
});

describe("the significators", () => {
  it("carry the group number that earned each place, and say what it means", async () => {
    stub(data);
    render(<KpView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "House significators" }));
    expect(table.getAllByRole("row").length).toBeGreaterThan(1);
    // The legend is present, so a reader can decode the ranking.
    expect(screen.getByText(/in the star of an occupant/)).toBeInTheDocument();
    expect(screen.getByText(/the lord itself/)).toBeInTheDocument();
  });
});

describe("the variants", () => {
  it("tell a reader how to make KP read the way KP practice does", async () => {
    stub(data);
    render(<KpView birth={birth} />);
    expect((await screen.findAllByText(/V-13-18/)).length).toBe(2);
    expect(screen.getByRole("note")).toHaveTextContent(/Krishnamurti/);
  });
});

describe("when the engine cannot answer", () => {
  it("says so rather than rendering empty tables", async () => {
    stub({ error: "latitude 91 out of range" }, 400);
    render(<KpView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("latitude 91");
  });
});
