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
    /// Minutes of clock time either side of the birth moment for which the
    /// lagna stays in this rasi, and the nearer of the two. Every house is
    /// counted from the lagna, so `margin_minutes` is how wrong the birth
    /// time can be before the whole reading shifts by a house.
    pub holds_for: LagnaHoldView,
}

#[derive(Debug, Clone, Serialize)]
pub struct LagnaHoldView {
    pub minutes_in: f64,
    pub minutes_left: f64,
    pub margin_minutes: f64,
    /// True when the search hit its 12-hour cap instead of finding a
    /// boundary, which happens only at extreme latitudes.
    pub capped: bool,
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

// ---------------------------------------------------------------------------
// Jaimini core (phase 13A). Specification: docs/phase13/DESIGN.md.
//
// The engine serialises its enums as identifiers - "rahu", "karka", "mars" -
// which are keys, not labels. Displaying them raw put lowercase names on the
// Jaimini tab, and "mars" where every other view in this app says "Kuja".
// So the display names are produced here, from the same `name()` methods the
// rest of the views use, and the stable id travels alongside for keying.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct KarakaView {
    /// Stable id, for keying and for tests: "atma", "amatya", and so on.
    pub id: &'static str,
    pub abbrev: &'static str,
    pub name: &'static str,
    pub signifies: &'static str,
    pub graha: &'static str,
    pub rasi: &'static str,
    pub advancement: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArudhaView {
    pub bhava: u8,
    /// "AL", "A7", "UL" - what practice calls it.
    pub label: String,
    pub bhava_rasi: &'static str,
    pub lord: &'static str,
    pub lord_rasi: &'static str,
    pub count: u8,
    pub raw: &'static str,
    pub rasi: &'static str,
    pub adjusted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgalaPairView {
    pub kind: &'static str,
    pub argala_house: u8,
    pub counter_house: u8,
    pub argala_rasi: &'static str,
    pub counter_rasi: &'static str,
    pub argala_grahas: Vec<&'static str>,
    pub counter_grahas: Vec<&'static str>,
    pub verdict: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgalaView {
    pub rasi: &'static str,
    pub pairs: Vec<ArgalaPairView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JaiminiView {
    pub karakas: Vec<KarakaView>,
    pub padas: Vec<ArudhaView>,
    pub argala: Vec<ArgalaView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

fn karaka_id(k: lagn_core::jaimini::Karaka) -> &'static str {
    use lagn_core::jaimini::Karaka as K;
    match k {
        K::Atma => "atma", K::Amatya => "amatya", K::Bhratri => "bhratri", K::Matri => "matri",
        K::Pitri => "pitri", K::Putra => "putra", K::Gnati => "gnati", K::Dara => "dara",
    }
}

fn verdict_name(v: lagn_core::jaimini::ArgalaVerdict) -> &'static str {
    use lagn_core::jaimini::ArgalaVerdict as V;
    match v {
        V::Stands => "stands",
        V::Neutralised => "neutralised",
        V::Overcome => "overcome",
        V::None => "none",
    }
}

pub fn jaimini_view(j: &lagn_core::jaimini::Jaimini) -> JaiminiView {
    JaiminiView {
        karakas: j
            .karakas
            .assigned
            .iter()
            .map(|a| KarakaView {
                id: karaka_id(a.karaka),
                abbrev: a.karaka.abbrev(),
                name: a.karaka.name(),
                signifies: a.karaka.signifies(),
                graha: a.graha.name(),
                rasi: a.rasi.name(),
                advancement: a.advancement,
            })
            .collect(),
        padas: j
            .padas
            .iter()
            .map(|p| ArudhaView {
                bhava: p.bhava,
                label: p.label(),
                bhava_rasi: p.bhava_rasi.name(),
                lord: p.lord.name(),
                lord_rasi: p.lord_rasi.name(),
                count: p.count,
                raw: p.raw.name(),
                rasi: p.rasi.name(),
                adjusted: p.adjusted,
            })
            .collect(),
        argala: j
            .argala
            .iter()
            .map(|a| ArgalaView {
                rasi: a.rasi.name(),
                pairs: a
                    .pairs
                    .iter()
                    .map(|p| {
                        let (ah, ch) = p.kind.houses();
                        ArgalaPairView {
                            kind: p.kind.name(),
                            argala_house: ah,
                            counter_house: ch,
                            argala_rasi: p.argala_rasi.name(),
                            counter_rasi: p.counter_rasi.name(),
                            argala_grahas: p.argala_grahas.iter().map(|g| g.name()).collect(),
                            counter_grahas: p.counter_grahas.iter().map(|g| g.name()).collect(),
                            verdict: verdict_name(p.verdict),
                        }
                    })
                    .collect(),
            })
            .collect(),
        variants: j.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Yogi, Avayogi and the Yoga sphuta (phase 13D).
// Specification: docs/phase13/YOGI.md.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct YogiView {
    pub longitude: f64,
    pub degrees: String,
    pub rasi: &'static str,
    pub rasi_tamil: &'static str,
    pub nakshatra: &'static str,
    pub pada: u8,
    pub yogi: &'static str,
    pub avayogi_nakshatra: &'static str,
    pub avayogi: &'static str,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

pub fn yogi_view(y: &lagn_core::yogi::Yogi) -> YogiView {
    YogiView {
        longitude: y.sphuta.longitude,
        degrees: lagn_core::format::dms(y.sphuta.degrees_in_rasi),
        rasi: y.sphuta.rasi.name(),
        rasi_tamil: y.sphuta.rasi.tamil_name(),
        nakshatra: y.sphuta.nakshatra.name(),
        pada: y.sphuta.pada,
        yogi: y.yogi.name(),
        avayogi_nakshatra: y.avayogi_nakshatra.name(),
        avayogi: y.avayogi.name(),
        variants: y.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Ashtakavarga for the professional surface (phase 13F).
//
// The engine has computed BAV, SAV and the prastara since phase 2B, with
// fixed per-graha totals and a fixed SAV total of 337 - which this view
// reports alongside so a reader can see the invariant hold rather than take
// the grid on trust. Nothing new is computed here; this is presentation.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BavRowView {
    pub graha: &'static str,
    /// Bindus by sign, Mesha first.
    pub bindus: Vec<u8>,
    /// Bindus by house from the lagna, 1 to 12.
    pub by_house: Vec<u8>,
    pub total: u32,
    /// The fixed total this graha's BAV always comes to.
    pub expected_total: u32,
}

/// The full grid, for the professional surface.
///
/// Deliberately separate from `AshtakavargaView`, the lean summary that rides
/// on every chart response: this one carries the fixed-total invariants and a
/// seven-by-twelve contributor matrix, which would be waste on a request that
/// only wants a chart.
#[derive(Debug, Clone, Serialize)]
pub struct AshtakavargaGridView {
    pub lagna: &'static str,
    pub signs: Vec<&'static str>,
    pub rows: Vec<BavRowView>,
    pub sav: Vec<u8>,
    pub sav_by_house: Vec<u8>,
    pub sav_total: u32,
    /// 337, always, for every chart ever cast.
    pub sav_expected_total: u32,
    /// Which contributors gave each bindu, per graha and sign.
    pub contributors: Vec<Vec<Vec<&'static str>>>,
}

pub fn ashtakavarga_grid_view(
    chart: &Chart,
    av: &lagn_core::Ashtakavarga,
) -> AshtakavargaGridView {
    use lagn_core::ashtakavarga::{BAV_TOTALS, SAV_TOTAL};
    let seven = lagn_core::relationship::SEVEN;

    AshtakavargaGridView {
        lagna: chart.lagna.rasi.name(),
        signs: lagn_core::Rasi::ALL.iter().map(|r| r.name()).collect(),
        rows: seven
            .iter()
            .enumerate()
            .map(|(i, &g)| {
                let bav = av.bav_of(g).expect("the seven have a BAV");
                BavRowView {
                    graha: g.name(),
                    bindus: bav.to_vec(),
                    by_house: (1..=12u8)
                        .map(|h| {
                            let sign = lagn_core::Rasi::from_index(
                                chart.lagna.rasi.index() as i32 + h as i32 - 1,
                            );
                            bav[sign.index() as usize]
                        })
                        .collect(),
                    total: bav.iter().map(|&x| x as u32).sum(),
                    expected_total: BAV_TOTALS[i],
                }
            })
            .collect(),
        sav: av.sav.to_vec(),
        sav_by_house: (1..=12u8).map(|h| av.sav_in_house(h)).collect(),
        sav_total: av.sav.iter().map(|&x| x as u32).sum(),
        sav_expected_total: SAV_TOTAL,
        contributors: seven
            .iter()
            .map(|&g| {
                lagn_core::Rasi::ALL
                    .iter()
                    .map(|&r| {
                        av.contributors(g, r)
                            .unwrap_or_default()
                            .into_iter()
                            .map(contributor_name)
                            .collect()
                    })
                    .collect()
            })
            .collect(),
    }
}

fn contributor_name(c: lagn_core::Contributor) -> &'static str {
    use lagn_core::Contributor as C;
    match c {
        C::Sun => "Surya",
        C::Moon => "Chandra",
        C::Mars => "Kuja",
        C::Mercury => "Budha",
        C::Jupiter => "Guru",
        C::Venus => "Shukra",
        C::Saturn => "Shani",
        C::Lagna => "Lagna",
    }
}

// ---------------------------------------------------------------------------
// Yogini dasha (phase 13D). Specification: docs/phase13/YOGINI.md.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct YoginiPeriodView {
    pub yogini: &'static str,
    pub lord: &'static str,
    pub years: f64,
    pub start: String,
    pub end: String,
    pub cycle: u32,
    pub start_jd: f64,
    pub end_jd: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<YoginiPeriodView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YoginiRunningView {
    pub as_of_utc: String,
    pub maha: &'static str,
    pub antar: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct YoginiView {
    pub janma_nakshatra: &'static str,
    pub birth_yogini: &'static str,
    pub birth_lord: &'static str,
    pub balance_years: f64,
    /// When the yogini running at birth gives way.
    pub balance_until: String,
    pub cycle_years: f64,
    pub periods: Vec<YoginiPeriodView>,
    /// Depends on today, so excluded from parity, as the Vimshottari view does.
    pub running_now: Option<YoginiRunningView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

pub fn yogini_view(
    chart: &Chart,
    d: &lagn_core::yogini::YoginiDasha,
    now: Option<f64>,
) -> YoginiView {
    let tz = chart.birth.moment.utc_offset_hours;
    let one = |p: &lagn_core::yogini::YoginiPeriod| YoginiPeriodView {
        yogini: p.yogini.name(),
        lord: p.lord.name(),
        years: p.yogini.years(),
        start: civil_date(p.start_jd, tz),
        end: civil_date(p.end_jd, tz),
        cycle: p.cycle,
        start_jd: p.start_jd,
        end_jd: p.end_jd,
        children: Vec::new(),
    };
    YoginiView {
        janma_nakshatra: d.janma_nakshatra.name(),
        birth_yogini: d.birth_yogini.name(),
        birth_lord: d.birth_yogini.lord().name(),
        balance_years: d.balance_years,
        balance_until: civil_date(d.periods[0].end_jd, tz),
        cycle_years: lagn_core::yogini::CYCLE_YEARS,
        periods: d
            .periods
            .iter()
            .map(|p| YoginiPeriodView { children: p.children.iter().map(one).collect(), ..one(p) })
            .collect(),
        running_now: now.and_then(|jd| {
            d.at(jd).map(|c| YoginiRunningView {
                as_of_utc: {
                    let t = jd_to_civil(jd, 0.0);
                    format!("{:04}-{:02}-{:02}", t.year, t.month, t.day)
                },
                maha: c.maha.name(),
                antar: c.antar.name(),
            })
        }),
        variants: d.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Shadbala (phase 13E). Specification: docs/phase13/SHADBALA.md.
//
// Ships labelled: every value depends on variant choices no astrologer has
// signed off, and one of the six components is not computed. The caveat and
// the variant IDs travel with the numbers rather than beside them.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BalaRowView {
    pub graha: &'static str,
    pub sthana: f64,
    pub uchcha: f64,
    pub saptavargaja: f64,
    pub ojayugma: f64,
    pub kendra: f64,
    pub drekkana: f64,
    pub dig: f64,
    pub kala: f64,
    pub nathonnatha: f64,
    pub paksha: f64,
    pub tribhaga: f64,
    pub abda: f64,
    pub masa: f64,
    pub vara: f64,
    pub hora: f64,
    pub ayana: f64,
    pub cheshta: f64,
    pub cheshta_from_ayana: bool,
    pub naisargika: f64,
    pub total_virupas: f64,
    pub total_rupas: f64,
    pub customary_minimum_rupas: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BalaView {
    /// Always false while Drik is held.
    pub drik_included: bool,
    pub components_computed: u8,
    pub caveat: String,
    pub rows: Vec<BalaRowView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

pub fn bala_view(b: &lagn_core::bala::Bala) -> BalaView {
    BalaView {
        drik_included: b.drik_included,
        components_computed: b.components_computed,
        caveat: b.caveat.to_string(),
        rows: b
            .grahas
            .iter()
            .map(|g| BalaRowView {
                graha: g.graha.name(),
                sthana: g.sthana.total,
                uchcha: g.sthana.uchcha,
                saptavargaja: g.sthana.saptavargaja,
                ojayugma: g.sthana.ojayugma,
                kendra: g.sthana.kendra,
                drekkana: g.sthana.drekkana,
                dig: g.dig,
                kala: g.kala.total,
                nathonnatha: g.kala.nathonnatha,
                paksha: g.kala.paksha,
                tribhaga: g.kala.tribhaga,
                abda: g.kala.abda,
                masa: g.kala.masa,
                vara: g.kala.vara,
                hora: g.kala.hora,
                ayana: g.kala.ayana,
                cheshta: g.cheshta.value,
                cheshta_from_ayana: g.cheshta.from_ayana,
                naisargika: g.naisargika,
                total_virupas: g.total_virupas,
                total_rupas: g.total_rupas,
                customary_minimum_rupas: g.customary_minimum_rupas,
            })
            .collect(),
        variants: b.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// The annual chart, Muntha and kaksha (phase 13F/13G).
// Specification: docs/phase13/VARSHA-KAKSHA.md.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct KakshaView {
    pub graha: &'static str,
    pub rasi: &'static str,
    pub degrees: String,
    pub kaksha: u8,
    pub owner: String,
    pub supported: bool,
    pub bindus: u8,
}

/// A position in the annual chart. Deliberately leaner than `PositionView`:
/// that one carries dignity, combustion and avastha, which need a full
/// `Analysis` the annual chart has no use for.
#[derive(Debug, Clone, Serialize)]
pub struct VarshaPositionView {
    pub graha: &'static str,
    pub rasi: &'static str,
    pub rasi_tamil: &'static str,
    pub degrees: String,
    pub house: u8,
    pub retrograde: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VarshaView {
    pub age: u32,
    /// The moment the year begins, in the birth place's own wall clock.
    pub begins: String,
    /// How far the annual Sun is from the natal Sun, in degrees. Shown so a
    /// reader can see the solve closed rather than take it on trust.
    pub sun_error: f64,
    pub lagna: &'static str,
    pub muntha: &'static str,
    pub muntha_house: u8,
    pub positions: Vec<VarshaPositionView>,
    pub kaksha: Vec<KakshaView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

pub fn varsha_view(v: &lagn_core::varsha::Varshaphala) -> VarshaView {
    let tz = v.chart.birth.moment.utc_offset_hours;
    let d = jd_to_civil(v.return_jd, tz);
    VarshaView {
        age: v.age,
        begins: format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            d.year, d.month, d.day, d.hour, d.minute
        ),
        sun_error: v.sun_error,
        lagna: v.chart.lagna.rasi.name(),
        muntha: v.muntha.name(),
        muntha_house: v.muntha_house,
        positions: v
            .chart
            .placements
            .iter()
            .map(|p| VarshaPositionView {
                graha: p.graha.name(),
                rasi: p.rasi.name(),
                rasi_tamil: p.rasi.tamil_name(),
                degrees: lagn_core::format::dms(p.degrees_in_rasi),
                house: p.house,
                retrograde: p.retrograde,
            })
            .collect(),
        kaksha: v
            .kaksha
            .iter()
            .map(|k| KakshaView {
                graha: k.graha.name(),
                rasi: k.rasi.name(),
                degrees: lagn_core::format::dms(lagn_core::Rasi::degrees_within(k.longitude)),
                kaksha: k.kaksha,
                owner: format!("{:?}", k.owner),
                supported: k.supported,
                bindus: k.bindus,
            })
            .collect(),
        variants: v.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Krishnamurti Paddhati (phase 13H). Specification: docs/phase13/KP.md.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct LordsView {
    pub longitude: f64,
    pub degrees: String,
    pub rasi: &'static str,
    pub nakshatra: &'static str,
    pub pada: u8,
    pub sign_lord: &'static str,
    pub star_lord: &'static str,
    pub sub_lord: &'static str,
    pub sub_sub_lord: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct KpCuspView {
    pub house: u8,
    #[serde(flatten)]
    pub lords: LordsView,
}

#[derive(Debug, Clone, Serialize)]
pub struct KpGrahaView {
    pub graha: &'static str,
    #[serde(flatten)]
    pub lords: LordsView,
}

#[derive(Debug, Clone, Serialize)]
pub struct RulingPlanetView {
    pub role: String,
    pub graha: &'static str,
    pub sub_lord: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignificatorView {
    pub graha: &'static str,
    /// 1 to 4, strongest first.
    pub rank: u8,
    pub because: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct HouseSignificatorsView {
    pub house: u8,
    pub significators: Vec<SignificatorView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KpView {
    pub ascendant: LordsView,
    pub grahas: Vec<KpGrahaView>,
    pub cusps: Vec<KpCuspView>,
    pub ruling_planets: Vec<RulingPlanetView>,
    pub significators: Vec<HouseSignificatorsView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

fn lords_view(l: &lagn_core::kp::Lords) -> LordsView {
    LordsView {
        longitude: l.longitude,
        degrees: lagn_core::format::dms(l.longitude - 30.0 * (l.rasi.index() as f64)),
        rasi: l.rasi.name(),
        nakshatra: l.nakshatra.name(),
        pada: l.pada,
        sign_lord: l.sign_lord.name(),
        star_lord: l.star_lord.name(),
        sub_lord: l.sub_lord.name(),
        sub_sub_lord: l.sub_sub_lord.name(),
    }
}

pub fn kp_view(kp: &lagn_core::kp::Kp) -> KpView {
    KpView {
        ascendant: lords_view(&kp.ascendant),
        grahas: kp
            .grahas
            .iter()
            .map(|(g, l)| KpGrahaView { graha: g.name(), lords: lords_view(l) })
            .collect(),
        cusps: kp
            .cusps
            .iter()
            .map(|c| KpCuspView { house: c.house, lords: lords_view(&c.lords) })
            .collect(),
        ruling_planets: kp
            .ruling_planets
            .iter()
            .map(|r| RulingPlanetView {
                role: r.role.to_string(),
                graha: r.graha.name(),
                sub_lord: r.sub_lord.map(|g| g.name()),
            })
            .collect(),
        significators: kp
            .significators
            .iter()
            .map(|h| HouseSignificatorsView {
                house: h.house,
                significators: h
                    .significators
                    .iter()
                    .map(|s| SignificatorView {
                        graha: s.graha.name(),
                        rank: s.group.rank(),
                        because: s.group.describe(),
                    })
                    .collect(),
            })
            .collect(),
        variants: kp.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Upagrahas and the time lagnas (phase 13C).
// Specification: docs/phase13/UPAGRAHA.md.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PointView {
    pub name: String,
    /// Formatted as degrees, minutes and seconds within the sign.
    pub degrees: String,
    pub longitude: f64,
    pub rasi: &'static str,
    pub rasi_tamil: &'static str,
    pub house: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayPartView {
    #[serde(flatten)]
    pub point: PointView,
    pub ruler: &'static str,
    pub part: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpagrahaView {
    pub at_night: bool,
    pub vara: &'static str,
    pub vara_lord: &'static str,
    pub hours_since_sunrise: f64,
    pub sun_offsets: Vec<PointView>,
    pub day_parts: Vec<DayPartView>,
    pub time_lagnas: Vec<PointView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

fn point_view(p: &lagn_core::upagraha::Point) -> PointView {
    PointView {
        name: p.name.to_string(),
        degrees: lagn_core::format::dms(p.degrees_in_rasi),
        longitude: p.longitude,
        rasi: p.rasi.name(),
        rasi_tamil: p.rasi.tamil_name(),
        house: p.house,
    }
}

pub fn upagraha_view(u: &lagn_core::upagraha::Upagrahas) -> UpagrahaView {
    UpagrahaView {
        at_night: u.at_night,
        vara: u.vara.english(),
        vara_lord: u.vara.lord().name(),
        hours_since_sunrise: u.hours_since_sunrise,
        sun_offsets: u.sun_offsets.iter().map(point_view).collect(),
        day_parts: u
            .day_parts
            .iter()
            .map(|d| DayPartView { point: point_view(&d.point), ruler: d.ruler.name(), part: d.part })
            .collect(),
        time_lagnas: u.time_lagnas.iter().map(point_view).collect(),
        variants: u.variants.clone(),
    }
}

// ---------------------------------------------------------------------------
// Chara dasha (phase 13B). Specification: docs/phase13/CHARA-DASHA.md.
//
// The engine returns Julian Days. They are formatted here, on the server,
// because the browser does no calendar arithmetic (DESIGN.md section 1,
// rule 2) - a Pro surface is no excuse to start.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CharaLengthView {
    pub rasi: &'static str,
    pub lord: &'static str,
    pub lord_rasi: &'static str,
    /// Which way the count ran: this sign's own parity (V-13-10).
    pub direction: &'static str,
    pub count: u8,
    pub years: f64,
    pub lord_at_home: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharaPeriodView {
    pub rasi: &'static str,
    pub start: String,
    pub end: String,
    pub cycle: u32,
    pub start_jd: f64,
    pub end_jd: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CharaPeriodView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharaRunningView {
    pub as_of_utc: String,
    pub maha: &'static str,
    pub antar: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharaView {
    pub lagna: &'static str,
    /// Why the sequence runs the way it does (V-13-9).
    pub lagna_is_odd: bool,
    pub direction: &'static str,
    pub cycle_years: f64,
    pub year_length_days: f64,
    pub lengths: Vec<CharaLengthView>,
    pub periods: Vec<CharaPeriodView>,
    /// Non-deterministic by nature (depends on today); excluded from parity,
    /// exactly as the Vimshottari view does it.
    pub running_now: Option<CharaRunningView>,
    pub variants: Vec<lagn_core::jaimini::VariantChoice>,
}

fn direction_name(d: lagn_core::chara::Direction) -> &'static str {
    match d {
        lagn_core::chara::Direction::Direct => "direct",
        lagn_core::chara::Direction::Reverse => "reverse",
    }
}

pub fn chara_view(chart: &Chart, d: &lagn_core::chara::CharaDasha, now: Option<f64>) -> CharaView {
    let tz = chart.birth.moment.utc_offset_hours;
    let period = |p: &lagn_core::chara::CharaPeriod| CharaPeriodView {
        rasi: p.rasi.name(),
        start: civil_date(p.start_jd, tz),
        end: civil_date(p.end_jd, tz),
        cycle: p.cycle,
        start_jd: p.start_jd,
        end_jd: p.end_jd,
        children: Vec::new(),
    };
    CharaView {
        lagna: d.lagna.name(),
        lagna_is_odd: d.lagna.is_odd(),
        direction: direction_name(d.direction),
        cycle_years: d.cycle_years(),
        year_length_days: d.year_length.days(),
        lengths: d
            .lengths
            .iter()
            .map(|l| CharaLengthView {
                rasi: l.rasi.name(),
                lord: l.lord.name(),
                lord_rasi: l.lord_rasi.name(),
                direction: direction_name(l.direction),
                count: l.count,
                years: l.years,
                lord_at_home: l.lord_at_home,
            })
            .collect(),
        periods: d
            .periods
            .iter()
            .map(|p| CharaPeriodView { children: p.children.iter().map(period).collect(), ..period(p) })
            .collect(),
        running_now: now.and_then(|jd| {
            d.at(jd).map(|c| CharaRunningView {
                as_of_utc: {
                    let t = jd_to_civil(jd, 0.0);
                    format!("{:04}-{:02}-{:02}", t.year, t.month, t.day)
                },
                maha: c.maha.name(),
                antar: c.antar.name(),
            })
        }),
        variants: d.variants.clone(),
    }
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
            holds_for: {
                // A chart that computed cannot fail here: the same ascendant
                // at a nearby instant. If it somehow does, report no margin
                // rather than dropping the whole response.
                let w = chart.lagna_window().unwrap_or(lagn_core::LagnaWindow {
                    minutes_in: 0.0,
                    minutes_left: 0.0,
                });
                LagnaHoldView {
                    minutes_in: w.minutes_in,
                    minutes_left: w.minutes_left,
                    margin_minutes: w.margin_minutes(),
                    capped: w.minutes_in >= 719.0 || w.minutes_left >= 719.0,
                }
            },
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
    /// How the period rules judge this stretch for this area: "favourable",
    /// "mixed", "asks for care", or "no period factor applies".
    pub verdict: &'static str,
    /// True when the topic's own timing rules single out this stretch.
    pub live: bool,
    /// Ranking across every stretch in range: best, good, mixed, caution,
    /// not judged.
    pub rank: &'static str,
    /// "past", "now" or "ahead", against the server's clock. Someone asking
    /// when to do a thing needs the stretches still to come; the frontend
    /// cannot work that out because it does no calendar arithmetic.
    pub when: &'static str,
    pub score: i32,
    /// Lines that name the area, so "supports" never stands on its own.
    pub supports: Vec<String>,
    pub cautions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicResponse {
    pub report: TopicReport,
    pub windows: Vec<WindowView>,
    /// Title, summary and disclaimer from the topic catalogue.
    pub meta: Option<lagn_rules::catalogue::TopicMeta>,
    /// The written interpretation (DESIGN section 4a).
    pub writeup: Option<lagn_rules::writeup::WriteUp>,
}

pub fn topic_response(
    report: TopicReport,
    tz: f64,
    judged: &[lagn_rules::reading::TopicWindow],
    now_jd: Option<f64>,
) -> TopicResponse {
    // Every stretch in range, ranked - not only the ones the timing rules name.
    // Listing only those left holes a reader could not distinguish from a
    // quiet period. `rule` names the timing rule where the topic is live.
    let rule_id = report.timing.first().map(|w| w.rule.clone()).unwrap_or_default();
    let windows = judged
        .iter()
        .map(|w| {
            let j = Some(w);
            WindowView {
                rule: if w.live { rule_id.clone() } else { String::new() },
                start: civil_date(w.start_jd, tz),
                end: civil_date(w.end_jd, tz),
                maha: w.maha.name(),
                antar: w.antar.name(),
                matched: w.matched.iter().map(|g| g.name()).collect(),
                verdict: j.map(|j| j.verdict.label()).unwrap_or("no period factor applies"),
                live: j.map(|j| j.live).unwrap_or(false),
                rank: j.map(|j| j.rank.label()).unwrap_or("not judged"),
                when: match now_jd {
                    Some(n) if n >= w.end_jd => "past",
                    Some(n) if n >= w.start_jd => "now",
                    Some(_) => "ahead",
                    // No clock given: nothing is claimed about when it falls.
                    None => "unknown",
                },
                score: j.map(|j| j.score).unwrap_or(0),
                supports: j.map(|j| j.supports.clone()).unwrap_or_default(),
                cautions: j.map(|j| j.cautions.clone()).unwrap_or_default(),
            }
        })
        .collect();
    TopicResponse { report, windows, meta: None, writeup: None }
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
