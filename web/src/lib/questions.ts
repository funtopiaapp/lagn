// Matching what someone typed to a question the engine can actually answer.
//
// The typing is for finding; the answer always comes from the question they
// chose. Nothing is inferred from free text: a reading derived from a guess at
// what a sentence meant would not be reproducible, and reproducibility is the
// whole claim of this app. It also means no text and no birth detail leaves
// the device, because the matching happens here.

import type { Question } from "../types";

export interface Suggestion {
  question: Question;
  /** Higher is a better match. Only used to order; never shown. */
  score: number;
}

/** Words too common to tell two questions apart. */
const STOP = new Set([
  "is", "it", "a", "an", "the", "to", "for", "of", "my", "me", "i", "this", "that",
  "good", "time", "when", "should", "can", "will", "do", "am", "be", "and", "or",
  "now", "right", "auspicious", "best",
]);

function words(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter((w) => w.length > 1 && !STOP.has(w));
}

/**
 * Questions matching what was typed, best first.
 *
 * Scoring, in order of weight: a keyword phrase appearing whole in the text,
 * then a typed word appearing in a keyword, then a typed word appearing in the
 * question itself. An empty or too-short query matches nothing, so the box
 * stays quiet until there is something to go on.
 */
export function suggest(all: Question[], typed: string, limit = 6): Suggestion[] {
  const q = typed.trim().toLowerCase();
  if (q.length < 2) return [];
  const typedWords = words(q);

  const scored: Suggestion[] = [];
  for (const question of all) {
    let score = 0;
    for (const k of question.keywords) {
      const key = k.toLowerCase();
      // A whole phrase present in the text is the strongest signal: "buy a
      // house" beats a stray "house".
      if (q.includes(key)) score += 10 + key.length;
      for (const w of typedWords) {
        if (key === w) score += 6;
        else if (key.includes(w) || w.includes(key)) score += 2;
      }
    }
    const text = question.question.toLowerCase();
    for (const w of typedWords) if (text.includes(w)) score += 1;
    if (score > 0) scored.push({ question, score });
  }

  // Ties break on the question's own order, which is the catalogue's, so the
  // list is stable for the same input.
  const order = new Map(all.map((x, i) => [x.id, i]));
  scored.sort((a, b) => b.score - a.score || (order.get(a.question.id) ?? 0) - (order.get(b.question.id) ?? 0));
  return scored.slice(0, limit);
}
