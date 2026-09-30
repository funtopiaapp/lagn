//! The native library and the HTTP server must produce the same reading.
//!
//! Both call `lagn_server::api`, so this test is what keeps that true: it runs
//! the real axum router in-process and the real C ABI side by side over many
//! random births, and compares the JSON.
//!
//! Specification: `docs/phase10/DESIGN.md` acceptance criterion 1.

use std::ffi::{CStr, CString};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use lagn_core::Ephemeris;
use lagn_server::{router, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// The FFI library, initialised once, exactly as an app would.
fn init_ffi() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let r = root();
        let cfg = json!({
            "ephemeris": r.join("ephe").to_str().unwrap(),
            "corpus": r.join("corpus").to_str().unwrap(),
            "places": r.join("data/places.tsv").to_str().unwrap(),
            "tzdb": r.join("data/zoneinfo").to_str().unwrap(),
        })
        .to_string();
        let c = CString::new(cfg).unwrap();
        let v = ffi_json(|| unsafe { lagn_ffi::lagn_init(c.as_ptr()) });
        assert_eq!(v["ok"], true, "FFI init failed: {v}");
    });
}

fn ffi_json(f: impl FnOnce() -> *mut std::ffi::c_char) -> Value {
    let p = f();
    assert!(!p.is_null());
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
    unsafe { lagn_ffi::lagn_string_free(p) };
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("FFI returned non-JSON: {e}: {s}"))
}

fn server() -> axum::Router {
    static STATE: std::sync::OnceLock<Arc<AppState>> = std::sync::OnceLock::new();
    let s = STATE
        .get_or_init(|| {
            let r = root();
            Ephemeris::set_ephemeris_path(r.join("ephe")).unwrap();
            Arc::new(
                AppState::load(&r.join("corpus"), &r.join("data/places.tsv"), Some(&r.join("data/zoneinfo")), None)
                    .unwrap(),
            )
        })
        .clone();
    router(s, None)
}

async fn http_json(method: &str, path: &str, body: Option<&Value>) -> Value {
    let req = match body {
        Some(b) => Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => Request::builder().method(method).uri(path).body(Body::empty()).unwrap(),
    };
    let res = server().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "{path} returned {}", res.status());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Strip the fields that depend on the wall clock rather than on the chart.
