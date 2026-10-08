// A reading as a PDF, laid out from the structured reading rather than from a
// screenshot of it. A rasterised page loses selectable text, searchable text
// and accessibility, makes a far larger file, and would need blob: image URLs
// that this app's Content-Security-Policy forbids.
//
// Everything is generated on the device. The file is never uploaded: sharing
// it is the one action that deliberately sends birth details anywhere, and the
// person does that themselves through the share sheet.

import type { jsPDF } from "jspdf";

import type { BirthInput, TopicMeta, TopicResponse } from "../types";

/** A4 in points, with margins that leave a comfortable measure for prose. */
const PAGE = { w: 595.28, h: 841.89, margin: 56 };
const BODY = 10.5;
const LEAD = 14.5;

/** One reading, as the app already has it in hand. */
export interface PdfTopic {
  meta: Pick<TopicMeta, "id" | "title" | "summary" | "disclaimer">;
  data: TopicResponse;
}

export interface PdfInput {
  birth: BirthInput;
  lagna: string;
  /** Readings to include, in the order they should appear. */
  topics: PdfTopic[];
  /** The engine's own version line, so a forwarded file says what made it. */
  version?: string;
}

// jsPDF and the optional dependencies it reaches for are around 450 kB. Almost
// nobody opening the app shares a PDF, so it is fetched when they do rather
// than carried by every first visit.
async function loadJsPdf(): Promise<typeof jsPDF> {
  const m = await import("jspdf");
  return m.jsPDF;
}

class Writer {
  private doc: jsPDF;
  private y: number;

  constructor(ctor: typeof jsPDF) {
    this.doc = new ctor({ unit: "pt", format: "a4" });
    this.y = PAGE.margin;
  }

  private room(need: number) {
    if (this.y + need > PAGE.h - PAGE.margin - 24) {
      this.doc.addPage();
      this.y = PAGE.margin;
    }
  }

  heading(text: string, size = 15) {
    this.room(size + 14);
    this.y += 8;
    this.doc.setFont("helvetica", "bold").setFontSize(size).setTextColor(20);
    this.doc.text(text, PAGE.margin, this.y);
    this.y += size + 4;
  }

  para(text: string, opts: { bold?: boolean; grey?: boolean; size?: number; indent?: number } = {}) {
    const size = opts.size ?? BODY;
    const indent = opts.indent ?? 0;
    const width = PAGE.w - PAGE.margin * 2 - indent;
    this.doc
      .setFont("helvetica", opts.bold ? "bold" : "normal")
      .setFontSize(size)
      .setTextColor(opts.grey ? 110 : 30);
    for (const line of this.doc.splitTextToSize(text, width) as string[]) {
      this.room(LEAD);
      this.doc.text(line, PAGE.margin + indent, this.y);
      this.y += LEAD;
    }
    this.y += 3;
  }

  rule() {
    this.room(12);
    this.doc.setDrawColor(210).line(PAGE.margin, this.y, PAGE.w - PAGE.margin, this.y);
    this.y += 10;
  }

  /** Page numbers last, once the page count is known. */
  finish(footer: string): Blob {
    const pages = this.doc.getNumberOfPages();
    for (let i = 1; i <= pages; i++) {
      this.doc.setPage(i);
      this.doc.setFont("helvetica", "normal").setFontSize(8).setTextColor(130);
      this.doc.text(footer, PAGE.margin, PAGE.h - 30);
      this.doc.text(`${i} of ${pages}`, PAGE.w - PAGE.margin, PAGE.h - 30, { align: "right" });
    }
    return this.doc.output("blob");
  }
}

/** A short, filesystem-safe name for the file. */
export function pdfName(birth: BirthInput, topics: PdfTopic[]): string {
  const what = topics.length === 1 ? (topics[0]?.meta.title ?? "reading") : "reading";
  const slug = what.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
  return `lagn-${slug}-${birth.date}.pdf`;
}

/** One piece of the document, before anything is drawn. */
export interface Block {
  kind: "h1" | "h2" | "h3" | "p" | "rule";
  text?: string;
  bold?: boolean;
  grey?: boolean;
  size?: number;
  indent?: number;
}

/**
 * The document's content, in order, as plain blocks.
 *
 * Separate from the drawing so that what a shared file must contain - every
 * disclaimer and the provenance line - can be asserted
 * without a PDF parser.
 */
