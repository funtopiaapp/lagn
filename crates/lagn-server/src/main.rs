//! `lagn-server`: serves the API and the web app.

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use lagn_core::Ephemeris;
use lagn_server::{router_with, AppState, Cors};

/// Hosting platforms hand the port over in $PORT and expect the process to
/// listen on every interface. Locally, listening only on the loopback is the
/// safer default.
fn default_addr() -> String {
    match std::env::var("PORT").ok().filter(|p| p.parse::<u16>().is_ok()) {
        Some(port) => format!("0.0.0.0:{port}"),
        None => "127.0.0.1:8080".to_string(),
    }
}

#[derive(Parser)]
#[command(name = "lagn-server", about = "HTTP API and web app for the lagn kernel")]
struct Args {
    /// Address to listen on. Defaults to 127.0.0.1:8080, or 0.0.0.0:$PORT
    /// when PORT is set, which is how most hosting platforms assign one.
    #[arg(long)]
    addr: Option<String>,
    #[arg(long, default_value = "ephe")]
    ephe: PathBuf,
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,
    #[arg(long, default_value = "data/places.tsv")]
    places: PathBuf,
    /// IANA tz database compiled by scripts/update_tzdb.sh. The newest of this,
    /// the OS copy and the bundled copy is used.
    #[arg(long, default_value = "data/zoneinfo")]
    tzdb: PathBuf,
    /// Built web app (web/dist). Omit to serve the API only.
    #[arg(long = "static")]
    static_dir: Option<PathBuf>,
    /// File holding the review token. Omit to disable review mode entirely.
    /// A file, not a flag, so the token never appears in the process list.
    #[arg(long)]
    review_token_file: Option<PathBuf>,
    /// Origin allowed to call the API cross-origin, e.g. capacitor://localhost
    /// for the iOS app or https://localhost for the Android app. Repeatable.
    /// Omit for same-origin only.
    #[arg(long = "allow-origin")]
    allow_origin: Vec<String>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let a = Args::parse();
    Ephemeris::set_ephemeris_path(&a.ephe).map_err(|e| e.to_string())?;
    let token = match &a.review_token_file {
        Some(p) => Some(std::fs::read_to_string(p).map_err(|e| format!("read {}: {e}", p.display()))?.trim().to_string()),
        None => None,
    };
    let review = token.is_some();
    let state = Arc::new(AppState::load(&a.corpus, &a.places, Some(&a.tzdb), token)?);
    if let Some(d) = &a.static_dir {
        if !d.join("index.html").is_file() {
            return Err(format!("{} has no index.html; build the web app first", d.display()));
        }
    }
    let state_tz = state.tzdb.version_label();
    let cors = Cors::parse(&a.allow_origin)?;
    let app = router_with(state, a.static_dir.as_deref(), &cors);
    let addr = a.addr.clone().unwrap_or_else(default_addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| format!("bind {addr}: {e}"))?;
    eprintln!(
        "lagn-server listening on http://{}  (review mode: {}; tz database: {})",
        addr, if review { "enabled" } else { "disabled" }, state_tz
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; })
        .await
        .map_err(|e| e.to_string())
}
