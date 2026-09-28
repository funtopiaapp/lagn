//! UTC offset suggestions from the IANA tz database.
//!
//! A suggestion, never a silent conversion: the user confirms it, and the
//! confirmed value is what reaches the kernel. Specification:
//! `docs/phase4/DESIGN.md` section 3.

use jiff::civil::DateTime;
use jiff::tz::{AmbiguousOffset, TimeZone};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// On or after 1970-01-01, the period the tz database guarantees.
    Reliable,
    /// Before 1970: plausible, but confirm against the birth record.
    Historical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// One valid offset.
    Unique,
    /// The local time occurred twice (clocks went back); both offsets listed.
    Ambiguous,
    /// The local time never occurred (clocks went forward).
    Nonexistent,
    /// Before standard time: local mean time of the birth place itself.
    LocalMeanTime,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Candidate {
    pub offset_hours: f64,
    /// e.g. "+05:30" or "+05:21:10".
    pub offset_text: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suggestion {
    pub timezone: String,
    pub kind: Kind,
    pub confidence: Confidence,
    /// Best candidate first. Empty only for `Nonexistent`.
    pub candidates: Vec<Candidate>,
    pub notes: Vec<String>,
    pub tzdb_version: String,
}

/// One candidate tz database.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TzCandidate {
    pub source: String,
    /// IANA release, e.g. "2026d". `None` if the copy carries no version.
    pub version: Option<String>,
}

/// The tz database the server uses: the newest of the copies available.
///
/// IANA corrects *historical* offsets several times a year (2025c fixed Baja
/// California 1953-75; 2026d fixed Colombia 1992 and Iran 1979), so a stale
/// copy gives wrong birth offsets. Three copies are considered:
///
/// - `shipped`: `data/zoneinfo`, compiled from IANA's latest release by
///   `scripts/update_tzdb.sh` before each release;
/// - `system`: the operating system's copy, kept current by OS updates;
/// - `bundled`: the copy compiled into jiff (`jiff-tzdb`).
///
/// IANA release names sort correctly as strings ("2026c" < "2026d").
/// A candidate database and a way to open it.
type Opener = Box<dyn Fn() -> Option<jiff::tz::TimeZoneDatabase>>;

pub struct TzDb {
    pub db: jiff::tz::TimeZoneDatabase,
    pub in_use: TzCandidate,
    pub considered: Vec<TzCandidate>,
}

fn dir_version(dir: &std::path::Path) -> Option<String> {
    // "+VERSION": IANA's zic output (ours) and macOS. "tzdata.zi": Debian, Alpine.
    if let Ok(v) = std::fs::read_to_string(dir.join("+VERSION")) {
        return Some(v.trim().to_string());
    }
    std::fs::read_to_string(dir.join("tzdata.zi"))
        .ok()
        .and_then(|zi| zi.lines().next().and_then(|l| l.strip_prefix("# version ")).map(|v| v.trim().to_string()))
}

impl TzDb {
    /// Pick the newest of the shipped, system and bundled databases.
    pub fn newest(shipped_dir: Option<&std::path::Path>) -> TzDb {
        let system_dir = std::path::PathBuf::from(std::env::var("TZDIR").unwrap_or_else(|_| "/usr/share/zoneinfo".into()));
        let mut options: Vec<(TzCandidate, Opener)> = Vec::new();

        if let Some(dir) = shipped_dir.filter(|d| d.is_dir()) {
            let d = dir.to_path_buf();
            options.push((
                TzCandidate { source: format!("shipped ({})", d.display()), version: dir_version(&d) },
                Box::new(move || jiff::tz::TimeZoneDatabase::from_dir(&d).ok()),
            ));
        }
        if system_dir.is_dir() {
            let d = system_dir.clone();
            options.push((
                TzCandidate { source: format!("system ({})", d.display()), version: dir_version(&d) },
                Box::new(move || jiff::tz::TimeZoneDatabase::from_dir(&d).ok()),
            ));
        }
        options.push((
            TzCandidate { source: "bundled (jiff-tzdb)".into(), version: jiff_tzdb::VERSION.map(String::from) },
            Box::new(|| Some(jiff::tz::TimeZoneDatabase::bundled())),
        ));

        let considered: Vec<TzCandidate> = options.iter().map(|(c, _)| c.clone()).collect();
        // Newest version first; an unversioned copy ranks last. Ties keep the
        // listed order: shipped, then system, then bundled.
        let mut order: Vec<usize> = (0..options.len()).collect();
        order.sort_by(|&a, &b| options[b].0.version.cmp(&options[a].0.version));
        for i in order {
            if let Some(db) = (options[i].1)() {
                if db.get("Asia/Kolkata").is_ok() {
                    return TzDb { db, in_use: options[i].0.clone(), considered };
                }
            }
        }
        TzDb { db: jiff::tz::TimeZoneDatabase::bundled(), in_use: considered.last().unwrap().clone(), considered }
    }

    pub fn version_label(&self) -> String {
        format!("{} {}", self.in_use.version.as_deref().unwrap_or("unversioned"), self.in_use.source)
    }
}

pub fn offset_text(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let s = seconds.unsigned_abs();
    let (h, m, sec) = (s / 3600, (s / 60) % 60, s % 60);
    if sec == 0 { format!("{sign}{h:02}:{m:02}") } else { format!("{sign}{h:02}:{m:02}:{sec:02}") }
}

