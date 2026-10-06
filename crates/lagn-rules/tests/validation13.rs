//! Replay the thirteen charts of the independent 1.0 validation pass.
//!
//! These are the only charts whose engine output has been graded against
//! externally researched lives (tests/QA-VALIDATION-1.0.md). All thirteen
//! reproduced exactly, and 128 of the 130 topic scores in that report matched
//! the engine cell for cell, so a change in any of these numbers is a change
//! someone should have intended.
//!
//! The fixture locks the numbers; it does not bless them. Several of these
//! scores are wrong about the documented life - the marriage verdicts for
//! subjects 03, 06 and 10 most of all. When the recalibration lands, these
//! expectations move on purpose: rerun `scripts/make_validation13.py` and say
//! in the commit which subjects moved and why.

use std::path::{Path, PathBuf};

use lagn_core::{BirthData, BirthMoment, Chart, ChartSettings, DerivationSettings, Ephemeris};
use lagn_rules::model::Sex;
use lagn_rules::resolve::{evaluate_topic, Mode};
use lagn_rules::{FactBase, NativeInfo};
use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn init_ephemeris() {
    let dir = root().join("ephe").canonicalize().expect("ephe directory");
    Ephemeris::set_ephemeris_path(dir).expect("set ephemeris path");
}

fn fixture() -> Value {
    let p = root().join("tests/validation13.json");
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    serde_json::from_str(&text).expect("validation13.json is valid JSON")
}

/// `YYYY-MM-DD` and `HH:MM` as the fixture writes them.
fn moment(date: &str, time: &str, tz: f64) -> BirthMoment {
    let d: Vec<&str> = date.split('-').collect();
    let t: Vec<&str> = time.split(':').collect();
    BirthMoment {
        year: d[0].parse().expect("year"),
        month: d[1].parse().expect("month"),
        day: d[2].parse().expect("day"),
        hour: t[0].parse().expect("hour"),
        minute: t[1].parse().expect("minute"),
        second: 0.0,
        utc_offset_hours: tz,
    }
}

fn facts_for(s: &Value) -> FactBase {
    let b = &s["birth"];
    let birth = BirthData {
        moment: moment(
            b["date"].as_str().expect("date"),
            b["time"].as_str().expect("time"),
            b["tz"].as_f64().expect("tz"),
        ),
        latitude: b["lat"].as_f64().expect("lat"),
        longitude: b["lon"].as_f64().expect("lon"),
        place_name: String::new(),
    };
    let sex = match b["sex"].as_str() {
        Some("female") => Some(Sex::Female),
        Some("male") => Some(Sex::Male),
        _ => None,
    };
    let chart = Chart::compute(birth, ChartSettings::default()).expect("chart computes");
    FactBase::new(chart, DerivationSettings::default(), NativeInfo { sex })
}

#[test]
fn every_subject_reproduces_its_lagna() {
    init_ephemeris();
    let doc = fixture();
    let subjects = doc["subjects"].as_array().expect("subjects array");
    assert_eq!(subjects.len(), 13, "the validation pass covered thirteen charts");

    for s in subjects {
        let code = s["code"].as_str().expect("code");
        let f = facts_for(s);
        let lagna = f.chart.lagna;
        let want = &s["lagna"];

        assert_eq!(
            serde_json::to_value(lagna.rasi).expect("rasi serialises"),
            want["rasi"],
            "subject {code}: lagna rasi moved"
        );
        // An arcsecond: far tighter than any boundary that matters, loose
        // enough to survive a Swiss Ephemeris point release.
        let got = lagna.degrees_in_rasi;
        let expected = want["degrees_in_rasi"].as_f64().expect("degrees_in_rasi");
        assert!(
            (got - expected).abs() < 1.0 / 3600.0,
            "subject {code}: lagna moved from {expected}° to {got}° in the rasi"
        );
    }
}

