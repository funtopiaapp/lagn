//! The 27 nakshatras, their padas, and their Vimshottari lords.
//!
//! A nakshatra spans 13 deg 20 min; a pada is a quarter of that, 3 deg 20 min.
//! 27 x 4 = 108 padas, which is exactly the number of navamsas in the zodiac -
//! the two divisions are the same grid, and `varga` relies on that identity.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

/// Span of one nakshatra in degrees: 360/27.
pub const NAKSHATRA_SPAN: f64 = 360.0 / 27.0;
/// Span of one pada in degrees: 360/108.
pub const PADA_SPAN: f64 = 360.0 / 108.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Nakshatra {
    Ashwini = 0, Bharani, Krittika, Rohini, Mrigashira, Ardra,
    Punarvasu, Pushya, Ashlesha, Magha, PurvaPhalguni, UttaraPhalguni,
    Hasta, Chitra, Swati, Vishakha, Anuradha, Jyeshtha,
    Mula, PurvaAshadha, UttaraAshadha, Shravana, Dhanishta, Shatabhisha,
    PurvaBhadrapada, UttaraBhadrapada, Revati,
}

impl Nakshatra {
    pub const ALL: [Nakshatra; 27] = {
        use Nakshatra::*;
        [
            Ashwini, Bharani, Krittika, Rohini, Mrigashira, Ardra,
            Punarvasu, Pushya, Ashlesha, Magha, PurvaPhalguni, UttaraPhalguni,
            Hasta, Chitra, Swati, Vishakha, Anuradha, Jyeshtha,
            Mula, PurvaAshadha, UttaraAshadha, Shravana, Dhanishta, Shatabhisha,
            PurvaBhadrapada, UttaraBhadrapada, Revati,
        ]
    };

    #[inline]
    pub fn from_index(i: i32) -> Nakshatra {
        Nakshatra::ALL[i.rem_euclid(27) as usize]
    }

    #[inline]
    pub fn index(self) -> u8 {
        self as u8
    }

    /// The nakshatra containing a sidereal longitude.
    ///
    /// Derived from the 108-cell pada grid rather than from a separate floor
    /// on the 27-cell grid, so the two can never disagree. See
    /// [`NakshatraPosition::from_longitude`].
    #[inline]
    pub fn from_longitude(lon: f64) -> Nakshatra {
        Nakshatra::from_index(absolute_pada(lon) / 4)
    }

    /// Vimshottari dasha lord.
    ///
    /// The nine lords repeat in a fixed cycle three times across the 27
    /// nakshatras, beginning with Ketu at Ashwini.
    pub fn lord(self) -> Graha {
        const CYCLE: [Graha; 9] = [
            Graha::Ketu, Graha::Venus, Graha::Sun, Graha::Moon, Graha::Mars,
            Graha::Rahu, Graha::Jupiter, Graha::Saturn, Graha::Mercury,
        ];
        CYCLE[(self.index() % 9) as usize]
    }

    /// Sanskrit name.
    pub fn name(self) -> &'static str {
        use Nakshatra::*;
        match self {
            Ashwini => "Ashwini", Bharani => "Bharani", Krittika => "Krittika",
            Rohini => "Rohini", Mrigashira => "Mrigashira", Ardra => "Ardra",
            Punarvasu => "Punarvasu", Pushya => "Pushya", Ashlesha => "Ashlesha",
            Magha => "Magha", PurvaPhalguni => "Purva Phalguni",
            UttaraPhalguni => "Uttara Phalguni", Hasta => "Hasta",
            Chitra => "Chitra", Swati => "Swati", Vishakha => "Vishakha",
            Anuradha => "Anuradha", Jyeshtha => "Jyeshtha", Mula => "Mula",
            PurvaAshadha => "Purva Ashadha", UttaraAshadha => "Uttara Ashadha",
            Shravana => "Shravana", Dhanishta => "Dhanishta",
            Shatabhisha => "Shatabhisha", PurvaBhadrapada => "Purva Bhadrapada",
            UttaraBhadrapada => "Uttara Bhadrapada", Revati => "Revati",
        }
    }

    /// Tamil name, as spoken and as printed on a jathagam.
    pub fn tamil_name(self) -> &'static str {
        use Nakshatra::*;
        match self {
            Ashwini => "Ashwini", Bharani => "Bharani", Krittika => "Karthigai",
            Rohini => "Rohini", Mrigashira => "Mrigasheersham",
            Ardra => "Thiruvathirai", Punarvasu => "Punarpoosam",
            Pushya => "Poosam", Ashlesha => "Ayilyam", Magha => "Magam",
            PurvaPhalguni => "Pooram", UttaraPhalguni => "Uthiram",
            Hasta => "Hastham", Chitra => "Chithirai", Swati => "Swathi",
            Vishakha => "Visakam", Anuradha => "Anusham", Jyeshtha => "Kettai",
            Mula => "Moolam", PurvaAshadha => "Pooradam",
            UttaraAshadha => "Uthiradam", Shravana => "Thiruvonam",
            Dhanishta => "Avittam", Shatabhisha => "Sathayam",
            PurvaBhadrapada => "Poorattathi", UttaraBhadrapada => "Uthirattathi",
            Revati => "Revathi",
        }
    }
}

