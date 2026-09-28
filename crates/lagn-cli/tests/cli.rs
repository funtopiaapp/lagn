//! End-to-end tests of the `lagn` binary.
//!
//! Everything else tests the library. These test what a user actually runs:
//! argument parsing, exit codes, error messages, and the JSON contract.

use std::process::{Command, Output};

fn lagn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lagn"))
        .args(args)
        .env_remove("LAGN_EPHE_PATH")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to spawn lagn")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

const CHENNAI: &[&str] = &[
    "--date", "1985-06-21", "--time", "14:30",
    "--lat", "13.0827", "--lon", "80.2707", "--tz", "5.5",
];

fn with<'a>(sub: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut v = vec![sub];
    v.extend_from_slice(CHENNAI);
    v.extend_from_slice(extra);
    v
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

#[test]
fn chart_succeeds_and_prints_the_expected_lagna() {
    let o = lagn(&with("chart", &[]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("Lagna        Tula"), "lagna missing:\n{out}");
    assert!(out.contains("Pushya"), "janma nakshatra missing");
    assert!(out.contains("Poosam"), "Tamil nakshatra name missing");
    assert!(out.contains("South Indian"), "square chart missing");
}

#[test]
fn chart_json_is_valid_and_complete() {
    let o = lagn(&with("chart", &["--json"]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("valid JSON");
    assert_eq!(v["placements"].as_array().unwrap().len(), 9);
    assert_eq!(v["lagna"]["rasi"], "tula");
    assert_eq!(v["settings"]["ayanamsa"], "lahiri");
    assert_eq!(v["settings"]["node_type"], "mean");
    assert_eq!(v["settings"]["house_system"], "whole_sign");
    assert!(v["jd_ut"].as_f64().unwrap() > 2_400_000.0);
}

#[test]
fn identical_invocations_produce_byte_identical_output() {
    let a = stdout(&lagn(&with("chart", &["--json"])));
    let b = stdout(&lagn(&with("chart", &["--json"])));
    assert_eq!(a, b, "CLI output is not deterministic");
    let c = stdout(&lagn(&with("dasha", &["--json"])));
    let d = stdout(&lagn(&with("dasha", &["--json"])));
    assert_eq!(c, d, "dasha output is not deterministic");
}

#[test]
fn navamsa_flag_adds_the_d9_chart() {
    let without = stdout(&lagn(&with("chart", &[])));
    let with_d9 = stdout(&lagn(&with("chart", &["--navamsa"])));
    assert!(!without.contains("D-9 Navamsa"));
    assert!(with_d9.contains("D-9 Navamsa"));
}

#[test]
fn dasha_reports_the_known_balance_and_sequence() {
    let o = lagn(&with("dasha", &["--levels", "1"]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("Balance at birth: Shani 8y 10m 18d"), "balance wrong:\n{out}");
    // Mahadasha sequence after Shani is fixed.
    let order = ["Shani", "Budha", "Ketu", "Shukra", "Surya", "Chandra", "Kuja", "Rahu", "Guru"];
    let mut last = 0;
    for lord in order {
        let pos = out[last..].find(lord).unwrap_or_else(|| panic!("{lord} missing or out of order"));
        last += pos + lord.len();
    }
}

#[test]
fn dasha_levels_control_depth() {
    let l1 = stdout(&lagn(&with("dasha", &["--levels", "1"]))).lines().count();
    let l2 = stdout(&lagn(&with("dasha", &["--levels", "2"]))).lines().count();
    let l3 = stdout(&lagn(&with("dasha", &["--levels", "3"]))).lines().count();
    assert!(l1 < l2 && l2 < l3, "levels did not deepen output: {l1} {l2} {l3}");
}

#[test]
fn dasha_on_date_returns_a_three_level_chain() {
    let o = lagn(&with("dasha", &["--on", "2026-09-20"]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("Shukra / Rahu / Shani"), "unexpected chain:\n{out}");
}

#[test]
fn every_ayanamsa_and_year_length_option_is_accepted() {
    for a in ["lahiri", "lahiri-icrc", "true-chitra", "raman", "krishnamurti", "true-revati", "yukteshwar"] {
        let o = lagn(&with("chart", &["--ayanamsa", a, "--json"]));
        assert!(o.status.success(), "ayanamsa {a}: {}", stderr(&o));
    }
    for y in ["julian", "gregorian", "tropical", "sidereal-solar", "savana"] {
        let o = lagn(&with("dasha", &["--year-length", y, "--json"]));
        assert!(o.status.success(), "year length {y}: {}", stderr(&o));
    }
    for n in ["mean", "true"] {
        let o = lagn(&with("chart", &["--node", n, "--json"]));
        assert!(o.status.success(), "node {n}: {}", stderr(&o));
    }
}

#[test]
fn negative_coordinates_are_parsed_as_values_not_flags() {
    // Regression: clap originally treated "-33.8688" as an unknown flag.
    let o = lagn(&[
        "chart", "--date", "1978-11-03", "--time", "03:20",
        "--lat", "-33.8688", "--lon", "151.2093", "--tz", "11", "--json",
    ]);
    assert!(o.status.success(), "southern latitude rejected: {}", stderr(&o));
    let o = lagn(&[
        "chart", "--date", "1965-02-14", "--time", "22:45",
        "--lat", "40.7128", "--lon", "-74.0060", "--tz", "-5", "--json",
    ]);
    assert!(o.status.success(), "western longitude rejected: {}", stderr(&o));
}

#[test]
fn seconds_in_the_time_are_honoured() {
    let a: serde_json::Value = serde_json::from_str(&stdout(&lagn(&[
        "chart", "--date", "1985-06-21", "--time", "14:30:00",
        "--lat", "13.0827", "--lon", "80.2707", "--json",
    ]))).unwrap();
    let b: serde_json::Value = serde_json::from_str(&stdout(&lagn(&[
        "chart", "--date", "1985-06-21", "--time", "14:30:30",
        "--lat", "13.0827", "--lon", "80.2707", "--json",
    ]))).unwrap();
    let d = (b["jd_ut"].as_f64().unwrap() - a["jd_ut"].as_f64().unwrap()) * 86400.0;
    assert!((d - 30.0).abs() < 1e-3, "30 seconds became {d}");
}

// ---------------------------------------------------------------------------
// Failure paths: must exit non-zero with a message, never print a chart
// ---------------------------------------------------------------------------

fn assert_fails(args: &[&str], needle: &str) {
    let o = lagn(args);
    assert!(!o.status.success(), "{args:?} should have failed but printed:\n{}", stdout(&o));
    let err = stderr(&o).to_lowercase();
    assert!(
        err.contains(&needle.to_lowercase()),
        "{args:?}: expected {needle:?} in stderr, got:\n{err}"
    );
    assert!(
        !stdout(&o).contains("Lagna"),
        "{args:?}: printed a chart despite failing"
    );
}

#[test]
fn impossible_dates_fail_cleanly() {
    for date in ["1985-02-30", "1985-04-31", "1985-13-01", "1985-00-10", "1900-02-29"] {
        assert_fails(
            &["chart", "--date", date, "--time", "12:00", "--lat", "13", "--lon", "80"],
            "invalid calendar date",
        );
    }
}

#[test]
fn malformed_dates_and_times_fail_cleanly() {
    for date in ["1985/06/21", "21-06-1985x", "not-a-date", "1985-06"] {
        let o = lagn(&["chart", "--date", date, "--time", "12:00", "--lat", "13", "--lon", "80"]);
        assert!(!o.status.success(), "date {date:?} was accepted");
    }
    for time in ["25:00", "12:60", "12:00:60", "noon", "12"] {
        let o = lagn(&["chart", "--date", "1985-06-21", "--time", time, "--lat", "13", "--lon", "80"]);
        assert!(!o.status.success(), "time {time:?} was accepted");
    }
}

#[test]
fn out_of_range_coordinates_fail_cleanly() {
    assert_fails(&["chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "91", "--lon", "80"], "latitude");
    assert_fails(&["chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "13", "--lon", "181"], "longitude");
    assert_fails(&["chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "90", "--lon", "0"], "undefined");
    assert_fails(&["chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "NaN", "--lon", "80"], "latitude");
}

#[test]
fn out_of_range_offset_fails_cleanly() {
    assert_fails(
        &["chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "13", "--lon", "80", "--tz", "15"],
        "utc offset",
    );
}

#[test]
fn years_outside_coverage_fail_cleanly() {
    for date in ["1100-06-15", "3100-06-15"] {
        assert_fails(
            &["chart", "--date", date, "--time", "12:00", "--lat", "13", "--lon", "80"],
            "year outside",
        );
    }
}

#[test]
fn invalid_dasha_levels_are_rejected() {
    for lv in ["0", "4", "-1", "two"] {
        let o = lagn(&with("dasha", &["--levels", lv]));
        assert!(!o.status.success(), "--levels {lv} was accepted");
    }
}

#[test]
fn unknown_enum_values_are_rejected() {
    assert!(!lagn(&with("chart", &["--ayanamsa", "fagan"])).status.success());
    assert!(!lagn(&with("chart", &["--node", "osculating"])).status.success());
    assert!(!lagn(&with("dasha", &["--year-length", "lunar"])).status.success());
}

#[test]
fn a_missing_ephemeris_directory_is_reported_not_ignored() {
    let o = lagn(&[
        "--ephe", "/definitely/not/a/real/path",
        "chart", "--date", "1985-06-21", "--time", "12:00", "--lat", "13", "--lon", "80",
    ]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("ephemeris"), "stderr: {}", stderr(&o));
}

#[test]
fn missing_required_arguments_fail_with_usage() {
    let o = lagn(&["chart", "--date", "1985-06-21"]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("--time") || stderr(&o).contains("required"));
}

#[test]
fn help_and_version_work() {
    assert!(lagn(&["--help"]).status.success());
    assert!(lagn(&["--version"]).status.success());
    assert!(lagn(&["chart", "--help"]).status.success());
    assert!(lagn(&["dasha", "--help"]).status.success());
}

#[test]
fn a_pre_reform_birth_lists_its_first_dasha_from_the_entered_date() {
    // Regression QA-7: this used to print 1400-06-24.
    let o = lagn(&[
        "dasha", "--date", "1400-06-15", "--time", "12:00",
        "--lat", "13.0827", "--lon", "80.2707", "--tz", "5.3514", "--levels", "1",
    ]);
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let first = stdout(&o).lines().find(|l| l.contains('*')).unwrap_or("").to_string();
    assert!(first.contains("1400-06-15"), "first dasha row was: {first}");
}

// ---------------------------------------------------------------------------
// Phase 2 commands
// ---------------------------------------------------------------------------

#[test]
fn vargas_json_has_sixteen_vargas_with_nine_grahas_each() {
    let o = lagn(&with("vargas", &["--json"]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let vargas = v["vargas"].as_array().unwrap();
    assert_eq!(vargas.len(), 16);
    for b in vargas {
        assert_eq!(b["grahas"].as_object().unwrap().len(), 9, "{}", b["varga"]);
        assert!(b["lagna"].is_string());
    }
    assert_eq!(v["settings"]["node_aspects"], "none");
}

#[test]
fn known_chart_vargas_match_hand_computation() {
    // Chennai 1985-06-21 14:30: Sun at Mithuna 6 deg 16 min, worked by hand
    // from DESIGN.md section 3 during development.
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("vargas", &["--json"])))).unwrap();
    let sun = |d: &str| -> String {
        v["vargas"].as_array().unwrap().iter()
            .find(|b| b["varga"] == d).unwrap()["grahas"]["sun"]["sign"]
            .as_str().unwrap().to_string()
    };
    for (d, want) in [
        ("d1", "mithuna"), ("d2", "simha"), ("d3", "mithuna"), ("d4", "mithuna"),
        ("d7", "karka"), ("d10", "simha"), ("d12", "simha"), ("d16", "meena"),
        ("d20", "dhanus"), ("d24", "makara"), ("d27", "meena"), ("d30", "kumbha"),
        ("d40", "dhanus"), ("d45", "kanya"), ("d60", "mithuna"),
    ] {
        assert_eq!(sun(d), want, "Sun in {d}");
    }
}

#[test]
fn details_reports_known_dignities_and_combustion() {
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("details", &["--json"])))).unwrap();
    let cond = |g: &str| v["conditions"].as_array().unwrap().iter().find(|c| c["graha"] == g).unwrap().clone();
    assert_eq!(cond("saturn")["dignity"], "exalted");
    assert_eq!(cond("jupiter")["dignity"], "debilitated");
    assert_eq!(cond("moon")["dignity"], "own_sign");
    assert_eq!(cond("mars")["dignity"], "great_enemy");
    assert_eq!(cond("mars")["combust"], true, "Mars is 8 deg from the Sun, orb 17");
    assert_eq!(cond("mercury")["combust"], false, "Mercury is 15.5 deg away, orb 14");
    assert!(cond("rahu")["dignity"].is_null());
    assert_eq!(v["relations"].as_array().unwrap().len(), 42);
}

#[test]
fn ashtakavarga_totals_are_fixed() {
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("ashtakavarga", &["--json"])))).unwrap();
    let totals: Vec<u64> = v["bav"].as_array().unwrap().iter()
        .map(|row| row.as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).sum())
        .collect();
    assert_eq!(totals, vec![48, 49, 39, 54, 56, 52, 39]);
    let sav: u64 = v["sav"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).sum();
    assert_eq!(sav, 337);
}

#[test]
fn variant_flags_are_recorded_and_change_results() {
    let base: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("details", &["--json"])))).unwrap();
    let alt: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("details", &[
        "--json", "--node-aspects", "five-seven-nine", "--jagradadi", "natural",
        "--varga-mt", "as-own-sign", "--temporary", "same-varga",
    ])))).unwrap();
    assert_eq!(alt["settings"]["node_aspects"], "five_seven_nine");
    assert_eq!(alt["settings"]["jagradadi_basis"], "natural");
    assert_eq!(alt["settings"]["varga_moolatrikona"], "as_own_sign");
    assert_eq!(alt["settings"]["temporary_source"], "same_varga");
    let rahu_aspects = |v: &serde_json::Value| {
        v["conditions"].as_array().unwrap().iter().find(|c| c["graha"] == "rahu").unwrap()["aspects"]
            .as_array().unwrap().len()
    };
    assert_eq!(rahu_aspects(&base), 0);
    assert_eq!(rahu_aspects(&alt), 3);
}

