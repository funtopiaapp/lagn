//! Request parsing and response view models.
//!
//! Responses carry the kernel's raw output unchanged (the parity tests compare
//! it bit for bit with the library) plus display strings produced here by the
//! engine, so the browser never formats a degree or computes a date
//! (DESIGN.md section 1).

use lagn_core::format::dms;
use lagn_core::{
    jd_to_civil, Analysis, Ashtakavarga, BirthData, BirthMoment, Chart, ChartSettings,
    DerivationSettings, Dignity, Graha, Varga, Vimshottari,
};
use lagn_rules::resolve::TopicReport;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BirthInput {
    /// YYYY-MM-DD, civil calendar (Julian before 1582-10-15).
    pub date: String,
    /// HH:MM or HH:MM:SS, local wall clock.
    pub time: String,
    pub latitude: f64,
    pub longitude: f64,
    /// The confirmed offset. Never looked up silently.
    pub utc_offset_hours: f64,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub settings: Option<ChartSettings>,
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Strict YYYY-MM-DD.
pub fn parse_date(s: &str) -> Result<(i32, u32, u32), String> {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 || p[0].len() != 4 || p[1].len() != 2 || p[2].len() != 2 || !p.iter().all(|x| digits(x)) {
        return Err(format!("date must be YYYY-MM-DD, got {s:?}"));
    }
    Ok((p[0].parse().unwrap(), p[1].parse().unwrap(), p[2].parse().unwrap()))
}

/// Strict HH:MM or HH:MM:SS.
pub fn parse_time(s: &str) -> Result<(u32, u32, u32), String> {
    let p: Vec<&str> = s.split(':').collect();
    if !(2..=3).contains(&p.len()) || !p.iter().all(|x| x.len() == 2 && digits(x)) {
        return Err(format!("time must be HH:MM or HH:MM:SS, got {s:?}"));
    }
    Ok((p[0].parse().unwrap(), p[1].parse().unwrap(), if p.len() == 3 { p[2].parse().unwrap() } else { 0 }))
}

impl BirthInput {
    pub fn to_chart(&self) -> Result<Chart, String> {
        let (year, month, day) = parse_date(&self.date)?;
        let (hour, minute, second) = parse_time(&self.time)?;
        let birth = BirthData {
            moment: BirthMoment {
                year, month, day, hour, minute,
                second: second as f64,
                utc_offset_hours: self.utc_offset_hours,
            },
            latitude: self.latitude,
            longitude: self.longitude,
            place_name: self.place.clone(),
        };
        Chart::compute(birth, self.settings.unwrap_or_default()).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

/// Civil date in the birth's offset, as the engine reckons it.
pub fn civil_date(jd: f64, offset: f64) -> String {
    let d = jd_to_civil(jd, offset);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

fn dignity_label(d: Option<Dignity>) -> &'static str {
    match d {
        None => "",
        Some(d) => lagn_rules::explain::dignity(d),
    }
}

// ---------------------------------------------------------------------------
// Chart response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PositionView {
    pub key: Graha,
    pub name: &'static str,
    pub abbrev: &'static str,
    pub rasi: &'static str,
    pub rasi_tamil: &'static str,
    pub degrees: String,
    pub nakshatra: &'static str,
    pub nakshatra_tamil: &'static str,
    pub pada: u8,
    pub house: u8,
    pub retrograde: bool,
    pub dignity: &'static str,
    pub combust: bool,
    pub baladi: String,
    pub jagradadi: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LagnaView {
    pub rasi: &'static str,
    pub rasi_tamil: &'static str,
    pub degrees: String,
    pub nakshatra: &'static str,
    pub nakshatra_tamil: &'static str,
    pub pada: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct VargaCellView {
    pub key: Graha,
    pub abbrev: &'static str,
    pub sign: &'static str,
    pub sign_index: u8,
    pub house: u8,
    pub dignity: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VargaView {
    pub varga: Varga,
    pub label: String,
    pub lagna: &'static str,
    pub lagna_index: u8,
    pub grahas: Vec<VargaCellView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeriodView {
    pub lord: &'static str,
    pub start: String,
    pub end: String,
    /// True when this period began before birth and is shown from birth.
    pub from_birth: bool,
    /// True when "now" (server clock) falls inside this period.
    pub current: bool,
    pub start_jd: f64,
    pub end_jd: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<PeriodView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningView {
    pub as_of_utc: String,
    pub maha: &'static str,
    pub antar: &'static str,
    pub pratyantar: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashaView {
    pub janma_nakshatra: &'static str,
    pub janma_nakshatra_tamil: &'static str,
    pub birth_lord: &'static str,
    pub balance_years: f64,
    /// When the birth mahadasha ends - exact, instead of an approximate y/m/d.
    pub balance_until: String,
    pub year_length_days: f64,
    pub mahadashas: Vec<PeriodView>,
    /// Non-deterministic by nature (depends on today); excluded from parity.
    pub running_now: Option<RunningView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignView {
    pub index: u8,
    pub name: &'static str,
    pub tamil: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct BavRow {
    pub graha: &'static str,
    pub bindus: [u8; 12],
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AshtakavargaView {
    /// Indexed by sign, Mesha first.
    pub rows: Vec<BavRow>,
    pub sav: [u8; 12],
    /// SAV rotated so index 0 is the 1st house.
    pub sav_by_house: [u8; 12],
    pub sav_total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChartResponse {
    pub input: BirthInput,
    /// The twelve signs, for labelling empty chart cells.
    pub signs: Vec<SignView>,
    pub lagna_index: u8,
    pub ashtakavarga: AshtakavargaView,
    pub julian_day_ut: f64,
    pub ayanamsa: String,
    pub lagna: LagnaView,
    pub positions: Vec<PositionView>,
    pub vargas: Vec<VargaView>,
    pub dasha: DashaView,
    /// Raw kernel output, unchanged.
    pub raw: RawChart,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawChart {
    pub chart: Chart,
    pub analysis: Analysis,
    pub ashtakavarga: Ashtakavarga,
    pub vimshottari: Vimshottari,
}

fn periods(ps: &[lagn_core::DashaPeriod], birth_jd: f64, tz: f64, now: Option<f64>) -> Vec<PeriodView> {
    ps.iter()
        .filter(|p| p.end_jd > birth_jd)
        .map(|p| {
            let start_jd = p.start_jd.max(birth_jd);
            PeriodView {
                lord: p.lord.name(),
                start: civil_date(start_jd, tz),
                end: civil_date(p.end_jd, tz),
                from_birth: p.start_jd < birth_jd,
                current: now.is_some_and(|n| p.contains(n)),
                start_jd,
                end_jd: p.end_jd,
                children: periods(&p.children, birth_jd, tz, now),
            }
        })
        .collect()
}

/// Julian Day of "now", from the system clock.
pub fn jd_now() -> f64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    2440587.5 + secs / 86400.0
}

pub fn chart_response(input: BirthInput, chart: Chart, settings: &DerivationSettings, now_jd: Option<f64>) -> ChartResponse {
    let tz = chart.birth.moment.utc_offset_hours;
    let analysis = chart.analyse(settings);
    let ashtakavarga = Ashtakavarga::compute(&chart);
    let vim = Vimshottari::compute(&chart);

    let positions = chart
        .placements
        .iter()
        .map(|p| {
            let c = analysis.conditions.iter().find(|k| k.graha == p.graha).expect("nine conditions");
            PositionView {
                key: p.graha,
                name: p.graha.name(),
                abbrev: p.graha.abbrev(),
                rasi: p.rasi.name(),
                rasi_tamil: p.rasi.tamil_name(),
                degrees: dms(p.degrees_in_rasi),
                nakshatra: p.nakshatra.nakshatra.name(),
                nakshatra_tamil: p.nakshatra.nakshatra.tamil_name(),
                pada: p.nakshatra.pada,
                house: p.house,
                retrograde: p.retrograde,
                dignity: dignity_label(c.dignity),
                combust: c.combust,
                baladi: format!("{:?}", c.baladi),
                jagradadi: c.jagradadi.map(|j| format!("{j:?}")).unwrap_or_default(),
            }
        })
        .collect();

    let vargas = Varga::ALL
        .into_iter()
        .map(|v| {
            let vc = chart.varga(v);
            VargaView {
                varga: v,
                label: v.label(),
                lagna: vc.lagna.name(),
                lagna_index: vc.lagna.index(),
                grahas: Graha::ALL
                    .into_iter()
                    .map(|g| VargaCellView {
                        key: g,
                        abbrev: g.abbrev(),
                        sign: vc.sign_of(g).name(),
                        sign_index: vc.sign_of(g).index(),
                        house: vc.house_of(g),
                        dignity: dignity_label(chart.dignity(g, v, settings)),
                    })
                    .collect(),
            }
        })
        .collect();

    let running_now = now_jd.and_then(|jd| {
        vim.at(jd).map(|c| {
            let d = jd_to_civil(jd, 0.0);
            RunningView {
                as_of_utc: format!("{:04}-{:02}-{:02}", d.year, d.month, d.day),
                maha: c.maha.name(),
                antar: c.antar.name(),
                pratyantar: c.pratyantar.name(),
            }
        })
    });

    let dasha = DashaView {
        janma_nakshatra: vim.janma_nakshatra.name(),
        janma_nakshatra_tamil: vim.janma_nakshatra.tamil_name(),
        birth_lord: vim.birth_lord.name(),
        balance_years: vim.balance_years,
        balance_until: civil_date(vim.mahadashas[0].end_jd, tz),
        year_length_days: vim.year_length.days(),
        mahadashas: periods(&vim.mahadashas, vim.birth_jd, tz, now_jd),
        running_now,
    };

    let ashtakavarga_view = AshtakavargaView {
        rows: lagn_core::relationship::SEVEN
            .iter()
            .enumerate()
            .map(|(i, g)| BavRow {
                graha: g.name(),
                bindus: ashtakavarga.bav[i],
                total: ashtakavarga.bav[i].iter().map(|&x| x as u32).sum(),
            })
            .collect(),
        sav: ashtakavarga.sav,
        sav_by_house: core::array::from_fn(|h| ashtakavarga.sav_in_house(h as u8 + 1)),
        sav_total: ashtakavarga.sav.iter().map(|&x| x as u32).sum(),
    };

    ChartResponse {
        signs: lagn_core::Rasi::ALL
            .into_iter()
            .map(|r| SignView { index: r.index(), name: r.name(), tamil: r.tamil_name() })
            .collect(),
        lagna_index: chart.lagna.rasi.index(),
        ashtakavarga: ashtakavarga_view,
        julian_day_ut: chart.jd_ut,
        ayanamsa: dms(chart.ayanamsa_value),
        lagna: LagnaView {
            rasi: chart.lagna.rasi.name(),
            rasi_tamil: chart.lagna.rasi.tamil_name(),
            degrees: dms(chart.lagna.degrees_in_rasi),
            nakshatra: chart.lagna.nakshatra.nakshatra.name(),
            nakshatra_tamil: chart.lagna.nakshatra.nakshatra.tamil_name(),
            pada: chart.lagna.nakshatra.pada,
        },
        positions,
        vargas,
        dasha,
        raw: RawChart { chart: chart.clone(), analysis, ashtakavarga, vimshottari: vim },
        input,
    }
}

// ---------------------------------------------------------------------------
// Topic response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct WindowView {
    pub rule: String,
    pub start: String,
    pub end: String,
    pub maha: &'static str,
    pub antar: &'static str,
    pub matched: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicResponse {
    pub report: TopicReport,
    pub windows: Vec<WindowView>,
    /// Title, summary and disclaimer from the topic catalogue.
    pub meta: Option<lagn_rules::catalogue::TopicMeta>,
    /// Pariharams for the effective afflicting findings.
    pub pariharams: Vec<lagn_rules::pariharam::Suggested>,
    /// The written interpretation (DESIGN section 4a).
    pub writeup: Option<lagn_rules::writeup::WriteUp>,
}

pub fn topic_response(report: TopicReport, tz: f64) -> TopicResponse {
    let windows = report
        .timing
        .iter()
        .map(|w| WindowView {
            rule: w.rule.clone(),
            start: civil_date(w.start_jd, tz),
            end: civil_date(w.end_jd, tz),
            maha: w.maha.name(),
            antar: w.antar.name(),
            matched: w.matched.iter().map(|g| g.name()).collect(),
        })
        .collect();
    TopicResponse { report, windows, meta: None, pariharams: Vec::new(), writeup: None }
}

// ---------------------------------------------------------------------------
// Phase 6: sensitive periods and transits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct TransitView {
    pub start: String,
    pub end: String,
    pub label: &'static str,
    pub graha: &'static str,
    pub sign: &'static str,
    pub house_from_moon: u8,
    pub supportive: bool,
    pub bindus: Option<u8>,
    pub bav_supports: Option<bool>,
}

pub fn transit_view(t: &lagn_core::TransitWindow, tz: f64) -> TransitView {
    TransitView {
        start: civil_date(t.start_jd, tz),
        end: civil_date(t.end_jd, tz),
        label: t.kind.label(),
        graha: t.graha.name(),
        sign: t.sign.name(),
        house_from_moon: t.house_from_moon,
        supportive: t.supportive,
        bindus: t.bindus,
        bav_supports: t.bav_supports,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PeriodView6 {
    pub start: String,
    pub end: String,
    pub maha_name: &'static str,
    pub antar_name: &'static str,
    /// True when the window contains the moment of the request.
    pub current: bool,
    #[serde(flatten)]
    pub detail: lagn_rules::reading::PeriodDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeriodsResponse {
    pub mode: lagn_rules::Mode,
    pub withheld: std::collections::BTreeMap<String, usize>,
    pub moon_sign: &'static str,
    pub windows: Vec<PeriodView6>,
    pub transits: Vec<TransitView>,
}

pub fn periods_response(
    r: lagn_rules::reading::SensitivePeriods,
    t: &lagn_rules::reading::TransitReport,
    tz: f64,
    now_jd: f64,
) -> PeriodsResponse {
    PeriodsResponse {
        mode: r.mode,
        withheld: r.withheld,
        moon_sign: t.moon_sign.name(),
        windows: r
            .windows
            .into_iter()
            .map(|d| PeriodView6 {
                start: civil_date(d.window.start_jd, tz),
                end: civil_date(d.window.end_jd, tz),
                maha_name: d.window.maha.name(),
                antar_name: d.window.antar.name(),
                current: d.window.start_jd <= now_jd && now_jd < d.window.end_jd,
                detail: d,
            })
            .collect(),
        transits: t.windows.iter().map(|w| transit_view(w, tz)).collect(),
    }
}
