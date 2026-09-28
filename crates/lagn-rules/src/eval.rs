//! Three-valued evaluator. Specification: `docs/phase3/DESIGN.md` section 4.

use lagn_core::{Graha, Varga};
use serde::{Deserialize, Serialize};

use crate::explain;
use crate::facts::FactBase;
use crate::model::{Condition, FromRef};

/// Kleene truth value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truth {
    True,
    False,
    Unknown,
}

impl Truth {
    pub fn from_bool(b: bool) -> Truth {
        if b { Truth::True } else { Truth::False }
    }

    /// Kleene AND over any number of operands. Empty is True.
    pub fn all(values: &[Truth]) -> Truth {
        if values.contains(&Truth::False) {
            Truth::False
        } else if values.contains(&Truth::Unknown) {
            Truth::Unknown
        } else {
            Truth::True
        }
    }

    /// Kleene OR over any number of operands. Empty is False.
    pub fn any(values: &[Truth]) -> Truth {
        if values.contains(&Truth::True) {
            Truth::True
        } else if values.contains(&Truth::Unknown) {
            Truth::Unknown
        } else {
            Truth::False
        }
    }

}

/// Kleene NOT.
impl std::ops::Not for Truth {
    type Output = Truth;
    fn not(self) -> Truth {
        match self {
            Truth::True => Truth::False,
            Truth::False => Truth::True,
            Truth::Unknown => Truth::Unknown,
        }
    }
}

/// One evaluated atomic condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEntry {
    /// The condition in plain language.
    pub condition: String,
    pub value: Truth,
    /// The facts it read, in plain language.
    pub facts: String,
}

fn list(gs: &[Graha]) -> String {
    if gs.is_empty() {
        "none".into()
    } else {
        gs.iter().map(|g| g.name()).collect::<Vec<_>>().join(", ")
    }
}

fn vtag(v: Varga) -> String {
    if v == Varga::D1 { String::new() } else { format!(" in D-{}", v.number()) }
}

