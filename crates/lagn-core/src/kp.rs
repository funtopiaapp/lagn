//! The Krishnamurti Paddhati: the four lords of a longitude, Placidus cusps,
//! ruling planets and significators.
//!
//! Specification: `docs/phase13/KP.md`.
//!
//! Everything here derives from two things this engine already has: the
//! Vimshottari period lengths, and the Placidus cusps Swiss Ephemeris
//! returns. Nothing is a table to be recalled, which is why KP could be built
//! without external reference output when several smaller groups could not.
//!
//! The division is self-similar. A nakshatra splits into nine subs in
//! Vimshottari order from its own lord, each proportional to that lord's
//! years out of 120; a sub splits into nine sub-subs by the identical rule.
//! One function therefore serves every depth, and the property that it tiles
//! exactly follows from the nine periods summing to 120.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::dasha::{dasha_years, CYCLE, TOTAL_YEARS};
use crate::jaimini::VariantChoice;
use crate::nakshatra::{Nakshatra, NAKSHATRA_SPAN};
use crate::rasi::Rasi;
use lagn_ephem::{norm360, EphemError, Ephemeris, Graha, HouseSystem};
use serde::{Deserialize, Serialize};

/// One division of a span, with the lord that owns it.
///
/// `start` and `end` are in whatever frame the caller passed to [`divide`].
/// Inside [`lords`] that frame is *degrees within the nakshatra*, not absolute
/// longitude - see the comment there for why.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Division {
    pub lord: Graha,
    pub start: f64,
    pub end: f64,
}

impl Division {
    pub fn width(&self) -> f64 {
        self.end - self.start
    }

    pub fn contains(&self, lon: f64) -> bool {
        lon >= self.start && lon < self.end
    }
}

/// Split `start..end` into nine, in Vimshottari order from `first`, each part
/// proportional to its lord's years out of 120.
///
/// This is the whole of the KP division, used at every depth.
///
/// It takes both endpoints rather than a start and a width, and that is not
/// cosmetic. Computing a nakshatra's end as `start + span` while its
/// neighbour's start is `span * k` makes the two differ by an ulp, and a
/// longitude landing in that sliver belongs to no sub at all. The dense sweep
/// in `tests/kp.rs` found exactly that at 93.33333333333333 degrees. Passing
/// the end through means consecutive nakshatras share a boundary bit for bit.
pub fn divide(start: f64, end: f64, first: Graha) -> Vec<Division> {
    let i = CYCLE
        .iter()
        .position(|&g| g == first)
        .expect("every KP lord is a Vimshottari lord");
    let span = end - start;
    let mut out = Vec::with_capacity(9);
    let mut at = start;
    for j in 0..9 {
        let lord = CYCLE[(i + j) % 9];
        // The last division ends exactly on `end`: accumulating nine
        // proportions would leave a rounding crumb on the boundary.
        let next = if j == 8 { end } else { at + span * dasha_years(lord) / TOTAL_YEARS };
        out.push(Division { lord, start: at, end: next });
        at = next;
    }
    out
}



/// The four lords of a sidereal longitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lords {
    pub longitude: f64,
    pub rasi: Rasi,
    pub nakshatra: Nakshatra,
    /// Pada, 1 to 4. Carried because KP practice quotes it alongside.
    pub pada: u8,
    pub sign_lord: Graha,
    pub star_lord: Graha,
    pub sub_lord: Graha,
    pub sub_sub_lord: Graha,
}

/// The four lords of a longitude. Section 3.
pub fn lords(longitude: f64) -> Lords {
    let lon = norm360(longitude);
    let rasi = Rasi::from_longitude(lon);
    let pos = crate::nakshatra::NakshatraPosition::from_longitude(lon);
    let nakshatra = pos.nakshatra;
    let star_lord = nakshatra.lord();

    // The division is measured in degrees *within the nakshatra*, not in
    // absolute longitude.
    //
    // This matters at an exact nakshatra boundary. `NakshatraPosition` decides
    // which nakshatra a longitude belongs to with a single floor, which
    // nakshatra.rs documents as the one source of truth for both the 27-fold
    // and the 108-fold division. Re-deriving the nakshatra's start as
    // `SPAN * index` and comparing absolute longitudes against it disagrees
    // with that floor by an ulp, and the dense sweep in tests/kp.rs caught it
    // at 226.66666666666666 degrees: the floor said Jyeshtha, the
    // multiplication said Anuradha. Working from `degrees_within` - which
    // comes from the same floor - makes the disagreement impossible rather
    // than unlikely.
    let within = pos.degrees_within;
    let subs = divide(0.0, NAKSHATRA_SPAN, star_lord);
    let sub = subs
        .iter()
        .find(|d| d.contains(within))
        .copied()
        // Only reachable at the very top of the last sub through floating
        // point; the last division is the right answer there.
        .unwrap_or(subs[8]);

    let sub_subs = divide(sub.start, sub.end, sub.lord);
    let sub_sub = sub_subs
        .iter()
        .find(|d| d.contains(within))
        .copied()
        .unwrap_or(sub_subs[8]);

    Lords {
        longitude: lon,
        rasi,
        nakshatra,
        pada: pos.pada,
        sign_lord: rasi.lord(),
        star_lord,
        sub_lord: sub.lord,
        sub_sub_lord: sub_sub.lord,
    }
}

