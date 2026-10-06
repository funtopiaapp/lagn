// A shared PDF is forwarded, screenshotted and read months later with none of
// the app around it. So what it must contain is not a nicety: every reading's
// disclaimer, the provenance line saying the rules are AI-reviewed, and the
// pariharam framing all have to travel inside the file.

import { describe, expect, it, vi } from "vitest";

import { canShareFiles, pdfName, readingBlocks, sharePdf, type PdfTopic } from "./pdf";
import type { BirthInput } from "../types";

const birth: BirthInput = {
  date: "1981-12-21", time: "14:10:00", latitude: 8.88113, longitude: 76.58469,
  utc_offset_hours: 5.5, place: "Kollam, Kerala",
};

const topic = (over: Partial<PdfTopic["meta"]> = {}, withPariharam = false): PdfTopic => ({
  meta: { id: "health", title: "Health", summary: "Constitution and care.", ...over },
  data: {
    report: {
      topic: over.id ?? "health", mode: "production", results: [], withheld: {},
      supporting: ["a"], afflicting: ["b", "c"], cancelled: [], unknown: [],
      score: -2, label: "afflicted",
    },
    windows: [],
    pariharams: withPariharam
      ? [{
          pariharam: {
            id: "graha.saturn", title: "For Saturn (Shani)", triggers: [], deity: "Shani",
            day: "Saturday", practices: ["Light a sesame-oil lamp"], places: ["Thirunallar"],
            charity: "Black sesame",
          },
          because: ["The 6th lord is in the lagna"],
        }]
      : [],
    writeup: {
      summary: ["On balance, the indications call for care.", over.disclaimer ?? ""].filter(Boolean),
      sections: [{
        kind: "care", heading: "What calls for care",
        paragraphs: ["These classical indications apply:"],
        points: [{
          rule: "health.x", title: "Saturn in the lagna", text: "Saturn sits in the lagna.",
          meaning: "Joints want regular movement.", polarity: -1, because: ["Shani in house 1"],
        }],
      }],
    },
  } as unknown as PdfTopic["data"],
});

const text = (t: PdfTopic[]) =>
  readingBlocks({ birth, lagna: "Mesha 6°45'", topics: t }).map((b) => b.text ?? "").join("\n");

describe("the shared PDF's contents", () => {
  it("carries every reading's own disclaimer", () => {
    const health = topic({ id: "health", title: "Health", disclaimer: "Consult a qualified doctor." });
    const past = topic({ id: "past_life", title: "Past life", disclaimer: "Not a claim about events." });
    const out = text([health, past]);
    expect(out).toContain("Consult a qualified doctor.");
    expect(out).toContain("Not a claim about events.");
    // Once each, not twice: the write-up summary repeats it and that copy is
    // dropped.
    expect(out.split("Consult a qualified doctor.").length - 1).toBe(1);
  });

  it("says the rules are AI-reviewed, because a forwarded file has no other context", () => {
    expect(text([topic()])).toContain("AI-reviewed, not astrologer-reviewed");
  });

  it("keeps the pariharam framing with the pariharams", () => {
    const out = text([topic({}, true)]);
    expect(out).toContain("No gemstones and no paid services");
    expect(out).toContain("never a substitute for medical, legal or financial advice");
    expect(out).toContain("For Saturn (Shani)");
    expect(out).toContain("Light a sesame-oil lamp");
    // And why it was suggested, so the file is as traceable as the app.
    expect(out).toContain("Because: The 6th lord is in the lagna");
  });

  it("states the birth details it was computed from, and the closing caveat", () => {
    const out = text([topic()]);
    expect(out).toContain("1981-12-21 at 14:10, Kollam, Kerala (UTC+5.5)");
    expect(out).toContain("Lagna Mesha 6°45'");
    expect(out).toContain("predicts no event");
  });

  it("keeps each finding's own justification", () => {
    const out = text([topic()]);
    expect(out).toContain("Saturn in the lagna (-1)");
    expect(out).toContain("What this means for you: Joints want regular movement.");
    expect(out).toContain("Because: Shani in house 1");
  });

  it("names the file after the reading and the birth date", () => {
    expect(pdfName(birth, [topic({ title: "Living abroad" })])).toBe("lagn-living-abroad-1981-12-21.pdf");
    // More than one reading is the whole reading, not a list of titles.
    expect(pdfName(birth, [topic(), topic()])).toBe("lagn-reading-1981-12-21.pdf");
  });
});

describe("sharing the file", () => {
  it("falls back to saving when the browser cannot share files", async () => {
    vi.stubGlobal("navigator", { ...navigator, share: undefined, canShare: undefined });
    const click = vi.fn();
    const orig = document.createElement.bind(document);
    vi.spyOn(document, "createElement").mockImplementation((tag: string) => {
      const el = orig(tag) as HTMLAnchorElement;
      if (tag === "a") el.click = click;
      return el;
    });
    vi.stubGlobal("URL", { ...URL, createObjectURL: () => "blob:x", revokeObjectURL: () => {} });

    const how = await sharePdf(new Blob(["x"], { type: "application/pdf" }), "a.pdf", "t");
    expect(how).toBe("saved");
    expect(click).toHaveBeenCalled();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("reports a cancelled share as shared, so it does not also download", async () => {
    const file = new File(["x"], "a.pdf", { type: "application/pdf" });
    vi.stubGlobal("navigator", {
      ...navigator,
      canShare: () => true,
      share: () => Promise.reject(new DOMException("cancelled", "AbortError")),
    });
    expect(canShareFiles(file)).toBe(true);
    const how = await sharePdf(new Blob(["x"]), "a.pdf", "t");
    expect(how).toBe("shared");
    vi.unstubAllGlobals();
  });
});
