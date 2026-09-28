//! Family readings: a member's own chart beside the native's relational
//! reading of them. Specification: `docs/phase6/DESIGN.md` section 4.
//!
//! The server stays stateless: both birth records arrive with each request,
//! and members are stored only on the user's device.

use serde::{Deserialize, Serialize};

use crate::catalogue::{Focus, Karaka, TopicMeta};
use crate::writeup::{write_up, WriteUp};
use crate::corpus::Corpus;
use crate::facts::FactBase;
use crate::model::Scope;
use crate::pariharam::{suggest, Suggested};
use crate::resolve::{evaluate_topic, Mode, RuleResult, TopicReport};

/// Placeholder for the member's name, filled in by whatever displays the
/// text, so names never leave the device.
pub const NAME_TOKEN: &str = "{member}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Spouse,
    Child,
    Mother,
    Father,
}

impl Relation {
    /// The native's topic that reads this relation, and the tag that narrows
    /// it (the parents topic covers both parents).
    pub fn relational(self) -> (&'static str, Option<&'static str>) {
        match self {
            Relation::Spouse => ("marriage", None),
            Relation::Child => ("progeny", None),
            Relation::Mother => ("parents", Some("parent:mother")),
            Relation::Father => ("parents", Some("parent:father")),
        }
    }

    /// Topics read on the member's own chart. The first is compared with the
    /// native's relational reading (variant FA-1).
    pub fn own_topics(self) -> &'static [&'static str] {
        match self {
            Relation::Spouse => &["marriage", "health"],
            Relation::Child => &["health", "education", "marriage"],
            Relation::Mother | Relation::Father => &["health", "later_life"],
        }
    }

    pub fn house(self) -> u8 {
        match self {
            Relation::Spouse => 7,
            Relation::Child => 5,
            Relation::Mother => 4,
            Relation::Father => 9,
        }
    }
}

/// Which way a reading leans: the sign of its net score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lean {
    Favourable,
    Balanced,
    CallsForCare,
}

