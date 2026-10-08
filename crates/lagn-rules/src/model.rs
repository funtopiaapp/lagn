//! The rule language. Specification: `docs/phase3/DESIGN.md` section 3.
//!
//! Every struct denies unknown fields: a typo in the corpus is a load error,
//! never a silently ignored condition.

use lagn_core::{Dignity, FunctionalNature, Graha, Rasi, SambandhaKind, Varga};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn d1() -> Varga {
    Varga::D1
}
fn one() -> u8 {
    1
}
fn is_d1(v: &Varga) -> bool {
    *v == Varga::D1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub topic: String,
    pub title: String,
    pub tradition: Tradition,
    pub when: Condition,
    /// `natal` (default): evaluated once per chart. `period`: evaluated once
    /// per Vimshottari antardasha, and may use the period-lord references.
    #[serde(default, skip_serializing_if = "Scope::is_natal")]
    pub scope: Scope,
    /// -3..=3. Integer, so topic scores are exact sums.
    pub polarity: i8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timing: Vec<GrahaRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// The graha the rule is about, resolved per chart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<GrahaRef>,
    pub text: Text,
    pub source: Source,
    pub review: Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Natal,
    Period,
}

impl Scope {
    fn is_natal(&self) -> bool {
        *self == Scope::Natal
    }
}

/// Which lord of the running period a period rule refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodLevel {
    Maha,
    Antar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tradition {
    Parashari,
    Tamil,
    Kerala,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub en: String,
    /// What this tends to mean in a person's life, in plain, gentle words
    /// (DESIGN section 4b). Shown under `en` wherever the rule is displayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// A citation checked against the text, or null. Never invented.
    pub reference: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub status: ReviewStatus,
    #[serde(default)]
    pub reviewer: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sex {
    Female,
    Male,
}

// ---------------------------------------------------------------------------
// References
// ---------------------------------------------------------------------------

/// A graha, named directly, as the lord of a house, or (in period rules
/// only) as the lord of the running mahadasha or antardasha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrahaRef {
    Named(Graha),
    LordOf { house: u8, varga: Varga },
    Period(PeriodLevel),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PeriodRaw {
    period: PeriodLevel,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LordOfRaw {
    lord_of: u8,
    #[serde(default = "d1", skip_serializing_if = "is_d1")]
    varga: Varga,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GrahaRefRaw {
    Named(Graha),
    Lord(LordOfRaw),
    Period(PeriodRaw),
}

impl<'de> Deserialize<'de> for GrahaRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match GrahaRefRaw::deserialize(d)? {
            GrahaRefRaw::Named(g) => GrahaRef::Named(g),
            GrahaRefRaw::Lord(l) => GrahaRef::LordOf { house: l.lord_of, varga: l.varga },
            GrahaRefRaw::Period(p) => GrahaRef::Period(p.period),
        })
    }
}

impl Serialize for GrahaRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match *self {
            GrahaRef::Named(g) => GrahaRefRaw::Named(g).serialize(s),
            GrahaRef::LordOf { house, varga } => {
                GrahaRefRaw::Lord(LordOfRaw { lord_of: house, varga }).serialize(s)
            }
            GrahaRef::Period(p) => GrahaRefRaw::Period(PeriodRaw { period: p }).serialize(s),
        }
    }
}

/// The point `graha_from` counts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FromRef {
    Lagna,
    Graha(GrahaRef),
}

impl<'de> Deserialize<'de> for FromRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        if v.as_str() == Some("lagna") {
            return Ok(FromRef::Lagna);
        }
        serde_json::from_value::<GrahaRef>(v)
            .map(FromRef::Graha)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for FromRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            FromRef::Lagna => s.serialize_str("lagna"),
            FromRef::Graha(g) => g.serialize(s),
        }
    }
}

