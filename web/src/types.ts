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

export interface TopicMeta { id: string; title: string; summary: string; disclaimer?: string; ages: [number, number] }
export interface BhavaMeaning { house: number; name: string; significations: string[] }
export interface TopicsResponse { topics: TopicMeta[]; bhavas: BhavaMeaning[] }

export interface Pariharam {
  id: string; title: string; triggers: string[]; deity: string | null; day: string | null;
  practices: string[]; places: string[]; charity: string | null;
}
export interface Suggested { pariharam: Pariharam; because: string[] }

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
  pariharams: Suggested[]; explanation: string[];
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
  windows: { rule: string; start: string; end: string; maha: string; antar: string; matched: string[] }[];
  meta?: TopicMeta | null;
  pariharams?: Suggested[];
  writeup?: WriteUp | null;
}

export interface PoruthamResult { kind: string; verdict: "matching" | "not_matching" | "not_evaluated"; critical: boolean; detail: string }
export interface MatchResponse {
  mode: "production" | "review";
  bride: { nakshatra: string; rasi: string };
  groom: { nakshatra: string; rasi: string };
  results: [PoruthamResult, string][];
  withheld: number; reviewers: string[]; matched: number; evaluated: number; critical_failures: string[];
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
  own: { meta: TopicMeta | null; report: TopicResponse["report"]; lean: Lean | null; pariharams: Suggested[]; writeup: WriteUp | null }[];
  agreement: "agree" | "differ" | "inconclusive";
  compared_with: string;
  comparison: string[];
}

export interface WriteUpPoint { rule: string; title: string; text: string; meaning?: string | null; polarity: number; because: string[] }
export type SectionKind = "house" | "karakas" | "varga" | "supporting" | "care" | "noted" | "dosha" | "eased" | "unknown" | "timing" | "karmic_axis" | "bridge" | "life_carried" | "bonds" | "debts";
export interface WriteUpSection { kind: SectionKind; heading: string; paragraphs: string[]; points?: WriteUpPoint[] }
export interface WriteUp { summary: string[]; sections: WriteUpSection[] }