/// One Placidus cusp, with its lords.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cusp {
    /// 1 to 12.
    pub house: u8,
    pub lords: Lords,
}

/// A ruling planet, with the sub practice weighs it by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RulingPlanet {
    /// Why it rules: "day lord", "Moon sign lord", and so on.
    pub role: Cow<'static, str>,
    pub graha: Graha,
    /// The sub lord of the point this role was read from, where there is one.
    pub sub_lord: Option<Graha>,
}

/// Which of the four significator groups a graha falls in, strongest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignificatorGroup {
    /// In the star of a graha occupying the house.
    StarOfOccupant,
    /// Occupying the house.
    Occupant,
    /// In the star of the lord of the house.
    StarOfLord,
    /// The lord of the house.
    Lord,
}

impl SignificatorGroup {
    pub const ALL: [SignificatorGroup; 4] = [
        SignificatorGroup::StarOfOccupant,
        SignificatorGroup::Occupant,
        SignificatorGroup::StarOfLord,
        SignificatorGroup::Lord,
    ];

    /// 1 to 4, as practice numbers them.
    pub const fn rank(self) -> u8 {
        match self {
            SignificatorGroup::StarOfOccupant => 1,
            SignificatorGroup::Occupant => 2,
            SignificatorGroup::StarOfLord => 3,
            SignificatorGroup::Lord => 4,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            SignificatorGroup::StarOfOccupant => "in the star of an occupant",
            SignificatorGroup::Occupant => "occupies the house",
            SignificatorGroup::StarOfLord => "in the star of the house lord",
            SignificatorGroup::Lord => "lord of the house",
        }
    }
}

/// One graha signifying one house.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Significator {
    pub graha: Graha,
    pub group: SignificatorGroup,
}

/// Every significator of one house, strongest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HouseSignificators {
    pub house: u8,
    pub significators: Vec<Significator>,
}

/// A whole KP reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kp {
    /// The ascendant's own lords, which KP reads first.
    pub ascendant: Lords,
    pub moon: Lords,
    /// Each graha's four lords.
    pub grahas: Vec<(Graha, Lords)>,
    /// The twelve Placidus cusps.
    pub cusps: Vec<Cusp>,
    pub ruling_planets: Vec<RulingPlanet>,
    pub significators: Vec<HouseSignificators>,
    pub variants: Vec<VariantChoice>,
}

