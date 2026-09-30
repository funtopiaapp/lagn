//! Phase 4: HTTP API over the verified kernel. Specification:
//! `docs/phase4/DESIGN.md`. The server computes nothing itself: every value
//! comes from `lagn-core` and `lagn-rules`, unchanged.

pub mod api;
#[cfg(feature = "http")]
pub mod http;
pub mod offset;
pub mod places;
pub mod view;

use std::path::Path;

use lagn_rules::corpus::Corpus;

use crate::places::PlaceIndex;

#[cfg(feature = "http")]
pub use crate::http::{router, router_with, Cors, BODY_LIMIT, CSP};

/// Minimum review-token length accepted at startup. A policy of the engine,
/// not of the transport: loading state enforces it on every platform.
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
