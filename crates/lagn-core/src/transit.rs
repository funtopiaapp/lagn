//! Transits (gochara) of the slow grahas, counted from the natal Moon.
//! Specification: `docs/phase6/DESIGN.md` section 3.5.

use lagn_ephem::{EphemError, Ephemeris, Graha};
use serde::{Deserialize, Serialize};

use crate::ashtakavarga::Ashtakavarga;
use crate::chart::Chart;
use crate::rasi::Rasi;

/// Search step in days. Half a day, so that a planet stationing within
/// arcminutes of a sign boundary cannot cross and re-cross between samples.
const STEP: f64 = 0.5;
/// Bisection resolution: one minute of time.
const RESOLUTION: f64 = 1.0 / 1440.0;

/// The grahas transits are computed for.
pub const TRANSIT_GRAHAS: [Graha; 4] = [Graha::Saturn, Graha::Jupiter, Graha::Rahu, Graha::Ketu];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ingress {
    pub graha: Graha,
    /// Julian Day (UT) of entering `to`, to within one minute.
    pub jd_ut: f64,
    pub from: Rasi,
    pub to: Rasi,
}

fn sign_at(eph: &Ephemeris, g: Graha, jd: f64) -> Result<Rasi, EphemError> {
    Ok(Rasi::from_longitude(eph.position(jd, g)?.longitude))
}

/// Every sidereal sign ingress of `g` in `[start, end)`, in time order.
pub fn ingresses(eph: &Ephemeris, g: Graha, start: f64, end: f64) -> Result<Vec<Ingress>, EphemError> {
    let mut out = Vec::new();
    let mut t = start;
    let mut sign = sign_at(eph, g, t)?;
    while t < end {
        let next = (t + STEP).min(end);
        let s2 = sign_at(eph, g, next)?;
        if s2 != sign {
            let (mut lo, mut hi) = (t, next);
            while hi - lo > RESOLUTION {
                let mid = 0.5 * (lo + hi);
                if sign_at(eph, g, mid)? == sign { lo = mid } else { hi = mid }
            }
            let to = sign_at(eph, g, hi)?;
            out.push(Ingress { graha: g, jd_ut: hi, from: sign, to });
            sign = to;
            // Continue from the crossing, so a quick re-crossing is not skipped.
            t = hi;
            continue;
        }
        t = next;
    }
    Ok(out)
}

/// A graha's stay in one sign, clipped to the requested range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignStay {
    pub graha: Graha,
    pub sign: Rasi,
    pub start_jd: f64,
    pub end_jd: f64,
}

pub fn sign_stays(eph: &Ephemeris, g: Graha, start: f64, end: f64) -> Result<Vec<SignStay>, EphemError> {
    let mut stays = Vec::new();
    let mut from = start;
    let mut sign = sign_at(eph, g, start)?;
    for i in ingresses(eph, g, start, end)? {
        stays.push(SignStay { graha: g, sign, start_jd: from, end_jd: i.jd_ut });
        from = i.jd_ut;
        sign = i.to;
    }
    stays.push(SignStay { graha: g, sign, start_jd: from, end_jd: end });
    Ok(stays)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitKind {
    SadeSatiRising,
    SadeSatiPeak,
    SadeSatiSetting,
    Ardhashtama,
    /// Variant T-1: Saturn 7th or 10th from the Moon.
    Kantaka,
    Ashtama,
    GuruFavourable,
    NodeFavourable,
}

impl TransitKind {
    pub fn supportive(self) -> bool {
        matches!(self, TransitKind::GuruFavourable | TransitKind::NodeFavourable)
    }

    pub fn label(self) -> &'static str {
        match self {
            TransitKind::SadeSatiRising => "Sade sati (rising): Saturn 12th from the Moon",
            TransitKind::SadeSatiPeak => "Sade sati (peak): Saturn over the Moon sign",
            TransitKind::SadeSatiSetting => "Sade sati (setting): Saturn 2nd from the Moon",
            TransitKind::Ardhashtama => "Ardhashtama shani: Saturn 4th from the Moon",
            TransitKind::Kantaka => "Kantaka shani: Saturn 7th or 10th from the Moon",
            TransitKind::Ashtama => "Ashtama shani: Saturn 8th from the Moon",
            TransitKind::GuruFavourable => "Favourable Guru transit",
            TransitKind::NodeFavourable => "Favourable Rahu/Ketu transit",
        }
    }
}

