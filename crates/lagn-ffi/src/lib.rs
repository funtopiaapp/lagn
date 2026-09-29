//! A C ABI over the engine, so an iOS or Android app runs the whole reading
//! on the device with no server and no network.
//!
//! Every function takes JSON and returns JSON, in exactly the shapes the HTTP
//! API uses, because both call `lagn_server::api`. A parity test asserts the
//! two agree byte for byte.
//!
//! # Contract
//!
//! - Every function returns a NUL-terminated UTF-8 C string that the caller
//!   **must** release with [`lagn_string_free`]. Null is returned only if the
//!   process is out of memory.
//! - Failures come back as `{"error": "..."}`, the same shape the HTTP API
//!   uses, so one client path handles both transports.
//! - [`lagn_init`] must be called once before anything else. It is idempotent
//!   and safe to call again (for example after the app reloads its data).
//! - Every function is safe to call from any thread. The Swiss Ephemeris has a
//!   global lock inside `lagn-ephem`, so calls serialise there.
//!
//! Specification: `docs/phase10/DESIGN.md`.

use std::ffi::{c_char, CStr, CString};
use std::sync::{Arc, OnceLock, RwLock};

use lagn_core::Ephemeris;
use lagn_rules::resolve::Mode;
use lagn_server::{api, ApiError, AppState};

#[cfg(feature = "jni")]
mod android;

/// The loaded reference data. `None` until `lagn_init` succeeds.
fn state() -> &'static RwLock<Option<Arc<AppState>>> {
    static STATE: OnceLock<RwLock<Option<Arc<AppState>>>> = OnceLock::new();
    STATE.get_or_init(|| RwLock::new(None))
}

// ---------------------------------------------------------------------------
// Strings across the boundary
// ---------------------------------------------------------------------------

