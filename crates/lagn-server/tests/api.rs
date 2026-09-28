//! Phase 4 QA: the HTTP layer, driven in-process.
//! Specification: docs/phase4/DESIGN.md section 7.

use std::sync::{Arc, OnceLock};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use lagn_core::*;
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo};
use lagn_server::{router, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

const TOKEN: &str = "qa-review-token-0123456789abcdef";

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn state(token: Option<&str>) -> Arc<AppState> {
    static EPHE: OnceLock<()> = OnceLock::new();
    EPHE.get_or_init(|| Ephemeris::set_ephemeris_path(root().join("ephe")).unwrap());
    Arc::new(AppState::load(&root().join("corpus"), &root().join("data/places.tsv"), Some(&root().join("data/zoneinfo")), token.map(String::from)).unwrap())
}

fn app(token: Option<&str>) -> axum::Router {
    static WITH: OnceLock<Arc<AppState>> = OnceLock::new();
    static WITHOUT: OnceLock<Arc<AppState>> = OnceLock::new();
    let s = match token {
        Some(_) => WITH.get_or_init(|| state(Some(TOKEN))).clone(),
        None => WITHOUT.get_or_init(|| state(None)).clone(),
    };
    router(s, None)
}

async fn call(app: axum::Router, req: Request<Body>) -> (StatusCode, Value, String) {
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).to_string();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null), text)
}

fn post(path: &str, body: &Value, token: Option<&str>) -> Request<Body> {
    let mut b = Request::post(path).header("content-type", "application/json");
    if let Some(t) = token {
        b = b.header("x-review-token", t);
    }
    b.body(Body::from(body.to_string())).unwrap()
}

fn get(path: &str) -> Request<Body> {
    Request::get(path).body(Body::empty()).unwrap()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { let mut x = self.0; x ^= x >> 12; x ^= x << 25; x ^= x >> 27; self.0 = x; x.wrapping_mul(0x2545_F491_4F6C_DD1D) }
    fn int(&mut self, lo: i64, hi: i64) -> i64 { lo + (self.next() % ((hi - lo + 1) as u64)) as i64 }
    fn unit(&mut self) -> f64 { (self.next() >> 11) as f64 / (1u64 << 53) as f64 }
}

fn random_birth(r: &mut Rng) -> Value {
    let offset = [5.5, 0.0, -5.0, 9.0, 5.352778][r.int(0, 4) as usize];
    json!({
        "date": format!("{:04}-{:02}-{:02}", r.int(1300, 2390), r.int(1, 12), r.int(1, 28)),
        "time": format!("{:02}:{:02}:{:02}", r.int(0, 23), r.int(0, 59), r.int(0, 59)),
        "latitude": r.unit() * 170.0 - 85.0,
        "longitude": r.unit() * 360.0 - 180.0,
        "utc_offset_hours": offset,
    })
}

/// The same birth computed directly by the library, bypassing HTTP.
fn library_chart(b: &Value) -> Chart {
    let d: Vec<i32> = b["date"].as_str().unwrap().split('-').map(|x| x.parse().unwrap()).collect();
    let t: Vec<u32> = b["time"].as_str().unwrap().split(':').map(|x| x.parse().unwrap()).collect();
    Chart::compute(BirthData {
        moment: BirthMoment {
            year: d[0], month: d[1] as u32, day: d[2] as u32, hour: t[0], minute: t[1], second: t[2] as f64,
            utc_offset_hours: b["utc_offset_hours"].as_f64().unwrap(),
        },
        latitude: b["latitude"].as_f64().unwrap(),
        longitude: b["longitude"].as_f64().unwrap(),
        place_name: String::new(),
    }, ChartSettings::default()).unwrap()
}

// ===========================================================================
// Criterion 1: parity with the verified library
// ===========================================================================

#[tokio::test]
async fn chart_endpoint_raw_output_equals_the_library_bit_for_bit() {
    let mut r = Rng(0xAB1_0001);
    let settings = DerivationSettings::default();
    let mut compared = 0;
    for _ in 0..1000 {
        let b = random_birth(&mut r);
        let (st, v, text) = call(app(None), post("/api/chart", &json!({ "birth": b }), None)).await;
        assert_eq!(st, StatusCode::OK, "{text}");
        let c = library_chart(&b);
        let raw = &v["raw"];
        assert_eq!(raw["chart"], serde_json::to_value(&c).unwrap(), "chart differs for {b}");
        assert_eq!(raw["analysis"], serde_json::to_value(c.analyse(&settings)).unwrap(), "analysis differs for {b}");
        assert_eq!(raw["ashtakavarga"], serde_json::to_value(Ashtakavarga::compute(&c)).unwrap(), "ashtakavarga differs");
        assert_eq!(raw["vimshottari"], serde_json::to_value(Vimshottari::compute(&c)).unwrap(), "dasha differs");
        // Display fields must agree with the raw values they describe.
        for (p, pv) in c.placements.iter().zip(v["positions"].as_array().unwrap()) {
            assert_eq!(pv["degrees"], lagn_core::format::dms(p.degrees_in_rasi));
            assert_eq!(pv["rasi"], p.rasi.name());
            assert_eq!(pv["house"], p.house);
            assert_eq!(pv["pada"], p.nakshatra.pada);
        }
        for (vv, varga) in v["vargas"].as_array().unwrap().iter().zip(Varga::ALL) {
            let vc = c.varga(varga);
            for (cell, g) in vv["grahas"].as_array().unwrap().iter().zip(Graha::ALL) {
                assert_eq!(cell["sign_index"], vc.sign_of(g).index());
                assert_eq!(cell["house"], vc.house_of(g));
            }
        }
        let av = Ashtakavarga::compute(&c);
        for h in 1..=12u8 {
            assert_eq!(v["ashtakavarga"]["sav_by_house"][(h - 1) as usize], av.sav_in_house(h));
        }
        compared += 1;
    }
    assert_eq!(compared, 1000);
}

