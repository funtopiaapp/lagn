// Shapes returned by lagn-server. Only the fields the UI reads are declared.
// Every value here was computed by the verified kernel; the UI displays it
// and does no astrology or calendar arithmetic of its own.

export type Sex = "female" | "male";

export interface BirthInput {
  date: string;
  time: string;
  latitude: number;
  longitude: number;
  utc_offset_hours: number;
  place: string;
}

export interface Place {
  id: number;
  name: string;
  admin1: string;
  country: string;
  latitude: number;
  longitude: number;
  timezone: string;
  population: number;
  matched_alias?: string;
}

export interface OffsetCandidate { offset_hours: number; offset_text: string; label: string }
export interface OffsetSuggestion {
  timezone: string;
  kind: "unique" | "ambiguous" | "nonexistent" | "local_mean_time";
  confidence: "reliable" | "historical";
  candidates: OffsetCandidate[];
  notes: string[];
  tzdb_version: string;
}

export interface SignView { index: number; name: string; tamil: string }
export interface Position {
  key: string; name: string; abbrev: string;
  rasi: string; rasi_tamil: string; degrees: string;
  nakshatra: string; nakshatra_tamil: string; pada: number;
  house: number; retrograde: boolean; dignity: string; combust: boolean;
  baladi: string; jagradadi: string;
}
export interface VargaCell { key: string; abbrev: string; sign: string; sign_index: number; house: number; dignity: string }
/** How long the lagna stays in its rasi either side of the birth moment, in
 *  minutes. `margin_minutes` is the nearer edge: how wrong the birth time can
 *  be before every house shifts. `capped` means no boundary was found within
 *  twelve hours, which happens only at extreme latitudes. */
export interface LagnaHold { minutes_in: number; minutes_left: number; margin_minutes: number; capped: boolean }

export interface VargaView { varga: string; label: string; lagna: string; lagna_index: number; grahas: VargaCell[] }
export interface Period {
  lord: string; start: string; end: string; from_birth: boolean; current: boolean;
  children?: Period[];
}
export interface DashaView {
  janma_nakshatra: string; janma_nakshatra_tamil: string; birth_lord: string;
  balance_until: string; year_length_days: number; mahadashas: Period[];
  running_now: { as_of_utc: string; maha: string; antar: string; pratyantar: string } | null;
}
export interface AshtakavargaView {
  rows: { graha: string; bindus: number[]; total: number }[];
  sav: number[]; sav_by_house: number[]; sav_total: number;
}
export interface ChartResponse {
  input: BirthInput;
  signs: SignView[];
  lagna_index: number;
  julian_day_ut: number;
  ayanamsa: string;
  lagna: { rasi: string; rasi_tamil: string; degrees: string; nakshatra: string; nakshatra_tamil: string; pada: number; holds_for?: LagnaHold };
  positions: Position[];
  vargas: VargaView[];
  dasha: DashaView;
  ashtakavarga: AshtakavargaView;
}

export type Truth = "true" | "false" | "unknown";
export interface TraceEntry { condition: string; value: Truth; facts: string }
export interface RuleResult {
  id: string; title: string; status: "draft" | "approved" | "rejected"; reviewer: string | null;
  polarity: number; outcome: Truth; effective: boolean; cancelled_by: string | null;
  text: string; impact?: string | null; trace: TraceEntry[]; tags?: string[]; subject?: string;
}

/** A matter a reader can ask about, mapped to the reading that answers it.
 *  `keywords` are match terms only and are never shown. */
export interface Question { id: string; question: string; topic: string; keywords: string[] }

export interface TopicMeta { id: string; title: string; summary: string; disclaimer?: string; ages: [number, number] }
export interface BhavaMeaning { house: number; name: string; significations: string[] }
export interface TopicsResponse { questions: Question[]; topics: TopicMeta[]; bhavas: BhavaMeaning[] }

