//! The API operations, independent of transport.
//!
//! The HTTP handlers and the native FFI both call these functions, so the two
//! cannot drift apart: an on-device reading is the same computation, and the
//! same JSON, as one served over HTTP. Nothing here knows about axum, sockets
//! or headers. Specification: `docs/phase10/DESIGN.md`.

use std::sync::Arc;

use lagn_core::{DerivationSettings, Ephemeris};
use lagn_rules::corpus::{summary, CorpusSummary};
use lagn_rules::report::MatchReport;
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo};
use serde::{Deserialize, Serialize};

use crate::view::BirthInput;
use crate::{offset, places, view, ApiError, AppState, ATTRIBUTION};

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartRequest {
    pub birth: BirthInput,
    #[serde(default)]
    pub derivation: Option<DerivationSettings>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicRequest {
    pub birth: BirthInput,
    #[serde(default)]
    pub derivation: Option<DerivationSettings>,
    #[serde(default)]
    pub sex: Option<lagn_rules::Sex>,
    /// Defaults to the topic's catalogue ages (18-45 if it has none).
    #[serde(default)]
    pub from_age: Option<f64>,
    #[serde(default)]
    pub to_age: Option<f64>,
    #[serde(default)]
    pub mode: Mode,
}

fn d_from() -> f64 { 0.0 }
fn d_to() -> f64 { 80.0 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodsRequest {
    pub birth: BirthInput,
    #[serde(default)]
    pub derivation: Option<DerivationSettings>,
    #[serde(default)]
    pub sex: Option<lagn_rules::Sex>,
    #[serde(default = "d_from")]
    pub from_age: f64,
    #[serde(default = "d_to")]
    pub to_age: f64,
    #[serde(default)]
    pub mode: Mode,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    pub birth: BirthInput,
    #[serde(default)]
    pub sex: Option<lagn_rules::Sex>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyRequest {
    pub native: Person,
    pub member: Person,
    pub relation: lagn_rules::family::Relation,
    #[serde(default)]
    pub derivation: Option<DerivationSettings>,
    #[serde(default)]
    pub mode: Mode,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRequest {
    pub bride: BirthInput,
    pub groom: BirthInput,
    #[serde(default)]
    pub mode: Mode,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacesQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OffsetQuery {
    pub tz: String,
    pub date: String,
    pub time: String,
    pub longitude: Option<f64>,
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct VersionInfo {
    pub engine: &'static str,
    pub swiss_ephemeris: String,
    pub tzdb: String,
    pub tzdb_considered: Vec<offset::TzCandidate>,
    pub platform: String,
    pub places: usize,
    pub corpus: CorpusSummary,
    pub poruthams_approved: usize,
    pub review_enabled: bool,
    pub attribution: &'static str,
}

#[derive(Serialize)]
pub struct TopicsResponse {
    pub topics: Vec<lagn_rules::catalogue::TopicMeta>,
    pub bhavas: Vec<lagn_rules::catalogue::BhavaMeaning>,
    /// The matters a reader can ask about, each mapped to a topic. Served with
    /// the catalogue so the question box works offline and the matching
    /// happens on the device.
    pub questions: Vec<lagn_rules::catalogue::Question>,
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

pub fn check_ages(from: f64, to: f64) -> Result<(), ApiError> {
    if !(from >= 0.0 && to > from && to <= 120.0) {
        return Err(ApiError::BadRequest("age range must satisfy 0 <= from_age < to_age <= 120".into()));
    }
    Ok(())
}

pub fn version(s: &AppState) -> VersionInfo {
    let poruthams_approved = s
        .corpus
        .porutham
        .as_ref()
        .map(|m| m.values().filter(|r| r.status == lagn_rules::ReviewStatus::Approved).count())
        .unwrap_or(0);
    VersionInfo {
        engine: env!("CARGO_PKG_VERSION"),
        swiss_ephemeris: Ephemeris::library_version(),
        tzdb: s.tzdb.version_label(),
        tzdb_considered: s.tzdb.considered.clone(),
        platform: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        places: s.places.len(),
        corpus: summary(&s.corpus),
        poruthams_approved,
        review_enabled: s.review_token.is_some(),
        attribution: ATTRIBUTION,
    }
}

pub fn places(s: &AppState, q: &PlacesQuery) -> Result<Vec<places::Place>, ApiError> {
    if q.q.chars().count() > 100 {
        return Err(ApiError::BadRequest("query too long".into()));
    }
    Ok(s.places.search(&q.q, q.limit.unwrap_or(10).clamp(1, 50)))
}

pub fn offset(s: &AppState, q: &OffsetQuery) -> Result<offset::Suggestion, ApiError> {
    let date = view::parse_date(&q.date).map_err(ApiError::BadRequest)?;
    let time = view::parse_time(&q.time).map_err(ApiError::BadRequest)?;
    offset::suggest(&s.tzdb, &q.tz, date, time, q.longitude).map_err(ApiError::BadRequest)
}

/// Jaimini core for a chart: chara karakas, arudha padas, argala.
///
/// No review gate and no mode. Like a panchangam, every value here is a
/// computation rather than an interpretation, so there is nothing for a
/// reviewer to approve - but the variant defaults that produced it travel
/// with the response, which is what section 6 of the phase 13 design requires.
pub fn jaimini(req: ChartRequest) -> Result<view::JaiminiView, ApiError> {
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    Ok(view::jaimini_view(&lagn_core::jaimini::Jaimini::compute(&c)))
}

/// Chara dasha for a chart. Like `jaimini`, computation rather than
/// interpretation, so no review gate - and the variant defaults travel with
/// the response (CHARA-DASHA.md section 6).
/// Upagrahas and the time lagnas. Computation, so no review gate; the
/// variant defaults travel with the response.
/// A KP reading. Computation, so no review gate.
pub fn kp(req: ChartRequest) -> Result<view::KpView, ApiError> {
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let eph = lagn_core::Ephemeris::new(c.settings.ayanamsa, c.settings.node_type);
    let kp = lagn_core::kp::Kp::compute(&eph, &c).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok(view::kp_view(&kp))
}

pub fn upagraha(req: ChartRequest) -> Result<view::UpagrahaView, ApiError> {
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let eph = lagn_core::Ephemeris::new(c.settings.ayanamsa, c.settings.node_type);
    let u = lagn_core::upagraha::Upagrahas::compute(&eph, &c)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok(view::upagraha_view(&u))
}

pub fn chara(req: ChartRequest) -> Result<view::CharaView, ApiError> {
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let d = lagn_core::chara::CharaDasha::compute(&c);
    Ok(view::chara_view(&c, &d, Some(view::jd_now())))
}

pub fn chart(req: ChartRequest) -> Result<view::ChartResponse, ApiError> {
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let d = req.derivation.unwrap_or_default();
    Ok(view::chart_response(req.birth, c, &d, Some(view::jd_now())))
}

/// The topic catalogue. Only approved topics that have approved natal rules
/// are listed, so the app never offers a topic that would come back empty.
pub fn topics(s: &AppState) -> TopicsResponse {
    let topics = s
        .corpus
        .topics
        .iter()
        .filter(|t| Mode::Production.admits(t.review.status))
        .filter(|t| {
            s.corpus.rules.iter().any(|r| {
                r.topic == t.id && r.scope == lagn_rules::Scope::Natal && Mode::Production.admits(r.review.status)
            })
        })
        .cloned()
        .collect();
    let bhavas = s
        .corpus
        .bhavas
        .as_ref()
        .filter(|b| Mode::Production.admits(b.review.status))
        .map(|b| b.houses.clone())
        .unwrap_or_default();
    let questions = s
        .corpus
        .questions
        .as_ref()
        .filter(|q| Mode::Production.admits(q.review.status))
        .map(|q| q.questions.clone())
        .unwrap_or_default();
    TopicsResponse { topics, bhavas, questions }
}

pub fn topic(s: &Arc<AppState>, name: &str, req: TopicRequest, mode: Mode) -> Result<view::TopicResponse, ApiError> {
    // Period rules are served by the periods operation, not as a topic.
    if !s.corpus.rules.iter().any(|r| r.topic == name && r.scope == lagn_rules::Scope::Natal) {
        return Err(ApiError::NotFound(format!("no rules for topic {name:?}")));
    }
    let meta = s.corpus.topics.iter().find(|t| t.id == name && mode.admits(t.review.status)).cloned();
    let [dfrom, dto] = meta.as_ref().map(|m| m.ages).unwrap_or([18.0, 45.0]);
    let (from_age, to_age) = (req.from_age.unwrap_or(dfrom), req.to_age.unwrap_or(dto));
    check_ages(from_age, to_age)?;

    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let tz = c.birth.moment.utc_offset_hours;
    // Mutable because judging the windows binds the period lords per window.
    let mut facts = FactBase::new(c, req.derivation.unwrap_or_default(), NativeInfo { sex: req.sex });
    let report = evaluate_topic(name, &s.corpus.rules, &facts, mode, (from_age, to_age));
    // Each timing window judged by the period rules for the same stretch, so
    // the reading says whether a window favours this area and not only that it
    // is listed. Transits are left out on purpose: they cost close to a second
    // over a wide age range, and the periods operation already carries them.
    let title = meta.as_ref().map(|m| m.title.clone()).unwrap_or_else(|| name.to_string());
    let judged = lagn_rules::reading::topic_timing(
        &s.corpus, &mut facts, mode, &title, &report.timing, (from_age, to_age),
    );
    let writeup = meta.as_ref().map(|m| {
        lagn_rules::writeup::write_up(&s.corpus, m, &m.focus, &report, &facts, (from_age, to_age), |_| true)
    });
    Ok(view::TopicResponse { meta, writeup, ..view::topic_response(report, tz, &judged, Some(crate::view::jd_now())) })
}

/// Sensitive periods: every antardasha in the age range with its period rules,
/// focus houses and overlapping transits.
pub fn periods(s: &Arc<AppState>, req: PeriodsRequest, mode: Mode) -> Result<view::PeriodsResponse, ApiError> {
    check_ages(req.from_age, req.to_age)?;
    let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
    let tz = c.birth.moment.utc_offset_hours;
    let t = lagn_rules::reading::transits(&c, req.from_age, req.to_age)
        .map_err(|e| ApiError::Internal(format!("transits: {e}")))?;
    let mut facts = FactBase::new(c, req.derivation.unwrap_or_default(), NativeInfo { sex: req.sex });
    let r = lagn_rules::reading::sensitive_periods(&s.corpus, &mut facts, mode, (req.from_age, req.to_age), &t.windows);
    Ok(view::periods_response(r, &t, tz, view::jd_now()))
}

/// A family member's own chart beside the native's relational reading. Both
/// birth records arrive with the request; nothing is stored.
pub fn family(s: &Arc<AppState>, req: FamilyRequest, mode: Mode) -> Result<lagn_rules::family::FamilyReading, ApiError> {
    let d = req.derivation.unwrap_or_default();
    let n = req.native.birth.to_chart().map_err(|e| ApiError::BadRequest(format!("native: {e}")))?;
    let m = req.member.birth.to_chart().map_err(|e| ApiError::BadRequest(format!("member: {e}")))?;
    let native = FactBase::new(n, d, NativeInfo { sex: req.native.sex });
    let member = FactBase::new(m, d, NativeInfo { sex: req.member.sex });
    Ok(lagn_rules::family::family_reading(&s.corpus, &native, &member, req.relation, mode))
}

pub fn match_(s: &Arc<AppState>, req: MatchRequest, mode: Mode) -> Result<MatchReport, ApiError> {
    let b = req.bride.to_chart().map_err(|e| ApiError::BadRequest(format!("bride: {e}")))?;
    let g = req.groom.to_chart().map_err(|e| ApiError::BadRequest(format!("groom: {e}")))?;
    // The whole report: the poruthams plus papasamyam, dasa sandhi and
    // Chevvai dosha on both sides, which need the charts and not just the
    // two stars.
    Ok(lagn_rules::report::match_report_full(&s.corpus, &b, &g, mode))
}

// ---------------------------------------------------------------------------
// A day's own timings
// ---------------------------------------------------------------------------

/// A run of days at one place. No birth chart: these timings belong to the day
/// and the place, not to a person.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DayRequest {
    /// First date, YYYY-MM-DD. Omit it to start from today: the frontend does
    /// no calendar arithmetic (DESIGN.md section 1, rule 2), so a calendar
    /// walks weeks by sending an offset and letting the engine do the sums.
    #[serde(default)]
    pub date: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub utc_offset_hours: f64,
    /// How many days from the start. Defaults to one.
    #[serde(default)]
    pub days: Option<u32>,
    /// Days to shift the start by, forwards or back. This is what a calendar
    /// sends to move between weeks.
    #[serde(default)]
    pub offset_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaySegmentView {
    pub start: String,
    pub end: String,
    pub part: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayView {
    pub date: String,
    pub vara: &'static str,
    pub vara_tamil: &'static str,
    pub sunrise: String,
    pub sunset: String,
    pub tithi: u8,
    pub tithi_name: &'static str,
    pub paksha: &'static str,
    pub nakshatra: &'static str,
    pub nakshatra_tamil: &'static str,
    pub pada: u8,
    pub yoga: &'static str,
    pub karana: &'static str,
    pub rahu_kalam: DaySegmentView,
    pub yamagandam: DaySegmentView,
    pub kuligai: DaySegmentView,
    pub abhijit: DaySegmentView,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaysResponse {
    pub days: Vec<DayView>,
}

/// At most this many days in one request: a calendar shows a week or a month,
/// and each day costs two rise/set searches.
const MAX_DAYS: u32 = 62;

pub fn days(req: DayRequest) -> Result<DaysResponse, ApiError> {
    let n = req.days.unwrap_or(1);
    if !(1..=MAX_DAYS).contains(&n) {
        return Err(ApiError::BadRequest(format!("days must be 1..={MAX_DAYS}")));
    }
    if !req.utc_offset_hours.is_finite() || req.utc_offset_hours.abs() > 14.0 {
        return Err(ApiError::BadRequest("utc_offset_hours must be within +/-14".into()));
    }

    let offset = req.offset_days.unwrap_or(0);
    if !(-40_000..=40_000).contains(&offset) {
        return Err(ApiError::BadRequest("offset_days is out of range".into()));
    }

    let tz = req.utc_offset_hours;
    let eph = lagn_core::Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
    // Local midnight of the starting date, in UT.
    let start_midnight = match &req.date {
        Some(text) => {
            let p: Vec<&str> = text.split('-').collect();
            if p.len() != 3 {
                return Err(ApiError::BadRequest("date must be YYYY-MM-DD".into()));
            }
            let (y, m, d): (i32, u32, u32) = (
                p[0].parse().map_err(|_| ApiError::BadRequest("bad year".into()))?,
                p[1].parse().map_err(|_| ApiError::BadRequest("bad month".into()))?,
                p[2].parse().map_err(|_| ApiError::BadRequest("bad day".into()))?,
            );
            lagn_core::validate_civil_date(y, m, d).map_err(|e| ApiError::BadRequest(e.to_string()))?;
            lagn_core::julian_day_ut(y, m, d, 0.0, lagn_core::Calendar::Gregorian)
                .map_err(|e| ApiError::BadRequest(e.to_string()))?
        }
        // Today at the place: truncate the local date, then take its midnight.
        None => {
            let now = crate::view::jd_now();
            let c = lagn_core::jd_to_civil(now, tz);
            lagn_core::julian_day_ut(c.year, c.month, c.day, 0.0, lagn_core::Calendar::Gregorian)
                .map_err(|e| ApiError::BadRequest(e.to_string()))?
        }
    };
    let midnight = start_midnight - tz / 24.0 + offset as f64;

    let hm = |jd: f64| {
        let c = lagn_core::jd_to_civil(jd, tz);
        format!("{:02}:{:02}", c.hour, c.minute)
    };
    let seg = |s: &lagn_core::DaySegment| DaySegmentView {
        start: hm(s.start_jd),
        end: hm(s.end_jd),
        part: s.part,
    };

    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        // Inside the polar circles a day can have no sunrise. That is the
        // truth about the day, not a failure, so the run stops there rather
        // than failing the whole request.
        let sun = match eph.sun_day(midnight + i as f64, req.latitude, req.longitude) {
            Ok(s) => s,
            Err(_) if i > 0 => break,
            Err(e) => return Err(ApiError::BadRequest(e.to_string())),
        };
        let t = lagn_core::day_timings(sun.sunrise_jd, sun.sunset_jd);
        let s = eph.position(sun.sunrise_jd, lagn_core::Graha::Sun)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let mo = eph.position(sun.sunrise_jd, lagn_core::Graha::Moon)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let pan = lagn_core::panchanga(s.longitude, mo.longitude, sun.sunrise_jd);
        let c = lagn_core::jd_to_civil(sun.sunrise_jd, tz);
        out.push(DayView {
            date: format!("{:04}-{:02}-{:02}", c.year, c.month, c.day),
            vara: t.vara.english(),
            vara_tamil: t.vara.tamil_name(),
            sunrise: hm(sun.sunrise_jd),
            sunset: hm(sun.sunset_jd),
            tithi: pan.tithi,
            tithi_name: lagn_core::day::tithi_name(pan.tithi),
            paksha: lagn_core::day::paksha(pan.tithi),
            nakshatra: pan.nakshatra.nakshatra.name(),
            nakshatra_tamil: pan.nakshatra.nakshatra.tamil_name(),
            pada: pan.nakshatra.pada,
            yoga: lagn_core::day::yoga_name(pan.yoga),
            karana: lagn_core::day::karana_name(pan.karana),
            rahu_kalam: seg(&t.rahu_kalam),
            yamagandam: seg(&t.yamagandam),
            kuligai: seg(&t.kuligai),
            abhijit: seg(&t.abhijit),
        });
    }
    Ok(DaysResponse { days: out })
}
