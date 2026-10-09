//! Assembling a birth chart from a birth moment and a place.

use lagn_ephem::{
    Angles, Ayanamsa, BirthMoment, Ephemeris, EphemError, Graha, HouseSystem, NodeType, Position,
};
use serde::{Deserialize, Serialize};

use crate::dasha::YearLength;
use crate::nakshatra::NakshatraPosition;
use crate::rasi::Rasi;
use crate::varga::{navamsa_sign, Varga};

/// Where and when someone was born. The three inputs the user actually gives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BirthData {
    pub moment: BirthMoment,
    /// Degrees north-positive.
    pub latitude: f64,
    /// Degrees east-positive.
    pub longitude: f64,
    pub place_name: String,
}

/// Every choice that changes the numbers. Recorded on the chart so a reading
/// can be reproduced years later, and so two charts are only ever compared
/// when they were computed under the same conventions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChartSettings {
    pub ayanamsa: Ayanamsa,
    pub node_type: NodeType,
    pub house_system: HouseSystem,
    pub year_length: YearLength,
}

impl Default for ChartSettings {
    /// South Indian / Tamil-Kerala defaults.
    fn default() -> Self {
        Self {
            ayanamsa: Ayanamsa::Lahiri,
            node_type: NodeType::Mean,
            house_system: HouseSystem::WholeSign,
            year_length: YearLength::default(),
        }
    }
}

/// One graha, fully resolved onto every grid Phase 1 knows about.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub graha: Graha,
    /// Sidereal longitude, `[0, 360)`.
    pub longitude: f64,
    pub rasi: Rasi,
    /// Degrees within the rasi, `[0, 30)`.
    pub degrees_in_rasi: f64,
    pub nakshatra: NakshatraPosition,
    /// D-9 sign.
    pub navamsa: Rasi,
    /// Bhava, 1..=12, counted from the lagna.
    pub house: u8,
    /// True vakri motion. Always false for Rahu and Ketu, whose retrogradation
    /// is definitional rather than interpretive.
    pub retrograde: bool,
    pub speed_longitude: f64,
}

impl Placement {
    fn build(graha: Graha, pos: Position, lagna_rasi: Rasi) -> Placement {
        let rasi = Rasi::from_longitude(pos.longitude);
        Placement {
            graha,
            longitude: pos.longitude,
            rasi,
            degrees_in_rasi: Rasi::degrees_within(pos.longitude),
            nakshatra: NakshatraPosition::from_longitude(pos.longitude),
            navamsa: navamsa_sign(pos.longitude),
            house: lagna_rasi.houses_to(rasi),
            retrograde: !graha.is_chhaya() && pos.is_retrograde(),
            speed_longitude: pos.speed_longitude,
        }
    }
}

/// The lagna, resolved the same way a graha is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lagna {
    pub longitude: f64,
    pub rasi: Rasi,
    pub degrees_in_rasi: f64,
    pub nakshatra: NakshatraPosition,
    pub navamsa: Rasi,
}

/// A computed birth chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub birth: BirthData,
    pub settings: ChartSettings,
    /// Julian Day in UT - the canonical instant everything derives from.
    pub jd_ut: f64,
    /// Ayanamsa in degrees at birth, recorded for auditability.
    pub ayanamsa_value: f64,
    pub lagna: Lagna,
    pub angles: Angles,
    /// The nine grahas in traditional order.
    pub placements: [Placement; 9],
}

/// How long the lagna holds its rasi, either side of the birth moment. See
/// [`Chart::lagna_window`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LagnaWindow {
    /// Minutes from the lagna entering this rasi up to the birth moment.
    pub minutes_in: f64,
    /// Minutes from the birth moment until the lagna leaves this rasi.
    pub minutes_left: f64,
}

impl LagnaWindow {
    /// The nearer edge: how wrong the birth time can be before every house in
    /// the reading moves by one.
    pub fn margin_minutes(&self) -> f64 {
        self.minutes_in.min(self.minutes_left)
    }
}

