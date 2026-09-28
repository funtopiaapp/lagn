//! Phase 4: HTTP API over the verified kernel. Specification:
//! `docs/phase4/DESIGN.md`. The server computes nothing itself: every value
//! comes from `lagn-core` and `lagn-rules`, unchanged.

pub mod offset;
pub mod places;
pub mod view;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use lagn_core::{DerivationSettings, Ephemeris};
use lagn_rules::corpus::{summary, Corpus, CorpusSummary};
use lagn_rules::report::{match_report, review_sheet, MatchReport};
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo, StarPos};
use serde::{Deserialize, Serialize};
use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::places::PlaceIndex;
use crate::view::BirthInput;

/// Maximum request body.
pub const BODY_LIMIT: usize = 16 * 1024;
/// Minimum review-token length the server accepts at startup.
pub const MIN_TOKEN_LEN: usize = 16;
pub const ATTRIBUTION: &str = "Place data © GeoNames (CC BY 4.0)";

pub struct AppState {
    pub corpus: Corpus,
    pub places: PlaceIndex,
    pub tzdb: offset::TzDb,
    /// `None` disables review mode entirely.
    pub review_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Forbidden(String),
    NotFound(String),
    TooLarge,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            ApiError::TooLarge => (StatusCode::PAYLOAD_TOO_LARGE, format!("request body exceeds {BODY_LIMIT} bytes")),
            ApiError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

type Body = Result<Bytes, axum::extract::rejection::BytesRejection>;

/// Parse a JSON body. Oversized bodies (rejected by `DefaultBodyLimit`) and
/// malformed JSON both come back in the uniform `{"error": ...}` shape.
fn parse_json<T: for<'de> Deserialize<'de>>(body: Body) -> Result<T, ApiError> {
    let bytes = body.map_err(|e| {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE { ApiError::TooLarge } else { ApiError::BadRequest(e.body_text()) }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ApiError::BadRequest(format!("invalid request: {e}")))
}

/// Run kernel work off the async executor. Swiss Ephemeris serialises behind
/// its own lock, so this bounds contention to the blocking pool.
async fn compute<T: Send + 'static>(f: impl FnOnce() -> Result<T, ApiError> + Send + 'static) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| ApiError::Internal("computation failed".into()))?
}

// ---------------------------------------------------------------------------
// The review gate at the boundary
// ---------------------------------------------------------------------------

