//! Shadbala: five of the six strengths, in virupas.
//!
//! Specification: `docs/phase13/SHADBALA.md`.
//!
//! This is the one group in the engine that ships *labelled*. `docs/phase2`
//! held it from the start because several sub-components have more than one
//! published formula and no reference output is available here. The product
//! owner's decision was to build it with the alternatives recorded, so every
//! value carries the variant IDs it depends on, nothing here appears on the
//! Lite surface, and no corpus rule may consume it - a reviewed
//! interpretation resting on an unverified number would launder the caveat
//! away.
//!
//! Drik bala is **not computed**. It needs sphuta drishti, which is
//! degree-based and disputed; the engine's whole-sign drishti would produce a
//! plausible number with no error bar. Totals say they are five of six.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::dignity::{self, Dignity};
use crate::jaimini::VariantChoice;
use crate::rasi::Rasi;
use crate::relationship::SEVEN;
use crate::varga::Varga;
use lagn_ephem::{norm360, Graha};
use serde::{Deserialize, Serialize};

/// Virupas in a rupa.
pub const RUPA: f64 = 60.0;

/// The seven vargas Saptavargaja bala reads.
pub const SAPTAVARGA: [Varga; 7] =
    [Varga::D1, Varga::D2, Varga::D3, Varga::D7, Varga::D9, Varga::D12, Varga::D30];

/// Shortest angular distance between two longitudes, 0 to 180.
fn arc(a: f64, b: f64) -> f64 {
    let d = norm360(a - b);
    if d > 180.0 { 360.0 - d } else { d }
}

/// The dignity ladder for Saptavargaja, halving from moolatrikona down.
///
/// Exalted reads as moolatrikona and debilitated as great enemy, because the
/// ladder has seven rungs and nine dignities exist (V-13-26).
pub fn ladder(d: Dignity) -> f64 {
    match d {
        Dignity::Exalted | Dignity::Moolatrikona => 45.0,
        Dignity::OwnSign => 30.0,
        Dignity::GreatFriend => 22.5,
        Dignity::Friend => 15.0,
        Dignity::Neutral => 7.5,
        Dignity::Enemy => 3.75,
        Dignity::GreatEnemy | Dignity::Debilitated => 1.875,
    }
}

/// Deep exaltation longitude, from `docs/phase2/DESIGN.md` section 4.1.
fn deep_exaltation(g: Graha) -> Option<f64> {
    dignity::deep_exaltation_longitude(g)
}

/// Sthana bala and its five parts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sthana {
    pub uchcha: f64,
    pub saptavargaja: f64,
    pub ojayugma: f64,
    pub kendra: f64,
    pub drekkana: f64,
    pub total: f64,
}

/// Which third of a sign a longitude falls in, 1 to 3.
fn drekkana_of(longitude: f64) -> u8 {
    (Rasi::degrees_within(norm360(longitude)) / 10.0).floor().min(2.0) as u8 + 1
}

/// The drekkana a graha is at home in: male 1st, neutral 2nd, female 3rd.
fn own_drekkana(g: Graha) -> u8 {
    match g {
        Graha::Sun | Graha::Mars | Graha::Jupiter => 1,
        Graha::Mercury | Graha::Saturn => 2,
        Graha::Moon | Graha::Venus => 3,
        // Not scored for the nodes; never reached.
        _ => 0,
    }
}

/// True when the graha wants an odd sign for Ojayugma bala.
fn wants_odd(g: Graha) -> bool {
    !matches!(g, Graha::Moon | Graha::Venus)
}

