// Upagrahas on the professional surface.
//
// The fixture is taken verbatim from /api/upagraha rather than written by
// hand. Hand-writing a fixture is how the Jaimini one drifted from the engine
// once already - it carried properly cased names while the endpoint was still
// emitting raw identifiers, and the tests were happy about it.

import { render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { UpagrahaView } from "./UpagrahaView";
import type { UpagrahaResponse, YogiResponse } from "../types";

const birth = { date: "1981-12-21", time: "14:10:00", latitude: 8.8932, longitude: 76.6141, utc_offset_hours: 5.5, place: "Kollam" };

afterEach(() => vi.unstubAllGlobals());

const data: UpagrahaResponse = {
  "at_night": false,
  "vara": "Monday",
  "vara_lord": "Chandra",
  "hours_since_sunrise": 7.611137330532074,
  "sun_offsets": [
    {
      "name": "Dhuma",
      "degrees": "19\u00b007'51.14\"",
      "longitude": 19.13087380470097,
      "rasi": "Mesha",
      "rasi_tamil": "Mesham",
      "house": 1
    },
    {
      "name": "Vyatipata",
      "degrees": "10\u00b052'08.85\"",
      "longitude": 340.86912619529903,
      "rasi": "Meena",
      "rasi_tamil": "Meenam",
      "house": 12
    },
    {
      "name": "Parivesha",
      "degrees": "10\u00b052'08.85\"",
      "longitude": 160.86912619529903,
      "rasi": "Kanya",
      "rasi_tamil": "Kanni",
      "house": 6
    },
    {
      "name": "Indrachapa",
      "degrees": "19\u00b007'51.14\"",
      "longitude": 199.13087380470097,
      "rasi": "Tula",
      "rasi_tamil": "Thulam",
      "house": 7
    },
    {
      "name": "Upaketu",
      "degrees": "05\u00b047'51.14\"",
      "longitude": 215.79754047136763,
      "rasi": "Vrischika",
      "rasi_tamil": "Viruchigam",
      "house": 8
    }
  ],
  "day_parts": [
    {
      "name": "Kaala",
      "degrees": "24\u00b033'01.78\"",
      "longitude": 24.55049618474494,
      "rasi": "Mesha",
      "rasi_tamil": "Mesham",
      "house": 1,
      "ruler": "Surya",
      "part": 7
    },
    {
      "name": "Mrityu",
      "degrees": "24\u00b059'37.98\"",
      "longitude": 264.9938839789933,
      "rasi": "Dhanus",
      "rasi_tamil": "Dhanusu",
      "house": 9,
      "ruler": "Kuja",
      "part": 2
    },
    {
      "name": "Ardhaprahara",
      "degrees": "16\u00b042'31.01\"",
      "longitude": 286.7086141669722,
      "rasi": "Makara",
      "rasi_tamil": "Magaram",
      "house": 10,
      "ruler": "Budha",
      "part": 3
    },
    {
      "name": "Yamaghantaka",
      "degrees": "10\u00b021'01.08\"",
      "longitude": 310.35030038376783,
      "rasi": "Kumbha",
      "rasi_tamil": "Kumbam",
      "house": 11,
      "ruler": "Guru",
      "part": 4
    },
    {
      "name": "Gulika",
      "degrees": "00\u00b046'41.26\"",
      "longitude": 0.7781288626599832,
      "rasi": "Mesha",
      "rasi_tamil": "Mesham",
      "house": 1,
      "ruler": "Shani",
      "part": 6
    }
  ],
  "time_lagnas": [
    {
      "name": "Bhava lagna",
      "degrees": "29\u00b038'29.73\"",
      "longitude": 359.64159320973397,
      "rasi": "Meena",
      "rasi_tamil": "Meenam",
      "house": 12
    },
    {
      "name": "Hora lagna",
      "degrees": "23\u00b048'31.15\"",
      "longitude": 113.80865316771508,
      "rasi": "Karka",
      "rasi_tamil": "Kadagam",
      "house": 4
    },
    {
      "name": "Ghati lagna",
      "degrees": "06\u00b018'35.39\"",
      "longitude": 96.30983304165841,
      "rasi": "Karka",
      "rasi_tamil": "Kadagam",
      "house": 4
    }
  ],
  "variants": [
    {
      "id": "V-13-14",
      "question": "which moment of its part gives an upagraha",
      "chosen": "the start of the part"
    },
    {
      "id": "V-13-15",
      "question": "night-birth part sequence",
      "chosen": "starts from the lord 5th from the weekday lord"
    }
  ]
} as UpagrahaResponse;

const yogiData = {
  "longitude": 170.03788136545234,
  "degrees": "20\u00b002'16.37\"",
  "rasi": "Kanya",
  "rasi_tamil": "Kanni",
  "nakshatra": "Hasta",
  "pada": 4,
  "yogi": "Chandra",
  "avayogi_nakshatra": "Jyeshtha",
  "avayogi": "Budha",
  "variants": [
    {
      "id": "V-13-38",
      "question": "the Yoga sphuta offset",
      "chosen": "93 degrees 20 minutes, which is seven nakshatras exactly"
    },
    {
      "id": "V-13-39",
      "question": "counting to the Avayogi",
      "chosen": "the 6th nakshatra from the Yogi's, counting the Yogi's as 1"
    },
    {
      "id": "V-13-40",
      "question": "Duplicate Yogi",
      "chosen": "not computed; the accounts disagree and no identity settles it"
    }
  ]
} as YogiResponse;

/// Routed by URL: this view makes two calls now, and answering both with the
/// same body would have the Yogi card render an upagraha response.
function stub(body: unknown, status = 200) {
  vi.stubGlobal("fetch", vi.fn((url: string) =>
    Promise.resolve(
      url.includes("/api/yogi")
        ? new Response(JSON.stringify(yogiData), { status: 200 })
        : new Response(JSON.stringify(body), { status }),
    ),
  ));
}

describe("the three groups", () => {
  it("are kept apart, because each is computed differently", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    expect(await screen.findByRole("table", { name: "Upagrahas from the Sun" })).toBeInTheDocument();
    expect(screen.getByRole("table", { name: "Upagrahas from the day division" })).toBeInTheDocument();
    expect(screen.getByRole("table", { name: "Time lagnas" })).toBeInTheDocument();
  });

  it("says whether the day or the night was divided, and from which lord", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    expect(await screen.findByText(/Monday/)).toBeInTheDocument();
    expect(screen.getByText(/the first part belongs to the lord of the weekday/)).toBeInTheDocument();
  });
});

