//! The HTTP transport. Everything axum-shaped lives here, behind the `http`
//! feature, so the engine can be built for targets where no server exists -
//! WebAssembly in a browser, or an app on a phone. The operations themselves
//! are in `crate::api`, and both transports call those.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use lagn_rules::report::{review_sheet, MatchReport};
use lagn_rules::resolve::Mode;
use serde::Deserialize;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{api, offset, places, view, ApiError, AppState};

/// Maximum request body.
pub const BODY_LIMIT: usize = 16 * 1024;

type ApiResult<T> = Result<Json<T>, ApiError>;

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

async fn version(State(s): State<Arc<AppState>>) -> Json<api::VersionInfo> {
    Json(api::version(&s))
}

async fn places_search(State(s): State<Arc<AppState>>, q: Result<Query<api::PlacesQuery>, axum::extract::rejection::QueryRejection>) -> ApiResult<Vec<places::Place>> {
    let Query(q) = q.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    api::places(&s, &q).map(Json)
}

async fn offset_suggest(State(s): State<Arc<AppState>>, q: Result<Query<api::OffsetQuery>, axum::extract::rejection::QueryRejection>) -> ApiResult<offset::Suggestion> {
    let Query(q) = q.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    api::offset(&s, &q).map(Json)
}

async fn chart(body: Body) -> ApiResult<view::ChartResponse> {
    let req: api::ChartRequest = parse_json(body)?;
    compute(move || api::chart(req)).await.map(Json)
}

async fn topics(State(s): State<Arc<AppState>>) -> Json<api::TopicsResponse> {
    Json(api::topics(&s))
}

async fn topic(State(s): State<Arc<AppState>>, UrlPath(name): UrlPath<String>, headers: HeaderMap, body: Body) -> ApiResult<view::TopicResponse> {
    let req: api::TopicRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || api::topic(&s, &name, req, mode)).await.map(Json)
}

async fn periods(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<view::PeriodsResponse> {
    let req: api::PeriodsRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || api::periods(&s, req, mode)).await.map(Json)
}

/// A day's timings need no chart and no review gate: a panchangam states what
/// the day is, and nothing here is interpreted.
async fn days(body: Body) -> ApiResult<api::DaysResponse> {
    let req: api::DayRequest = parse_json(body)?;
    compute(move || api::days(req)).await.map(Json)
}

async fn family(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<lagn_rules::family::FamilyReading> {
    let req: api::FamilyRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || api::family(&s, req, mode)).await.map(Json)
}

async fn match_(State(s): State<Arc<AppState>>, headers: HeaderMap, body: Body) -> ApiResult<MatchReport> {
    let req: api::MatchRequest = parse_json(body)?;
    let mode = authorise(&s, &headers, req.mode)?;
    compute(move || api::match_(&s, req, mode)).await.map(Json)
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
// 'wasm-unsafe-eval' permits compiling WebAssembly - the engine running in
// the browser - without permitting eval() of JavaScript, which stays blocked.
/// The policy actually sent. `LAGN_COUNTER_ORIGIN` adds one host to `img-src`
/// so a visit counter can be shown when the app is served by this server; the
/// static build does the same through its meta tag. The host is allowed as an
/// image and nothing else - it may not run scripts or be connected to - and a
/// value that is not a bare origin is ignored rather than trusted.
pub fn csp() -> String {
    match std::env::var("LAGN_COUNTER_ORIGIN") {
        Ok(origin)
            if origin.starts_with("https://")
                && !origin.contains(|c: char| c.is_whitespace() || c == ';' || c == ',')
                && origin.matches('/').count() == 2 =>
        {
            CSP.replace("img-src 'self' data:", &format!("img-src 'self' data: {origin}"))
        }
        _ => CSP.to_string(),
    }
}

pub const CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; \
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
        .route("/api/days", post(days))
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
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_str(&csp()).unwrap_or_else(|_| HeaderValue::from_static(CSP)),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")))
}