export interface FocusHouse { house: number; name: string; significations: string[]; via: ("occupies" | "rules" | "dispositor")[] }
export interface TransitNote { label: string; start_jd: number; end_jd: number }
export interface TransitView {
  start: string; end: string; label: string; graha: string; sign: string; house_from_moon: number;
  supportive: boolean; bindus: number | null; bav_supports: boolean | null;
}
export interface PeriodWindow {
  start: string; end: string; maha_name: string; antar_name: string; current: boolean;
  score: number; sensitive: boolean;
  amplifiers: RuleResult[]; negators: RuleResult[]; noted: RuleResult[]; cancelled: [string, string][];
  focus: FocusHouse[]; pressures: TransitNote[]; supports: TransitNote[];
  explanation: string[];
}
export interface PeriodsResponse {
  mode: "production" | "review"; withheld: Record<string, number>; moon_sign: string;
  windows: PeriodWindow[]; transits: TransitView[];
}

export interface TopicResponse {
  report: {
    topic: string; mode: "production" | "review"; results: RuleResult[];
    withheld: Record<string, number>; supporting: string[]; afflicting: string[];
    cancelled: [string, string][]; unknown: string[]; score: number;
    label: "supportive" | "afflicted" | "mixed" | "neutral";
  };
  windows: TimingWindow[];
  meta?: TopicMeta | null;
  writeup?: WriteUp | null;
}

export interface PoruthamResult { kind: string; verdict: "matching" | "not_matching" | "not_evaluated"; critical: boolean; detail: string }
/** One porutham with what it measures and what its verdict is read to mean. */
export interface PoruthamExplained {
  kind: string; name: string; verdict: "matching" | "not_matching" | "not_evaluated";
  critical: boolean; detail: string; measures: string; means: string; touches: string;
}
/** Papasamyam: the papa count from one reference point. */
export interface PapaFrom { from: string; count: number; placements: [string, number][] }
export interface Papa { from: PapaFrom[]; total: number }
export interface PapaComparison { bride: Papa; groom: Papa; balanced: boolean; difference: number }
/** One coincidence of mahadasha changes between the two charts. */
export interface DasaSandhi {
  bride_jd: number; bride_from: string; bride_to: string;
  groom_jd: number; groom_from: string; groom_to: string; months_apart: number;
}
/** Chevvai dosha on both sides, which is how a report states it. */
export interface ChevvaiBoth {
  bride: boolean; groom: boolean; bride_counts: string[]; groom_counts: string[]; mutual: boolean;
}

/** The results gathered into the areas of married life they speak to. */
export interface MatchArea { touches: string; matching: string[]; not_matching: string[]; critical: boolean }
export interface MatchResponse {
  mode: "production" | "review";
  bride: { nakshatra: string; rasi: string };
  groom: { nakshatra: string; rasi: string };
  results: [PoruthamResult, string][];
  withheld: number; reviewers: string[]; matched: number; evaluated: number; critical_failures: string[];
  /** What each porutham measures and what its verdict means. Empty when the
   *  explainer is not reviewed. */
  explained: PoruthamExplained[];
  areas: MatchArea[];
  /** How to read the whole match, in the order a reader needs it. */
  reading: string[];
  papa?: PapaComparison;
  dasa_sandhi?: DasaSandhi[];
  chevvai?: ChevvaiBoth;
}

export interface VersionInfo {
  engine: string; swiss_ephemeris: string; tzdb: string; platform: string;
  places: number; corpus: { rules: number; by_status: Record<string, number> };
  poruthams_approved: number; review_enabled: boolean; attribution: string;
}

export type Lean = "favourable" | "balanced" | "calls_for_care";
export interface FamilyReading {
  relation: "spouse" | "child" | "mother" | "father";
  relational: { topic: string; house: number; supporting: RuleResult[]; afflicting: RuleResult[]; score: number; lean: Lean | null; writeup: WriteUp | null };
  own: { meta: TopicMeta | null; report: TopicResponse["report"]; lean: Lean | null; writeup: WriteUp | null }[];
  agreement: "agree" | "differ" | "inconclusive";
  compared_with: string;
  comparison: string[];
}