#[tokio::test]
async fn dasha_display_dates_come_from_the_engine() {
    let mut r = Rng(0xAB1_0002);
    for _ in 0..100 {
        let b = random_birth(&mut r);
        let (_, v, _) = call(app(None), post("/api/chart", &json!({ "birth": b }), None)).await;
        let c = library_chart(&b);
        let tz = c.birth.moment.utc_offset_hours;
        for m in v["dasha"]["mahadashas"].as_array().unwrap() {
            let (s, e) = (m["start_jd"].as_f64().unwrap(), m["end_jd"].as_f64().unwrap());
            let d = |jd: f64| { let x = jd_to_civil(jd, tz); format!("{:04}-{:02}-{:02}", x.year, x.month, x.day) };
            assert_eq!(m["start"], d(s));
            assert_eq!(m["end"], d(e));
            assert!(s >= c.jd_ut - 1e-9, "a period starts before birth in the display");
        }
        // At most one current period per level, and it lies on one chain.
        let cur: Vec<&Value> = v["dasha"]["mahadashas"].as_array().unwrap().iter().filter(|m| m["current"] == true).collect();
        assert!(cur.len() <= 1);
    }
}

#[tokio::test]
async fn topic_and_match_endpoints_equal_the_library() {
    let mut r = Rng(0xAB1_0003);
    let corpus = lagn_rules::load(&root().join("corpus")).unwrap();
    for i in 0..150 {
        let b = random_birth(&mut r);
        let sex = ["female", "male"][i % 2];
        let (st, v, text) = call(app(Some(TOKEN)), post("/api/topic/marriage",
            &json!({ "birth": b, "sex": sex, "from_age": 10, "to_age": 70, "mode": "review" }), Some(TOKEN))).await;
        assert_eq!(st, StatusCode::OK, "{text}");
        let facts = FactBase::new(library_chart(&b), DerivationSettings::default(),
            NativeInfo { sex: Some(if sex == "female" { lagn_rules::Sex::Female } else { lagn_rules::Sex::Male }) });
        let want = evaluate_topic("marriage", &corpus.rules, &facts, Mode::Review, (10.0, 70.0));
        assert_eq!(v["report"], serde_json::to_value(&want).unwrap(), "topic report differs for {b}");

        let g = random_birth(&mut r);
        let (st, mv, _) = call(app(Some(TOKEN)), post("/api/match", &json!({ "bride": b, "groom": g, "mode": "review" }), Some(TOKEN))).await;
        assert_eq!(st, StatusCode::OK);
        let (cb, cg) = (library_chart(&b), library_chart(&g));
        let star = |c: &Chart| lagn_rules::StarPos { nakshatra: c.janma_nakshatra().nakshatra, rasi: c.janma_rasi() };
        let want = lagn_rules::report::match_report(&corpus, star(&cb), star(&cg), Mode::Review);
        assert_eq!(mv, serde_json::to_value(&want).unwrap());
    }
}

// ===========================================================================
// Criterion 2: the gate at the boundary
// ===========================================================================

