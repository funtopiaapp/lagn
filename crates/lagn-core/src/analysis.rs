//! Chart-level derivations: dignity in any varga, relationships, aspects and
//! conditions, under an explicit set of variant choices.
//!
//! Specification: `docs/phase2/DESIGN.md` sections 4-6 and the variant register
//! in section 8. V-3 to V-6 are switchable here; the rest are fixed until an
//! astrologer asks for an alternative.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::chart::Chart;
use crate::condition::{self, BaladiAvastha, JagradadiAvastha, WAR_CAPABLE, YUDDHA_ORB};
use crate::dignity::{self, Dignity, VargaMoolatrikona};
use crate::drishti::{self, NodeAspects};
use crate::rasi::Rasi;
use crate::relationship::{self, CompoundRelation, NaturalRelation, SEVEN};
use crate::varga::Varga;

/// Which chart supplies the temporary relationship when judging dignity in a
/// divisional chart. Variant V-5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporarySource {
    /// Always the rasi chart.
    #[default]
    Rasi,
    /// The divisional chart being judged.
    SameVarga,
}

/// Which relationship Jagradadi reads. Variant V-6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationBasis {
    #[default]
    Compound,
    Natural,
}

/// The switchable interpretive variants for Phase 2 derivations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DerivationSettings {
    /// V-3.
    pub node_aspects: NodeAspects,
    /// V-4.
    pub varga_moolatrikona: VargaMoolatrikona,
    /// V-5.
    pub temporary_source: TemporarySource,
    /// V-6.
    pub jagradadi_basis: RelationBasis,
}

impl Chart {
    /// Dignity of a graha in a varga. D-1 is degree-aware (DESIGN 4.1); every
    /// other varga is sign-only (4.2). `None` for Rahu and Ketu (V-1).
    pub fn dignity(&self, g: Graha, varga: Varga, s: &DerivationSettings) -> Option<Dignity> {
        if g.is_chhaya() {
            return None;
        }
        let p = self.placement(g);
        let (zone, occupied) = if varga == Varga::D1 {
            (dignity::rasi_zone(g, p.longitude), p.rasi)
        } else {
            let sign = varga.sign_of(p.longitude);
            (dignity::varga_zone(g, sign, s.varga_moolatrikona), sign)
        };
        let lord = occupied.lord();
        let temp_chart = match s.temporary_source {
            TemporarySource::Rasi => Varga::D1,
            TemporarySource::SameVarga => varga,
        };
        let temp_self = temp_chart.sign_of(p.longitude);
        let temp_lord = temp_chart.sign_of(self.placement(lord).longitude);
        dignity::resolve(g, zone, occupied, temp_self, temp_lord)
    }

    /// Compound relationship of `of` toward `toward`, temporary part from D-1.
    pub fn compound_relation(&self, of: Graha, toward: Graha) -> Option<CompoundRelation> {
        relationship::compound_by_signs(
            of,
            toward,
            self.placement(of).rasi,
            self.placement(toward).rasi,
        )
    }

    /// Signs aspected by a graha by graha drishti, in D-1.
    pub fn aspected_signs(&self, g: Graha, nodes: NodeAspects) -> Vec<Rasi> {
        drishti::signs_aspected(g, self.placement(g).rasi, nodes)
    }

    /// Grahas casting graha drishti on a sign, in D-1.
    pub fn grahas_aspecting(&self, target: Rasi, nodes: NodeAspects) -> Vec<Graha> {
        Graha::ALL
            .into_iter()
            .filter(|&g| drishti::graha_aspects(g, self.placement(g).rasi, target, nodes))
            .collect()
    }

    /// Pairs in graha yuddha, closest first, with their separation.
    pub fn graha_yuddha(&self) -> Vec<(Graha, Graha, f64)> {
        let mut out = Vec::new();
        for (i, &a) in WAR_CAPABLE.iter().enumerate() {
            for &b in &WAR_CAPABLE[i + 1..] {
                let d = condition::arc(self.placement(a).longitude, self.placement(b).longitude);
                if d <= YUDDHA_ORB {
                    out.push((a, b, d));
                }
            }
        }
        out.sort_by(|x, y| x.2.total_cmp(&y.2));
        out
    }

    /// Everything above, gathered for one chart.
    pub fn analyse(&self, s: &DerivationSettings) -> Analysis {
        let sun = self.placement(Graha::Sun).longitude;
        let wars = self.graha_yuddha();
        let conditions = Graha::ALL
            .into_iter()
            .map(|g| {
                let p = self.placement(g);
                let d1 = self.dignity(g, Varga::D1, s);
                GrahaCondition {
                    graha: g,
                    dignity: d1,
                    baladi: condition::baladi(p.longitude),
                    jagradadi: d1.map(|d| self.jagradadi_for(g, d, s)),
                    combust: condition::is_combust(g, p.longitude, p.retrograde, sun),
                    distance_from_sun: condition::arc(p.longitude, sun),
                    at_war_with: wars
                        .iter()
                        .filter_map(|&(a, b, _)| {
                            if a == g { Some(b) } else if b == g { Some(a) } else { None }
                        })
                        .collect(),
                    aspects: self.aspected_signs(g, s.node_aspects),
                }
            })
            .collect();

        let mut relations = Vec::new();
        for of in SEVEN {
            for toward in SEVEN {
                if of != toward {
                    relations.push(RelationEntry {
                        of,
                        toward,
                        natural: relationship::natural(of, toward).expect("seven grahas"),
                        compound: self.compound_relation(of, toward).expect("seven grahas"),
                    });
                }
            }
        }

        Analysis { settings: *s, conditions, relations, wars }
    }

    fn jagradadi_for(&self, g: Graha, d: Dignity, s: &DerivationSettings) -> JagradadiAvastha {
        match s.jagradadi_basis {
            RelationBasis::Compound => condition::jagradadi(d),
            RelationBasis::Natural => match d {
                Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign => JagradadiAvastha::Jagrat,
                Dignity::Debilitated => JagradadiAvastha::Sushupti,
                _ => match relationship::natural(g, self.placement(g).rasi.lord()) {
                    Some(NaturalRelation::Enemy) => JagradadiAvastha::Sushupti,
                    _ => JagradadiAvastha::Swapna,
                },
            },
        }
    }
}

/// Per-graha conditions in D-1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrahaCondition {
    pub graha: Graha,
    /// D-1 dignity. `None` for the nodes.
    pub dignity: Option<Dignity>,
    pub baladi: BaladiAvastha,
    /// `None` for the nodes.
    pub jagradadi: Option<JagradadiAvastha>,
    pub combust: bool,
    /// Shortest arc to the Sun, degrees.
    pub distance_from_sun: f64,
    pub at_war_with: Vec<Graha>,
    /// Signs this graha aspects by graha drishti.
    pub aspects: Vec<Rasi>,
}

/// One directed relationship.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationEntry {
    pub of: Graha,
    pub toward: Graha,
    pub natural: NaturalRelation,
    pub compound: CompoundRelation,
}

/// All Phase 2A derivations for a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// The variants these results were computed under.
    pub settings: DerivationSettings,
    pub conditions: Vec<GrahaCondition>,
    pub relations: Vec<RelationEntry>,
    pub wars: Vec<(Graha, Graha, f64)>,
}