/** One of a topic's timing windows, with how the period rules judge the
 *  stretch for that topic. `supports` and `cautions` name the area, so a line
 *  never says "supports" without saying what it supports. */
export type TimingVerdict = "favourable" | "mixed" | "asks for care" | "no period factor applies";
/** Ranking across every stretch in range, best first. */
export type TimingRank = "best" | "good" | "mixed" | "caution" | "not judged";
export interface TimingWindow {
  rule: string; start: string; end: string; maha: string; antar: string; matched: string[];
  verdict: TimingVerdict; score: number; supports: string[]; cautions: string[];
  /** True when the topic's own timing rules single out this stretch. */
  live: boolean;
  rank: TimingRank;
  /** Against the server's clock. The frontend does no calendar arithmetic, so
   *  it cannot work this out for itself. */
  when: "past" | "now" | "ahead" | "unknown";
}

/** One named part of a day. `part` is which eighth (or which fifteenth, for
 *  Abhijit) of the daylight it occupies. */
export interface DaySegment { start: string; end: string; part: number }
/** A day's panchanga and the parts of it tradition marks out. */
export interface DayView {
  date: string; vara: string; vara_tamil: string; sunrise: string; sunset: string;
  tithi: number; tithi_name: string; paksha: string;
  nakshatra: string; nakshatra_tamil: string; pada: number; yoga: string; karana: string;
  rahu_kalam: DaySegment; yamagandam: DaySegment; kuligai: DaySegment; abhijit: DaySegment;
}
export interface DaysResponse { days: DayView[] }

export interface WriteUpPoint { rule: string; title: string; text: string; meaning?: string | null; polarity: number; because: string[] }
export type SectionKind = "house" | "karakas" | "varga" | "supporting" | "care" | "noted" | "dosha" | "eased" | "unknown" | "timing" | "karmic_axis" | "bridge" | "life_carried" | "bonds" | "debts";
export interface WriteUpSection { kind: SectionKind; heading: string; paragraphs: string[]; points?: WriteUpPoint[] }
export interface WriteUp { summary: string[]; sections: WriteUpSection[] }

// --- Jaimini (phase 13A, professional surface) --------------------------
// Specification: docs/phase13/DESIGN.md sections 4, 5 and 7.

/** Stable karaka id, for keying. The labels come from the engine. */
export type Karaka =
  | "atma" | "amatya" | "bhratri" | "matri" | "pitri" | "putra" | "gnati" | "dara";
export interface KarakaAssignment {
  id: Karaka;
  abbrev: string; name: string; signifies: string;
  graha: string; rasi: string;
  /** Degrees travelled through the sign. Reversed for Rahu (V-13-2). */
  advancement: number;
}
export interface ArudhaPada {
  bhava: number;
  /** "AL", "A7", "UL" - what practice calls it. */
  label: string;
  bhava_rasi: string; lord: string; lord_rasi: string;
  count: number;
  /** Where the count lands before the exception is applied. */
  raw: string;
  rasi: string;
  /** True when the pada would have fallen on its own bhava or the 7th from
   *  it, and the 10th was taken instead (V-13-8). */
  adjusted: boolean;
}
export type ArgalaVerdict = "stands" | "neutralised" | "overcome" | "none";
export interface ArgalaPair {
  kind: string; argala_house: number; counter_house: number;
  argala_rasi: string; counter_rasi: string;
  argala_grahas: string[]; counter_grahas: string[]; verdict: ArgalaVerdict;
}
export interface Argala { rasi: string; pairs: ArgalaPair[] }
/** A variant choice in force, named so a practitioner knows the scheme. */
export interface VariantChoice { id: string; question: string; chosen: string }
export interface JaiminiResponse {
  karakas: KarakaAssignment[];
  padas: ArudhaPada[];
  argala: Argala[];
  variants: VariantChoice[];
}