impl Chart {
    /// Compute a chart. Deterministic: same inputs, same bytes out, forever.
    pub fn compute(birth: BirthData, settings: ChartSettings) -> Result<Chart, EphemError> {
        let jd_ut = birth.moment.to_jd_ut()?;
        Chart::from_jd(jd_ut, birth, settings)
    }

    /// Compute a chart for an exact Julian Day rather than for a civil moment.
    ///
    /// The annual chart needs this. Its moment comes from solving for a solar
    /// return and is precise to a fraction of a second; routing it back
    /// through `BirthMoment` would round it to the civil second and throw
    /// away the precision the solve just established. So the JD is used as
    /// given, and the civil fields are filled in from it for display only.
    pub fn compute_at(
        jd_ut: f64,
        latitude: f64,
        longitude: f64,
        place_name: String,
        utc_offset_hours: f64,
        settings: ChartSettings,
    ) -> Result<Chart, EphemError> {
        let d = lagn_ephem::jd_to_civil(jd_ut, utc_offset_hours);
        let birth = BirthData {
            moment: BirthMoment {
                year: d.year,
                month: d.month,
                day: d.day,
                hour: d.hour,
                minute: d.minute,
                second: d.second,
                utc_offset_hours,
            },
            latitude,
            longitude,
            place_name,
        };
        Chart::from_jd(jd_ut, birth, settings)
    }

    /// The shared body of [`Chart::compute`] and [`Chart::compute_at`].
    fn from_jd(jd_ut: f64, birth: BirthData, settings: ChartSettings) -> Result<Chart, EphemError> {
        let eph = Ephemeris::new(settings.ayanamsa, settings.node_type);

        let (angles, _cusps) =
            eph.angles(jd_ut, birth.latitude, birth.longitude, settings.house_system)?;
        let lagna_rasi = Rasi::from_longitude(angles.ascendant);

        let positions = eph.positions(jd_ut)?;
        let mut placements: [Placement; 9] = core::array::from_fn(|i| {
            let (g, p) = positions[i];
            Placement::build(g, p, lagna_rasi)
        });
        placements.sort_by_key(|p| p.graha);

        Ok(Chart {
            ayanamsa_value: eph.ayanamsa_value(jd_ut)?,
            lagna: Lagna {
                longitude: angles.ascendant,
                rasi: lagna_rasi,
                degrees_in_rasi: Rasi::degrees_within(angles.ascendant),
                nakshatra: NakshatraPosition::from_longitude(angles.ascendant),
                navamsa: navamsa_sign(angles.ascendant),
            },
            angles,
            placements,
            jd_ut,
            birth,
            settings,
        })
    }

