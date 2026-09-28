//! The facts a rule can read. Everything is derived from Phases 1-2; this
//! module adds only reference resolution and the benefic/malefic class.

use lagn_core::{
    condition, drishti, Ashtakavarga, Chart, DerivationSettings, Dignity, Graha, Rasi, Varga,
    Vimshottari,
};

use crate::model::{GrahaRef, GrahaSet, PeriodLevel, SetName, Sex};

/// Optional facts about the native that are not in the birth chart.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeInfo {
    pub sex: Option<Sex>,
}

/// Everything a rule may read about one chart.
pub struct FactBase {
    /// (mahadasha lord, antardasha lord) while evaluating period rules.
    pub period: Option<(Graha, Graha)>,
    pub chart: Chart,
    pub settings: DerivationSettings,
    pub ashtakavarga: Ashtakavarga,
    pub dasha: Vimshottari,
    pub native: NativeInfo,
}

impl FactBase {
    pub fn new(chart: Chart, settings: DerivationSettings, native: NativeInfo) -> FactBase {
        FactBase {
            period: None,
            ashtakavarga: Ashtakavarga::compute(&chart),
            dasha: Vimshottari::compute(&chart),
            chart,
            settings,
            native,
        }
    }

    /// Resolve a reference to a concrete graha. `None` only for a period-lord
    /// reference outside period evaluation, which the validator forbids.
    pub fn resolve(&self, r: GrahaRef) -> Option<Graha> {
        match r {
            GrahaRef::Named(g) => Some(g),
            GrahaRef::LordOf { house, varga } => Some(self.rasi_of_house(house, varga).lord()),
            GrahaRef::Period(PeriodLevel::Maha) => self.period.map(|p| p.0),
            GrahaRef::Period(PeriodLevel::Antar) => self.period.map(|p| p.1),
        }
    }

    pub fn lagna(&self, varga: Varga) -> Rasi {
        varga.sign_of(self.chart.lagna.longitude)
    }

    pub fn rasi_of_house(&self, house: u8, varga: Varga) -> Rasi {
        Rasi::from_index(self.lagna(varga).index() as i32 + house as i32 - 1)
    }

    pub fn sign_of(&self, g: Graha, varga: Varga) -> Rasi {
        varga.sign_of(self.chart.placement(g).longitude)
    }

    pub fn house_of(&self, g: Graha, varga: Varga) -> u8 {
        self.lagna(varga).houses_to(self.sign_of(g, varga))
    }

    /// `None` for the nodes, which is what makes a dignity test Unknown.
    pub fn dignity(&self, g: Graha, varga: Varga) -> Option<Dignity> {
        self.chart.dignity(g, varga, &self.settings)
    }

    pub fn combust(&self, g: Graha) -> bool {
        let p = self.chart.placement(g);
        condition::is_combust(g, p.longitude, p.retrograde, self.chart.placement(Graha::Sun).longitude)
    }

    pub fn retrograde(&self, g: Graha) -> bool {
        self.chart.placement(g).retrograde
    }

    /// Graha drishti in D-1, honouring the node-aspect variant.
    pub fn aspects_sign(&self, g: Graha, target: Rasi) -> bool {
        drishti::graha_aspects(g, self.sign_of(g, Varga::D1), target, self.settings.node_aspects)
    }

    /// Moon waxing: elongation from the Sun in [0, 180).
    pub fn moon_waxing(&self) -> bool {
        let sun = self.chart.placement(Graha::Sun).longitude;
        let moon = self.chart.placement(Graha::Moon).longitude;
        let elong = (moon - sun).rem_euclid(360.0);
        elong < 180.0
    }

    /// Variant R-1.
    pub fn is_benefic(&self, g: Graha) -> bool {
        match g {
            Graha::Jupiter | Graha::Venus | Graha::Mercury => true,
            Graha::Moon => self.moon_waxing(),
            Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Rahu | Graha::Ketu => false,
        }
    }

    /// Members of a set, in traditional graha order, without duplicates.
    pub fn members(&self, set: &GrahaSet) -> Vec<Graha> {
        let mut out: Vec<Graha> = match set {
            GrahaSet::Named(SetName::Any) => Graha::ALL.to_vec(),
            GrahaSet::Named(SetName::Benefics) => {
                Graha::ALL.into_iter().filter(|&g| self.is_benefic(g)).collect()
            }
            GrahaSet::Named(SetName::Malefics) => {
                Graha::ALL.into_iter().filter(|&g| !self.is_benefic(g)).collect()
            }
            GrahaSet::Explicit(e) => e.grahas.iter().filter_map(|&r| self.resolve(r)).collect(),
        };
        out.sort();
        out.dedup();
        out
    }
}