#[tokio::test]
async fn review_mode_is_impossible_without_the_exact_token() {
    let b = random_birth(&mut Rng(9));
    let topic = json!({ "birth": b, "mode": "review" });
    let m = json!({ "bride": b, "groom": b, "mode": "review" });
    let upper = TOKEN.to_uppercase();
    let prefix = &TOKEN[..TOKEN.len() - 1];
    let longer = format!("{TOKEN}x");
    let bad: [Option<&str>; 6] = [None, Some(""), Some("wrong"), Some(upper.as_str()), Some(prefix), Some(longer.as_str())];

    for tok in bad {
        for (path, body) in [("/api/topic/marriage", &topic), ("/api/match", &m)] {
            let (st, v, _) = call(app(Some(TOKEN)), post(path, body, tok)).await;
            assert_eq!(st, StatusCode::FORBIDDEN, "{path} with token {tok:?}");
            assert!(v["error"].is_string());
        }
        let mut req = Request::get("/api/review-sheet?topic=marriage");
        if let Some(t) = tok { req = req.header("x-review-token", t); }
        let (st, _, _) = call(app(Some(TOKEN)), req.body(Body::empty()).unwrap()).await;
        assert_eq!(st, StatusCode::FORBIDDEN, "review sheet with token {tok:?}");
    }
    // A server started without a token refuses review mode even for the "right" token.
    let (st, _, _) = call(app(None), post("/api/topic/marriage", &topic, Some(TOKEN))).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    // With the token, review mode works and shows drafts.
    let (st, v, _) = call(app(Some(TOKEN)), post("/api/topic/marriage", &topic, Some(TOKEN))).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["report"]["mode"], "review");
    assert!(!v["report"]["results"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn production_mode_returns_only_approved_content_whatever_headers_are_sent() {
    let b = random_birth(&mut Rng(10));
    for tok in [None, Some(TOKEN), Some("junk")] {
        let (st, v, _) = call(app(Some(TOKEN)), post("/api/topic/marriage", &json!({ "birth": b }), tok)).await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(v["report"]["mode"], "production");
        for r in v["report"]["results"].as_array().unwrap() {
            assert_eq!(r["status"], "approved", "unapproved rule in production");
        }
        let (_, mv, _) = call(app(Some(TOKEN)), post("/api/match", &json!({ "bride": b, "groom": b }), tok)).await;
        for r in mv["results"].as_array().unwrap() {
            assert_eq!(r[1], "approved", "unapproved porutham in production");
        }
    }
}

#[test]
fn weak_review_tokens_are_refused_at_startup() {
    let r = AppState::load(&root().join("corpus"), &root().join("data/places.tsv"), None, Some("short".into()));
    assert!(r.is_err());
}

// ===========================================================================
// Criterion 3 (in part): offsets pinned from the system tz database (2026b)
// ===========================================================================

async fn offset(q: &str) -> (StatusCode, Value) {
    let (s, v, _) = call(app(None), get(&format!("/api/offset?{q}"))).await;
    (s, v)
}

#[tokio::test]
async fn asia_kolkata_history_matches_the_tz_database() {
    // Expected values read from the system tz database with Python zoneinfo
    // during design (docs/phase4/DESIGN.md criterion 3), not from memory.
    let cases = [
        ("1860-01-01", "12:00", "+05:53:20"), // HMT
        ("1890-03-14", "04:22", "+05:21:10"), // MMT
        ("1906-01-02", "12:00", "+05:30"),    // IST
        ("1941-12-01", "12:00", "+06:30"),    // wartime
        ("1942-06-01", "12:00", "+05:30"),    // the 1942 interlude
        ("1943-05-05", "14:00", "+06:30"),    // wartime again
        ("1945-10-16", "12:00", "+05:30"),
        ("1985-06-21", "14:30", "+05:30"),
    ];
    for (date, time, want) in cases {
        let (st, v) = offset(&format!("tz=Asia/Kolkata&date={date}&time={time}")).await;
        assert_eq!(st, StatusCode::OK);
        let texts: Vec<&str> = v["candidates"].as_array().unwrap().iter().map(|c| c["offset_text"].as_str().unwrap()).collect();
        assert!(texts.contains(&want), "{date} {time}: got {texts:?}, want {want}");
        let conf = if date >= "1970-01-01" { "reliable" } else { "historical" };
        assert_eq!(v["confidence"], conf, "{date}");
    }
}

#[tokio::test]
async fn pre_standard_time_suggests_the_birth_places_own_mean_time_first() {
    // Before 1854 the tz database gives Kolkata's LMT for all of India. For a
    // Madurai birth the birth place's own mean time must come first.
    let (_, v) = offset("tz=Asia/Kolkata&date=1850-03-01&time=06:00&longitude=78.1195").await;
    assert_eq!(v["kind"], "local_mean_time");
    let c = v["candidates"].as_array().unwrap();
    // 78.1195 degrees x 4 minutes = 312.478 min = 5h12m28.68s -> rounds to 29 s.
    assert_eq!(c[0]["offset_text"], "+05:12:29");
    assert_eq!(c[1]["offset_text"], "+05:53:28");
    // Pre-1582: never consults the (Gregorian) tz library; 1500-02-29 is a valid Julian date.
    let (st, v) = offset("tz=Asia/Kolkata&date=1500-02-29&time=06:00&longitude=78.1195").await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["candidates"][0]["offset_text"], "+05:12:29");
    // ...and without a longitude it cannot answer.
    let (st, _) = offset("tz=Asia/Kolkata&date=1500-02-28&time=06:00").await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn dst_gaps_and_overlaps_are_surfaced_not_guessed() {
    let (_, v) = offset("tz=America/New_York&date=2021-03-14&time=02:30").await;
    assert_eq!(v["kind"], "nonexistent");
    assert!(v["candidates"].as_array().unwrap().is_empty());
    let (_, v) = offset("tz=America/New_York&date=2021-11-07&time=01:30").await;
    assert_eq!(v["kind"], "ambiguous");
    let t: Vec<&str> = v["candidates"].as_array().unwrap().iter().map(|c| c["offset_text"].as_str().unwrap()).collect();
    assert_eq!(t, ["-04:00", "-05:00"]);
}

#[tokio::test]
async fn offset_rejects_bad_input() {
    for q in ["tz=Mars/Olympus&date=2000-01-01&time=12:00", "tz=Asia/Kolkata&date=1985-02-30&time=12:00",
              "tz=Asia/Kolkata&date=1582-10-10&time=12:00", "tz=Asia/Kolkata&date=2000-01-01&time=24:00",
              "tz=Asia/Kolkata&date=2000-1-1&time=12:00", "tz=Asia/Kolkata&date=2000-01-01", "",
              "tz=Asia/Kolkata&date=1500-01-01&time=12:00&longitude=200"] {
        let (st, v) = offset(q).await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{q:?} -> {v}");
        assert!(v["error"].is_string(), "{q:?}: error body must be JSON");
    }
}

// ===========================================================================
// Criterion 4: places
// ===========================================================================

#[tokio::test]
async fn historical_names_resolve_to_the_modern_city() {
    // Regression QA-P4-2: "Calcutta" finds Kolkata before Calcutta, South Africa.
    for (old, new) in [("Madras", "Chennai"), ("Bombay", "Mumbai"), ("Calcutta", "Kolkata"),
                       ("Trivandrum", "Thiruvananthapuram"), ("Cochin", "Kochi")] {
        let (st, v, _) = call(app(None), get(&format!("/api/places?q={old}&limit=1"))).await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(v[0]["name"], new, "{old}");
        assert_eq!(v[0]["matched_alias"], old);
        assert_eq!(v[0]["timezone"], "Asia/Kolkata");
    }
}

#[test]
fn place_index_is_internally_valid() {
    let idx = lagn_server::places::PlaceIndex::load(&root().join("data/places.tsv")).unwrap();
    assert!(idx.len() > 30_000);
    // Every Indian place: inside India's bounding box and on Indian time.
    let text = std::fs::read_to_string(root().join("data/places.tsv")).unwrap();
    let mut indian = 0;
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        assert!(jiff::tz::TimeZone::get(f[8]).is_ok(), "unknown zone {} for {}", f[8], f[1]);
        if f[4] == "IN" {
            indian += 1;
            let (lat, lon): (f64, f64) = (f[6].parse().unwrap(), f[7].parse().unwrap());
            assert!((6.0..=37.5).contains(&lat) && (68.0..=97.5).contains(&lon), "{} at {lat},{lon}", f[1]);
            assert_eq!(f[8], "Asia/Kolkata", "{}", f[1]);
        }
    }
    assert!(indian > 3000);
}

#[tokio::test]
async fn search_is_deterministic_ranked_and_bounded() {
    let (_, a, _) = call(app(None), get("/api/places?q=mad&limit=10")).await;
    let (_, b, _) = call(app(None), get("/api/places?q=mad&limit=10")).await;
    assert_eq!(a, b);
    assert_eq!(a.as_array().unwrap().len(), 10);
    // No exact "mad" place exists, so all ten are prefix hits: population order.
    let pops: Vec<u64> = a.as_array().unwrap().iter().map(|p| p["population"].as_u64().unwrap()).collect();
    assert!(pops.windows(2).all(|w| w[0] >= w[1]), "prefix hits must be in population order: {pops:?}");
    // Regression QA-P4-1: airport codes are not names. "MAD" must not surface Madrid as an alias hit.
    assert!(a.as_array().unwrap().iter().all(|p| p["matched_alias"] != "MAD"), "IATA code matched as a name");
    let names: Vec<&str> = a.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Madurai"), "{names:?}");
    let (_, v, _) = call(app(None), get("/api/places?q=chennai&limit=500")).await;
    assert!(v.as_array().unwrap().len() <= 50, "limit must be capped");
    let (st, _, _) = call(app(None), get(&format!("/api/places?q={}", "a".repeat(101)))).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    let (st, v, _) = call(app(None), get("/api/places?q=%E0%AE%9A%E0%AF%86%E0%AE%A9%E0%AF%8D%E0%AE%A9%E0%AF%88")).await; // Tamil script
    assert_eq!(st, StatusCode::OK);
    assert!(v.is_array());
}

// ===========================================================================
// Criterion 5: robustness
// ===========================================================================

#[tokio::test]
async fn malformed_and_random_payloads_never_crash_and_always_get_4xx() {
    let mut r = Rng(0xF022);
    let valid = json!({ "birth": random_birth(&mut r) });
    let base = valid.to_string();
    let mut cases: Vec<String> = vec![
        "".into(), "{".into(), "null".into(), "[]".into(), "\"x\"".into(), "{}".into(),
        r#"{"birth":{}}"#.into(), r#"{"birth":null}"#.into(),
        base.replace("\"date\"", "\"dat\""), base.replace("latitude", "lat"),
        base[..base.len() / 2].to_string(),
        r#"{"birth":{"date":"1985-06-21","time":"14:30","latitude":1e999,"longitude":80,"utc_offset_hours":5.5}}"#.into(),
        r#"{"birth":{"date":"1985-06-21","time":"14:30","latitude":"13","longitude":80,"utc_offset_hours":5.5}}"#.into(),
        r#"{"birth":{"date":"1985-06-21","time":"14:30","latitude":95,"longitude":80,"utc_offset_hours":5.5}}"#.into(),
        r#"{"birth":{"date":"1985-06-21","time":"14:30","latitude":90,"longitude":80,"utc_offset_hours":5.5}}"#.into(),
        r#"{"birth":{"date":"1985-06-21","time":"14:30","latitude":13,"longitude":80,"utc_offset_hours":99}}"#.into(),
        r#"{"birth":{"date":"0985-06-21","time":"14:30","latitude":13,"longitude":80,"utc_offset_hours":5.5}}"#.into(),
        r#"{"birth":{"date":"1985-06-21","time":"2:30","latitude":13,"longitude":80,"utc_offset_hours":5.5}}"#.into(),
    ];
    // Random byte mutations of a valid body.
    for _ in 0..400 {
        let mut bytes = base.clone().into_bytes();
        for _ in 0..r.int(1, 6) {
            let i = r.int(0, bytes.len() as i64 - 1) as usize;
            bytes[i] = (r.next() % 256) as u8;
        }
        cases.push(String::from_utf8_lossy(&bytes).to_string());
    }
    for body in &cases {
        for path in ["/api/chart", "/api/topic/marriage", "/api/match"] {
            let req = Request::post(path).header("content-type", "application/json").body(Body::from(body.clone())).unwrap();
            let (st, v, text) = call(app(None), req).await;
            assert!(st == StatusCode::OK || st.is_client_error(), "{path} {body:?} -> {st} {text}");
            if st.is_client_error() {
                assert!(v["error"].is_string(), "{path}: non-JSON error body {text:?}");
            }
        }
    }
}

#[tokio::test]
async fn oversized_bodies_get_a_json_413() {
    let big = format!(r#"{{"x":"{}"}}"#, "a".repeat(lagn_server::BODY_LIMIT + 10));
    let req = Request::post("/api/chart").header("content-type", "application/json").body(Body::from(big)).unwrap();
    let (st, v, _) = call(app(None), req).await;
    assert_eq!(st, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn unknown_endpoints_and_topics_are_json_404s() {
    let (st, v, _) = call(app(None), get("/api/nope")).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert!(v["error"].is_string());
    let (st, _, _) = call(app(None), post("/api/topic/cooking", &json!({ "birth": random_birth(&mut Rng(3)) }), None)).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn concurrent_requests_agree_with_sequential_ones() {
    let mut r = Rng(0xC0C0);
    let births: Vec<Value> = (0..40).map(|_| random_birth(&mut r)).collect();
    let mut sequential = Vec::new();
    for b in &births {
        let (_, v, _) = call(app(None), post("/api/chart", &json!({ "birth": b }), None)).await;
        sequential.push(v["raw"].clone());
    }
    let handles: Vec<_> = births.iter().map(|b| {
        let body = json!({ "birth": b });
        tokio::spawn(async move { call(app(None), post("/api/chart", &body, None)).await.1["raw"].clone() })
    }).collect();
    for (h, want) in handles.into_iter().zip(sequential) {
        assert_eq!(h.await.unwrap(), want, "concurrent result differs from sequential");
    }
}

#[tokio::test]
async fn version_reports_the_stack() {
    let (st, v, _) = call(app(None), get("/api/version")).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["swiss_ephemeris"], "2.10.03");
    assert_eq!(v["review_enabled"], false);
    let shipped = lagn_rules::load(&root().join("corpus")).unwrap().rules.len();
    assert_eq!(v["corpus"]["rules"], shipped);
    assert!(v["attribution"].as_str().unwrap().contains("GeoNames"));
}

#[test]
fn no_alternate_name_is_an_airport_code() {
    let text = std::fs::read_to_string(root().join("data/places.tsv")).unwrap();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        for a in f[3].split('|').filter(|a| !a.is_empty()) {
            assert!(!(a.chars().all(|c| c.is_ascii_uppercase()) && a.len() <= 4), "{} has code-like alias {a}", f[1]);
        }
    }
}

#[tokio::test]
async fn tz_data_includes_the_2025c_baja_california_correction() {
    // Regression QA-P4-3. tzdb 2025c corrected Baja California to follow
    // California's DST in 1953 and 1961-75. The first design compiled in
    // tzdb 2025b and gave -08:00 here. Expected value read from the system tz
    // database (2026b) with Python zoneinfo. If this fails, the host's tzdata
    // is stale: update the OS tzdata package (DESIGN.md section 3).
    let (st, v) = offset("tz=America/Tijuana&date=1973-05-18&time=23:19:54").await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["candidates"][0]["offset_text"], "-07:00", "stale tz data: {}", v["tzdb_version"]);
    let (_, v) = offset("tz=America/Tijuana&date=1953-06-01&time=12:00").await;
    assert_eq!(v["candidates"][0]["offset_text"], "-07:00", "stale tz data: {}", v["tzdb_version"]);
}

#[tokio::test]
async fn the_newest_tz_database_is_used_and_reported() {
    let (_, v, _) = call(app(None), get("/api/version")).await;
    let shipped = std::fs::read_to_string(root().join("data/zoneinfo/+VERSION")).unwrap().trim().to_string();
    let considered = v["tzdb_considered"].as_array().unwrap();
    assert!(considered.len() >= 2, "{considered:?}");
    let newest = considered.iter().filter_map(|c| c["version"].as_str()).max().unwrap();
    assert!(v["tzdb"].as_str().unwrap().starts_with(newest), "in use {} but newest considered is {newest}", v["tzdb"]);
    assert!(newest >= shipped.as_str());
}

#[tokio::test]
async fn tz_data_includes_the_2026d_corrections() {
    // IANA 2026d: Iran's 1979-05-26 spring forward was at 00:00, not 24:00.
    // So 1979-05-26 00:30 local did not exist. (Expected from the 2026d
    // release notes; earlier databases treat 00:30 as valid +03:30.)
    let (_, v) = offset("tz=Asia/Tehran&date=1979-05-26&time=00:30").await;
    assert_eq!(v["kind"], "nonexistent", "stale tz data: {}", v["tzdb_version"]);
}

// ===========================================================================
// Phase 5: CORS for wrapped apps, and security headers
// ===========================================================================

fn app_cors(origins: &[&str]) -> axum::Router {
    let cors = lagn_server::Cors::parse(&origins.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap();
    lagn_server::router_with(state(Some(TOKEN)), None, &cors)
}

async fn raw(app: axum::Router, req: Request<Body>) -> axum::http::Response<axum::body::Body> {
    app.oneshot(req).await.unwrap()
}

#[tokio::test]
async fn no_cors_headers_unless_origins_are_configured() {
    let req = Request::get("/api/health").header("origin", "capacitor://localhost").body(Body::empty()).unwrap();
    let res = raw(app(None), req).await;
    assert!(res.headers().get("access-control-allow-origin").is_none(), "same-origin by default");
}

#[tokio::test]
async fn listed_origins_are_allowed_and_others_are_not() {
    let app = app_cors(&["capacitor://localhost", "https://localhost"]);
    for origin in ["capacitor://localhost", "https://localhost"] {
        let req = Request::get("/api/health").header("origin", origin).body(Body::empty()).unwrap();
        let res = raw(app.clone(), req).await;
        assert_eq!(res.headers().get("access-control-allow-origin").unwrap(), origin);
    }
    for origin in ["https://evil.example", "http://localhost", "capacitor://localhost.evil", "null"] {
        let req = Request::get("/api/health").header("origin", origin).body(Body::empty()).unwrap();
        let res = raw(app.clone(), req).await;
        assert!(res.headers().get("access-control-allow-origin").is_none(), "{origin} was allowed");
    }
}

#[tokio::test]
async fn preflight_allows_the_review_token_header_only_for_listed_origins() {
    let app = app_cors(&["capacitor://localhost"]);
    let pre = |origin: &str| Request::builder().method("OPTIONS").uri("/api/topic/marriage")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type,x-review-token")
        .body(Body::empty()).unwrap();
    let ok = raw(app.clone(), pre("capacitor://localhost")).await;
    let allowed = ok.headers().get("access-control-allow-headers").map(|v| v.to_str().unwrap().to_lowercase()).unwrap_or_default();
    assert!(allowed.contains("x-review-token") && allowed.contains("content-type"), "{allowed}");
    assert_eq!(ok.headers().get("access-control-allow-origin").unwrap(), "capacitor://localhost");
    let bad = raw(app, pre("https://evil.example")).await;
    assert!(bad.headers().get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn cross_origin_requests_still_obey_the_review_gate() {
    // CORS lets a listed origin *call* the API; it never lets anyone skip the token.
    let app = app_cors(&["capacitor://localhost"]);
    let body = json!({ "birth": random_birth(&mut Rng(77)), "mode": "review" });
    let req = Request::post("/api/topic/marriage").header("origin", "capacitor://localhost")
        .header("content-type", "application/json").body(Body::from(body.to_string())).unwrap();
    let (st, _, _) = call(app, req).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
}

#[test]
fn malformed_allow_origins_are_refused_at_startup() {
    for bad in ["*", "https://*.example.com", "localhost", "https://example.com/path", "ftp://example.com", "https://"] {
        assert!(lagn_server::Cors::parse(&[bad.to_string()]).is_err(), "{bad} accepted");
    }
    assert!(lagn_server::Cors::parse(&["https://app.example.com:8443".into(), "capacitor://localhost".into()]).is_ok());
}

#[tokio::test]
async fn every_response_carries_the_security_headers() {
    for req in [get("/api/health"), get("/api/nope"), post("/api/chart", &json!({}), None)] {
        let res = raw(app(None), req).await;
        let h = res.headers();
        assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(h.get("referrer-policy").unwrap(), "no-referrer");
        let csp = h.get("content-security-policy").unwrap().to_str().unwrap();
        for d in ["default-src 'self'", "script-src 'self'", "frame-ancestors 'none'", "object-src 'none'"] {
            assert!(csp.contains(d), "CSP missing {d}: {csp}");
        }
    }
}

#[tokio::test]
async fn static_files_never_vary_by_origin_even_with_cors_enabled() {
    // Regression QA-P5-1: with --allow-origin set, the CORS layer stamped the
    // web app's files with `Vary: origin`, the service worker's precached
    // copies stopped matching, and the app opened to a blank screen offline.
    let dist = root().join("web/dist");
    if !dist.join("index.html").exists() {
        eprintln!("web/dist not built; skipping");
        return;
    }
    let cors = lagn_server::Cors::parse(&["capacitor://localhost".to_string()]).unwrap();
    let app = lagn_server::router_with(state(None), Some(&dist), &cors);
    for path in ["/", "/index.html", "/manifest.webmanifest", "/sw.js"] {
        let req = Request::get(path).header("origin", "capacitor://localhost").body(Body::empty()).unwrap();
        let res = raw(app.clone(), req).await;
        assert_eq!(res.status(), StatusCode::OK, "{path}");
        assert!(res.headers().get("vary").is_none(), "{path} varies: {:?}", res.headers().get("vary"));
        assert!(res.headers().get("access-control-allow-origin").is_none(), "{path} has CORS headers");
    }
    // The API still gets CORS.
    let req = Request::get("/api/health").header("origin", "capacitor://localhost").body(Body::empty()).unwrap();
    assert!(raw(app, req).await.headers().get("access-control-allow-origin").is_some());
}

// ===========================================================================
// Phase 6: topics catalogue, sensitive periods, pariharams
// ===========================================================================

fn modern_birth(r: &mut Rng) -> Value {
    // Transits need the ephemeris over the whole age range, so stay well
    // inside the shipped files.
    json!({
        "date": format!("{:04}-{:02}-{:02}", r.int(1900, 2020), r.int(1, 12), r.int(1, 28)),
        "time": format!("{:02}:{:02}:00", r.int(0, 23), r.int(0, 59)),
        "latitude": 8.0 + r.unit() * 12.0,
        "longitude": 74.0 + r.unit() * 10.0,
        "utc_offset_hours": 5.5,
    })
}

#[tokio::test]
async fn topics_lists_every_answerable_topic_with_its_disclaimers() {
    let (st, v, text) = call(app(None), get("/api/topics")).await;
    assert_eq!(st, StatusCode::OK, "{text}");
    let ids: Vec<&str> = v["topics"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap()).collect();
    for want in ["marriage", "career", "wealth", "education", "health", "vitality", "progeny", "parents", "later_life"] {
        assert!(ids.contains(&want), "{want} missing from {ids:?}");
    }
    assert!(!ids.contains(&"periods"), "period rules are not a topic");
    for t in v["topics"].as_array().unwrap() {
        if ["health", "vitality"].contains(&t["id"].as_str().unwrap()) {
            assert!(t["disclaimer"].as_str().unwrap().contains("doctor"));
        }
    }
    assert_eq!(v["bhavas"].as_array().unwrap().len(), 12);
    // Every listed topic answers.
    let mut r = Rng(0x6_0001);
    let b = modern_birth(&mut r);
    for id in &ids {
        let (st, tv, text) = call(app(None), post(&format!("/api/topic/{id}"), &json!({ "birth": b }), None)).await;
        assert_eq!(st, StatusCode::OK, "{id}: {text}");
        assert_eq!(tv["meta"]["id"], *id);
    }
    let (st, _, _) = call(app(None), post("/api/topic/periods", &json!({ "birth": b }), None)).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "period rules must not be served as a topic");
}

#[tokio::test]
async fn topic_ages_default_to_the_catalogue_and_pariharams_follow_the_findings() {
    let corpus = lagn_rules::load(&root().join("corpus")).unwrap();
    let mut r = Rng(0x6_0002);
    let mut suggested = 0;
    for _ in 0..40 {
        let b = modern_birth(&mut r);
        for id in ["career", "progeny", "marriage", "health"] {
            let meta = corpus.topics.iter().find(|t| t.id == id).unwrap();
            let (st, v, text) = call(app(None), post(&format!("/api/topic/{id}"), &json!({ "birth": b }), None)).await;
            assert_eq!(st, StatusCode::OK, "{text}");
            let facts = FactBase::new(library_chart(&b), DerivationSettings::default(), NativeInfo::default());
            let want = evaluate_topic(id, &corpus.rules, &facts, Mode::Production, (meta.ages[0], meta.ages[1]));
            assert_eq!(v["report"], serde_json::to_value(&want).unwrap(), "{id}: default ages not applied");
            let wantw = lagn_rules::writeup::write_up(&corpus, meta, &meta.focus, &want, &facts, (meta.ages[0], meta.ages[1]), |_| true);
            assert_eq!(v["writeup"], serde_json::to_value(&wantw).unwrap(), "{id}: write-up differs from the library");
            let wantp = lagn_rules::pariharam::suggest(&corpus, &want.results, &[], Mode::Production);
            assert_eq!(v["pariharams"], serde_json::to_value(&wantp).unwrap());
            suggested += wantp.len();
            // Every suggestion is explained by an effective afflicting finding.
            for p in v["pariharams"].as_array().unwrap() {
                assert!(!p["because"].as_array().unwrap().is_empty());
            }
        }
    }
    assert!(suggested > 0, "no pariharam was ever suggested; the path is untested");
}

#[tokio::test]
async fn periods_endpoint_equals_the_library_and_is_internally_consistent() {
    let corpus = lagn_rules::load(&root().join("corpus")).unwrap();
    let mut r = Rng(0x6_0003);
    for _ in 0..12 {
        let b = modern_birth(&mut r);
        let body = json!({ "birth": b, "from_age": 20, "to_age": 60 });
        let (st, v, text) = call(app(None), post("/api/periods", &body, None)).await;
        assert_eq!(st, StatusCode::OK, "{text}");
        let c = library_chart(&b);
        let t = lagn_rules::reading::transits(&c, 20.0, 60.0).unwrap();
        let mut facts = FactBase::new(c, DerivationSettings::default(), NativeInfo::default());
        let want = lagn_rules::reading::sensitive_periods(&corpus, &mut facts, Mode::Production, (20.0, 60.0), &t.windows);
        let windows = v["windows"].as_array().unwrap();
        assert_eq!(windows.len(), want.windows.len());
        for (got, w) in windows.iter().zip(&want.windows) {
            let mut wv = serde_json::to_value(w).unwrap();
            let mut gv = got.clone();
            for k in ["start", "end", "maha_name", "antar_name", "current"] {
                gv.as_object_mut().unwrap().remove(k);
                wv.as_object_mut().unwrap().remove(k);
            }
            assert_eq!(gv, wv);
            // Consistency: sensitivity follows the score; every pressure overlaps the window.
            assert_eq!(w.window.sensitive, w.window.score <= lagn_rules::resolve::SENSITIVE_AT);
            for p in w.pressures.iter().chain(&w.supports) {
                assert!(p.start_jd >= w.window.start_jd && p.end_jd <= w.window.end_jd && p.start_jd < p.end_jd);
            }
            assert!(!w.focus.is_empty() && w.focus.iter().all(|f| (1..=12).contains(&f.house) && !f.significations.is_empty()));
            assert!(!w.explanation.is_empty());
        }
        // Windows tile the range without gaps.
        for pair in want.windows.windows(2) {
            assert!((pair[0].window.end_jd - pair[1].window.start_jd).abs() < 1e-9);
        }
        assert!(windows.iter().filter(|w| w["current"] == true).count() <= 1);
    }
    let mut r = Rng(0x6_0004);
    let b = modern_birth(&mut r);
    for bad in [json!({ "birth": b, "from_age": 50, "to_age": 40 }), json!({ "birth": b, "to_age": 500 }), json!({ "birth": b, "extra": 1 })] {
        let (st, _, _) = call(app(None), post("/api/periods", &bad, None)).await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{bad}");
    }
    let (st, _, _) = call(app(None), post("/api/periods", &json!({ "birth": b, "mode": "review" }), None)).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "review mode without a token");
}

#[tokio::test]
async fn family_endpoint_equals_the_library_and_rejects_bad_input() {
    use lagn_rules::family::{family_reading, Relation};
    let corpus = lagn_rules::load(&root().join("corpus")).unwrap();
    let mut r = Rng(0x6_0008);
    for (i, (rel, name)) in [(Relation::Spouse, "spouse"), (Relation::Child, "child"), (Relation::Mother, "mother"), (Relation::Father, "father")]
        .into_iter()
        .cycle()
        .take(24)
        .enumerate()
    {
        let (nb, mb) = (modern_birth(&mut r), modern_birth(&mut r));
        let sex = ["female", "male"][i % 2];
        let body = json!({ "native": { "birth": nb, "sex": sex }, "member": { "birth": mb }, "relation": name });
        let (st, v, text) = call(app(None), post("/api/family", &body, None)).await;
        assert_eq!(st, StatusCode::OK, "{text}");
        let native = FactBase::new(library_chart(&nb), DerivationSettings::default(),
            NativeInfo { sex: Some(if sex == "female" { lagn_rules::Sex::Female } else { lagn_rules::Sex::Male }) });
        let member = FactBase::new(library_chart(&mb), DerivationSettings::default(), NativeInfo::default());
        let want = family_reading(&corpus, &native, &member, rel, Mode::Production);
        assert_eq!(v, serde_json::to_value(&want).unwrap());
        // Production never shows an unapproved rule, here either.
        for o in v["own"].as_array().unwrap() {
            assert!(o["report"]["results"].as_array().unwrap().iter().all(|x| x["status"] == "approved"));
        }
    }
    let b = modern_birth(&mut r);
    for bad in [
        json!({ "native": { "birth": b }, "member": { "birth": b }, "relation": "cousin" }),
        json!({ "native": { "birth": b }, "relation": "child" }),
        json!({ "native": { "birth": b }, "member": { "birth": { "date": "2001-02-30", "time": "10:00:00", "latitude": 13, "longitude": 80, "utc_offset_hours": 5.5 } }, "relation": "child" }),
    ] {
        let (st, _, _) = call(app(None), post("/api/family", &bad, None)).await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{bad}");
    }
    let (st, _, _) = call(app(None), post("/api/family", &json!({ "native": { "birth": b }, "member": { "birth": b }, "relation": "child", "mode": "review" }), None)).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
}