fn lmt_candidate(longitude: f64) -> Candidate {
    let secs = (longitude * 240.0).round() as i32; // 4 minutes of time per degree
    Candidate {
        offset_hours: secs as f64 / 3600.0,
        offset_text: offset_text(secs),
        label: format!("Local mean time at longitude {longitude:.4}"),
    }
}

/// Suggest the offset for a local date and time in an IANA zone.
///
/// `longitude` (east-positive) is needed for births before standard time, when
/// the correct offset is the mean time of the birth place itself - not the tz
/// database's "LMT", which belongs to the zone's reference city (Kolkata for
/// all of India).
pub fn suggest(
    tzdb: &TzDb,
    tz_name: &str,
    (year, month, day): (i32, u32, u32),
    (hour, minute, second): (u32, u32, u32),
    longitude: Option<f64>,
) -> Result<Suggestion, String> {
    let tz: TimeZone = tzdb.db.get(tz_name).map_err(|_| format!("unknown time zone {tz_name:?}"))?;
    // The kernel's civil-date rules: Julian before 1582-10-15, the reform gap
    // rejected, February checked in the right calendar.
    lagn_core::validate_civil_date(year, month, day).map_err(|e| e.to_string())?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(format!("invalid time {hour:02}:{minute:02}:{second:02}"));
    }
    if let Some(lon) = longitude {
        if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
            return Err(format!("longitude {lon} out of range [-180, 180]"));
        }
    }
    let version = tzdb.version_label();

    // Before the Gregorian reform no standard time existed anywhere, and the
    // tz library's calendar is proleptic Gregorian - it would misread a Julian
    // date (1500-02-29 is valid Julian, invalid Gregorian). Local mean time of
    // the birth place is the only defensible answer.
    if (year, month, day) < (1582, 10, 15) {
        let lon = longitude.ok_or("a longitude is needed to compute local mean time for a date before 1582")?;
        return Ok(Suggestion {
            timezone: tz_name.to_string(),
            kind: Kind::LocalMeanTime,
            confidence: Confidence::Historical,
            candidates: vec![lmt_candidate(lon)],
            notes: vec!["Before standard time existed anywhere: local mean time of the birth place.".into()],
            tzdb_version: version,
        });
    }

    let confidence = if (year, month, day) >= (1970, 1, 1) { Confidence::Reliable } else { Confidence::Historical };
    let mut notes = Vec::new();
    if confidence == Confidence::Historical {
        notes.push("The tz database guarantees offsets only from 1970. Confirm this offset against the birth record.".into());
    }
    if tz_name == "Asia/Kolkata" && confidence == Confidence::Historical {
        notes.push("The tz database models all of India as one zone, so it can't represent local times that differed in particular cities.".into());
    }

    let dt = DateTime::new(year as i16, month as i8, day as i8, hour as i8, minute as i8, second as i8, 0)
        .map_err(|e| format!("invalid date or time: {e}"))?;
    let amb = tz.to_ambiguous_timestamp(dt);
    let abbr_at = |ts: jiff::Timestamp| tz.to_offset_info(ts).abbreviation().to_string();
    let mk = |secs: i32, abbr: &str| Candidate {
        offset_hours: secs as f64 / 3600.0,
        offset_text: offset_text(secs),
        label: if abbr.is_empty() { tz_name.to_string() } else { format!("{abbr} ({tz_name})") },
    };

    let (kind, candidates) = match amb.offset() {
        AmbiguousOffset::Unambiguous { offset } => {
            let ts = amb.compatible().map_err(|e| e.to_string())?;
            let abbr = abbr_at(ts);
            let secs = offset.seconds();
            if abbr == "LMT" {
                notes.push(format!(
                    "Before standard time. The tz database's LMT ({}) is the mean time of {tz_name}'s reference city; the mean time of the birth place itself is usually what a record of that era means.",
                    offset_text(secs)
                ));
                let zone_lmt = mk(secs, "LMT of the zone's reference city");
                match longitude {
                    Some(lon) => (Kind::LocalMeanTime, vec![lmt_candidate(lon), zone_lmt]),
                    None => (Kind::LocalMeanTime, vec![zone_lmt]),
                }
            } else {
                (Kind::Unique, vec![mk(secs, &abbr)])
            }
        }
        AmbiguousOffset::Fold { before, after } => {
            notes.push("This local time occurred twice (clocks were set back). Choose the offset in force at the birth.".into());
            let (e, l) = (amb.earlier().map_err(|e| e.to_string())?, amb.later().map_err(|e| e.to_string())?);
            (Kind::Ambiguous, vec![mk(before.seconds(), &abbr_at(e)), mk(after.seconds(), &abbr_at(l))])
        }
        AmbiguousOffset::Gap { .. } => {
            notes.push("This local time did not exist (clocks were set forward). Check the recorded time.".into());
            (Kind::Nonexistent, vec![])
        }
    };
    Ok(Suggestion { timezone: tz_name.to_string(), kind, confidence, candidates, notes, tzdb_version: version })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_text_formats_seconds_only_when_present() {
        assert_eq!(offset_text(19800), "+05:30");
        assert_eq!(offset_text(19270), "+05:21:10");
        assert_eq!(offset_text(-18000), "-05:00");
        assert_eq!(offset_text(0), "+00:00");
    }
}