#[test]
fn every_topic_score_reproduces() {
    init_ephemeris();
    let corpus = lagn_rules::corpus::load(&root().join("corpus")).expect("corpus loads");
    let doc = fixture();

    let mut checked = 0usize;
    let mut drifted: Vec<String> = Vec::new();

    for s in doc["subjects"].as_array().expect("subjects array") {
        let code = s["code"].as_str().expect("code");
        let f = facts_for(s);

        for (topic, want) in s["topics"].as_object().expect("topics object") {
            let ages = want["ages"].as_array().expect("ages");
            let range = (
                ages[0].as_f64().expect("from_age"),
                ages[1].as_f64().expect("to_age"),
            );
            let report = evaluate_topic(topic, &corpus.rules, &f, Mode::Production, range);
            checked += 1;

            let got = report.score;
            let expected = want["score"].as_i64().expect("score") as i32;
            if got != expected {
                drifted.push(format!("  subject {code} {topic}: {expected:+} -> {got:+}"));
                continue;
            }
            // The counts catch a rule swapped for another of equal polarity,
            // which the score alone would hide.
            let sup = report.supporting.len();
            let aff = report.afflicting.len();
            let (want_sup, want_aff) = (
                want["supporting"].as_u64().expect("supporting") as usize,
                want["afflicting"].as_u64().expect("afflicting") as usize,
            );
            if (sup, aff) != (want_sup, want_aff) {
                drifted.push(format!(
                    "  subject {code} {topic}: score held at {expected:+} but factors changed, \
                     supporting {want_sup} -> {sup}, afflicting {want_aff} -> {aff}"
                ));
            }
        }
    }

    assert_eq!(checked, 130, "thirteen subjects across ten topics");
    assert!(
        drifted.is_empty(),
        "the validated corpus moved in {} of {checked} cells:\n{}\n\n\
         If this was deliberate, rerun scripts/make_validation13.py and say in the \
         commit which subjects moved and why.",
        drifted.len(),
        drifted.join("\n")
    );
}

/// D2 from the validation report: a period that renders with the same start
/// and end date. It was not reproducible on any of the thirteen charts, but
/// the shape that would produce it is real - the view clamps a period that
/// began before birth up to the birth moment, and a period ending later the
/// same day then renders as zero-length. This pins the absence so a change to
/// the dasha tree cannot quietly introduce one.
#[test]
fn no_dasha_period_renders_as_zero_length() {
    use lagn_core::Vimshottari;

    init_ephemeris();
    let doc = fixture();

    for s in doc["subjects"].as_array().expect("subjects array") {
        let code = s["code"].as_str().expect("code");
        let f = facts_for(s);
        let tz = f.chart.birth.moment.utc_offset_hours;
        let vim = Vimshottari::compute(&f.chart);
        let birth_jd = vim.birth_jd;

        // The civil date a period boundary renders as, in the birth's offset.
        let day = |jd: f64| {
            let c = lagn_core::jd_to_civil(jd, tz);
            (c.year, c.month, c.day)
        };

        fn walk(
            ps: &[lagn_core::DashaPeriod],
            birth_jd: f64,
            day: &dyn Fn(f64) -> (i32, u32, u32),
            path: &mut Vec<&'static str>,
            out: &mut Vec<String>,
        ) {
            for p in ps {
                // The view drops periods that ended before birth and clamps
                // the rest, so mirror both here.
                if p.end_jd <= birth_jd {
                    continue;
                }
                let start = p.start_jd.max(birth_jd);
                path.push(p.lord.name());
                if day(start) == day(p.end_jd) {
                    let (y, m, d) = day(start);
                    out.push(format!("{} on {y:04}-{m:02}-{d:02}", path.join("/")));
                }
                walk(&p.children, birth_jd, day, path, out);
                path.pop();
            }
        }

        let mut found = Vec::new();
        walk(&vim.mahadashas, birth_jd, &day, &mut Vec::new(), &mut found);
        assert!(
            found.is_empty(),
            "subject {code}: {} dasha period(s) would render as zero-length:\n  {}",
            found.len(),
            found.join("\n  ")
        );
    }
}