export function readingBlocks(input: PdfInput): Block[] {
  const { birth, topics } = input;
  const out: Block[] = [];
  const p = (text: string, o: Omit<Block, "kind" | "text"> = {}) => out.push({ kind: "p", text, ...o });

  out.push({ kind: "h1", text: "Lagn — a reading" });
  p(
    `${birth.date} at ${birth.time.slice(0, 5)}, ${birth.place || `${birth.latitude.toFixed(4)}, ${birth.longitude.toFixed(4)}`}` +
      ` (UTC${birth.utc_offset_hours >= 0 ? "+" : ""}${birth.utc_offset_hours}). Lagna ${input.lagna}.`,
  );
  p(
    "Vedic astrology in the South Indian tradition, computed deterministically: the same birth " +
      "details always give the same reading. Lahiri ayanamsha, whole-sign houses, Vimshottari dasha.",
    { grey: true },
  );
  p(
    "The interpretive rules behind this reading are AI-reviewed, not astrologer-reviewed. Every " +
      "finding below names the classical principle it rests on so it can be checked.",
    { grey: true },
  );
  out.push({ kind: "rule" });

  for (const t of topics) {
    const rep = t.data.report;
    out.push({ kind: "h2", text: t.meta.title });
    if (t.meta.summary) p(t.meta.summary, { grey: true });
    const sign = rep.score > 0 ? "+" : "";
    p(
      `Overall: ${rep.label}${rep.score !== 0 ? ` (score ${sign}${rep.score})` : ""}. ` +
        `${rep.supporting.length} supporting and ${rep.afflicting.length} afflicting factors apply.`,
      { bold: true },
    );
    // The disclaimer, before anything it qualifies.
    if (t.meta.disclaimer) p(t.meta.disclaimer, { grey: true });

    const writeup = t.data.writeup;
    if (writeup) {
      for (const q of writeup.summary) if (q !== t.meta.disclaimer) p(q);
      for (const sec of writeup.sections) {
        out.push({ kind: "h3", text: sec.heading });
        for (const q of sec.paragraphs) p(q);
        for (const pt of sec.points ?? []) {
          p(`${pt.title} (${pt.polarity > 0 ? "+" : ""}${pt.polarity})`, { bold: true, indent: 14 });
          p(pt.text, { indent: 14 });
          if (pt.meaning) p(`What this means for you: ${pt.meaning}`, { indent: 14 });
          if (pt.because.length > 0) p(`Because: ${pt.because.join("; ")}`, { grey: true, indent: 14 });
        }
      }
    }

    out.push({ kind: "rule" });
  }

  p(
    "This reading describes tendencies the tradition reads in a chart. It is not medical, legal " +
      "or financial advice, and it predicts no event.",
    { grey: true },
  );
  if (input.version) p(input.version, { grey: true, size: 8 });
  return out;
}

/**
 * Build the document.
 *
 * Every disclaimer travels inside the file. A PDF is forwarded, screenshotted
 * and read months later with none of the app around it, so a reading stripped
 * of its disclaimer would be worse than the app - and the provenance line
 * saying the rules are AI-reviewed has to go with it for the same reason.
 */
export async function buildReadingPdf(input: PdfInput): Promise<Blob> {
  const w = new Writer(await loadJsPdf());
  for (const b of readingBlocks(input)) {
    switch (b.kind) {
      case "h1": w.heading(b.text ?? "", 20); break;
      case "h2": w.heading(b.text ?? "", 15); break;
      case "h3": w.heading(b.text ?? "", 12); break;
      case "rule": w.rule(); break;
      default: w.para(b.text ?? "", b);
    }
  }
  return w.finish(`Lagn · ${input.birth.date} · generated on this device, not uploaded`);
}

/** True when this browser can hand a file to the OS share sheet. */
export function canShareFiles(file: File): boolean {
  try {
    return typeof navigator.share === "function"
      && typeof navigator.canShare === "function"
      && navigator.canShare({ files: [file] });
  } catch {
    return false;
  }
}

/**
 * Share the file, or fall back to saving it.
 *
 * The share sheet is the only route by which a web page can put a file into
 * WhatsApp or an email: `mailto:` cannot carry an attachment and WhatsApp's
 * URL scheme takes text only. Desktop browsers mostly cannot share files, so
 * there the honest behaviour is to save it and let the person attach it.
 */
export async function sharePdf(blob: Blob, name: string, title: string): Promise<"shared" | "saved"> {
  const file = new File([blob], name, { type: "application/pdf" });
  if (canShareFiles(file)) {
    try {
      await navigator.share({ files: [file], title });
      return "shared";
    } catch (e) {
      // A cancelled share is not a failure and must not become a download.
      if (e instanceof DOMException && e.name === "AbortError") return "shared";
    }
  }
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.rel = "noopener";
  document.body.append(a);
  a.click();
  a.remove();
  // Revoking immediately can cancel the download in some browsers.
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
  return "saved";
}
