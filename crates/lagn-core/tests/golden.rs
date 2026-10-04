//! Replay every golden fixture in `tests/golden/`.
//!
//! See that directory's README for what a fixture does and does not prove.

use std::path::{Path, PathBuf};

use lagn_core::{
    BirthData, BirthMoment, Chart, ChartSettings, Ephemeris, Graha, Vimshottari,
};
use serde_json::Value;

/// Degrees. 1 arcsec - far tighter than any boundary that matters, loose
/// enough to survive a Swiss Ephemeris point release.
const TOL_DEG: f64 = 1.0 / 3600.0;
/// Days. Dasha boundaries within a minute of each other are the same boundary.
const TOL_JD: f64 = 1.0 / 1440.0;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

fn init_ephemeris() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe")
        .canonicalize()
        .expect("ephe directory");
    Ephemeris::set_ephemeris_path(dir).expect("set ephemeris path");
}

fn graha_by_name(s: &str) -> Graha {
    match s {
        "sun" => Graha::Sun, "moon" => Graha::Moon, "mars" => Graha::Mars,
        "mercury" => Graha::Mercury, "jupiter" => Graha::Jupiter,
        "venus" => Graha::Venus, "saturn" => Graha::Saturn,
        "rahu" => Graha::Rahu, "ketu" => Graha::Ketu,
        other => panic!("unknown graha in fixture: {other}"),
    }
}

fn parse_date(s: &str) -> (i32, u32, u32) {
    let p: Vec<&str> = s.split('-').collect();
    (p[0].parse().unwrap(), p[1].parse().unwrap(), p[2].parse().unwrap())
}

fn parse_time(s: &str) -> (u32, u32, f64) {
    let p: Vec<&str> = s.split(':').collect();
    (
        p[0].parse().unwrap(),
        p[1].parse().unwrap(),
        if p.len() > 2 { p[2].parse().unwrap() } else { 0.0 },
    )
}

struct Report {
    total: usize,
    verified: usize,
}