/// D7: how much an uncertain birth time actually matters, per chart.
///
/// Every house is counted from the lagna, so the margin to the next rasi is
/// the honest measure of that. Subject-06 is the case that makes the point:
/// the lagna sits at Tula 29°48', minutes from Vrischika, yet the reading
/// spoke with the same authority as any other.
#[test]
fn the_lagna_margin_matches_a_recomputed_boundary() {
    init_ephemeris();
    let doc = fixture();

    for s in doc["subjects"].as_array().expect("subjects array") {
        let code = s["code"].as_str().expect("code");
        let f = facts_for(s);
        let w = f.chart.lagna_window().expect("lagna window");

        // Both edges are positive and the margin is the nearer of the two.
        assert!(w.minutes_in > 0.0 && w.minutes_left > 0.0, "subject {code}: {w:?}");
        assert_eq!(w.margin_minutes(), w.minutes_in.min(w.minutes_left));

        // A chart cast one minute inside each edge must still hold the same
        // rasi, and one minute outside must not. That is the claim the number
        // makes, checked against a fresh chart rather than the same search.
        let settings = ChartSettings::default();
        let tz = f.chart.birth.moment.utc_offset_hours;
        // Shift through the Julian Day and come back to a civil moment, so the
        // offset stays a real wall-clock time across a midnight or a month end.
        let at = |delta_minutes: f64| {
            let c = lagn_core::jd_to_civil(f.chart.jd_ut + delta_minutes / 1440.0, tz);
            let mut b = f.chart.birth.clone();
            b.moment = BirthMoment {
                year: c.year, month: c.month, day: c.day,
                hour: c.hour, minute: c.minute, second: c.second,
                utc_offset_hours: tz,
            };
            Chart::compute(b, settings).map(|c| c.lagna.rasi)
        };
        let here = f.chart.lagna.rasi;
        for (edge, dir) in [(w.minutes_in, -1.0), (w.minutes_left, 1.0)] {
            if edge >= 12.0 * 60.0 {
                continue; // reported as the cap, not a located boundary
            }
            assert_eq!(
                at(dir * (edge - 1.0)).expect("chart inside the edge"),
                here,
                "subject {code}: lagna should still be {here:?} a minute inside the edge at {edge} min"
            );
            assert_ne!(
                at(dir * (edge + 1.0)).expect("chart outside the edge"),
                here,
                "subject {code}: lagna should have left {here:?} a minute past the edge at {edge} min"
            );
        }
    }
}

#[test]
fn subject_06_is_the_knife_edge_chart() {
    init_ephemeris();
    let doc = fixture();
    let s = doc["subjects"]
        .as_array()
        .expect("subjects array")
        .iter()
        .find(|s| s["code"] == "06")
        .expect("subject 06");

    let f = facts_for(s);
    let w = f.chart.lagna_window().expect("lagna window");
    // Tula 29°48': the reading's entire house framework turns over within a
    // quarter of an hour. Whatever else changes, this chart must keep warning.
    assert!(
        w.margin_minutes() < 20.0,
        "subject 06 should sit minutes from a rasi boundary, got {:.1} min",
        w.margin_minutes()
    );
}

/// Not an assertion, a printout: `cargo test -p lagn-rules --test validation13
/// -- --nocapture report_lagna_margins` shows how exposed each chart is.
#[test]
fn report_lagna_margins() {
    init_ephemeris();
    let doc = fixture();
    println!("\n  subject  lagna              margin   holds (before/after)  confidence");
    for s in doc["subjects"].as_array().expect("subjects array") {
        let f = facts_for(s);
        let w = f.chart.lagna_window().expect("lagna window");
        println!(
            "  {:>7}  {:<9} {:>5.2}°  {:>6.0}m   {:>5.0}m / {:>5.0}m        {}",
            s["code"].as_str().unwrap_or("?"),
            format!("{:?}", f.chart.lagna.rasi),
            f.chart.lagna.degrees_in_rasi,
            w.margin_minutes(),
            w.minutes_in,
            w.minutes_left,
            s["time_confidence"].as_str().unwrap_or("?"),
        );
    }
    println!();
}