/// Evaluate a condition. Every child of `all`/`any` is evaluated - no
/// short-circuiting - so the trace is complete and independent of order.
pub fn eval(c: &Condition, f: &FactBase, trace: &mut Vec<TraceEntry>) -> Truth {
    // Record an atomic result. A plain function rather than a closure, so it
    // doesn't hold a borrow of `trace` across the recursive calls below.
    fn record(trace: &mut Vec<TraceEntry>, c: &Condition, value: Truth, facts: String) -> Truth {
        trace.push(TraceEntry { condition: explain::condition(c), value, facts });
        value
    }
    macro_rules! atom {
        ($value:expr, $facts:expr $(,)?) => {
            record(trace, c, $value, $facts)
        };
    }
    // Resolve a reference, or record Unknown and return. Only a period-lord
    // reference outside period evaluation can fail, and the validator forbids it.
    macro_rules! res {
        ($r:expr) => {
            match f.resolve($r) {
                Some(g) => g,
                None => return atom!(Truth::Unknown, "the period lord is defined only in period rules".into()),
            }
        };
    }
    match c {
        Condition::All(cs) => {
            let vs: Vec<Truth> = cs.iter().map(|x| eval(x, f, trace)).collect();
            Truth::all(&vs)
        }
        Condition::Any(cs) => {
            let vs: Vec<Truth> = cs.iter().map(|x| eval(x, f, trace)).collect();
            Truth::any(&vs)
        }
        Condition::Not(x) => !eval(x, f, trace),
        Condition::Always {} => atom!(Truth::True, "always true".into()),

        Condition::GrahaInHouse { graha, houses, varga } => {
            let g = res!(*graha);
            let h = f.house_of(g, *varga);
            atom!(Truth::from_bool(houses.contains(&h)), format!("{} in house {h}{}", g.name(), vtag(*varga)))
        }
        Condition::GrahaInSign { graha, signs, varga } => {
            let g = res!(*graha);
            let s = f.sign_of(g, *varga);
            atom!(Truth::from_bool(signs.contains(&s)), format!("{} in {}{}", g.name(), s.name(), vtag(*varga)))
        }
        Condition::GrahaFrom { graha, from, houses } => {
            let g = res!(*graha);
            let (origin, label) = match from {
                FromRef::Lagna => (f.lagna(Varga::D1), "the lagna".to_string()),
                FromRef::Graha(r) => {
                    let o = res!(*r);
                    (f.sign_of(o, Varga::D1), o.name().to_string())
                }
            };
            let h = origin.houses_to(f.sign_of(g, Varga::D1));
            atom!(Truth::from_bool(houses.contains(&h)), format!("{} is house {h} from {label}", g.name()))
        }
        Condition::Dignity { graha, any_of, varga } => {
            let g = res!(*graha);
            match f.dignity(g, *varga) {
                Some(d) => atom!(
                    Truth::from_bool(any_of.contains(&d)),
                    format!("{} is {}{}", g.name(), explain::dignity(d), vtag(*varga)),
                ),
                None => atom!(Truth::Unknown, format!("{} has no dignity in this system (variant V-1)", g.name())),
            }
        }
        Condition::HouseOccupied { house, by, min, varga } => {
            let members = f.members(by);
            let present: Vec<Graha> =
                members.into_iter().filter(|&g| f.house_of(g, *varga) == *house).collect();
            atom!(
                Truth::from_bool(present.len() >= *min as usize),
                format!("house {house}{} holds {}", vtag(*varga), list(&present)),
            )
        }
        Condition::HouseAspected { house, by, min } => {
            let target = f.rasi_of_house(*house, Varga::D1);
            let aspecting: Vec<Graha> =
                f.members(by).into_iter().filter(|&g| f.aspects_sign(g, target)).collect();
            atom!(
                Truth::from_bool(aspecting.len() >= *min as usize),
                format!("house {house} ({}) aspected by {}", target.name(), list(&aspecting)),
            )
        }
        Condition::Conjunct { a, b } => {
            let (ga, gb) = (res!(*a), res!(*b));
            if ga == gb {
                return atom!(Truth::False, format!("both refer to {} - not a conjunction", ga.name()));
            }
            let (sa, sb) = (f.sign_of(ga, Varga::D1), f.sign_of(gb, Varga::D1));
            atom!(
                Truth::from_bool(sa == sb),
                format!("{} in {}, {} in {}", ga.name(), sa.name(), gb.name(), sb.name()),
            )
        }
        Condition::AspectsGraha { from, to } => {
            let (gf, gt) = (res!(*from), res!(*to));
            let target = f.sign_of(gt, Varga::D1);
            atom!(
                Truth::from_bool(f.aspects_sign(gf, target)),
                format!("{} in {}, {} in {}", gf.name(), f.sign_of(gf, Varga::D1).name(), gt.name(), target.name()),
            )
        }
        Condition::Combust { graha } => {
            let g = res!(*graha);
            let d = lagn_core::condition::arc(
                f.chart.placement(g).longitude,
                f.chart.placement(Graha::Sun).longitude,
            );
            atom!(Truth::from_bool(f.combust(g)), format!("{} is {d:.2} deg from the Sun", g.name()))
        }
        Condition::Retrograde { graha } => {
            let g = res!(*graha);
            atom!(Truth::from_bool(f.retrograde(g)), format!("{} {}", g.name(), if f.retrograde(g) { "is retrograde" } else { "is direct" }))
        }
        Condition::Sav { house, min, max } => {
            let v = f.ashtakavarga.sav_in_house(*house);
            let ok = min.is_none_or(|m| v >= m) && max.is_none_or(|m| v <= m);
            atom!(Truth::from_bool(ok), format!("SAV of house {house} is {v}"))
        }
        Condition::Native { sex } => match f.native.sex {
            Some(s) => atom!(Truth::from_bool(s == *sex), format!("native is {s:?}")),
            None => atom!(Truth::Unknown, "native's sex was not supplied".into()),
        },
        Condition::Functional { graha, is } => {
            let g = res!(*graha);
            match f.chart.functional_nature(g) {
                Some(n) => atom!(Truth::from_bool(is.contains(&n)), format!("{} is functionally {:?} for this lagna", g.name(), n)),
                None => atom!(Truth::Unknown, format!("{} has no functional nature (it rules no sign)", g.name())),
            }
        }
        Condition::RulesHouse { graha, houses } => {
            let g = res!(*graha);
            let ruled = lagn_core::functional::houses_ruled(f.chart.lagna.rasi, g);
            let hit = ruled.iter().any(|h| houses.contains(h));
            let shown: Vec<String> = ruled.iter().map(|h| h.to_string()).collect();
            atom!(Truth::from_bool(hit), format!("{} rules house(s) {}", g.name(), if shown.is_empty() { "none".into() } else { shown.join(", ") }))
        }
        Condition::Sambandha { a, b, kinds } => {
            let (ga, gb) = (res!(*a), res!(*b));
            if ga == gb {
                return atom!(Truth::False, format!("both refer to {} - no sambandha with itself", ga.name()));
            }
            let found = f.chart.sambandha(ga, gb, &f.settings);
            let hit = found.iter().any(|k| kinds.is_empty() || kinds.contains(k));
            let shown: Vec<String> = found.iter().map(|k| format!("{k:?}")).collect();
            atom!(Truth::from_bool(hit), format!("{} and {}: {}", ga.name(), gb.name(), if shown.is_empty() { "unrelated".into() } else { shown.join(", ") }))
        }
        Condition::NeechaBhanga { graha } => {
            let g = res!(*graha);
            match f.chart.neecha_bhanga(g, &f.settings) {
                Some(v) => atom!(Truth::from_bool(v), format!("{} {}", g.name(), if v { "is debilitated and the debilitation is cancelled" } else { "is not in cancelled debilitation" })),
                None => atom!(Truth::Unknown, format!("{} has no debilitation in this system", g.name())),
            }
        }
    }
}

/// A minimal justification for a condition that evaluated True, as the facts
/// behind it: every child of an `all`, the first true child of an `any`, and
/// the fact behind a `not`. Unlike the trace, it never lists facts from
/// branches that did not decide the result. DESIGN section 4a.
pub fn justify(c: &Condition, f: &FactBase) -> Vec<String> {
    let facts_of = |x: &Condition| -> Vec<String> {
        let mut t = Vec::new();
        eval(x, f, &mut t);
        t.into_iter().map(|e| e.facts).collect()
    };
    let mut out: Vec<String> = match c {
        Condition::All(cs) => cs.iter().flat_map(|x| justify(x, f)).collect(),
        Condition::Any(cs) => cs
            .iter()
            .find(|x| eval(x, f, &mut Vec::new()) == Truth::True)
            .map(|x| justify(x, f))
            .unwrap_or_default(),
        Condition::Not(x) => facts_of(x),
        Condition::Always {} => Vec::new(),
        atom => facts_of(atom),
    };
    out.dedup();
    out
}