/// A longitude resolved onto the nakshatra grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NakshatraPosition {
    pub nakshatra: Nakshatra,
    /// Pada within the nakshatra, 1..=4.
    pub pada: u8,
    /// Degrees traversed within the nakshatra, `[0, 13.333)`.
    pub degrees_within: f64,
    /// Fraction of the nakshatra already traversed, `[0, 1)`.
    /// This is what sets the balance of the birth Vimshottari dasha.
    pub fraction_traversed: f64,
}

/// Index of the pada containing a longitude, 0..=107.
///
/// This single floor is the one source of truth for both the 27-fold and the
/// 108-fold division. Computing them independently - a floor on `/13.333` for
/// the nakshatra and another on `/3.333` for the pada - lets rounding put them
/// one cell apart, which previously had to be papered over with a `.min(4)`
/// clamp that silently hid the inconsistency rather than preventing it.
///
/// 108 = 27 x 4 exactly, so deriving the nakshatra as `pada / 4` is not an
/// approximation; it is the same partition read at a coarser resolution.
#[inline]
fn absolute_pada(lon: f64) -> i32 {
    let lon = lagn_ephem::norm360(lon);
    // norm360 guarantees [0, 360), so this is already in 0..=107; the clamp is
    // belt-and-braces against a future change to that guarantee.
    ((lon / PADA_SPAN).floor() as i32).clamp(0, 107)
}

impl NakshatraPosition {
    pub fn from_longitude(lon: f64) -> NakshatraPosition {
        let lon = lagn_ephem::norm360(lon);
        let abs = absolute_pada(lon);
        let nakshatra = Nakshatra::from_index(abs / 4);
        let pada = (abs % 4) as u8 + 1;

        // Derived, so it can sit a rounding step outside its nominal span;
        // clamp to keep the documented [0, NAKSHATRA_SPAN) contract.
        let degrees_within =
            (lon - (nakshatra.index() as f64) * NAKSHATRA_SPAN).clamp(0.0, NAKSHATRA_SPAN);
        let fraction_traversed = (degrees_within / NAKSHATRA_SPAN).clamp(0.0, 1.0 - f64::EPSILON);

        NakshatraPosition { nakshatra, pada, degrees_within, fraction_traversed }
    }

    pub fn lord(&self) -> Graha {
        self.nakshatra.lord()
    }

    /// Absolute pada index across the zodiac, 0..=107.
    ///
    /// Equals the navamsa cell index: `Rasi::from_index(absolute_pada())` is
    /// the D-9 sign.
    #[inline]
    pub fn absolute_pada(&self) -> u8 {
        self.nakshatra.index() * 4 + (self.pada - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nakshatra_boundaries() {
        assert_eq!(Nakshatra::from_longitude(0.0), Nakshatra::Ashwini);
        assert_eq!(Nakshatra::from_longitude(13.3333), Nakshatra::Ashwini);
        assert_eq!(Nakshatra::from_longitude(13.3334), Nakshatra::Bharani);
        assert_eq!(Nakshatra::from_longitude(359.99), Nakshatra::Revati);
    }

    #[test]
    fn vimshottari_lords_cycle_three_times() {
        // Each of the nine lords must own exactly three nakshatras.
        for g in [
            Graha::Ketu, Graha::Venus, Graha::Sun, Graha::Moon, Graha::Mars,
            Graha::Rahu, Graha::Jupiter, Graha::Saturn, Graha::Mercury,
        ] {
            let n = Nakshatra::ALL.iter().filter(|nk| nk.lord() == g).count();
            assert_eq!(n, 3, "{} owns {n} nakshatras, expected 3", g.name());
        }
        assert_eq!(Nakshatra::Ashwini.lord(), Graha::Ketu);
        assert_eq!(Nakshatra::Bharani.lord(), Graha::Venus);
        assert_eq!(Nakshatra::Magha.lord(), Graha::Ketu);
        assert_eq!(Nakshatra::Mula.lord(), Graha::Ketu);
        assert_eq!(Nakshatra::Revati.lord(), Graha::Mercury);
    }

    #[test]
    fn padas_partition_each_nakshatra() {
        for i in 0..27 {
            let base = i as f64 * NAKSHATRA_SPAN;
            for p in 0..4 {
                let lon = base + p as f64 * PADA_SPAN + 0.001;
                let np = NakshatraPosition::from_longitude(lon);
                assert_eq!(np.nakshatra.index(), i, "nakshatra at {lon}");
                assert_eq!(np.pada, p + 1, "pada at {lon}");
            }
        }
    }

    #[test]
    fn absolute_pada_spans_exactly_108() {
        let first = NakshatraPosition::from_longitude(0.0);
        assert_eq!(first.absolute_pada(), 0);
        let last = NakshatraPosition::from_longitude(360.0 - 1e-9);
        assert_eq!(last.absolute_pada(), 107);
    }

    #[test]
    fn fraction_traversed_is_monotonic_within_a_nakshatra() {
        let a = NakshatraPosition::from_longitude(0.0);
        let b = NakshatraPosition::from_longitude(NAKSHATRA_SPAN / 2.0);
        assert!((a.fraction_traversed - 0.0).abs() < 1e-12);
        assert!((b.fraction_traversed - 0.5).abs() < 1e-12);
    }
}
