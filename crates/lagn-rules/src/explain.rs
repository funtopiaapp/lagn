//! Plain-language rendering of conditions, for traces and the review sheet.
//!
//! Generated from the rule itself, so what the astrologer reads is exactly
//! what the engine evaluates. Nothing here is hand-written per rule.

use lagn_core::{Dignity, Varga};

use crate::model::{Condition, FromRef, GrahaRef, GrahaSet, SetName, Sex};

pub fn ordinal(n: u8) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// "a", "a or b", "a, b or c".
pub fn or_list(items: &[String]) -> String {
    match items {
        [] => "(nothing)".into(),
        [one] => one.clone(),
        _ => format!("{} or {}", items[..items.len() - 1].join(", "), items[items.len() - 1]),
    }
}

fn houses(hs: &[u8]) -> String {
    let words: Vec<String> = hs.iter().map(|&h| ordinal(h)).collect();
    if hs.len() == 1 {
        format!("the {} house", words[0])
    } else {
        format!("the {} house", or_list(&words))
    }
}

fn in_varga(v: Varga) -> String {
    if v == Varga::D1 { String::new() } else { format!(" in the {}", v.label()) }
}

pub fn graha(r: &GrahaRef) -> String {
    match *r {
        GrahaRef::Named(g) => g.name().to_string(),
        GrahaRef::LordOf { house, varga } => {
            format!("the lord of the {}{}", ordinal(house), in_varga(varga))
        }
        GrahaRef::Period(crate::model::PeriodLevel::Maha) => "the mahadasha lord".to_string(),
        GrahaRef::Period(crate::model::PeriodLevel::Antar) => "the antardasha lord".to_string(),
    }
}

fn set(s: &GrahaSet) -> String {
    match s {
        GrahaSet::Named(SetName::Benefics) => "benefics".into(),
        GrahaSet::Named(SetName::Malefics) => "malefics".into(),
        GrahaSet::Named(SetName::Any) => "any graha".into(),
        GrahaSet::Explicit(e) => {
            let names: Vec<String> = e.grahas.iter().map(graha).collect();
            or_list(&names)
        }
    }
}

pub fn dignity(d: Dignity) -> &'static str {
    match d {
        Dignity::Exalted => "exalted",
        Dignity::Moolatrikona => "in moolatrikona",
        Dignity::OwnSign => "in its own sign",
        Dignity::GreatFriend => "in a great friend's sign",
        Dignity::Friend => "in a friend's sign",
        Dignity::Neutral => "in a neutral sign",
        Dignity::Enemy => "in an enemy's sign",
        Dignity::GreatEnemy => "in a great enemy's sign",
        Dignity::Debilitated => "debilitated",
    }
}

fn at_least(min: u8, what: String) -> String {
    if min <= 1 { what } else { format!("at least {min} of {what}") }
}

/// One condition as an English clause.
pub fn condition(c: &Condition) -> String {
    match c {
        Condition::All(cs) => {
            let parts: Vec<String> = cs.iter().map(condition).collect();
            format!("all of: [{}]", parts.join("; "))
        }
        Condition::Any(cs) => {
            let parts: Vec<String> = cs.iter().map(condition).collect();
            format!("any of: [{}]", parts.join("; "))
        }
        Condition::Not(x) => format!("NOT ({})", condition(x)),
        Condition::Always {} => "always".into(),
        Condition::GrahaInHouse { graha: g, houses: hs, varga } => {
            format!("{} is in {}{}", graha(g), houses(hs), in_varga(*varga))
        }
        Condition::GrahaInSign { graha: g, signs, varga } => {
            let names: Vec<String> = signs.iter().map(|s| s.name().to_string()).collect();
            format!("{} is in {}{}", graha(g), or_list(&names), in_varga(*varga))
        }
        Condition::GrahaFrom { graha: g, from, houses: hs } => {
            let origin = match from {
                FromRef::Lagna => "the lagna".to_string(),
                FromRef::Graha(r) => graha(r),
            };
            format!("{} is in {} counted from {origin}", graha(g), houses(hs))
        }
        Condition::Dignity { graha: g, any_of, varga } => {
            let names: Vec<String> = any_of.iter().map(|d| dignity(*d).to_string()).collect();
            format!("{} is {}{}", graha(g), or_list(&names), in_varga(*varga))
        }
        Condition::HouseOccupied { house, by, min, varga } => {
            format!("the {} house{} is occupied by {}", ordinal(*house), in_varga(*varga), at_least(*min, set(by)))
        }
        Condition::HouseAspected { house, by, min } => {
            format!("the {} house is aspected by {}", ordinal(*house), at_least(*min, set(by)))
        }
        Condition::Conjunct { a, b } => format!("{} and {} are in the same sign", graha(a), graha(b)),
        Condition::AspectsGraha { from, to } => format!("{} aspects {}", graha(from), graha(to)),
        Condition::Combust { graha: g } => format!("{} is combust", graha(g)),
        Condition::Retrograde { graha: g } => format!("{} is retrograde", graha(g)),
        Condition::Sav { house, min, max } => match (min, max) {
            (Some(a), Some(b)) => format!("the {} house has {a} to {b} SAV bindus", ordinal(*house)),
            (Some(a), None) => format!("the {} house has at least {a} SAV bindus", ordinal(*house)),
            (None, Some(b)) => format!("the {} house has at most {b} SAV bindus", ordinal(*house)),
            (None, None) => format!("the {} house has any number of SAV bindus", ordinal(*house)),
        },
        Condition::Functional { graha: g, is } => {
            let names: Vec<String> = is.iter().map(|n| match n {
                lagn_core::FunctionalNature::Yogakaraka => "the yogakaraka".to_string(),
                lagn_core::FunctionalNature::Benefic => "a functional benefic".to_string(),
                lagn_core::FunctionalNature::Malefic => "a functional malefic".to_string(),
                lagn_core::FunctionalNature::Neutral => "functionally neutral".to_string(),
            }).collect();
            format!("{} is {} for this lagna", graha(g), or_list(&names))
        }
        Condition::RulesHouse { graha: g, houses: hs } => format!("{} rules {}", graha(g), houses(hs)),
        Condition::Sambandha { a, b, kinds } => {
            let how = if kinds.is_empty() {
                "related (conjunction, mutual aspect or exchange)".to_string()
            } else {
                let k: Vec<String> = kinds.iter().map(|k| match k {
                    lagn_core::SambandhaKind::Conjunct => "conjunct".to_string(),
                    lagn_core::SambandhaKind::MutualAspect => "in mutual aspect".to_string(),
                    lagn_core::SambandhaKind::Exchange => "in exchange (parivartana)".to_string(),
                }).collect();
                or_list(&k)
            };
            format!("{} and {} are {how}", graha(a), graha(b))
        }
        Condition::NeechaBhanga { graha: g } => format!("{} is debilitated with the debilitation cancelled (neecha bhanga)", graha(g)),
        Condition::Native { sex } => format!(
            "the native is {}",
            match sex { Sex::Female => "female", Sex::Male => "male" }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals() {
        let got: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 23].iter().map(|&n| ordinal(n)).collect();
        assert_eq!(got, ["1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd"]);
    }

    #[test]
    fn lists() {
        assert_eq!(or_list(&["a".into()]), "a");
        assert_eq!(or_list(&["a".into(), "b".into()]), "a or b");
        assert_eq!(or_list(&["a".into(), "b".into(), "c".into()]), "a, b or c");
    }
}
