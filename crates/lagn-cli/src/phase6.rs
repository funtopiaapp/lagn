//! Phase 6 output: transits and period analysis.

use lagn_core::{jd_to_civil, Chart};
use lagn_rules::reading::SensitivePeriods;

fn date(jd: f64, tz: f64) -> String {
    let d = jd_to_civil(jd, tz);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

pub use lagn_rules::reading::{transits, TransitReport};

pub fn print_transits(c: &Chart, r: &TransitReport) {
    let tz = c.birth.moment.utc_offset_hours;
    println!("\n  Transits from the natal Moon sign ({})\n", r.moon_sign.name());
    for w in &r.windows {
        let extra = match (w.bindus, w.bav_supports, w.supportive) {
            (Some(b), Some(true), false) => format!("  [{b} bindus: milder]"),
            (Some(b), Some(true), true) => format!("  [{b} bindus: fuller]"),
            (Some(b), Some(false), true) => format!("  [{b} bindus: weaker]"),
            (Some(b), _, _) => format!("  [{b} bindus]"),
            _ => String::new(),
        };
        println!("  {} -> {}  {:<8} in {:<10} {}{}", date(w.start_jd, tz), date(w.end_jd, tz), w.graha.name(), w.sign.name(), w.kind.label(), extra);
    }
    println!();
}

pub fn print_periods(c: &Chart, r: &SensitivePeriods) {
    let tz = c.birth.moment.utc_offset_hours;
    println!("\n  Period analysis   mode: {:?}", r.mode);
    if !r.withheld.is_empty() {
        let parts: Vec<String> = r.withheld.iter().map(|(k, v)| format!("{v} {k}")).collect();
        println!("  Withheld by the review gate: {}", parts.join(", "));
    }
    for d in &r.windows {
        let w = &d.window;
        let mark = if w.sensitive { "SENSITIVE" } else { "" };
        println!("\n  {} -> {}  {} / {}  score {:+} {}", date(w.start_jd, tz), date(w.end_jd, tz), w.maha.name(), w.antar.name(), w.score, mark);
        for line in &d.explanation {
            println!("     {line}");
        }
    }
    println!();
}