/// A topic's ranked timing must cover its whole age range without holes.
///
/// It used to list only the stretches a topic's timing rules named. On a
/// Mesha-lagna chart whose 10th lord is Saturn, that meant career windows
/// jumped from 2029 to 2041 - the entire Mercury mahadasha vanished, because
/// its lord is not a timing lord. A reader could not tell that from a period
/// where nothing happens.
#[test]
fn ranked_timing_covers_the_range_with_no_gaps() {
    use lagn_rules::reading::topic_timing;

    init_ephemeris();
    let corpus = lagn_rules::corpus::load(&root().join("corpus")).expect("corpus loads");
    let doc = fixture();

    // The chart that exposed the hole, plus the fixture's thirteen.
    let kollam = serde_json::json!({
        "code": "kollam-1981",
        "birth": { "date": "1981-12-21", "time": "14:10", "lat": 8.88113, "lon": 76.58469, "tz": 5.5, "sex": "male" },
    });
    let mut subjects: Vec<&Value> = vec![&kollam];
    subjects.extend(doc["subjects"].as_array().expect("subjects array"));

    for s in subjects {
        let code = s["code"].as_str().expect("code");
        for (topic, ages) in [("career", (18.0, 65.0)), ("marriage", (18.0, 45.0)), ("wealth", (18.0, 80.0))] {
            let mut f = facts_for(s);
            let report = evaluate_topic(topic, &corpus.rules, &f, Mode::Production, ages);
            let judged = topic_timing(&corpus, &mut f, Mode::Production, topic, &report.timing, ages);
            if judged.is_empty() {
                continue; // a topic the corpus does not time by dasha
            }

            // Every stretch the timing rules named is still present.
            for w in &report.timing {
                assert!(
                    judged.iter().any(|j| j.maha == w.maha && j.antar == w.antar && j.live),
                    "{code} {topic}: {:?}/{:?} was named by the timing rules but is not marked live",
                    w.maha, w.antar
                );
            }

            // And the ranked list is contiguous: each stretch begins where the
            // last ended, so there is no span a reader cannot account for.
            let mut sorted = judged.clone();
            sorted.sort_by(|a, b| a.start_jd.partial_cmp(&b.start_jd).expect("finite"));
            for pair in sorted.windows(2) {
                let (a, b) = (&pair[0], &pair[1]);
                let gap_days = b.start_jd - a.end_jd;
                assert!(
                    gap_days.abs() < 1.0,
                    "{code} {topic}: {:.0} day gap between {:?}/{:?} and {:?}/{:?}",
                    gap_days, a.maha, a.antar, b.maha, b.antar
                );
            }
            // Every stretch carries a rank, so none is silently unexplained.
            assert!(judged.iter().all(|j| !j.rank.label().is_empty()));
        }
    }
}

/// The reported case, pinned: career timing for a Mesha-lagna chart born
/// 1981-12-21 at Kollam used to skip from 2029 to 2041 because Mercury, whose
/// mahadasha that is, is not one of its timing lords.
#[test]
fn the_mercury_mahadasha_is_no_longer_missing_from_career_timing() {
    use lagn_core::jd_to_civil;
    use lagn_rules::reading::topic_timing;

    init_ephemeris();
    let corpus = lagn_rules::corpus::load(&root().join("corpus")).expect("corpus loads");
    let s = serde_json::json!({
        "birth": { "date": "1981-12-21", "time": "14:10", "lat": 8.88113, "lon": 76.58469, "tz": 5.5, "sex": "male" },
    });
    let mut f = facts_for(&s);
    let ages = (18.0, 65.0);
    let report = evaluate_topic("career", &corpus.rules, &f, Mode::Production, ages);
    let judged = topic_timing(&corpus, &mut f, Mode::Production, "Career", &report.timing, ages);

    // Mercury's mahadasha runs 2029-2046 on this chart. Every year of it that
    // falls inside the age range must be represented by some stretch.
    let year = |jd: f64| jd_to_civil(jd, 5.5).year;
    for y in 2030..=2040 {
        assert!(
            judged.iter().any(|w| year(w.start_jd) <= y && y <= year(w.end_jd)),
            "no career stretch covers {y}; the Mercury mahadasha is missing again"
        );
    }
    // The timing rules genuinely name only a few of those stretches, which is
    // why the old list had a hole. Both facts should hold at once.
    let in_mercury: Vec<_> = judged.iter().filter(|w| (2030..=2040).contains(&year(w.start_jd))).collect();
    assert!(in_mercury.len() >= 6, "only {} stretches in the gap", in_mercury.len());
    assert!(
        in_mercury.iter().all(|w| !w.live),
        "the stretches in the old gap are listed, but none should be marked live"
    );
}