fn sthana(chart: &Chart, g: Graha, varga_dignity: &[(Varga, Option<Dignity>)]) -> Sthana {
    let p = chart.placement(g);

    // Uchcha: 60 at deep exaltation, 0 at deep debilitation, linear between.
    // Measured from the debilitation point, which is the exaltation point
    // opposed, so the formula needs only one of the two.
    let uchcha = deep_exaltation(g)
        .map(|ex| arc(p.longitude, norm360(ex + 180.0)) / 3.0)
        .unwrap_or(0.0);

    let saptavargaja: f64 = varga_dignity
        .iter()
        .filter_map(|(_, d)| d.map(ladder))
        .sum();

    let odd_wanted = wants_odd(g);
    let ojayugma = [p.rasi.is_odd(), p.navamsa.is_odd()]
        .iter()
        .filter(|&&is_odd| is_odd == odd_wanted)
        .count() as f64
        * 15.0;

    // Kendra, panapara and apoklima are a property of the house, not of the
    // sign: houses 1, 4, 7, 10 then 2, 5, 8, 11 then the rest.
    let kendra = match (p.house - 1) % 3 {
        0 => 60.0,
        1 => 30.0,
        _ => 15.0,
    };

    let drekkana = if drekkana_of(p.longitude) == own_drekkana(g) { 15.0 } else { 0.0 };

    Sthana {
        uchcha,
        saptavargaja,
        ojayugma,
        kendra,
        drekkana,
        total: uchcha + saptavargaja + ojayugma + kendra + drekkana,
    }
}

/// The house a graha is strongest in, for Dig bala.
fn strongest_house(g: Graha) -> u8 {
    match g {
        Graha::Jupiter | Graha::Mercury => 1,
        Graha::Sun | Graha::Mars => 10,
        Graha::Saturn => 7,
        Graha::Moon | Graha::Venus => 4,
        _ => 1,
    }
}

/// Dig bala: 60 at the strongest angle, 0 opposite, linear between.
fn dig(chart: &Chart, g: Graha) -> f64 {
    let p = chart.placement(g);
    // The cusp of the strongest house, in whole-sign terms: the start of the
    // sign that house occupies.
    let house_start = 30.0 * (chart.rasi_of_house(strongest_house(g)).index() as f64);
    60.0 * (1.0 - arc(p.longitude, house_start) / 180.0)
}

/// Kala bala and its parts. Several depend on disputed choices; each is
/// labelled in `Bala::variants`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Kala {
    pub nathonnatha: f64,
    pub paksha: f64,
    pub tribhaga: f64,
    pub abda: f64,
    pub masa: f64,
    pub vara: f64,
    pub hora: f64,
    pub ayana: f64,
    pub total: f64,
}

/// Mean obliquity of the ecliptic, degrees. Varies by about 0.013 degrees a
/// century, far below the precision Ayana bala is read to.
const OBLIQUITY: f64 = 23.4393;

/// Declination from sidereal ecliptic longitude and latitude.
///
/// The standard transform, which needs the *tropical* longitude - so the
/// chart's ayanamsa is added back before converting.
fn declination(sidereal_longitude: f64, latitude: f64, ayanamsa: f64) -> f64 {
    let lam = (sidereal_longitude + ayanamsa).to_radians();
    let beta = latitude.to_radians();
    let eps = OBLIQUITY.to_radians();
    (beta.sin() * eps.cos() + beta.cos() * eps.sin() * lam.sin())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

/// Declination of a point, exposed so a test can check the transform against
/// an astronomical fact rather than against itself.
pub fn declination_of(sidereal_longitude: f64, latitude: f64, ayanamsa: f64) -> f64 {
    declination(sidereal_longitude, latitude, ayanamsa)
}

/// The obliquity this engine uses, for the same reason.
pub const OBLIQUITY_DEGREES: f64 = OBLIQUITY;

/// Cheshta bala, and whether it was taken from Ayana instead (V-13-31).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cheshta {
    pub value: f64,
    /// True for Surya and Chandra, which never retrograde, so their Cheshta
    /// is taken from Ayana by the common convention.
    pub from_ayana: bool,
}

/// Naisargika bala: exactly `60 n / 7`, n from 7 down to 1.
pub fn naisargika(g: Graha) -> f64 {
    let n = match g {
        Graha::Sun => 7.0,
        Graha::Moon => 6.0,
        Graha::Venus => 5.0,
        Graha::Jupiter => 4.0,
        Graha::Mercury => 3.0,
        Graha::Mars => 2.0,
        Graha::Saturn => 1.0,
        _ => 0.0,
    };
    RUPA * n / 7.0
}