#[test]
fn every_varga_can_be_drawn() {
    for d in ["d1", "d2", "d3", "d4", "d7", "d9", "d10", "d12", "d16", "d20", "d24", "d27", "d30", "d40", "d45", "d60"] {
        let o = lagn(&with("chart", &["--varga", d]));
        assert!(o.status.success(), "{d}: {}", stderr(&o));
        if d != "d1" {
            assert!(stdout(&o).contains(&format!("D-{} ", &d[1..])), "{d} square chart missing");
        }
    }
    assert!(!lagn(&with("chart", &["--varga", "d5"])).status.success(), "d5 is not a varga");
}

#[test]
fn phase2_commands_are_deterministic_and_accept_southern_coordinates() {
    for cmd in ["vargas", "details", "ashtakavarga"] {
        let a = stdout(&lagn(&with(cmd, &["--json"])));
        let b = stdout(&lagn(&with(cmd, &["--json"])));
        assert_eq!(a, b, "{cmd} not deterministic");
        let o = lagn(&[cmd, "--date", "1978-11-03", "--time", "03:20", "--lat", "-33.8688",
                       "--lon", "151.2093", "--tz", "11", "--json"]);
        assert!(o.status.success(), "{cmd} southern: {}", stderr(&o));
    }
}

#[test]
fn phase2_commands_reject_invalid_input_like_phase_1() {
    for cmd in ["vargas", "details", "ashtakavarga"] {
        let o = lagn(&[cmd, "--date", "1985-02-30", "--time", "12:00", "--lat", "13", "--lon", "80"]);
        assert!(!o.status.success(), "{cmd} accepted 30 February");
        assert!(stderr(&o).contains("invalid calendar date"));
    }
    assert!(!lagn(&with("details", &["--node-aspects", "twelfth"])).status.success());
}