pub const VARIANTS: [VariantChoice; 4] = [
    VariantChoice {
        id: Cow::Borrowed("V-13-18"),
        question: Cow::Borrowed("ayanamsa for a KP reading"),
        chosen: Cow::Borrowed("the chart's own"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-19"),
        question: Cow::Borrowed("a graha qualifying in two significator groups"),
        chosen: Cow::Borrowed("listed once, in the strongest"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-20"),
        question: Cow::Borrowed("house occupancy for significators"),
        chosen: Cow::Borrowed("Placidus cusps"),
    },
    VariantChoice {
        id: Cow::Borrowed("V-13-21"),
        question: Cow::Borrowed("Rahu and Ketu as significators"),
        chosen: Cow::Borrowed("included, by the sign they occupy"),
    },
];

/// Which Placidus house a longitude falls in, given the twelve cusps.
///
/// Cusps are unequal under Placidus and the twelfth wraps past 360, so this
/// walks the arcs rather than dividing.
fn house_of(cusps: &[f64; 12], lon: f64) -> u8 {
    let lon = norm360(lon);
    for h in 0..12 {
        let a = cusps[h];
        let b = cusps[(h + 1) % 12];
        let span = norm360(b - a);
        if norm360(lon - a) < span {
            return h as u8 + 1;
        }
    }
    // Unreachable while the cusps tile the circle, which they do by
    // construction; the first house is the safe answer.
    1
}

impl Kp {
    pub fn compute(eph: &Ephemeris, chart: &Chart) -> Result<Kp, EphemError> {
        let (lat, lon) = (chart.birth.latitude, chart.birth.longitude);
        // KP reads houses as Placidus cusps (V-13-20), whatever house system
        // the rest of the chart was cast with.
        let (_, cusp_list) = eph.angles(chart.jd_ut, lat, lon, HouseSystem::Placidus)?;
        let mut cusp_lon = [0.0f64; 12];
        for (i, slot) in cusp_lon.iter_mut().enumerate() {
            *slot = norm360(cusp_list.0[i]);
        }

        let cusps: Vec<Cusp> = (0..12)
            .map(|i| Cusp { house: i as u8 + 1, lords: lords(cusp_lon[i]) })
            .collect();

        let grahas: Vec<(Graha, Lords)> = Graha::ALL
            .iter()
            .map(|&g| (g, lords(chart.placement(g).longitude)))
            .collect();
        let lords_of = |g: Graha| grahas.iter().find(|(x, _)| *x == g).map(|(_, l)| *l).expect("every graha");

        let ascendant = lords(chart.lagna.longitude);
        let moon = lords_of(Graha::Moon);

        // The five ruling planets, in the order practice lists them.
        let vara = crate::day::Vara::from_sunrise_jd(
            // The vara belongs to the sunrise that precedes the moment; the
            // same reckoning day.rs and upagraha.rs use.
            eph.sun_day(chart.jd_ut - 0.5, lat, lon)
                .map(|d| if chart.jd_ut < d.sunrise_jd {
                    d.sunrise_jd - 1.0
                } else {
                    d.sunrise_jd
                })
                .unwrap_or(chart.jd_ut),
        );
        let ruling_planets = vec![
            RulingPlanet { role: Cow::Borrowed("lord of the day"), graha: vara.lord(), sub_lord: None },
            RulingPlanet { role: Cow::Borrowed("Moon's sign lord"), graha: moon.sign_lord, sub_lord: Some(moon.sub_lord) },
            RulingPlanet { role: Cow::Borrowed("Moon's star lord"), graha: moon.star_lord, sub_lord: Some(moon.sub_lord) },
            RulingPlanet { role: Cow::Borrowed("ascendant's sign lord"), graha: ascendant.sign_lord, sub_lord: Some(ascendant.sub_lord) },
            RulingPlanet { role: Cow::Borrowed("ascendant's star lord"), graha: ascendant.star_lord, sub_lord: Some(ascendant.sub_lord) },
        ];

        // Significators, from the Placidus houses.
        let house_of_graha = |g: Graha| house_of(&cusp_lon, chart.placement(g).longitude);
        let significators = (1..=12u8)
            .map(|h| {
                let occupants: Vec<Graha> =
                    Graha::ALL.iter().copied().filter(|&g| house_of_graha(g) == h).collect();
                // The lord of the house is the sign lord of the cusp's sign
                // (V-13-21 reads the nodes the same way).
                let house_lord = cusps[(h - 1) as usize].lords.sign_lord;

                let mut found: Vec<Significator> = Vec::new();
                let push = |g: Graha, group: SignificatorGroup, found: &mut Vec<Significator>| {
                    // Listed once, in the strongest group it qualifies for
                    // (V-13-19). Groups are visited in order, so the first
                    // entry for a graha is already its strongest.
                    if !found.iter().any(|s| s.graha == g) {
                        found.push(Significator { graha: g, group });
                    }
                };
                for g in Graha::ALL {
                    if occupants.contains(&lords_of(g).star_lord) {
                        push(g, SignificatorGroup::StarOfOccupant, &mut found);
                    }
                }
                for &g in &occupants {
                    push(g, SignificatorGroup::Occupant, &mut found);
                }
                for g in Graha::ALL {
                    if lords_of(g).star_lord == house_lord {
                        push(g, SignificatorGroup::StarOfLord, &mut found);
                    }
                }
                push(house_lord, SignificatorGroup::Lord, &mut found);

                // Sorted by rank(), not by the enum's declaration order.
                //
                // These were the same thing, so a wrong rank() changed only
                // a number shown to the reader and nothing in the ordering -
                // which meant the N-version oracle, reading the ordering,
                // could not see it. One source of truth for strength closes
                // that: get rank() wrong now and the output order changes.
                found.sort_by_key(|s| s.group.rank());
                HouseSignificators { house: h, significators: found }
            })
            .collect();

        Ok(Kp {
            ascendant,
            moon,
            grahas,
            cusps,
            ruling_planets,
            significators,
            variants: VARIANTS.to_vec(),
        })
    }
}