/// The customary minimum strength in rupas. Reference only: these are the
/// thresholds for a *complete* Shadbala, and this engine computes five of the
/// six components, so nothing is tested against them.
pub fn customary_minimum(g: Graha) -> Option<f64> {
    Some(match g {
        Graha::Sun => 5.0,
        Graha::Moon => 6.0,
        Graha::Mars => 5.0,
        Graha::Mercury => 7.0,
        Graha::Jupiter => 6.5,
        Graha::Venus => 5.5,
        Graha::Saturn => 5.0,
        _ => return None,
    })
}

/// One graha's strengths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrahaBala {
    pub graha: Graha,
    pub sthana: Sthana,
    pub dig: f64,
    pub kala: Kala,
    pub cheshta: Cheshta,
    pub naisargika: f64,
    /// Sum of the five computed components, in virupas.
    pub total_virupas: f64,
    /// The same, in rupas.
    pub total_rupas: f64,
    /// Reference only; see `customary_minimum`.
    pub customary_minimum_rupas: Option<f64>,
}

/// Shadbala for a chart: five of the six components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bala {
    pub grahas: Vec<GrahaBala>,
    /// Always false while Drik bala is held, and stated rather than implied.
    pub drik_included: bool,
    /// How many of the six components the totals contain.
    pub components_computed: u8,
    /// Why a reader must not take these as verified numbers.
    pub caveat: Cow<'static, str>,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 7] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-26"),
        question: Cow::Borrowed("exalted and debilitated on the seven-rung ladder"),
        chosen: Cow::Borrowed("read as moolatrikona and great enemy"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-27"),
        question: Cow::Borrowed("Budha's nature for Paksha bala"),
        chosen: Cow::Borrowed("benefic"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-28"),
        question: Cow::Borrowed("lord of the year, for Abda bala"),
        chosen: Cow::Borrowed("weekday lord of the solar year's first day"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-29"),
        question: Cow::Borrowed("lord of the month, for Masa bala"),
        chosen: Cow::Borrowed("weekday lord of the solar month's first day"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-30"),
        question: Cow::Borrowed("hora length, and Budha's Ayana sign"),
        chosen: Cow::Borrowed("equal one-hour horas; Budha doubled and always positive"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-31"),
        question: Cow::Borrowed("Cheshta bala formula"),
        chosen: Cow::Borrowed("speed deviation from the mean; retrograde takes 60"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-32"),
        question: Cow::Borrowed("Drik bala"),
        chosen: Cow::Borrowed("not computed; the total is five of six"),
    },
];

/// Mean daily motion, degrees. Used only by Cheshta bala (V-13-31).
fn mean_speed(g: Graha) -> f64 {
    match g {
        Graha::Sun => 0.9856,
        Graha::Moon => 13.1764,
        Graha::Mars => 0.5240,
        Graha::Mercury => 1.3833,
        Graha::Jupiter => 0.0831,
        Graha::Venus => 1.6021,
        Graha::Saturn => 0.0335,
        _ => 1.0,
    }
}

impl Bala {
    /// Shadbala for a chart, deriving the day's bounds itself.
    ///
    /// Kala bala needs the sunrise and sunset that bound the birth, and the
    /// weekday lord of the governing sunrise. Every caller would otherwise
    /// repeat that, and two callers deriving it differently is exactly how a
    /// quantity ends up with two values.
    pub fn for_chart(
        eph: &lagn_ephem::Ephemeris,
        chart: &Chart,
    ) -> Result<Bala, lagn_ephem::EphemError> {
        let (lat, lon) = (chart.birth.latitude, chart.birth.longitude);
        let today = eph.sun_day(chart.jd_ut - 0.5, lat, lon)?;
        // A birth before sunrise belongs to the previous day, the same
        // reckoning day.rs and upagraha.rs use.
        let sunrise = if chart.jd_ut < today.sunrise_jd {
            eph.sun_day(chart.jd_ut - 1.5, lat, lon)?
        } else {
            today
        };
        let vara = crate::day::Vara::from_sunrise_jd(sunrise.sunrise_jd);
        Bala::compute(eph, chart, sunrise.sunrise_jd, sunrise.sunset_jd, vara.lord())
    }

    /// Compute Shadbala for a chart.
    ///
    /// `sunrise_jd` and `sunset_jd` bound the day the birth falls in, for the
    /// parts of Kala bala reckoned from them. `vara_lord` is the weekday lord
    /// of the governing sunrise.
    pub fn compute(
        eph: &lagn_ephem::Ephemeris,
        chart: &Chart,
        sunrise_jd: f64,
        sunset_jd: f64,
        vara_lord: Graha,
    ) -> Result<Bala, lagn_ephem::EphemError> {
        let sun = chart.placement(Graha::Sun);
        let moon = chart.placement(Graha::Moon);
        // The Moon's elongation, which is what Paksha bala measures.
        let elongation = arc(moon.longitude, sun.longitude);
        let day = chart.jd_ut >= sunrise_jd && chart.jd_ut < sunset_jd;

        // Hours from midnight, local to the birth's own offset. Only Kala
        // bala's Nathonnatha part needs this, and it needs it as a clock
        // position rather than a date.
        let m = &chart.birth.moment;
        let clock = m.hour as f64 + m.minute as f64 / 60.0 + m.second / 3600.0;

        let mut grahas = Vec::with_capacity(7);
        for &g in SEVEN.iter() {
                // Dignity in each of the seven vargas, for Saptavargaja.
                let varga_dignity: Vec<(Varga, Option<Dignity>)> = SAPTAVARGA
                    .iter()
                    .map(|&v| {
                        let vc = chart.varga(v);
                        let sign = vc.sign_of(g);
                        (v, dignity_in(chart, g, sign))
                    })
                    .collect();

                let sthana = sthana(chart, g, &varga_dignity);
                let dig_bala = dig(chart, g);

                // --- Kala bala ---
                // Nathonnatha: 60 at the graha's strong moment, 0 at its weak
                // one, twelve hours apart.
                let weak_at_midday = matches!(g, Graha::Moon | Graha::Mars | Graha::Saturn);
                let from_midnight = (clock - 0.0).abs().min(24.0 - clock);
                let nathonnatha = if g == Graha::Mercury {
                    RUPA
                } else if weak_at_midday {
                    // Strong at midnight: 60 there, 0 at midday.
                    RUPA * (1.0 - from_midnight / 12.0)
                } else {
                    RUPA * (from_midnight / 12.0)
                };

                // Paksha: benefics take the elongation third, malefics the
                // complement. Budha counts benefic (V-13-27).
                let benefic = matches!(g, Graha::Moon | Graha::Jupiter | Graha::Venus | Graha::Mercury);
                let paksha = if benefic { elongation / 3.0 } else { RUPA - elongation / 3.0 };

                // Tribhaga: which third of the day or night the birth is in.
                let span = if day { sunset_jd - sunrise_jd } else { 1.0 };
                let third = if day {
                    (((chart.jd_ut - sunrise_jd) / (span / 3.0)).floor() as i32).clamp(0, 2)
                } else {
                    0
                };
                let tribhaga_lord = if day {
                    [Graha::Mercury, Graha::Sun, Graha::Saturn][third as usize]
                } else {
                    Graha::Moon
                };
                // Guru always takes the full measure; otherwise only the
                // ruler of the third the birth falls in does.
                let tribhaga = if g == Graha::Jupiter || g == tribhaga_lord { RUPA } else { 0.0 };

                // Abda and Masa: the lord of the year and month (V-13-28,
                // V-13-29). Both reckoned from the weekday lord, which is the
                // documented default.
                let abda = if g == vara_lord { 15.0 } else { 0.0 };
                let masa = if g == vara_lord { 30.0 } else { 0.0 };
                let vara = if g == vara_lord { 45.0 } else { 0.0 };

                // Hora: equal one-hour horas from sunrise (V-13-30).
                let hora_index = (((chart.jd_ut - sunrise_jd) * 24.0).floor() as i64).rem_euclid(24);
                let hora_lord = hora_sequence(vara_lord, hora_index as usize);
                let hora = if g == hora_lord { RUPA } else { 0.0 };

                // Ayana: from declination.
                let p = chart.placement(g);
                // Ecliptic latitude is not on Placement, so it comes from the
                // ephemeris - the only thing Shadbala needs that the chart
                // does not already carry.
                let beta = eph.position(chart.jd_ut, g)?.latitude;
                let decl = declination(p.longitude, beta, chart.ayanamsa_value);
                let k = match g {
                    Graha::Moon | Graha::Saturn => -1.0,
                    _ => 1.0,
                };
                let base = RUPA * (OBLIQUITY + k * decl) / (2.0 * OBLIQUITY);
                let ayana = if g == Graha::Mercury {
                    (RUPA * (OBLIQUITY + decl.abs()) / (2.0 * OBLIQUITY) * 2.0).min(RUPA)
                } else {
                    base.clamp(0.0, RUPA)
                };

                let kala = Kala {
                    nathonnatha: nathonnatha.clamp(0.0, RUPA),
                    paksha: paksha.clamp(0.0, RUPA),
                    tribhaga,
                    abda,
                    masa,
                    vara,
                    hora,
                    ayana,
                    total: nathonnatha.clamp(0.0, RUPA)
                        + paksha.clamp(0.0, RUPA)
                        + tribhaga
                        + abda
                        + masa
                        + vara
                        + hora
                        + ayana,
                };

                // Cheshta (V-13-31).
                let cheshta = if matches!(g, Graha::Sun | Graha::Moon) {
                    Cheshta { value: ayana, from_ayana: true }
                } else if p.retrograde {
                    Cheshta { value: RUPA, from_ayana: false }
                } else {
                    let mean = mean_speed(g);
                    let dev = ((p.speed_longitude.abs() - mean).abs() / mean) * RUPA;
                    Cheshta { value: dev.clamp(0.0, RUPA), from_ayana: false }
                };

                let nais = naisargika(g);
                let total = sthana.total + dig_bala + kala.total + cheshta.value + nais;

                grahas.push(GrahaBala {
                    graha: g,
                    sthana,
                    dig: dig_bala,
                    kala,
                    cheshta,
                    naisargika: nais,
                    total_virupas: total,
                    total_rupas: total / RUPA,
                    customary_minimum_rupas: customary_minimum(g),
                });
        }

        Ok(Bala {
            grahas,
            drik_included: false,
            components_computed: 5,
            caveat: Cow::Borrowed(
                "Five of the six components. Drik bala needs sphuta drishti, which is \
                 degree-based and disputed, so it is not computed rather than approximated. \
                 Every value here depends on variant choices no astrologer has signed off, \
                 and the customary minimum strengths are thresholds for a complete Shadbala \
                 and so must not be compared against these totals.",
            ),
            variants: VARIANTS.to_vec(),
        })
    }
}

/// The hora lord `n` hours after sunrise, from the weekday lord.
///
/// Horas run in the Chaldean order, which is the weekday lords taken in
/// descending planetary speed: Shani, Guru, Kuja, Surya, Shukra, Budha,
/// Chandra. The first hora of a day belongs to that day's lord.
fn hora_sequence(vara_lord: Graha, n: usize) -> Graha {
    const CHALDEAN: [Graha; 7] = [
        Graha::Saturn,
        Graha::Jupiter,
        Graha::Mars,
        Graha::Sun,
        Graha::Venus,
        Graha::Mercury,
        Graha::Moon,
    ];
    let start = CHALDEAN.iter().position(|&g| g == vara_lord).unwrap_or(0);
    CHALDEAN[(start + n) % 7]
}

/// Dignity of a graha in a sign-only chart, for Saptavargaja.
///
/// Uses the engine's own `dignity::resolve`, which already falls back to the
/// compound relationship with the sign's lord when no zone applies - so the
/// ladder here reads the same dignities the rest of the app does, rather than
/// a second opinion about them. Temporary relationships come from D-1, which
/// is phase 2's variant V-5 default.
fn dignity_in(chart: &Chart, g: Graha, sign: Rasi) -> Option<Dignity> {
    let zone = dignity::varga_zone(g, sign, dignity::VargaMoolatrikona::AsMoolatrikona);
    let lord = sign.lord();
    dignity::resolve(
        g,
        zone,
        sign,
        chart.placement(g).rasi,
        chart.placement(lord).rasi,
    )
}