fn replay(path: &Path, report: &mut Report) {
    let raw = std::fs::read_to_string(path).expect("read fixture");
    let f: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{}: malformed JSON: {e}", path.display()));
    let id = f["id"].as_str().unwrap_or("<no id>");

    report.total += 1;
    if f["verified_against"].as_str().is_some() {
        report.verified += 1;
    }

    // Every file in this directory must be a chart fixture; anything else is
    // in the wrong place. Say so, rather than unwrapping into a bare None.
    let inp = &f["input"];
    assert!(
        inp["date"].is_string(),
        "{}: not a chart fixture - no input.date. Every .json in tests/golden/ \
         pins one birth chart; other fixtures belong elsewhere under tests/.",
        path.display()
    );
    let (year, month, day) = parse_date(inp["date"].as_str().unwrap());
    let (hour, minute, second) = parse_time(inp["time"].as_str().unwrap());

    let birth = BirthData {
        moment: BirthMoment {
            year, month, day, hour, minute, second,
            utc_offset_hours: inp["utc_offset_hours"].as_f64().unwrap(),
        },
        latitude: inp["latitude"].as_f64().unwrap(),
        longitude: inp["longitude"].as_f64().unwrap(),
        place_name: inp["place"].as_str().unwrap_or("").to_string(),
    };

    // Settings come through serde so a fixture cannot silently be replayed
    // under different conventions than it was recorded with.
    let settings: ChartSettings = serde_json::from_value(serde_json::json!({
        "ayanamsa": f["settings"]["ayanamsa"],
        "node_type": f["settings"]["node"],
        "house_system": f["settings"]["house_system"],
        "year_length": f["settings"]["year_length"],
    }))
    .unwrap_or_else(|e| panic!("{id}: bad settings: {e}"));

    let chart = Chart::compute(birth, settings)
        .unwrap_or_else(|e| panic!("{id}: chart computation failed: {e}"));
    let exp = &f["expected"];

    // --- lagna ---
    let el = &exp["lagna"];
    assert_eq!(
        serde_json::to_value(chart.lagna.rasi).unwrap(), el["rasi"],
        "{id}: lagna rasi"
    );
    assert!(
        (chart.lagna.degrees_in_rasi - el["degrees_in_rasi"].as_f64().unwrap()).abs() < TOL_DEG,
        "{id}: lagna degrees: got {}, expected {}",
        chart.lagna.degrees_in_rasi, el["degrees_in_rasi"]
    );
    assert_eq!(
        serde_json::to_value(chart.lagna.nakshatra.nakshatra).unwrap(), el["nakshatra"],
        "{id}: lagna nakshatra"
    );
    assert_eq!(chart.lagna.nakshatra.pada as u64, el["pada"].as_u64().unwrap(), "{id}: lagna pada");
    assert_eq!(
        serde_json::to_value(chart.lagna.navamsa).unwrap(), el["navamsa"],
        "{id}: lagna navamsa"
    );

    // --- grahas ---
    for (name, e) in exp["grahas"].as_object().expect("grahas object") {
        let g = graha_by_name(name);
        let p = chart.placement(g);
        assert_eq!(serde_json::to_value(p.rasi).unwrap(), e["rasi"], "{id}: {name} rasi");
        assert!(
            (p.degrees_in_rasi - e["degrees_in_rasi"].as_f64().unwrap()).abs() < TOL_DEG,
            "{id}: {name} degrees: got {}, expected {}",
            p.degrees_in_rasi, e["degrees_in_rasi"]
        );
        assert_eq!(
            serde_json::to_value(p.nakshatra.nakshatra).unwrap(), e["nakshatra"],
            "{id}: {name} nakshatra"
        );
        assert_eq!(p.nakshatra.pada as u64, e["pada"].as_u64().unwrap(), "{id}: {name} pada");
        assert_eq!(p.house as u64, e["house"].as_u64().unwrap(), "{id}: {name} house");
        assert_eq!(
            serde_json::to_value(p.navamsa).unwrap(), e["navamsa"],
            "{id}: {name} navamsa"
        );
        assert_eq!(p.retrograde, e["retrograde"].as_bool().unwrap(), "{id}: {name} retrograde");
    }

    // --- vimshottari ---
    let v = Vimshottari::compute(&chart);
    let ev = &exp["vimshottari"];
    assert_eq!(
        serde_json::to_value(v.janma_nakshatra).unwrap(), ev["janma_nakshatra"],
        "{id}: janma nakshatra"
    );
    assert_eq!(
        serde_json::to_value(v.birth_lord).unwrap(), ev["birth_lord"],
        "{id}: birth dasha lord"
    );
    assert!(
        (v.balance_years - ev["balance_years"].as_f64().unwrap()).abs() < 1e-5,
        "{id}: dasha balance: got {}, expected {}",
        v.balance_years, ev["balance_years"]
    );

    let expected_mds = ev["mahadashas"].as_array().unwrap();
    assert_eq!(v.mahadashas.len(), expected_mds.len(), "{id}: mahadasha count");
    for (got, e) in v.mahadashas.iter().zip(expected_mds) {
        assert_eq!(serde_json::to_value(got.lord).unwrap(), e["lord"], "{id}: mahadasha lord");
        assert!(
            (got.start_jd - e["start_jd"].as_f64().unwrap()).abs() < TOL_JD,
            "{id}: {:?} mahadasha start: got {}, expected {}",
            got.lord, got.start_jd, e["start_jd"]
        );
        assert!(
            (got.end_jd - e["end_jd"].as_f64().unwrap()).abs() < TOL_JD,
            "{id}: {:?} mahadasha end", got.lord
        );
    }
}

#[test]
fn all_golden_charts_replay() {
    init_ephemeris();
    let dir = golden_dir();
    let mut report = Report { total: 0, verified: 0 };

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();

    for p in &paths {
        replay(p, &mut report);
    }

    println!(
        "\n  golden charts: {} replayed, {} externally verified, {} regression-lock only",
        report.total,
        report.verified,
        report.total - report.verified
    );
    println!("  Phase 1 exit criterion: 50 externally verified\n");
}

/// Fails once the fixture set is complete, as a deliberate reminder that the
/// Phase 1 gate is external verification and not merely having 50 files.
#[test]
#[ignore = "enable when working through JHora verification"]
fn phase_1_exit_criterion() {
    init_ephemeris();
    let dir = golden_dir();
    let verified = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter(|p| {
            let raw = std::fs::read_to_string(p).unwrap();
            let f: Value = serde_json::from_str(&raw).unwrap();
            f["verified_against"].as_str().is_some()
        })
        .count();
    assert!(
        verified >= 50,
        "Phase 1 needs 50 JHora-verified charts, have {verified}"
    );
}