/// Constant-time comparison, so response timing can't leak the token.
fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn authorise(state: &AppState, headers: &HeaderMap, mode: Mode) -> Result<Mode, ApiError> {
    if mode == Mode::Production {
        return Ok(Mode::Production);
    }
    let Some(expected) = &state.review_token else {
        return Err(ApiError::Forbidden("review mode is not enabled on this server".into()));
    };
    let supplied = headers.get("x-review-token").map(|v| v.as_bytes()).unwrap_or(b"");
    if supplied.is_empty() || !same(supplied, expected.as_bytes()) {
        return Err(ApiError::Forbidden("review mode requires a valid X-Review-Token".into()));
    }
    Ok(Mode::Review)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

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

async fn version(State(s): State<Arc<AppState>>) -> Json<VersionInfo> {
    let poruthams_approved = s
        .corpus
        .porutham
        .as_ref()
        .map(|m| m.values().filter(|r| r.status == lagn_rules::ReviewStatus::Approved).count())
        .unwrap_or(0);
    Json(VersionInfo {
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
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacesQuery {
    q: String,
    limit: Option<usize>,
}

async fn places_search(State(s): State<Arc<AppState>>, q: Result<Query<PlacesQuery>, axum::extract::rejection::QueryRejection>) -> ApiResult<Vec<places::Place>> {
    let Query(q) = q.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    if q.q.chars().count() > 100 {
        return Err(ApiError::BadRequest("query too long".into()));
    }
    Ok(Json(s.places.search(&q.q, q.limit.unwrap_or(10).clamp(1, 50))))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OffsetQuery {
    tz: String,
    date: String,
    time: String,
    longitude: Option<f64>,
}

async fn offset_suggest(State(s): State<Arc<AppState>>, q: Result<Query<OffsetQuery>, axum::extract::rejection::QueryRejection>) -> ApiResult<offset::Suggestion> {
    let Query(q) = q.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    let date = view::parse_date(&q.date).map_err(ApiError::BadRequest)?;
    let time = view::parse_time(&q.time).map_err(ApiError::BadRequest)?;
    offset::suggest(&s.tzdb, &q.tz, date, time, q.longitude).map(Json).map_err(ApiError::BadRequest)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChartRequest {
    birth: BirthInput,
    #[serde(default)]
    derivation: Option<DerivationSettings>,
}

async fn chart(body: Body) -> ApiResult<view::ChartResponse> {
    let req: ChartRequest = parse_json(body)?;
    compute(move || {
        let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
        let d = req.derivation.unwrap_or_default();
        Ok(view::chart_response(req.birth, c, &d, Some(view::jd_now())))
    })
    .await
    .map(Json)
}

fn check_ages(from: f64, to: f64) -> Result<(), ApiError> {
    if !(from >= 0.0 && to > from && to <= 120.0) {
        return Err(ApiError::BadRequest("age range must satisfy 0 <= from_age < to_age <= 120".into()));
    }
    Ok(())
}

#[derive(Serialize)]
struct TopicsResponse {
    topics: Vec<lagn_rules::catalogue::TopicMeta>,
    bhavas: Vec<lagn_rules::catalogue::BhavaMeaning>,
}

/// The topic catalogue. Only approved topics that have approved natal rules
/// are listed, so the app never offers a topic that would come back empty.
async fn topics(State(s): State<Arc<AppState>>) -> Json<TopicsResponse> {
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
    Json(TopicsResponse { topics, bhavas })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TopicRequest {
    birth: BirthInput,
    #[serde(default)]
    derivation: Option<DerivationSettings>,
    #[serde(default)]
    sex: Option<lagn_rules::Sex>,
    /// Defaults to the topic's catalogue ages (18-45 if it has none).
    #[serde(default)]
    from_age: Option<f64>,
    #[serde(default)]
    to_age: Option<f64>,
    #[serde(default)]
    mode: Mode,
}

async fn topic(State(s): State<Arc<AppState>>, UrlPath(name): UrlPath<String>, headers: HeaderMap, body: Body) -> ApiResult<view::TopicResponse> {
    let req: TopicRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    // Period rules are served by /api/periods, not as a topic.
    if !s.corpus.rules.iter().any(|r| r.topic == name && r.scope == lagn_rules::Scope::Natal) {
        return Err(ApiError::NotFound(format!("no rules for topic {name:?}")));
    }
    let meta = s.corpus.topics.iter().find(|t| t.id == name && mode.admits(t.review.status)).cloned();
    let [dfrom, dto] = meta.as_ref().map(|m| m.ages).unwrap_or([18.0, 45.0]);
    let (from_age, to_age) = (req.from_age.unwrap_or(dfrom), req.to_age.unwrap_or(dto));
    check_ages(from_age, to_age)?;
    compute(move || {
        let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
        let tz = c.birth.moment.utc_offset_hours;
        let facts = FactBase::new(c, req.derivation.unwrap_or_default(), NativeInfo { sex: req.sex });
        let report = evaluate_topic(&name, &s.corpus.rules, &facts, mode, (from_age, to_age));
        let pariharams = lagn_rules::pariharam::suggest(&s.corpus, &report.results, &[], mode);
        let writeup = meta.as_ref().map(|m| {
            lagn_rules::writeup::write_up(&s.corpus, m, &m.focus, &report, &facts, (from_age, to_age), |_| true)
        });
        Ok(view::TopicResponse { meta, pariharams, writeup, ..view::topic_response(report, tz) })
    })
    .await
    .map(Json)
}

fn d_from() -> f64 { 0.0 }
fn d_to() -> f64 { 80.0 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PeriodsRequest {
    birth: BirthInput,
    #[serde(default)]
    derivation: Option<DerivationSettings>,
    #[serde(default)]
    sex: Option<lagn_rules::Sex>,
    #[serde(default = "d_from")]
    from_age: f64,
    #[serde(default = "d_to")]
    to_age: f64,
    #[serde(default)]
    mode: Mode,
}

/// Sensitive periods: every antardasha in the age range with its period
/// rules, focus houses, overlapping transits and pariharams.
async fn periods(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<view::PeriodsResponse> {
    let req: PeriodsRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    check_ages(req.from_age, req.to_age)?;
    compute(move || {
        let c = req.birth.to_chart().map_err(ApiError::BadRequest)?;
        let tz = c.birth.moment.utc_offset_hours;
        let t = lagn_rules::reading::transits(&c, req.from_age, req.to_age)
            .map_err(|e| ApiError::Internal(format!("transits: {e}")))?;
        let mut facts = FactBase::new(c, req.derivation.unwrap_or_default(), NativeInfo { sex: req.sex });
        let r = lagn_rules::reading::sensitive_periods(&s.corpus, &mut facts, mode, (req.from_age, req.to_age), &t.windows);
        Ok(view::periods_response(r, &t, tz, view::jd_now()))
    })
    .await
    .map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Person {
    birth: BirthInput,
    #[serde(default)]
    sex: Option<lagn_rules::Sex>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyRequest {
    native: Person,
    member: Person,
    relation: lagn_rules::family::Relation,
    #[serde(default)]
    derivation: Option<DerivationSettings>,
    #[serde(default)]
    mode: Mode,
}

/// A family member's own chart beside the native's relational reading. Both
/// birth records arrive with the request; nothing is stored.
async fn family(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<lagn_rules::family::FamilyReading> {
    let req: FamilyRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || {
        let d = req.derivation.unwrap_or_default();
        let n = req.native.birth.to_chart().map_err(|e| ApiError::BadRequest(format!("native: {e}")))?;
        let m = req.member.birth.to_chart().map_err(|e| ApiError::BadRequest(format!("member: {e}")))?;
        let native = FactBase::new(n, d, NativeInfo { sex: req.native.sex });
        let member = FactBase::new(m, d, NativeInfo { sex: req.member.sex });
        Ok(lagn_rules::family::family_reading(&s.corpus, &native, &member, req.relation, mode))
    })
    .await
    .map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchRequest {
    bride: BirthInput,
    groom: BirthInput,
    #[serde(default)]
    mode: Mode,
}

async fn match_(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<MatchReport> {
    let req: MatchRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || {
        let b = req.bride.to_chart().map_err(|e| ApiError::BadRequest(format!("bride: {e}")))?;
        let g = req.groom.to_chart().map_err(|e| ApiError::BadRequest(format!("groom: {e}")))?;
        let star = |c: &lagn_core::Chart| StarPos { nakshatra: c.janma_nakshatra().nakshatra, rasi: c.janma_rasi() };
        Ok(match_report(&s.corpus, star(&b), star(&g), mode))
    })
    .await
    .map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewSheetQuery {
    topic: String,
}

async fn review_sheet_md(State(s): State<Arc<AppState>>, headers: HeaderMap, q: Result<Query<ReviewSheetQuery>, axum::extract::rejection::QueryRejection>) -> Result<Response, ApiError> {
    let Query(q) = q.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    authorise(&s, &headers, Mode::Review)?;
    if !s.corpus.rules.iter().any(|r| r.topic == q.topic) {
        return Err(ApiError::NotFound(format!("no rules for topic {:?}", q.topic)));
    }
    let md = compute(move || Ok(review_sheet(&s.corpus, &q.topic, 1000))).await?;
    Ok(([("content-type", "text/markdown; charset=utf-8")], md).into_response())
}

async fn api_not_found() -> ApiError {
    ApiError::NotFound("no such API endpoint".into())
}

/// Origins allowed to call the API from another origin - the wrapped iOS and
/// Android apps (`capacitor://localhost`, `https://localhost`) or a separately
/// hosted web front end. Empty means same-origin only: no CORS headers at all.
#[derive(Debug, Clone, Default)]
pub struct Cors {
    pub origins: Vec<String>,
}

impl Cors {
    /// Validate an origin: scheme and host, no path, no wildcard.
    pub fn parse(origins: &[String]) -> Result<Cors, String> {
        for o in origins {
            let ok = (o.starts_with("https://") || o.starts_with("http://") || o.starts_with("capacitor://"))
                && !o.contains('*')
                && o.split("://").nth(1).is_some_and(|rest| !rest.is_empty() && !rest.contains('/'));
            if !ok {
                return Err(format!("invalid --allow-origin {o:?}: expected scheme://host[:port], no path or wildcard"));
            }
            HeaderValue::from_str(o).map_err(|_| format!("invalid --allow-origin {o:?}"))?;
        }
        Ok(Cors { origins: origins.to_vec() })
    }

    fn layer(&self) -> Option<CorsLayer> {
        if self.origins.is_empty() {
            return None;
        }
        let list: Vec<HeaderValue> = self.origins.iter().map(|o| HeaderValue::from_str(o).unwrap()).collect();
        Some(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(list))
                .allow_methods([Method::GET, Method::POST])
                // The review token crosses origins only for listed origins.
                .allow_headers([header::CONTENT_TYPE, HeaderName::from_static("x-review-token")])
                .max_age(Duration::from_secs(600)),
        )
    }
}

/// Content-Security-Policy for the web app served by this server.
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; \
connect-src 'self'; manifest-src 'self'; worker-src 'self'; font-src 'self'; object-src 'none'; \
base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

/// The full application. `static_dir` serves the built web app, with
/// `index.html` as the fallback for client-side routes.
pub fn router(state: Arc<AppState>, static_dir: Option<&Path>) -> Router {
    router_with(state, static_dir, &Cors::default())
}

/// As [`router`], with cross-origin access for the listed origins.
pub fn router_with(state: Arc<AppState>, static_dir: Option<&Path>, cors: &Cors) -> Router {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/version", get(version))
        .route("/api/places", get(places_search))
        .route("/api/offset", get(offset_suggest))
        .route("/api/chart", post(chart))
        .route("/api/topics", get(topics))
        .route("/api/topic/{name}", post(topic))
        .route("/api/periods", post(periods))
        .route("/api/family", post(family))
        .route("/api/match", post(match_))
        .route("/api/review-sheet", get(review_sheet_md))
        .route("/api/{*rest}", get(api_not_found).post(api_not_found))
        .with_state(state);
    // CORS belongs to the API only. Applied to the whole app it also stamped
    // the web app's own files with `Vary: origin`, and the service worker's
    // precached copies then failed to match the browser's requests, which
    // carry an Origin header: a blank screen offline (QA-P5-1).
    let api = match cors.layer() {
        Some(c) => api.layer(c),
        None => api,
    };

    let app = match static_dir {
        Some(dir) => {
            let index: PathBuf = dir.join("index.html");
            api.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)))
        }
        None => api,
    };
    app.layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
        // Backstop for anything that reads a body without the extractor.
        .layer(RequestBodyLimitLayer::new(BODY_LIMIT * 4))
        .layer(CatchPanicLayer::new())
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(20)))
        .layer(SetResponseHeaderLayer::if_not_present(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP)))
        .layer(SetResponseHeaderLayer::if_not_present(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")))
}

impl AppState {
    pub fn load(corpus_dir: &Path, places_file: &Path, tzdb_dir: Option<&Path>, review_token: Option<String>) -> Result<AppState, String> {
        if let Some(t) = &review_token {
            if t.len() < MIN_TOKEN_LEN {
                return Err(format!("review token must be at least {MIN_TOKEN_LEN} characters"));
            }
        }
        Ok(AppState {
            corpus: lagn_rules::load(corpus_dir).map_err(|e| e.to_string())?,
            places: PlaceIndex::load(places_file)?,
            tzdb: offset::TzDb::newest(tzdb_dir),
            review_token,
        })
    }
}