// --- Chara dasha (phase 13B, professional surface) ---------------------
// Specification: docs/phase13/CHARA-DASHA.md.
//
// Dates arrive already formatted. The engine works in Julian Days and the
// server turns them into dates, because the browser does no calendar
// arithmetic - a professional surface is no excuse to start.

export type CharaDirection = "direct" | "reverse";
export interface CharaLength {
  rasi: string; lord: string; lord_rasi: string;
  /** Which way the count ran: this sign's own parity (V-13-10). */
  direction: CharaDirection;
  count: number; years: number;
  /** True when the lord is at home, so the period is 12 rather than 0. */
  lord_at_home: boolean;
}
export interface CharaPeriod {
  rasi: string; start: string; end: string; cycle: number;
  start_jd: number; end_jd: number;
  children?: CharaPeriod[];
}
export interface CharaRunning { as_of_utc: string; maha: string; antar: string }
export interface CharaResponse {
  lagna: string;
  /** Why the sequence runs the way it does (V-13-9). */
  lagna_is_odd: boolean;
  direction: CharaDirection;
  cycle_years: number;
  year_length_days: number;
  lengths: CharaLength[];
  periods: CharaPeriod[];
  running_now: CharaRunning | null;
  variants: VariantChoice[];
}

// --- Upagrahas and time lagnas (phase 13C, professional surface) --------
// Specification: docs/phase13/UPAGRAHA.md.

export interface UpaPoint {
  name: string;
  /** Degrees, minutes and seconds within the sign, formatted by the engine. */
  degrees: string;
  longitude: number;
  rasi: string; rasi_tamil: string; house: number;
}
export interface UpaDayPart extends UpaPoint {
  ruler: string;
  /** Which eighth of the day or night, 1 to 7. */
  part: number;
}
export interface UpagrahaResponse {
  at_night: boolean;
  vara: string; vara_lord: string;
  hours_since_sunrise: number;
  sun_offsets: UpaPoint[];
  day_parts: UpaDayPart[];
  time_lagnas: UpaPoint[];
  variants: VariantChoice[];
}

// --- Krishnamurti Paddhati (phase 13H, professional surface) ------------
// Specification: docs/phase13/KP.md.

export interface KpLords {
  longitude: number;
  /** Degrees, minutes and seconds within the sign, formatted by the engine. */
  degrees: string;
  rasi: string; nakshatra: string; pada: number;
  sign_lord: string; star_lord: string; sub_lord: string; sub_sub_lord: string;
}
export interface KpGraha extends KpLords { graha: string }
export interface KpCusp extends KpLords { house: number }
export interface KpRulingPlanet { role: string; graha: string; sub_lord: string | null }
export interface KpSignificator {
  graha: string;
  /** 1 to 4, strongest first. */
  rank: number;
  because: string;
}
export interface KpHouseSignificators { house: number; significators: KpSignificator[] }
export interface KpResponse {
  ascendant: KpLords;
  grahas: KpGraha[];
  cusps: KpCusp[];
  ruling_planets: KpRulingPlanet[];
  significators: KpHouseSignificators[];
  variants: VariantChoice[];
}

// --- The annual chart, Muntha and kaksha (phase 13F/13G) ---------------
// Specification: docs/phase13/VARSHA-KAKSHA.md.

export interface VarshaPosition {
  graha: string; rasi: string; rasi_tamil: string;
  degrees: string; house: number; retrograde: boolean;
}
export interface KakshaReading {
  graha: string; rasi: string; degrees: string;
  /** 1 to 8, from the start of the sign. */
  kaksha: number;
  owner: string;
  /** True when the kaksha's owner gave a bindu there in the natal chart. */
  supported: boolean;
  bindus: number;
}
export interface VarshaResponse {
  age: number;
  begins: string;
  /** How far the annual Sun is from the natal Sun, in degrees. */
  sun_error: number;
  lagna: string; muntha: string; muntha_house: number;
  positions: VarshaPosition[];
  kaksha: KakshaReading[];
  variants: VariantChoice[];
}