fn out(s: String) -> *mut c_char {
    // A NUL inside would truncate the JSON, so replace rather than fail: the
    // caller still gets well-formed JSON it can parse.
    match CString::new(s.replace('\0', "\u{fffd}")) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn error_json(message: &str) -> *mut c_char {
    out(serde_json::json!({ "error": message }).to_string())
}

fn ok<T: serde::Serialize>(v: Result<T, ApiError>) -> *mut c_char {
    match v {
        Ok(v) => match serde_json::to_string(&v) {
            Ok(s) => out(s),
            Err(e) => error_json(&format!("could not serialise the result: {e}")),
        },
        Err(e) => error_json(&api_error_message(&e)),
    }
}

fn api_error_message(e: &ApiError) -> String {
    match e {
        ApiError::BadRequest(m) | ApiError::Forbidden(m) | ApiError::NotFound(m) | ApiError::Internal(m) => m.clone(),
        ApiError::TooLarge => "request too large".to_string(),
    }
}

/// Read a C string argument. `None` for null or invalid UTF-8.
///
/// # Safety
/// `p` must be null or a NUL-terminated string valid for the call.
unsafe fn input<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

/// Release a string returned by any function in this library.
///
/// # Safety
/// `p` must be null, or a pointer this library returned and has not already
/// been freed.
#[no_mangle]
pub unsafe extern "C" fn lagn_string_free(p: *mut c_char) {
    if !p.is_null() {
        drop(unsafe { CString::from_raw(p) });
    }
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/// Load the reference data. Call once at start-up.
///
/// `config_json`: `{"ephemeris": "...", "corpus": "...", "places": "...",
/// "tzdb": "..."}` - absolute directory or file paths inside the app's own
/// storage. `tzdb` may be omitted, in which case the copy compiled into the
/// binary is used.
///
/// Returns `{"ok": true, ...}` or `{"error": "..."}`.
///
/// # Safety
/// `config_json` must be a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn lagn_init(config_json: *const c_char) -> *mut c_char {
    let Some(text) = (unsafe { input(config_json) }) else {
        return error_json("lagn_init needs a JSON configuration string");
    };
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Config {
        ephemeris: String,
        corpus: String,
        places: String,
        #[serde(default)]
        tzdb: Option<String>,
    }
    let cfg: Config = match serde_json::from_str(text) {
        Ok(c) => c,
        Err(e) => return error_json(&format!("invalid configuration: {e}")),
    };
    if let Err(e) = Ephemeris::set_ephemeris_path(&cfg.ephemeris) {
        return error_json(&format!("ephemeris path {}: {e}", cfg.ephemeris));
    }
    let loaded = AppState::load(
        std::path::Path::new(&cfg.corpus),
        std::path::Path::new(&cfg.places),
        cfg.tzdb.as_deref().map(std::path::Path::new),
        // Reviewer mode is a server facility: on a device there is nowhere to
        // keep a secret, so the gate stays closed and only approved content is
        // ever produced.
        None,
    );
    match loaded {
        Ok(s) => {
            let s = Arc::new(s);
            let info = serde_json::json!({
                "ok": true,
                "version": serde_json::to_value(api::version(&s)).unwrap_or(serde_json::Value::Null),
            });
            *state().write().expect("state lock") = Some(s);
            out(info.to_string())
        }
        Err(e) => error_json(&format!("could not load the reference data: {e}")),
    }
}

/// True once `lagn_init` has succeeded.
#[no_mangle]
pub extern "C" fn lagn_ready() -> bool {
    state().read().map(|s| s.is_some()).unwrap_or(false)
}

fn with_state<F>(f: F) -> *mut c_char
where
    F: FnOnce(&Arc<AppState>) -> *mut c_char,
{
    let guard = match state().read() {
        Ok(g) => g,
        Err(_) => return error_json("the engine is in a failed state; restart the app"),
    };
    match guard.as_ref() {
        Some(s) => f(s),
        None => error_json("the engine is not initialised; call lagn_init first"),
    }
}

/// Parse a request body, or produce the error JSON for it.
fn parse<T: serde::de::DeserializeOwned>(body: Option<&str>) -> Result<T, *mut c_char> {
    let Some(text) = body else {
        return Err(error_json("a JSON request body is required"));
    };
    serde_json::from_str(text).map_err(|e| error_json(&format!("invalid request: {e}")))
}

macro_rules! endpoint {
    ($name:ident, $req:ty, |$state:ident, $r:ident| $body:expr) => {
        /// # Safety
        /// `request_json` must be a NUL-terminated UTF-8 string.
        #[no_mangle]
        pub unsafe extern "C" fn $name(request_json: *const c_char) -> *mut c_char {
            let text = unsafe { input(request_json) };
            with_state(|$state| {
                let $r: $req = match parse(text) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                ok($body)
            })
        }
    };
}

// ---------------------------------------------------------------------------
// Operations. These mirror the HTTP API one for one.
// ---------------------------------------------------------------------------

/// `{"engine": ..., "swiss_ephemeris": ..., "corpus": {...}, ...}`
#[no_mangle]
pub extern "C" fn lagn_version() -> *mut c_char {
    with_state(|s| ok::<api::VersionInfo>(Ok(api::version(s))))
}

/// The topic catalogue and the bhava meanings.
#[no_mangle]
pub extern "C" fn lagn_topics() -> *mut c_char {
    with_state(|s| ok::<api::TopicsResponse>(Ok(api::topics(s))))
}

endpoint!(lagn_chart, api::ChartRequest, |_s, req| api::chart(req));
endpoint!(lagn_periods, api::PeriodsRequest, |s, req| api::periods(s, req, Mode::Production));
endpoint!(lagn_family, api::FamilyRequest, |s, req| api::family(s, req, Mode::Production));
endpoint!(lagn_match, api::MatchRequest, |s, req| api::match_(s, req, Mode::Production));
endpoint!(lagn_places, api::PlacesQuery, |s, q| api::places(s, &q));
endpoint!(lagn_offset, api::OffsetQuery, |s, q| api::offset(s, &q));

/// A reading for one topic. `name` is a topic id from `lagn_topics`.
///
/// # Safety
/// Both arguments must be NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn lagn_topic(name: *const c_char, request_json: *const c_char) -> *mut c_char {
    let (name, text) = (unsafe { input(name) }, unsafe { input(request_json) });
    with_state(|s| {
        let Some(name) = name else {
            return error_json("a topic name is required");
        };
        let req: api::TopicRequest = match parse(text) {
            Ok(v) => v,
            Err(e) => return e,
        };
        ok(api::topic(s, name, req, Mode::Production))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Call through the real C ABI, as an app would, and return the JSON.
    fn call(f: impl FnOnce() -> *mut c_char) -> serde_json::Value {
        let p = f();
        assert!(!p.is_null(), "the library returned null");
        let text = unsafe { CStr::from_ptr(p) }.to_str().expect("utf-8").to_string();
        unsafe { lagn_string_free(p) };
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("not JSON: {e}: {text}"))
    }

    fn cstring(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    fn root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
    }

    fn init() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let r = root();
            let cfg = serde_json::json!({
                "ephemeris": r.join("ephe").to_str().unwrap(),
                "corpus": r.join("corpus").to_str().unwrap(),
                "places": r.join("data/places.tsv").to_str().unwrap(),
                "tzdb": r.join("data/zoneinfo").to_str().unwrap(),
            })
            .to_string();
            let v = call(|| unsafe { lagn_init(cstring(&cfg).as_ptr()) });
            assert_eq!(v["ok"], true, "init failed: {v}");
        });
    }

    const BIRTH: &str = r#"{"date":"1985-06-21","time":"14:30:00","latitude":13.08,"longitude":80.27,"utc_offset_hours":5.5}"#;

    #[test]
    fn an_uninitialised_library_says_so_instead_of_crashing() {
        // Before init on a fresh process this would be the "not initialised"
        // error; after it, a real answer. Either way it is JSON, never a crash.
        let v = call(|| lagn_version());
        assert!(v.get("engine").is_some() || v["error"].as_str().unwrap().contains("initialised"));
    }

    #[test]
    fn every_operation_answers_with_json_over_the_c_abi() {
        init();
        assert!(lagn_ready());

        let v = call(|| lagn_version());
        assert!(v["corpus"]["rules"].as_u64().unwrap() > 100);
        assert_eq!(v["review_enabled"], false, "a device build must not offer reviewer mode");

        let topics = call(|| lagn_topics());
        let ids: Vec<&str> = topics["topics"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap()).collect();
        assert!(ids.contains(&"marriage") && ids.contains(&"past_life"), "{ids:?}");
        assert_eq!(topics["bhavas"].as_array().unwrap().len(), 12);

        let body = cstring(&format!(r#"{{"birth":{BIRTH}}}"#));
        let chart = call(|| unsafe { lagn_chart(body.as_ptr()) });
        assert_eq!(chart["positions"].as_array().unwrap().len(), 9);

        for id in &ids {
            let name = cstring(id);
            let v = call(|| unsafe { lagn_topic(name.as_ptr(), body.as_ptr()) });
            assert!(v["report"]["topic"] == **id, "{id}: {v}");
            assert!(v["writeup"]["summary"].as_array().is_some_and(|s| !s.is_empty()), "{id}: no write-up");
        }

        let periods_body = cstring(&format!(r#"{{"birth":{BIRTH},"from_age":20,"to_age":40}}"#));
        let p = call(|| unsafe { lagn_periods(periods_body.as_ptr()) });
        assert!(!p["windows"].as_array().unwrap().is_empty());

        let fam = cstring(&format!(
            r#"{{"native":{{"birth":{BIRTH}}},"member":{{"birth":{BIRTH}}},"relation":"child"}}"#
        ));
        let f = call(|| unsafe { lagn_family(fam.as_ptr()) });
        assert!(f["comparison"].as_array().is_some_and(|c| !c.is_empty()));

        let m = cstring(&format!(r#"{{"bride":{BIRTH},"groom":{BIRTH}}}"#));
        let m = call(|| unsafe { lagn_match(m.as_ptr()) });
        // Nine, not ten: Vasya is rejected for want of a verified table, and a
        // device build only ever produces approved content.
        assert_eq!(m["results"].as_array().unwrap().len(), 9);

        let q = cstring(r#"{"q":"Chennai","limit":5}"#);
        let places = call(|| unsafe { lagn_places(q.as_ptr()) });
        assert!(!places.as_array().unwrap().is_empty());

        let o = cstring(r#"{"tz":"Asia/Kolkata","date":"1985-06-21","time":"14:30:00"}"#);
        let off = call(|| unsafe { lagn_offset(o.as_ptr()) });
        assert_eq!(off["candidates"][0]["offset_hours"], 5.5);
    }

    #[test]
    fn bad_input_comes_back_as_an_error_not_a_crash() {
        init();
        // Each of these fails at a different stage, and the message must say
        // which: a client showing "invalid request" for a bad latitude would
        // be useless to whoever typed it.
        let complete = |date: &str, time: &str, lat: &str| {
            format!(r#"{{"birth":{{"date":"{date}","time":"{time}","latitude":{lat},"longitude":80.27,"utc_offset_hours":5.5}}}}"#)
        };
        let cases: Vec<(String, &str)> = vec![
            ("".to_string(), "invalid request"),
            ("{".to_string(), "invalid request"),
            ("null".to_string(), "invalid request"),
            (r#"{"birth":null}"#.to_string(), "invalid request"),
            (r#"{"birth":{"date":"1985-06-21"}}"#.to_string(), "missing field"),
            (complete("not-a-date", "14:30:00", "13.08"), "date must be YYYY-MM-DD"),
            (complete("1985-02-30", "14:30:00", "13.08"), "invalid calendar date"),
            (complete("1985-06-21", "25:00:00", "13.08"), "invalid clock time"),
            (complete("1985-06-21", "14:30:00", "999"), "latitude 999 out of range"),
        ];
        for (bad, expected) in cases {
            let b = cstring(&bad);
            let v = call(|| unsafe { lagn_chart(b.as_ptr()) });
            let message = v["error"].as_str().unwrap_or_else(|| panic!("{bad:?} gave {v}"));
            assert!(message.contains(expected), "{bad:?} gave {message:?}, expected {expected:?}");
        }
        // Null pointers are handled.
        let v = call(|| unsafe { lagn_chart(std::ptr::null()) });
        assert!(v["error"].is_string());
        let v = call(|| unsafe { lagn_topic(std::ptr::null(), std::ptr::null()) });
        assert!(v["error"].is_string());
        // An unknown topic is a clean error.
        let (name, body) = (cstring("no-such-topic"), cstring(&format!(r#"{{"birth":{BIRTH}}}"#)));
        let v = call(|| unsafe { lagn_topic(name.as_ptr(), body.as_ptr()) });
        assert!(v["error"].as_str().unwrap().contains("no rules for topic"));
        // Freeing null is allowed.
        unsafe { lagn_string_free(std::ptr::null_mut()) };
    }

    #[test]
    fn reviewer_mode_cannot_be_reached_from_a_device() {
        init();
        // Even when a caller asks for review mode, production content is what
        // comes back: there is no token on a device and the gate stays shut.
        let body = cstring(&format!(r#"{{"birth":{BIRTH},"mode":"review"}}"#));
        let name = cstring("marriage");
        let v = call(|| unsafe { lagn_topic(name.as_ptr(), body.as_ptr()) });
        assert_eq!(v["report"]["mode"], "production");
        for r in v["report"]["results"].as_array().unwrap() {
            assert_eq!(r["status"], "approved");
        }
    }

    #[test]
    fn calls_are_safe_from_many_threads_at_once() {
        init();
        let threads: Vec<_> = (0..8)
            .map(|i| {
                std::thread::spawn(move || {
                    let body = cstring(&format!(r#"{{"birth":{BIRTH},"from_age":{},"to_age":{}}}"#, i, i + 20));
                    let v = call(|| unsafe { lagn_periods(body.as_ptr()) });
                    assert!(v["windows"].as_array().is_some());
                })
            })
            .collect();
        for t in threads {
            t.join().expect("a thread panicked");
        }
    }
}