// ---------------------------------------------------------------------------
// Phase 3 commands
// ---------------------------------------------------------------------------

const BRIDE: &str = "1992-03-15 06:20 13.0827 80.2707 5.5";
const GROOM: &str = "1988-11-02 21:45 9.9252 78.1198 5.5";

#[test]
fn rules_validate_reports_the_shipped_corpus() {
    let o = lagn(&["rules", "validate"]);
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("corpus valid"));
    assert!(out.contains("status approved"));
    assert!(!out.contains("status draft"), "an unreviewed draft is in the shipped corpus");
    assert!(out.contains("poruthams: 9/10 approved"), "Vasya stays rejected until a table exists");
}

#[test]
fn production_topic_output_contains_only_approved_rules_with_provenance() {
    let o = lagn(&with("topic", &["marriage", "--json"]));
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["mode"], "production");
    let results = v["results"].as_array().unwrap();
    assert!(!results.is_empty(), "approved rules must now appear in production");
    for r in results {
        assert_eq!(r["status"], "approved", "{} leaked into production", r["id"]);
        assert!(r["reviewer"].as_str().is_some_and(|x| !x.is_empty()), "{} has no reviewer", r["id"]);
    }
    assert_eq!(v["withheld"]["rejected"], 3);
    assert!(v["withheld"].get("draft").is_none());
}