    /// How long the lagna holds the rasi it is in, in minutes of clock time
    /// either side of this birth moment.
    ///
    /// Every house in a reading is counted from the lagna, so this is the
    /// number that decides how much an uncertain birth time actually matters.
    /// A chart with four hours of margin tolerates a remembered time; one with
    /// eight minutes does not, and a reading cast from a defaulted time should
    /// not speak with the same authority in both cases.
    ///
    /// Determinism: same inputs, same bytes out. The search below is a
    /// bracket-and-bisect on the ascendant's own sign, not a rate estimate,
    /// because a rasi can rise in minutes or hold for hours depending on
    /// latitude and which sign it is.
    pub fn lagna_window(&self) -> Result<LagnaWindow, EphemError> {
        let eph = Ephemeris::new(self.settings.ayanamsa, self.settings.node_type);
        let rasi_at = |jd: f64| -> Result<Rasi, EphemError> {
            let (a, _) = eph.angles(
                jd,
                self.birth.latitude,
                self.birth.longitude,
                self.settings.house_system,
            )?;
            Ok(Rasi::from_longitude(a.ascendant))
        };
        let here = self.lagna.rasi;

        // Far past any birth time's uncertainty. Inside the polar circles a
        // sign really can hold longer than this, and the answer is then
        // reported as the cap rather than a false precision.
        const CAP_MINUTES: f64 = 12.0 * 60.0;

        // `dir` is -1 looking back to when the lagna entered this rasi, +1
        // forward to when it leaves.
        let edge = |dir: f64| -> Result<f64, EphemError> {
            let (mut lo, mut hi) = (0.0f64, 1.0f64);
            while rasi_at(self.jd_ut + dir * hi / 1440.0)? == here {
                lo = hi;
                hi *= 2.0;
                if hi > CAP_MINUTES {
                    return Ok(CAP_MINUTES);
                }
            }
            // Sixteen halvings of at most 12 hours: under a tenth of a
            // second, far finer than any birth time is ever known.
            for _ in 0..16 {
                let mid = 0.5 * (lo + hi);
                if rasi_at(self.jd_ut + dir * mid / 1440.0)? == here {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            Ok(0.5 * (lo + hi))
        };

        Ok(LagnaWindow { minutes_in: edge(-1.0)?, minutes_left: edge(1.0)? })
    }

    pub fn placement(&self, graha: Graha) -> &Placement {
        self.placements
            .iter()
            .find(|p| p.graha == graha)
            .expect("all nine grahas are always present")
    }

    /// Janma rasi - the sign occupied by the Moon. In South India this, not
    /// the lagna, is what someone names when asked their rasi.
    pub fn janma_rasi(&self) -> Rasi {
        self.placement(Graha::Moon).rasi
    }

    /// Janma nakshatra - the birth star. The one thing every Tamil household
    /// knows about a chart, and the anchor of the Vimshottari dasha.
    pub fn janma_nakshatra(&self) -> NakshatraPosition {
        self.placement(Graha::Moon).nakshatra
    }

    /// Grahas occupying a bhava, 1..=12.
    pub fn grahas_in_house(&self, house: u8) -> Vec<Graha> {
        self.placements
            .iter()
            .filter(|p| p.house == house)
            .map(|p| p.graha)
            .collect()
    }

    /// The rasi that forms a bhava, under whole-sign.
    pub fn rasi_of_house(&self, house: u8) -> Rasi {
        Rasi::from_index(self.lagna.rasi.index() as i32 + (house as i32 - 1))
    }

    /// Bhavadhipati - the lord of a bhava.
    pub fn lord_of_house(&self, house: u8) -> Graha {
        self.rasi_of_house(house).lord()
    }

    /// The bhava a graha occupies.
    pub fn house_of(&self, graha: Graha) -> u8 {
        self.placement(graha).house
    }

    /// Project onto a divisional chart. The varga lagna is the varga sign of
    /// the ascendant, and houses are recounted from there.
    pub fn varga(&self, varga: Varga) -> VargaChart {
        let lagna = varga.sign_of(self.lagna.longitude);
        let mut signs = [(Graha::Sun, Rasi::Mesha, 0u8); 9];
        for (slot, p) in signs.iter_mut().zip(self.placements.iter()) {
            let sign = varga.sign_of(p.longitude);
            *slot = (p.graha, sign, lagna.houses_to(sign));
        }
        VargaChart { varga, lagna, placements: signs }
    }
}

/// A divisional chart: signs and houses only. Divisional charts carry no
/// degrees by construction, so nothing degree-based belongs here.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VargaChart {
    pub varga: Varga,
    pub lagna: Rasi,
    /// `(graha, sign, house)` in traditional graha order.
    pub placements: [(Graha, Rasi, u8); 9],
}

impl VargaChart {
    pub fn sign_of(&self, graha: Graha) -> Rasi {
        self.placements.iter().find(|(g, _, _)| *g == graha).unwrap().1
    }

    pub fn house_of(&self, graha: Graha) -> u8 {
        self.placements.iter().find(|(g, _, _)| *g == graha).unwrap().2
    }

    pub fn grahas_in_house(&self, house: u8) -> Vec<Graha> {
        self.placements
            .iter()
            .filter(|(_, _, h)| *h == house)
            .map(|(g, _, _)| *g)
            .collect()
    }
}