impl Lean {
    pub fn of(score: i32) -> Lean {
        match score.signum() {
            1 => Lean::Favourable,
            -1 => Lean::CallsForCare,
            _ => Lean::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Agreement {
    /// Both lean the same way.
    Agree,
    /// They lean opposite ways. Both are shown; neither overrides the other.
    Differ,
    /// At least one is balanced (net zero) or has no reviewed rules.
    Inconclusive,
}

pub fn agreement(a: Option<Lean>, b: Option<Lean>) -> Agreement {
    match (a, b) {
        (Some(x), Some(y)) if x == y && x != Lean::Balanced => Agreement::Agree,
        (Some(Lean::Favourable), Some(Lean::CallsForCare)) | (Some(Lean::CallsForCare), Some(Lean::Favourable)) => Agreement::Differ,
        _ => Agreement::Inconclusive,
    }
}

/// The native's reading of one relation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalReading {
    pub topic: String,
    pub house: u8,
    /// Effective scored results for this relation, in id order.
    pub supporting: Vec<RuleResult>,
    pub afflicting: Vec<RuleResult>,
    pub score: i32,
    /// None when no reviewed rule for this relation was admitted.
    pub lean: Option<Lean>,
    /// The native's chart read for this relation, as prose.
    pub writeup: Option<WriteUp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnReading {
    pub meta: Option<TopicMeta>,
    pub report: TopicReport,
    pub lean: Option<Lean>,
    pub pariharams: Vec<Suggested>,
    pub writeup: Option<WriteUp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyReading {
    pub relation: Relation,
    pub relational: RelationalReading,
    pub own: Vec<OwnReading>,
    /// The native's relational reading against the member's first own topic.
    pub agreement: Agreement,
    pub compared_with: String,
    /// "Together": what the two readings mean side by side, in plain words.
    pub comparison: Vec<String>,
}

impl Relation {
    /// "your spouse", for sentences.
    pub fn describe(self) -> &'static str {
        match self {
            Relation::Spouse => "your spouse",
            Relation::Child => "your child",
            Relation::Mother => "your mother",
            Relation::Father => "your father",
        }
    }

    /// What the native's own chart is read for, in plain words.
    pub fn read_for(self) -> &'static str {
        match self {
            Relation::Spouse => "marriage and the spouse",
            Relation::Child => "children",
            Relation::Mother => "the mother",
            Relation::Father => "the father",
        }
    }
}

impl Lean {
    pub fn plain(self) -> &'static str {
        match self {
            Lean::Favourable => "favourable",
            Lean::Balanced => "evenly balanced",
            Lean::CallsForCare => "calls for care",
        }
    }
}

/// The "Together" text: why two charts are read, what each says, and what it
/// means that they agree or differ. Fixed templates; nothing is merged or
/// averaged away. DESIGN phase 7 section 4.
fn comparison(relation: Relation, rel: &RelationalReading, own: Option<&OwnReading>, agreement: Agreement) -> Vec<String> {
    // Family names stay on the device, so the text carries a placeholder the
    // app fills in. Anything displaying this must substitute NAME_TOKEN.
    let who = NAME_TOKEN;
    let topic = own.and_then(|o| o.meta.as_ref().map(|m| m.title.to_lowercase())).unwrap_or_else(|| "their own life".into());
    let mut out = vec![format!(
        "Two charts are read here, and they answer different questions. Your own chart, read for {}, shows what this part of your life tends to bring you. {who}'s own chart shows their life, which is theirs and not yours. The tradition weighs both, and neither settles the other.",
        relation.read_for()
    )];
    match rel.lean {
        Some(l) => out.push(format!("Your chart, read for {}: {}. {}", relation.read_for(), l.plain(), match l {
            Lean::Favourable => "The indications there support this part of your life.",
            Lean::CallsForCare => "The indications there ask for patience and attention in this part of your life.",
            Lean::Balanced => "The supporting and challenging indications there carry equal weight.",
        })),
        None => out.push(format!("Your chart, read for {}: no reviewed indication applies, so nothing is claimed either way.", relation.read_for())),
    }
    match own.and_then(|o| o.lean) {
        Some(l) => out.push(format!("{}'s own chart, read for {}: {}. {}", who, topic, l.plain(), match l {
            Lean::Favourable => "That area of their life is well supported in their own chart.",
            Lean::CallsForCare => "That area of their life asks for attention in their own chart.",
            Lean::Balanced => "Supporting and challenging indications carry equal weight there.",
        })),
        None => out.push(format!("{}'s own chart: no reviewed indication applies for {topic}.", who)),
    }
    out.push(match agreement {
        Agreement::Agree => format!(
            "The two readings point the same way, which the tradition treats as the stronger signal: what your chart shows about {} and what {who}'s own chart shows agree with each other.",
            relation.describe()
        ),
        Agreement::Differ => format!(
            "The two readings point different ways. That is common, and it is not a contradiction: your chart describes what this relationship brings into your life, while {who}'s chart describes their own circumstances. Read both, and give weight to the one whose subject you are actually asking about.",
        ),
        Agreement::Inconclusive => "The two readings do not give a clear comparison, because at least one of them is evenly balanced or carries no reviewed indication. Read each on its own rather than drawing a conclusion from the pair.".to_string(),
    });
    out.push(format!(
        "Practically: for anything about {} themselves, their own chart is the one to follow. Your chart tells you about your side of the relationship.",
        relation.describe()
    ));
    out
}

fn lean_of(report: &TopicReport, keep: impl Fn(&RuleResult) -> bool) -> (i32, Option<Lean>) {
    let admitted: Vec<&RuleResult> = report.results.iter().filter(|r| keep(r)).collect();
    let score: i32 = admitted.iter().filter(|r| r.effective).map(|r| r.polarity as i32).sum();
    (score, (!admitted.is_empty()).then(|| Lean::of(score)))
}

fn ages(corpus: &Corpus, topic: &str) -> (f64, f64) {
    corpus.topics.iter().find(|t| t.id == topic).map(|t| (t.ages[0], t.ages[1])).unwrap_or((18.0, 45.0))
}

pub fn family_reading(corpus: &Corpus, native: &FactBase, member: &FactBase, relation: Relation, mode: Mode) -> FamilyReading {
    let (topic, tag) = relation.relational();
    let rep = evaluate_topic(topic, &corpus.rules, native, mode, ages(corpus, topic));
    let keep = |r: &RuleResult| tag.is_none_or(|t| r.tags.iter().any(|x| x == t));
    let (score, lean) = lean_of(&rep, keep);
    let pick = |pos: bool| -> Vec<RuleResult> {
        rep.results
            .iter()
            .filter(|r| keep(r) && r.effective && if pos { r.polarity > 0 } else { r.polarity < 0 })
            .cloned()
            .collect()
    };
    // The relational write-up explains only this relation's house and karaka.
    let meta = corpus.topics.iter().find(|t| t.id == topic && mode.admits(t.review.status));
    let focus = match relation {
        Relation::Spouse | Relation::Child => meta.map(|m| m.focus.clone()).unwrap_or_default(),
        Relation::Mother => relation_focus(meta, 4, lagn_core::Graha::Moon),
        Relation::Father => relation_focus(meta, 9, lagn_core::Graha::Sun),
    };
    let writeup = meta.map(|m| write_up(corpus, m, &focus, &rep, native, ages(corpus, topic), keep));
    let relational = RelationalReading {
        topic: topic.into(),
        house: relation.house(),
        supporting: pick(true),
        afflicting: pick(false),
        score,
        lean,
        writeup,
    };

    let own: Vec<OwnReading> = relation
        .own_topics()
        .iter()
        .filter(|t| corpus.rules.iter().any(|r| r.topic == **t && r.scope == Scope::Natal))
        .map(|t| {
            let report = evaluate_topic(t, &corpus.rules, member, mode, ages(corpus, t));
            let (_, lean) = lean_of(&report, |_| true);
            let pariharams = suggest(corpus, &report.results, &[], mode);
            let meta = corpus.topics.iter().find(|m| m.id == *t && mode.admits(m.review.status)).cloned();
            let writeup = meta.as_ref().map(|m| write_up(corpus, m, &m.focus, &report, member, ages(corpus, t), |_| true));
            OwnReading { meta, report, lean, pariharams, writeup }
        })
        .collect();
    let compared_with = relation.own_topics()[0].to_string();
    let first = own.iter().find(|o| o.report.topic == compared_with);
    let agreement = agreement(relational.lean, first.and_then(|o| o.lean));
    let comparison = comparison(relation, &relational, first, agreement);
    FamilyReading { relation, agreement, relational, own, compared_with, comparison }
}

/// One parent's slice of the parents focus: their house, their karaka, and
/// the matching divisional check.
fn relation_focus(meta: Option<&TopicMeta>, house: u8, karaka: lagn_core::Graha) -> Focus {
    let m = meta.map(|m| &m.focus);
    Focus {
        houses: vec![house],
        karakas: m
            .map(|f| f.karakas.iter().filter(|k| k.graha == karaka).cloned().collect::<Vec<Karaka>>())
            .unwrap_or_default(),
        vargas: m.map(|f| f.vargas.iter().filter(|v| v.house == house).cloned().collect()).unwrap_or_default(),
        // A relation's slice never carries the past-life sections.
        karmic_axis: false,
        bridge: false,
    }
}