#[test]
fn topic_output_matches_the_hand_checked_chart() {
    // Chennai 1985-06-21 14:30, worked by hand; re-checked after the review
    // changed four rules (docs/phase3/REVIEW-DECISIONS.md).
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("topic", &["marriage", "--json"])))).unwrap();
    let results = v["results"].as_array().unwrap();
    let get = |id: &str| results.iter().find(|r| r["id"] == id).unwrap_or_else(|| panic!("{id} missing")).clone();
    // Mars in Mithuna is the 12th from the Moon (Karka): dosha, now weighted -1.
    assert_eq!(get("marriage.kuja.from_moon")["effective"], true);
    assert_eq!(get("marriage.kuja.from_moon")["polarity"], -1);
    assert_eq!(get("marriage.kuja.from_lagna")["outcome"], "false", "Mars is 9th from Tula");
    assert_eq!(get("marriage.l7.combust")["effective"], true);
    // House 7 (Mesha) holds Rahu: in the narrowed malefic set.
    assert_eq!(get("marriage.h7.malefic_occupant")["effective"], true);
    // Venus is in the 7th: karako bhava nashaya is noted.
    assert_eq!(get("marriage.venus.karaka_in_7")["effective"], true);
    // Venus shares Mesha with Rahu.
    assert_eq!(get("marriage.venus.with_malefic")["effective"], true);
    assert_eq!(get("marriage.jupiter.debilitated_female")["outcome"], "unknown");
    assert_eq!(get("marriage.jupiter.strong_female")["outcome"], "false", "Kleene: False dominates Unknown");
    // Rejected rules never appear.
    assert!(results.iter().all(|r| r["id"] != "marriage.kuja.from_venus" && r["id"] != "marriage.h2.malefic_occupant"));
    assert!(!stdout(&lagn(&with("topic", &["marriage"]))).contains("[DRAFT]"));
}

