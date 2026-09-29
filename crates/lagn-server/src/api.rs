//! The API operations, independent of transport.
//!
//! The HTTP handlers and the native FFI both call these functions, so the two
//! cannot drift apart: an on-device reading is the same computation, and the
//! same JSON, as one served over HTTP. Nothing here knows about axum, sockets
//! or headers. Specification: `docs/phase10/DESIGN.md`.

use std::sync::Arc;

use lagn_core::{DerivationSettings, Ephemeris};
use lagn_rules::corpus::{summary, CorpusSummary};
use lagn_rules::report::{match_report, MatchReport};
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo, StarPos};
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
    TopicsResponse { topics, bhavas }
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
    let facts = FactBase::new(c, req.derivation.unwrap_or_default(), NativeInfo { sex: req.sex });
    let report = evaluate_topic(name, &s.corpus.rules, &facts, mode, (from_age, to_age));
    let pariharams = lagn_rules::pariharam::suggest(&s.corpus, &report.results, &[], mode);
    let writeup = meta.as_ref().map(|m| {
        lagn_rules::writeup::write_up(&s.corpus, m, &m.focus, &report, &facts, (from_age, to_age), |_| true)
    });
    Ok(view::TopicResponse { meta, pariharams, writeup, ..view::topic_response(report, tz) })
}

/// Sensitive periods: every antardasha in the age range with its period rules,
/// focus houses, overlapping transits and pariharams.
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
    let star = |c: &lagn_core::Chart| StarPos { nakshatra: c.janma_nakshatra().nakshatra, rasi: c.janma_rasi() };
    Ok(match_report(&s.corpus, star(&b), star(&g), mode))
}