/// Classify a transit house from the Moon.
pub fn classify(g: Graha, house_from_moon: u8) -> Option<TransitKind> {
    use TransitKind::*;
    match (g, house_from_moon) {
        (Graha::Saturn, 12) => Some(SadeSatiRising),
        (Graha::Saturn, 1) => Some(SadeSatiPeak),
        (Graha::Saturn, 2) => Some(SadeSatiSetting),
        (Graha::Saturn, 4) => Some(Ardhashtama),
        (Graha::Saturn, 7) | (Graha::Saturn, 10) => Some(Kantaka),
        (Graha::Saturn, 8) => Some(Ashtama),
        (Graha::Jupiter, 2 | 5 | 7 | 9 | 11) => Some(GuruFavourable),
        (Graha::Rahu | Graha::Ketu, 3 | 6 | 11) => Some(NodeFavourable),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransitWindow {
    pub kind: TransitKind,
    pub graha: Graha,
    pub sign: Rasi,
    pub house_from_moon: u8,
    pub start_jd: f64,
    pub end_jd: f64,
    pub supportive: bool,
    /// Bindus in the transit sign in this graha's own BAV (Saturn and Jupiter;
    /// the nodes have no BAV).
    pub bindus: Option<u8>,
    /// 4 or more bindus: the sign supports this graha. Classical ashtakavarga
    /// reads a supported Saturn transit as milder, and a supported Jupiter
    /// transit as fuller in its benefits.
    pub bav_supports: Option<bool>,
}

impl Chart {
    /// Gochara windows from the natal Moon sign in `[start, end)`.
    pub fn gochara(&self, start: f64, end: f64) -> Result<Vec<TransitWindow>, EphemError> {
        let eph = Ephemeris::new(self.settings.ayanamsa, self.settings.node_type);
        let moon = self.janma_rasi();
        let av = Ashtakavarga::compute(self);
        let mut out = Vec::new();
        for g in TRANSIT_GRAHAS {
            for stay in sign_stays(&eph, g, start, end)? {
                let h = moon.houses_to(stay.sign);
                let Some(kind) = classify(g, h) else { continue };
                let bindus = av.bindus(g, stay.sign);
                out.push(TransitWindow {
                    kind,
                    graha: g,
                    sign: stay.sign,
                    house_from_moon: h,
                    start_jd: stay.start_jd,
                    end_jd: stay.end_jd,
                    supportive: kind.supportive(),
                    bindus,
                    bav_supports: bindus.map(|b| b >= 4),
                });
            }
        }
        out.sort_by(|a, b| a.start_jd.total_cmp(&b.start_jd).then((a.graha as u8).cmp(&(b.graha as u8))));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_table() {
        use TransitKind::*;
        assert_eq!(classify(Graha::Saturn, 12), Some(SadeSatiRising));
        assert_eq!(classify(Graha::Saturn, 1), Some(SadeSatiPeak));
        assert_eq!(classify(Graha::Saturn, 2), Some(SadeSatiSetting));
        assert_eq!(classify(Graha::Saturn, 8), Some(Ashtama));
        assert_eq!(classify(Graha::Saturn, 3), None);
        assert_eq!(classify(Graha::Jupiter, 9), Some(GuruFavourable));
        assert_eq!(classify(Graha::Jupiter, 8), None);
        assert_eq!(classify(Graha::Ketu, 11), Some(NodeFavourable));
        assert_eq!(classify(Graha::Sun, 1), None);
        assert!(GuruFavourable.supportive() && !Ashtama.supportive());
    }
}