#[test]
fn supplying_the_native_sex_resolves_unknowns() {
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&with("topic", &["marriage", "--review", "--sex", "female", "--json"])))).unwrap();
    let r = v["results"].as_array().unwrap().iter().find(|r| r["id"] == "marriage.jupiter.debilitated_female").unwrap().clone();
    assert_eq!(r["outcome"], "true", "Jupiter is debilitated in this chart");
    assert!(v["unknown"].as_array().unwrap().is_empty());
}

#[test]
fn topic_rejects_bad_input() {
    assert!(!lagn(&with("topic", &["cooking"])).status.success(), "unknown topic accepted");
    assert!(!lagn(&with("topic", &["marriage", "--from-age", "40", "--to-age", "20"])).status.success());
    assert!(!lagn(&with("topic", &["marriage", "--sex", "other"])).status.success());
    let o = lagn(&with("topic", &["marriage", "--corpus", "/no/such/corpus"]));
    assert!(!o.status.success() && stderr(&o).contains("corpus"));
}

#[test]
fn an_invalid_corpus_is_refused_with_its_problems_listed() {
    let dir = std::env::temp_dir().join(format!("lagn-bad-corpus-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("r.json"), r#"[{"id":"t.a","topic":"t","title":"x","tradition":"parashari",
        "when":{"always":{}},"polarity":1,"overrides":["t.a"],"text":{"en":"x"},
        "source":{"reference":null,"note":"n"},"review":{"status":"approved"}}]"#).unwrap();
    let o = lagn(&["rules", "validate", "--corpus", dir.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(err.contains("overrides itself"), "{err}");
    assert!(err.contains("without a reviewer"), "{err}");
}

#[test]
fn match_shows_reviewed_poruthams_and_reports_the_hand_checked_pair() {
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&["match", "--bride", BRIDE, "--groom", GROOM, "--json"]))).unwrap();
    assert_eq!(v["mode"], "production");
    assert_eq!(v["results"].as_array().unwrap().len(), 9);
    assert_eq!(v["withheld"], 1, "Vasya is rejected");
    assert!(v["results"].as_array().unwrap().iter().all(|r| r[1] == "approved" && r[0]["kind"] != "vasya"));
    assert!(v["reviewers"].as_array().unwrap().iter().any(|r| r.as_str().unwrap().contains("AI")), "provenance missing");
    assert_eq!(v["bride"]["nakshatra"], "pushya");
    assert_eq!(v["groom"]["nakshatra"], "magha");
    assert_eq!(v["matched"], 4);
    assert_eq!(v["evaluated"], 9);
    assert!(v["critical_failures"].as_array().unwrap().is_empty());
}

#[test]
fn match_rejects_malformed_people() {
    for bad in ["1992-03-15 06:20 13.08 80.27", "1992-02-30 06:20 13.08 80.27 5.5", "garbage"] {
        let o = lagn(&["match", "--bride", bad, "--groom", GROOM, "--review"]);
        assert!(!o.status.success(), "accepted {bad:?}");
    }
    assert!(!lagn(&["match", "--bride", BRIDE]).status.success(), "missing groom accepted");
}

#[test]
fn match_table_covers_the_whole_input_space() {
    let v: serde_json::Value = serde_json::from_str(&stdout(&lagn(&["match", "--table"]))).unwrap();
    let rows = v.as_array().unwrap();
    assert_eq!(rows.len(), 108 * 108);
    assert!(rows.iter().all(|r| r["results"].as_array().unwrap().len() == 10));
}

#[test]
fn review_sheet_lists_every_rule_with_generated_conditions() {
    // One sheet per topic; each must list exactly that topic's rules.
    let validate = stdout(&lagn(&["rules", "validate"]));
    let per_topic: Vec<(String, usize)> = validate
        .lines()
        .filter_map(|l| l.trim().strip_prefix("topic "))
        .map(|l| {
            let (t, n) = l.split_once(':').unwrap();
            (t.to_string(), n.trim().parse().unwrap())
        })
        .collect();
    assert!(per_topic.len() >= 10, "expected every topic in the validator summary: {validate}");
    for (topic, n) in &per_topic {
        let o = lagn(&["review-sheet", "--topic", topic, "--samples", "20"]);
        assert!(o.status.success(), "{topic}: {}", stderr(&o));
        assert_eq!(stdout(&o).matches("- **Condition:**").count(), *n, "{topic}: every rule needs a generated condition line");
    }
    let o = lagn(&["review-sheet", "--samples", "50"]);
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    let md = stdout(&o);
    assert!(md.contains("## Poruthams"));
    assert!(md.contains("blocked: table to be supplied"));
    assert_eq!(md, stdout(&lagn(&["review-sheet", "--samples", "50"])), "review sheet is not deterministic");
}

#[test]
fn review_sheet_counts_each_status_separately() {
    // Regression: the sheet once labelled every non-approved rule "Draft".
    let md = stdout(&lagn(&["review-sheet", "--samples", "20"]));
    assert!(md.contains("| Approved | 32 |"), "{}", &md[..600]);
    assert!(md.contains("| Draft | 0 |"));
    assert!(md.contains("| Rejected | 3 |"));
}

#[test]
fn periods_without_transits_keeps_every_rule_result() {
    // --no-transits only drops the gochara overlay: the period rules, scores
    // and windows must be identical.
    let args = ["periods", "--date", "1985-06-21", "--time", "14:30", "--lat", "13.08", "--lon", "80.27",
                "--tz", "5.5", "--from-age", "20", "--to-age", "45", "--json"];
    let full: serde_json::Value = serde_json::from_str(&stdout(&lagn(&args))).unwrap();
    let mut bare_args = args.to_vec();
    bare_args.push("--no-transits");
    let bare: serde_json::Value = serde_json::from_str(&stdout(&lagn(&bare_args))).unwrap();

    let windows = full["windows"].as_array().unwrap();
    assert!(!windows.is_empty());
    assert_eq!(windows.len(), bare["windows"].as_array().unwrap().len());
    for (a, b) in windows.iter().zip(bare["windows"].as_array().unwrap()) {
        for field in ["maha", "antar", "score", "sensitive", "amplifiers", "negators", "noted", "cancelled", "focus"] {
            assert_eq!(a[field], b[field], "{field} differs without transits");
        }
        assert!(b["pressures"].as_array().unwrap().is_empty());
        assert!(b["supports"].as_array().unwrap().is_empty());
        // Explanations lose their transit lines and keep the rest.
        let lines = |v: &serde_json::Value| -> Vec<String> {
            v["explanation"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string())
                .filter(|l| !l.starts_with("Transit that")).collect()
        };
        assert_eq!(lines(a), lines(b));
    }
    // Some window in the full run did carry a transit, so the flag is meaningful.
    assert!(windows.iter().any(|w| !w["pressures"].as_array().unwrap().is_empty() || !w["supports"].as_array().unwrap().is_empty()));
}