describe("Gulika", () => {
  it("is shown with its ruler, its part and the name practice uses", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Upagrahas from the day division" }));
    const row = table.getByRole("row", { name: /Gulika/ });
    expect(row).toHaveTextContent("Shani");
    expect(row).toHaveTextContent("Mesha");
    // Mandi is the same point, and a practitioner looking for that name finds it.
    expect(row).toHaveTextContent(/Mandi/);
  });
});

describe("Upaketu", () => {
  it("states the identity that makes the whole chain checkable", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Upagrahas from the Sun" }));
    expect(table.getByRole("row", { name: /Upaketu/ })).toHaveTextContent("Vrischika");
    expect(screen.getByText(/exactly 30° behind the Sun/)).toBeInTheDocument();
  });
});

describe("the degrees", () => {
  it("come formatted from the engine, not rounded in the browser", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Time lagnas" }));
    expect(table.getByRole("row", { name: /Bhava lagna/ })).toHaveTextContent("29°38'29.73\"");
  });
});

describe("the variants", () => {
  it("call out the one that explains differences from other software", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    expect((await screen.findAllByText(/V-13-14/)).length).toBe(2);
    expect(screen.getByRole("note")).toHaveTextContent(/start/);
  });
});

describe("when the engine cannot answer", () => {
  it("says so rather than rendering empty tables", async () => {
    stub({ error: "no sunrise at this latitude on this date" }, 400);
    render(<UpagrahaView birth={birth} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("no sunrise");
  });
});

describe("Yogi and Avayogi", () => {
  it("appear on this tab, labelled as what they are rather than as upagrahas", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    const table = within(await screen.findByRole("table", { name: "Yogi and Avayogi" }));
    // Anchored: "Avayogi" also ends in "Yogi".
    expect(table.getByRole("row", { name: /^Yogi/ })).toHaveTextContent("Chandra");
    expect(table.getByRole("row", { name: /Avayogi/ })).toHaveTextContent("Budha");
    expect(table.getByRole("row", { name: /Yoga sphuta/ })).toHaveTextContent("Hasta");
    // Said plainly: these are derived points, not upagrahas.
    expect(screen.getByText(/Not upagrahas, but derived points/)).toBeInTheDocument();
    // And the identity that makes them checkable is on the page.
    expect(screen.getByText(/exactly seven nakshatras/)).toBeInTheDocument();
  });

  it("says the Duplicate Yogi is not computed", async () => {
    stub(data);
    render(<UpagrahaView birth={birth} />);
    expect(await screen.findByText(/Duplicate Yogi is not computed/)).toBeInTheDocument();
  });
});
