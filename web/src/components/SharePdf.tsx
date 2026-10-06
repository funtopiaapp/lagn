import { useState } from "react";

import { api, ApiError } from "../api";
import { buildReadingPdf, pdfName, sharePdf, type PdfTopic } from "../lib/pdf";
import type { BirthInput, Sex, TopicMeta, TopicResponse } from "../types";

interface Props {
  birth: BirthInput;
  lagna: string;
  sex?: Sex;
  reviewToken?: string;
  /** The reading already on screen, so sharing it costs no extra request. */
  current?: { meta: TopicMeta; data: TopicResponse };
  /** Every topic in the catalogue, for the whole-reading file. */
  all?: TopicMeta[];
}

/**
 * Share a reading as a PDF.
 *
 * On a phone this opens the OS share sheet, which is the only way a web page
 * can hand a file to WhatsApp or to an email: `mailto:` cannot carry an
 * attachment and WhatsApp's URL scheme takes text only. Desktop browsers
 * mostly cannot share files, so there it saves the file and says so.
 */
export function SharePdf({ birth, lagna, sex, reviewToken, current, all }: Props) {
  const [busy, setBusy] = useState<"" | "one" | "full">("");
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const emit = async (topics: PdfTopic[], title: string) => {
    const blob = await buildReadingPdf({ birth, lagna, topics });
    const how = await sharePdf(blob, pdfName(birth, topics), title);
    setNote(
      how === "shared"
        ? "Handed to your device to share."
        : "Saved to your downloads — attach it from there to send it.",
    );
  };

  const shareOne = async () => {
    if (!current) return;
    setBusy("one"); setError(null); setNote(null);
    try {
      await emit([{ meta: current.meta, data: current.data }], `Lagn — ${current.meta.title}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally { setBusy(""); }
  };

  const shareAll = async () => {
    if (!all || all.length === 0) return;
    setBusy("full"); setError(null); setNote(null);
    try {
      // Each reading is one request. They are fetched in catalogue order so
      // the file reads the same way every time.
      const topics: PdfTopic[] = [];
      for (const meta of all) {
        const data = await api.topic(meta.id, birth, { sex }, reviewToken || undefined);
        if (data.report.results.length > 0) topics.push({ meta, data });
      }
      if (topics.length === 0) {
        setError("No reviewed readings are available for this chart yet.");
        return;
      }
      await emit(topics, "Lagn — full reading");
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally { setBusy(""); }
  };

  return (
    <section className="share-pdf">
      <div className="row">
        {current && (
          <button type="button" onClick={() => void shareOne()} disabled={busy !== ""}>
            {busy === "one" ? "Preparing…" : `Share “${current.meta.title}” as a PDF`}
          </button>
        )}
        {all && all.length > 0 && (
          <button type="button" onClick={() => void shareAll()} disabled={busy !== ""}>
            {busy === "full" ? `Reading all ${all.length} topics…` : "Share the full reading as a PDF"}
          </button>
        )}
      </div>
      {note && <p className="hint" role="status">{note}</p>}
      {error && <p className="error" role="alert">{error}</p>}
      <p className="hint">
        The file is made on this device and is never uploaded. Sharing it is the one thing that
        sends your birth details anywhere, and you choose where.
      </p>
    </section>
  );
}