/// The two transports are called moments apart, so "is this the period running
/// right now" can legitimately differ; everything else must match exactly.
fn stable(mut v: Value) -> Value {
    fn walk(v: &mut Value) {
        match v {
            Value::Object(map) => {
                map.remove("current");
                map.remove("running_now");
                map.remove("as_of_utc");
                map.remove("platform"); // "aarch64-macos" either way, but not a reading
                for (_, x) in map.iter_mut() {
                    walk(x);
                }
            }
            Value::Array(items) => items.iter_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut v);
    v
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn int(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn birth(r: &mut Rng) -> Value {
    json!({
        "date": format!("{:04}-{:02}-{:02}", r.int(1900, 2020), r.int(1, 12), r.int(1, 28)),
        "time": format!("{:02}:{:02}:00", r.int(0, 23), r.int(0, 59)),
        "latitude": 8.0 + r.unit() * 12.0,
        "longitude": 74.0 + r.unit() * 10.0,
        "utc_offset_hours": 5.5,
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn the_device_library_and_the_server_give_the_same_reading() {
    init_ffi();
    // The catalogue, which drives everything else.
    let http_topics = http_json("GET", "/api/topics", None).await;
    let ffi_topics = ffi_json(|| lagn_ffi::lagn_topics());
    assert_eq!(stable(ffi_topics.clone()), stable(http_topics), "the topic catalogue differs");
    let ids: Vec<String> =
        ffi_topics["topics"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap().to_string()).collect();
    assert!(ids.len() >= 10);

    let mut r = Rng(0xA_0001);
    let mut compared = 0;
    for i in 0..12 {
        let b = birth(&mut r);

        // Chart.
        let body = json!({ "birth": b });
        let c_http = http_json("POST", "/api/chart", Some(&body)).await;
        let payload = CString::new(body.to_string()).unwrap();
        let c_ffi = ffi_json(|| unsafe { lagn_ffi::lagn_chart(payload.as_ptr()) });
        assert_eq!(stable(c_ffi), stable(c_http), "chart differs for {b}");
        compared += 1;

        // Every topic, including the write-ups.
        for id in &ids {
            let t_http = http_json("POST", &format!("/api/topic/{id}"), Some(&body)).await;
            let (name, payload) = (CString::new(id.as_str()).unwrap(), CString::new(body.to_string()).unwrap());
            let t_ffi = ffi_json(|| unsafe { lagn_ffi::lagn_topic(name.as_ptr(), payload.as_ptr()) });
            assert_eq!(stable(t_ffi), stable(t_http), "topic {id} differs for {b}");
            compared += 1;
        }

        // Sensitive periods, the heaviest computation.
        let pbody = json!({ "birth": b, "from_age": 20, "to_age": 45 });
        let p_http = http_json("POST", "/api/periods", Some(&pbody)).await;
        let payload = CString::new(pbody.to_string()).unwrap();
        let p_ffi = ffi_json(|| unsafe { lagn_ffi::lagn_periods(payload.as_ptr()) });
        assert_eq!(stable(p_ffi), stable(p_http), "periods differ for {b}");
        compared += 1;

        // Family and porutham matching.
        let other = birth(&mut r);
        let relation = ["spouse", "child", "mother", "father"][i % 4];
        let fbody = json!({ "native": { "birth": b }, "member": { "birth": other }, "relation": relation });
        let f_http = http_json("POST", "/api/family", Some(&fbody)).await;
        let payload = CString::new(fbody.to_string()).unwrap();
        let f_ffi = ffi_json(|| unsafe { lagn_ffi::lagn_family(payload.as_ptr()) });
        assert_eq!(stable(f_ffi), stable(f_http), "family differs");
        compared += 1;

        let mbody = json!({ "bride": b, "groom": other });
        let m_http = http_json("POST", "/api/match", Some(&mbody)).await;
        let payload = CString::new(mbody.to_string()).unwrap();
        let m_ffi = ffi_json(|| unsafe { lagn_ffi::lagn_match(payload.as_ptr()) });
        assert_eq!(stable(m_ffi), stable(m_http), "match differs");
        compared += 1;
    }

    // Place search and offset suggestions, which the birth form depends on.
    for q in ["Chennai", "Madurai", "Kochi", "zzzz"] {
        let h = http_json("GET", &format!("/api/places?q={q}&limit=5"), None).await;
        let payload = CString::new(json!({ "q": q, "limit": 5 }).to_string()).unwrap();
        let f = ffi_json(|| unsafe { lagn_ffi::lagn_places(payload.as_ptr()) });
        assert_eq!(f, h, "place search differs for {q}");
        compared += 1;
    }
    for (tz, date, time) in [("Asia/Kolkata", "1985-06-21", "14:30:00"), ("Asia/Kolkata", "1943-05-05", "14:00:00")] {
        let h = http_json("GET", &format!("/api/offset?tz={tz}&date={date}&time={time}"), None).await;
        let payload = CString::new(json!({ "tz": tz, "date": date, "time": time }).to_string()).unwrap();
        let f = ffi_json(|| unsafe { lagn_ffi::lagn_offset(payload.as_ptr()) });
        assert_eq!(f, h, "offset differs for {tz} {date}");
        compared += 1;
    }

    assert!(compared > 150, "only {compared} comparisons");
}

/// The C header is what an iOS app compiles against. If it drifts from the
/// library, the app fails to link, or worse, links to the wrong signature.
#[test]
fn the_header_declares_exactly_what_the_library_exports() {
    let src = include_str!("../src/lib.rs");
    let header = std::fs::read_to_string(root().join("include/lagn.h")).expect("include/lagn.h");

    // Exported names: every `pub ... extern "C" fn` in the source, plus the
    // ones the endpoint! macro generates.
    let mut exported: Vec<String> = src
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub unsafe extern \"C\" fn ").or_else(|| l.trim().strip_prefix("pub extern \"C\" fn ")))
        .filter_map(|l| l.split('(').next())
        .filter(|n| !n.starts_with('$')) // the endpoint! macro's own template line
        .map(str::to_string)
        .collect();
    exported.extend(
        src.lines()
            .filter_map(|l| l.trim().strip_prefix("endpoint!("))
            .filter_map(|l| l.split(',').next())
            .map(str::to_string),
    );
    exported.sort();
    exported.dedup();
    assert!(exported.len() >= 12, "found only {exported:?}");

    // Any top-level declaration of a lagn_* function, whatever its return type.
    let declared: Vec<String> = header
        .lines()
        .filter(|l| !l.starts_with(' ') && !l.starts_with('*') && l.contains("lagn_") && l.trim_end().ends_with(';'))
        .filter_map(|l| l.split('(').next())
        .map(|l| l.rsplit(['*', ' ']).next().unwrap().to_string())
        .filter(|n| n.starts_with("lagn_"))
        .collect();
    assert!(declared.len() >= 12, "the header parser found only {declared:?}");

    for name in &exported {
        assert!(declared.contains(name), "{name} is exported but not declared in include/lagn.h");
    }
    for name in &declared {
        assert!(exported.contains(name), "{name} is declared in include/lagn.h but not exported");
    }
}