/// A set of grahas for occupancy and aspect tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GrahaSet {
    Named(SetName),
    Explicit(ExplicitSet),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetName {
    Benefics,
    Malefics,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitSet {
    pub grahas: Vec<GrahaRef>,
}

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    Always {},
    GrahaInHouse {
        graha: GrahaRef,
        houses: Vec<u8>,
        #[serde(default = "d1", skip_serializing_if = "is_d1")]
        varga: Varga,
    },
    GrahaInSign {
        graha: GrahaRef,
        signs: Vec<Rasi>,
        #[serde(default = "d1", skip_serializing_if = "is_d1")]
        varga: Varga,
    },
    GrahaFrom {
        graha: GrahaRef,
        from: FromRef,
        houses: Vec<u8>,
    },
    Dignity {
        graha: GrahaRef,
        #[serde(rename = "in")]
        any_of: Vec<Dignity>,
        #[serde(default = "d1", skip_serializing_if = "is_d1")]
        varga: Varga,
    },
    HouseOccupied {
        house: u8,
        by: GrahaSet,
        #[serde(default = "one")]
        min: u8,
        #[serde(default = "d1", skip_serializing_if = "is_d1")]
        varga: Varga,
    },
    HouseAspected {
        house: u8,
        by: GrahaSet,
        #[serde(default = "one")]
        min: u8,
    },
    Conjunct {
        a: GrahaRef,
        b: GrahaRef,
    },
    AspectsGraha {
        from: GrahaRef,
        to: GrahaRef,
    },
    Combust {
        graha: GrahaRef,
    },
    Retrograde {
        graha: GrahaRef,
    },
    Sav {
        house: u8,
        #[serde(default)]
        min: Option<u8>,
        #[serde(default)]
        max: Option<u8>,
    },
    Native {
        sex: Sex,
    },
    /// Functional nature for the lagna (DESIGN 6, section 3.1).
    Functional {
        graha: GrahaRef,
        is: Vec<FunctionalNature>,
    },
    /// The graha rules any of these houses (D-1).
    RulesHouse {
        graha: GrahaRef,
        houses: Vec<u8>,
    },
    /// The two grahas are related: conjunction, mutual aspect or exchange.
    /// An empty `kinds` means any of the three.
    Sambandha {
        a: GrahaRef,
        b: GrahaRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        kinds: Vec<SambandhaKind>,
    },
    /// Debilitated and cancelled (DESIGN 6, section 3.3).
    NeechaBhanga {
        graha: GrahaRef,
    },
}

impl Condition {
    /// Number of atomic conditions: a rough specificity measure for reports.
    pub fn atoms(&self) -> usize {
        match self {
            Condition::All(v) | Condition::Any(v) => v.iter().map(Condition::atoms).sum(),
            Condition::Not(c) => c.atoms(),
            _ => 1,
        }
    }

    /// Every GrahaRef this condition mentions, including nested ones.
    pub fn refs(&self) -> Vec<GrahaRef> {
        let mut out = Vec::new();
        self.walk(&mut |c| match c {
            Condition::GrahaInHouse { graha, .. } | Condition::GrahaInSign { graha, .. }
            | Condition::Dignity { graha, .. } | Condition::Combust { graha } | Condition::Retrograde { graha }
            | Condition::Functional { graha, .. } | Condition::RulesHouse { graha, .. }
            | Condition::NeechaBhanga { graha } => out.push(*graha),
            Condition::GrahaFrom { graha, from, .. } => {
                out.push(*graha);
                if let FromRef::Graha(r) = from { out.push(*r); }
            }
            Condition::Conjunct { a, b } | Condition::Sambandha { a, b, .. } => { out.push(*a); out.push(*b); }
            Condition::AspectsGraha { from, to } => { out.push(*from); out.push(*to); }
            Condition::HouseOccupied { by: GrahaSet::Explicit(e), .. } | Condition::HouseAspected { by: GrahaSet::Explicit(e), .. } => {
                out.extend(e.grahas.iter().copied())
            }
            _ => {}
        });
        out
    }

    /// Visit every node, depth-first.
    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a Condition)) {
        f(self);
        match self {
            Condition::All(v) | Condition::Any(v) => v.iter().for_each(|c| c.walk(f)),
            Condition::Not(c) => c.walk(f),
            _ => {}
        }
    }
}
