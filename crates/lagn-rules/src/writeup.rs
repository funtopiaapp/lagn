//! Written interpretations: a topic reading as prose a lay reader can follow,
//! each statement justified by the chart fact behind it. Fixed templates only:
//! the same chart always gives the same text. Specification:
//! `docs/phase6/DESIGN.md` section 4a.

use lagn_core::{jd_to_civil, Dignity, Graha, Varga};
use serde::{Deserialize, Serialize};

use crate::catalogue::{Bhavas, Focus, TopicMeta};
use crate::corpus::Corpus;
use crate::eval::justify;
use crate::facts::FactBase;
use crate::resolve::{RuleResult, TopicReport};
use crate::Truth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteUp {
    /// The conclusion, first.
    pub summary: Vec<String>,
    pub sections: Vec<Section>,
}

/// What a section is, so a front end can present it without matching on the
/// heading text (phase 7 design section 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    House,
    Karakas,
    Varga,
    Supporting,
    Care,
    Noted,
    /// A named dosha's verdict, stated whether or not it applies.
    Dosha,
    Eased,
    Unknown,
    Timing,
    /// Ketu and Rahu: what is carried forward against what is asked now.
    KarmicAxis,
    /// Temperament, work and station carried forward.
    LifeCarried,
    /// Marriage and children, read as what those houses have already been through.
    Bonds,
    /// Debts (rina) and their settlement.
    Debts,
    /// The same houses, read through the native's other topics.
    Bridge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub kind: SectionKind,
    pub heading: String,
    pub paragraphs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<Point>,
}

/// One classical finding, with the facts that made it apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub rule: String,
    pub title: String,
    pub text: String,
    /// What it means for the reader, in plain words.
    pub meaning: Option<String>,
    pub polarity: i8,
    pub because: Vec<String>,
}

/// "Saturn (Shani)".
pub fn graha_name(g: Graha) -> String {
    let en = match g {
        Graha::Sun => "the Sun",
        Graha::Moon => "the Moon",
        Graha::Mars => "Mars",
        Graha::Mercury => "Mercury",
        Graha::Jupiter => "Jupiter",
        Graha::Venus => "Venus",
        Graha::Saturn => "Saturn",
        Graha::Rahu => "Rahu",
        Graha::Ketu => "Ketu",
    };
    if matches!(g, Graha::Rahu | Graha::Ketu) { en.to_string() } else { format!("{en} ({})", g.name()) }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

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

/// "a", "a and b", "a, b and c" - the engine never shows a reader a bare
/// comma-separated list.
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// Items that contain commas, joined with semicolons: "a, x; and b, y".
fn join_semi(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{}; and {}", init.join("; "), last),
    }
}

fn dignity_phrase(d: Dignity) -> &'static str {
    match d {
        Dignity::Exalted => "exalted, its strongest dignity",
        Dignity::Moolatrikona => "in its moolatrikona sign, very strong",
        Dignity::OwnSign => "in its own sign, strong and at ease",
        Dignity::GreatFriend => "in a great friend's sign, well supported",
        Dignity::Friend => "in a friend's sign, comfortable",
        Dignity::Neutral => "in a neutral sign",
        Dignity::Enemy => "in an enemy's sign, somewhat constrained",
        Dignity::GreatEnemy => "in a great enemy's sign, constrained",
        Dignity::Debilitated => "debilitated, its weakest dignity",
    }
}

/// What a lord's house position (from the lagna) means for the houses it rules.
fn position_phrase(h: u8) -> &'static str {
    match h {
        1 => "the lagna itself, which ties these matters closely to your own person and efforts",
        4 | 7 | 10 => "a kendra (angular house), a position of strength and visibility",
        5 | 9 => "a trikona, among the most auspicious houses, which classical texts read as favour and fortune",
        2 | 11 => "a house of wealth and gains, generally supportive",
        3 => "the 3rd, an upachaya house, where results grow through effort over time",
        6 => "the 6th, a dusthana, which classical texts read as effort, competition or obstacles for the houses it rules (it is also an upachaya, improving with time)",
        8 => "the 8th, a dusthana, which classical texts read as delays, changes or hidden difficulties for the houses it rules",
        _ => "the 12th, a dusthana, which classical texts read as expense, distance or withdrawal for the houses it rules",
    }
}

fn nature(f: &FactBase, g: Graha) -> &'static str {
    match g {
        Graha::Rahu | Graha::Ketu => "a shadow graha, treated as malefic",
        Graha::Moon if f.is_benefic(g) => "waxing, so counted as a natural benefic",
        Graha::Moon => "waning, so counted as a natural malefic",
        _ if f.is_benefic(g) => "a natural benefic",
        _ => "a natural malefic",
    }
}

/// "the Moon (Chandra), waxing, so counted as a natural benefic"
fn with_nature(f: &FactBase, g: Graha) -> String {
    format!("{}, {}", graha_name(g), nature(f, g))
}

/// The condition of one graha: house, sign, dignity, combustion, retrogression.
fn condition_of(f: &FactBase, g: Graha) -> String {
    let h = f.house_of(g, Varga::D1);
    format!("in your {} house, {}", ordinal(h), sign_condition(f, g))
}

/// Sign, dignity, combustion and retrogression, without the house.
fn sign_condition(f: &FactBase, g: Graha) -> String {
    let sign = f.sign_of(g, Varga::D1);
    let mut s = format!("in {} ({})", sign.name(), sign.tamil_name());
    if let Some(d) = f.dignity(g, Varga::D1) {
        s.push_str(&format!(", where it is {}", dignity_phrase(d)));
        if d == Dignity::Debilitated && f.chart.neecha_bhanga(g, &f.settings) == Some(true) {
            s.push_str(", though the debilitation is cancelled (neecha bhanga), which classical texts read as weakness that is overcome");
        }
    }
    if g != Graha::Sun && f.combust(g) {
        s.push_str("; it is combust (too close to the Sun), which dims its results");
    }
    if f.retrograde(g) && !matches!(g, Graha::Rahu | Graha::Ketu) {
        s.push_str("; it is retrograde, traditionally read as results that come with review or delay");
    }
    s
}

fn house_section(f: &FactBase, bhavas: Option<&Bhavas>, h: u8) -> Section {
    let meaning = bhavas.and_then(|b| b.houses.iter().find(|m| m.house == h));
    let sign = f.rasi_of_house(h, Varga::D1);
    let lord = sign.lord();
    let lh = f.house_of(lord, Varga::D1);
    let mut paragraphs = Vec::new();

    let what = meaning.map(|m| format!(", {} bhava, governs {}", m.name, join_semi(&m.significations))).unwrap_or_default();
    paragraphs.push(format!(
        "Your {} house{what}. It falls in {} ({}), so its lord is {}.",
        ordinal(h), sign.name(), sign.tamil_name(), graha_name(lord)
    ));
    let lord_line = if lh == h {
        format!("{} sits in this very house, {}. A lord in its own house protects and strengthens that house.", cap(&graha_name(lord)), sign_condition(f, lord))
    } else {
        format!("{} is {}. The {} house from the lagna is {}.", cap(&graha_name(lord)), condition_of(f, lord), ordinal(lh), position_phrase(lh))
    };
    paragraphs.push(lord_line);

    let occupants: Vec<Graha> = Graha::ALL.into_iter().filter(|&g| f.house_of(g, Varga::D1) == h).collect();
    if occupants.is_empty() {
        paragraphs.push("No graha occupies this house, so the condition of its lord carries the most weight.".into());
    } else {
        let list: Vec<String> = occupants.iter().map(|&g| with_nature(f, g)).collect();
        let benefics = occupants.iter().filter(|&&g| f.is_benefic(g)).count();
        let effect = match (benefics, occupants.len() - benefics) {
            (_, 0) => "Benefics here support the matters of this house.",
            (0, _) => "Malefics here bring effort and testing to the matters of this house, though they also give drive.",
            _ => "The benefics here support its matters, while the malefics bring effort and testing, though also drive.",
        };
        paragraphs.push(format!("This house is occupied by {}. {effect}", join_semi(&list)));
    }
    let aspecting: Vec<Graha> = Graha::ALL.into_iter().filter(|&g| f.aspects_sign(g, sign)).collect();
    if !aspecting.is_empty() {
        let list: Vec<String> = aspecting.iter().map(|&g| with_nature(f, g)).collect();
        let jupiter = if aspecting.contains(&Graha::Jupiter) { " Jupiter's aspect in particular is classically protective." } else { "" };
        paragraphs.push(format!("It receives the aspect (drishti) of {}.{jupiter}", join_semi(&list)));
    }
    let sav = f.ashtakavarga.sav_in_house(h);
    let judgement = match sav {
        s if s >= 30 => "well above the average of 28: a strongly supported house",
        28 | 29 => "at or just above the average of 28: an adequately supported house",
        25..=27 => "a little below the average of 28: support here needs effort",
        _ => "well below the average of 28: this house needs care and effort to deliver",
    };
    paragraphs.push(format!("In Ashtakavarga it holds {sav} bindus, {judgement}."));

    let heading = meaning.map(|m| format!("The {} house: {}", ordinal(h), m.name)).unwrap_or_else(|| format!("The {} house", ordinal(h)));
    Section { kind: SectionKind::House, heading, paragraphs, points: Vec::new() }
}

/// Temperament, work and station carried forward: Ketu's house gives the
/// station, its sign the temperament, and the lord of that sign says how the
/// life went. All text comes from `corpus/karma.json`. Phase 9 section 3.
fn life_carried_section(f: &FactBase, karma: Option<&crate::catalogue::Karma>) -> Option<Section> {
    let k = karma?;
    let house = f.house_of(Graha::Ketu, Varga::D1);
    let sign = f.sign_of(Graha::Ketu, Varga::D1);
    let station = k.station(house)?;
    let temperament = k.temperament(sign.index() + 1)?;
    let disp = sign.lord();
    let dh = f.house_of(disp, Varga::D1);
    let strong = matches!(
        f.dignity(disp, Varga::D1),
        Some(Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign | Dignity::GreatFriend | Dignity::Friend)
    ) && !matches!(dh, 6 | 8 | 12);
    let mut paragraphs = vec![
        format!(
            "The station — Ketu sits in your {} house, and the tradition reads that as {}.",
            ordinal(house), station
        ),
        format!(
            "The temperament — Ketu falls in {} ({}), read as a nature {}. That is the cast of mind you are held to arrive with, before this life adds anything to it.",
            temperament.name, sign.tamil_name(), temperament.temperament
        ),
        format!(
            "How that life went — Ketu gives its results through {}, the lord of the sign it occupies, which is {}. {}",
            graha_name(disp),
            condition_of(f, disp),
            if strong { &k.dispositor.strong } else { &k.dispositor.weak }
        ),
    ];

    // The kinds of life the placement is associated with. Several, always, and
    // phrased as what the tradition associates rather than as a claim about
    // the native - a chart can no more encode an occupation than a name. The
    // house gives the sphere and the dispositor the craft.
    let callings = k.callings(house);
    if !callings.is_empty() {
        let list: Vec<String> = callings.iter().map(|c| c.to_string()).collect();
        let mut t = format!(
            "The kinds of life it is associated with — readings of this placement traditionally \
             point to lives of {}.",
            join_and(&list)
        );
        if let Some(craft) = k.craft(disp) {
            t.push_str(&format!(
                " With {} as the dispositor, the tradition narrows that to work that {}.",
                graha_name(disp), craft
            ));
        }
        t.push_str(
            " These are possibilities of one kind, offered together and never one at a time: the \
             tradition associates them with the placement, and nothing here is a record of a past \
             identity or can be checked against one.",
        );
        paragraphs.push(t);
    }

    // Last, so it qualifies everything above it rather than interrupting.
    paragraphs.push(
        "Read all of this as orientation and pattern, which is what the placements can carry. A \
         chart holds nine grahas in twelve houses; it cannot hold a name, a year or a place, so \
         none is given."
            .to_string(),
    );

    Some(Section { kind: SectionKind::LifeCarried, heading: "The life carried forward".into(), paragraphs, points: Vec::new() })
}

/// Bonds carried forward. The 6th stands 12th from the 7th and is read as what
/// the house of marriage has already passed through; the 4th stands 12th from
/// the 5th and is read the same way for children (variant PL-4).
fn bonds_section(f: &FactBase, bhavas: Option<&Bhavas>) -> Section {
    let describe = |h: u8, from: u8, what: &str| -> String {
        let sign = f.rasi_of_house(h, Varga::D1);
        let lord = sign.lord();
        let occupants: Vec<Graha> = Graha::ALL.into_iter().filter(|&g| f.house_of(g, Varga::D1) == h).collect();
        let who = if occupants.is_empty() {
            "No graha stands there, so the condition of its lord carries the reading".to_string()
        } else {
            format!("It holds {}", join_semi(&occupants.iter().map(|&g| with_nature(f, g)).collect::<Vec<_>>()))
        };
        let name = bhavas.and_then(|b| b.houses.iter().find(|m| m.house == h)).map(|m| m.name.clone()).unwrap_or_default();
        format!(
            "{what} — your {} house ({name}) stands 12th from the {}, and the tradition reads the 12th from a house as what that house has already passed through. It falls in {} ({}), so its lord is {}, which is {}. {who}.",
            ordinal(h), ordinal(from), sign.name(), sign.tamil_name(), graha_name(lord), condition_of(f, lord)
        )
    };
    let mut paragraphs = vec![describe(6, 7, "The partner"), describe(4, 5, "Children")];
    let nodes_in = |h: u8| Graha::ALL.into_iter().filter(|&g| matches!(g, Graha::Rahu | Graha::Ketu) && f.house_of(g, Varga::D1) == h).collect::<Vec<_>>();
    if let Some(&g) = nodes_in(7).first() {
        paragraphs.push(format!("{} also sits in your 7th house itself, which the tradition reads as a marriage bond continued rather than newly begun: recognition on meeting, and a partnership that feels settled or unsettled faster than it should.", graha_name(g)));
    }
    if let Some(&g) = nodes_in(5).first() {
        paragraphs.push(format!("{} sits in your 5th house, the house of children, which is read as the serpent debt described below rather than as anything settled about children themselves.", graha_name(g)));
    }
    paragraphs.push("Both of these are read as patterns brought into the present, not as a record of who anyone was. What your own chart says about marriage and children today is in those readings, and they take precedence.".to_string());
    Section { kind: SectionKind::Bonds, heading: "Bonds carried forward".into(), paragraphs, points: Vec::new() }
}

/// The Ketu-Rahu axis. Ketu is what the tradition reads as finished and
/// carried; Rahu, always opposite, is what this life asks the native to take
/// up. Phase 8 design section 4.
fn karmic_axis_section(f: &FactBase, bhavas: Option<&Bhavas>) -> Section {
    let significations = |h: u8| -> String {
        bhavas
            .and_then(|b| b.houses.iter().find(|m| m.house == h))
            .map(|m| join_semi(&m.significations))
            .unwrap_or_default()
    };
    let (kh, rh) = (f.house_of(Graha::Ketu, Varga::D1), f.house_of(Graha::Rahu, Varga::D1));
    let (ks, rs) = (f.sign_of(Graha::Ketu, Varga::D1), f.sign_of(Graha::Rahu, Varga::D1));
    let disp = ks.lord();
    let mut paragraphs = vec![
        format!(
            "Ketu sits in your {} house, in {} ({}). The tradition reads Ketu as what is already finished: the ground you arrive on, where you need less instruction than others and feel less need to prove anything. That house governs {}.",
            ordinal(kh), ks.name(), ks.tamil_name(), significations(kh)
        ),
        format!(
            "Ketu rules no sign of its own, so it gives its results through the lord of the sign it occupies: {}, which is {}. That graha's condition colours everything this axis describes.",
            graha_name(disp), condition_of(f, disp)
        ),
        format!(
            "Rahu stands opposite, in your {} house, in {} ({}). That is read as the direction of this life: unfamiliar ground, taken up with appetite and some clumsiness, and the place where growth is asked of you. That house governs {}.",
            ordinal(rh), rs.name(), rs.tamil_name(), significations(rh)
        ),
        format!(
            "Held together: what comes easily to you sits in the {} house, and what this life keeps pushing you towards sits in the {}. The tradition does not say to abandon the first; it says the second is where the work is.",
            ordinal(kh), ordinal(rh)
        ),
    ];
    if kh == 12 || rh == 12 {
        paragraphs.push("With this axis touching the 12th house, the tradition adds that solitude, rest and the inner life are part of the pattern rather than a retreat from it.".into());
    }
    Section { kind: SectionKind::KarmicAxis, heading: "What you carry, and what is asked now".into(), paragraphs, points: Vec::new() }
}

/// The same houses read through the native's other topics: the interpretation
/// against the present life. No new judgements - each topic's own rules decide.
fn bridge_section(corpus: &Corpus, f: &FactBase, this_topic: &str, mode: crate::resolve::Mode) -> Option<Section> {
    let (kh, rh) = (f.house_of(Graha::Ketu, Varga::D1), f.house_of(Graha::Rahu, Varga::D1));
    let mut paragraphs = vec![
        "These same houses are read in other parts of your chart. Where a reading there is favourable, the tradition treats it as merit already earned showing itself; where it calls for care, that is the work this life is asking for.".to_string(),
    ];
    let mut any = false;
    for (house, label) in [(kh, "Ketu's house, what you carry"), (rh, "Rahu's house, what is asked now"), (5u8, "the 5th house of merit")] {
        let topics: Vec<&TopicMeta> = corpus
            .topics
            .iter()
            .filter(|t| t.id != this_topic && t.focus.houses.contains(&house) && mode.admits(t.review.status))
            .collect();
        if topics.is_empty() {
            paragraphs.push(format!("The {} house ({label}) is not the ground of any other reading in this app.", ordinal(house)));
            continue;
        }
        let mut parts = Vec::new();
        for t in topics {
            let rep = crate::resolve::evaluate_topic(&t.id, &corpus.rules, f, mode, (t.ages[0], t.ages[1]));
            let verdict = if rep.results.is_empty() {
                "has no reviewed indication".to_string()
            } else {
                match rep.score.signum() {
                    1 => format!("comes out favourable ({:+})", rep.score),
                    -1 => format!("calls for care ({:+})", rep.score),
                    _ => "comes out evenly balanced".to_string(),
                }
            };
            parts.push(format!("your {} reading, which {verdict}", t.title.to_lowercase()));
            any = true;
        }
        paragraphs.push(format!("The {} house ({label}) is also the ground of {}.", ordinal(house), join_semi(&parts)));
    }
    any.then(|| Section { kind: SectionKind::Bridge, heading: "Where this shows in the rest of your chart".into(), paragraphs, points: Vec::new() })
}

fn karaka_section(f: &FactBase, focus: &Focus) -> Option<Section> {
    let paragraphs: Vec<String> = focus
        .karakas
        .iter()
        .filter(|k| k.sex.is_none_or(|s| f.native.sex == Some(s)))
        .map(|k| format!("{}, the natural significator of {}, is {}.", cap(&graha_name(k.graha)), k.role, condition_of(f, k.graha)))
        .collect();
    (!paragraphs.is_empty()).then(|| Section { kind: SectionKind::Karakas, heading: "The significators (karakas)".into(), paragraphs, points: Vec::new() })
}

fn varga_section(f: &FactBase, focus: &Focus) -> Option<Section> {
    let paragraphs: Vec<String> = focus
        .vargas
        .iter()
        .map(|v| {
            let lord = f.rasi_of_house(v.house, Varga::D1).lord();
            let sign = f.sign_of(lord, v.varga);
            let d = f.dignity(lord, v.varga).map(|d| format!(", where it is {}", dignity_phrase(d))).unwrap_or_default();
            let verdict = match f.dignity(lord, v.varga) {
                Some(Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign) => " This confirms and strengthens the promise of the main chart.",
                Some(Dignity::Debilitated) => " This weakens what the main chart promises and calls for patience.",
                _ => "",
            };
            format!(
                "In the {} (D-{}), the divisional chart read for this area, the lord of your {} house, {}, falls in {} ({}){d}.{verdict}",
                v.varga.name(), v.varga.number(), ordinal(v.house), graha_name(lord), sign.name(), sign.tamil_name()
            )
        })
        .collect();
    (!paragraphs.is_empty()).then(|| Section { kind: SectionKind::Varga, heading: "Divisional chart confirmation".into(), paragraphs, points: Vec::new() })
}

fn point(r: &RuleResult, rules: &[crate::Rule], f: &FactBase) -> Point {
    let because = rules.iter().find(|x| x.id == r.id).map(|x| justify(&x.when, f)).unwrap_or_default();
    Point { rule: r.id.clone(), title: r.title.clone(), text: r.text.clone(), meaning: r.impact.clone(), polarity: r.polarity, because }
}

/// A definite verdict on a named dosha: present, present-but-cancelled, or
/// absent. Readers open the marriage topic to ask about Chevvai dosha, and
/// before this the subtitle promised an answer that only appeared when the
/// dosha happened to apply (defect D4). Silence is not an answer, so the
/// absent case is stated too.
fn dosha_section(meta: &TopicMeta, report: &TopicReport, results: &[&RuleResult]) -> Option<Section> {
    let note = meta.dosha.as_ref()?;
    let mine = |id: &str| id.starts_with(note.rule_prefix.as_str());
    let name = note.name.as_str();

    // Rule titles repeat the dosha's name ("Chevvai dosha counted from the
    // lagna"), which would stutter once the name is already in the sentence.
    // What is left keeps its own capitalisation, so "the Moon" survives.
    let detail = |title: &str| -> String {
        title
            .strip_prefix(name)
            .map(|rest| rest.trim_start().to_string())
            .filter(|rest| !rest.is_empty())
            .unwrap_or_else(|| title.to_string())
    };
    let list = |items: Vec<String>| items.join("; ");

    // Only the scored rules decide presence; the cancellation rules under the
    // same prefix carry polarity 0 and would otherwise read as the dosha.
    let present: Vec<String> = results
        .iter()
        .filter(|r| mine(&r.id) && r.effective && r.polarity < 0)
        .map(|r| detail(&r.title))
        .collect();
    // Each entry names the count that was cancelled and the exception that
    // cancelled it; naming only one of the two reads as the other.
    let title_of = |id: &String| -> String {
        report
            .results
            .iter()
            .find(|r| &r.id == id)
            .map(|r| r.title.clone())
            .unwrap_or_else(|| id.clone())
    };
    let cancelled: Vec<String> = report
        .cancelled
        .iter()
        .filter(|(id, _)| mine(id))
        .map(|(id, by)| format!("{} - {}", detail(&title_of(id)), detail(&title_of(by))))
        .collect();

    let paragraph = if !present.is_empty() {
        let mut t = format!("{name} applies in this chart: {}.", list(present));
        if !cancelled.is_empty() {
            t.push_str(&format!(
                " A further count of it is not scored, being cancelled by a classical exception: {}.",
                list(cancelled)
            ));
        }
        t
    } else if !cancelled.is_empty() {
        format!(
            "{name} would apply in this chart, but every count of it is cancelled by a classical \
             exception, so it is not scored against this reading: {}.",
            list(cancelled)
        )
    } else {
        format!("{name} does not apply in this chart. No count of it is present.")
    };

    Some(Section {
        kind: SectionKind::Dosha,
        heading: name.to_string(),
        paragraphs: vec![paragraph],
        points: Vec::new(),
    })
}

fn summary(meta: &TopicMeta, report: &TopicReport, results: &[&RuleResult]) -> Vec<String> {
    let pos: i32 = results.iter().filter(|r| r.effective && r.polarity > 0).map(|r| r.polarity as i32).sum();
    let neg: i32 = results.iter().filter(|r| r.effective && r.polarity < 0).map(|r| -(r.polarity as i32)).sum();
    let topic = meta.title.to_lowercase();
    let mut out = Vec::new();
    if results.is_empty() {
        out.push(format!("No reviewed interpretations for {topic} are available yet; the chart analysis below describes the underlying houses."));
    } else {
        out.push(match (pos, neg) {
            (0, 0) => format!("No classical rule for {topic} applies strongly either way in this chart. The houses and significators described below show the underlying condition."),
            (p, 0) => format!("The indications for {topic} are clearly supportive: every scored factor that applies is favourable (total weight +{p})."),
            (0, n) => format!("The indications for {topic} call for care and effort: the factors that apply are challenging (total weight -{n}), and none offsets them. Challenges in a chart describe where effort is needed, not a fixed outcome."),
            (p, n) if p > n => format!("On balance, the indications for {topic} are supportive: the favourable factors (+{p}) outweigh those that call for care (-{n})."),
            (p, n) if p < n => format!("On balance, {topic} calls for care and effort: the challenging factors (-{n}) outweigh the favourable ones (+{p}). Challenges describe where effort is needed, not a fixed outcome."),
            (p, _) => format!("The indications for {topic} are evenly balanced: favourable and challenging factors carry equal weight (+{p} and -{p}), so both sides below are worth reading."),
        });
    }
    if !report.cancelled.is_empty() {
        out.push(format!("{} challenging indication{} cancelled by a classical exception, as explained below.", report.cancelled.len(), if report.cancelled.len() == 1 { " is" } else { "s are" }));
    }
    if let Some(d) = &meta.disclaimer {
        out.push(d.clone());
    }
    out
}

/// The write-up for a topic report. `keep` narrows the results (a family
/// relation reads only its own rules); `focus` is what the analysis explains.
pub fn write_up(
    corpus: &Corpus,
    meta: &TopicMeta,
    focus: &Focus,
    report: &TopicReport,
    f: &FactBase,
    ages: (f64, f64),
    keep: impl Fn(&RuleResult) -> bool,
) -> WriteUp {
    let results: Vec<&RuleResult> = report.results.iter().filter(|r| keep(r)).collect();
    let bhavas = corpus.bhavas.as_ref();
    let mut sections: Vec<Section> = focus.houses.iter().map(|&h| house_section(f, bhavas, h)).collect();
    sections.extend(karaka_section(f, focus));
    sections.extend(varga_section(f, focus));
    sections.extend(dosha_section(meta, report, &results));
    if focus.karmic_axis {
        sections.extend(life_carried_section(f, corpus.karma.as_ref()));
        sections.push(karmic_axis_section(f, bhavas));
        sections.push(bonds_section(f, bhavas));
    }

    let timed: std::collections::BTreeSet<&str> = report.timing.iter().map(|w| w.rule.as_str()).collect();
    let pick = |pred: &dyn Fn(&RuleResult) -> bool| -> Vec<Point> {
        results.iter().filter(|r| r.effective && pred(r)).map(|r| point(r, &corpus.rules, f)).collect()
    };
    // Debts (rina) are gathered into their own section rather than listed
    // among the ordinary difficulties.
    let is_debt = |r: &RuleResult| r.tags.iter().any(|t| t.starts_with("rina:"));
    let supporting = pick(&|r| r.polarity > 0 && !is_debt(r));
    let caring = pick(&|r| r.polarity < 0 && !is_debt(r));
    let debts = pick(&|r| is_debt(r));
    // A cancellation rule is mentioned only where it cancels something ("What
    // eases it"); on its own it would announce relief from nothing.
    let cancels: std::collections::BTreeSet<&str> =
        corpus.rules.iter().filter(|x| !x.overrides.is_empty()).map(|x| x.id.as_str()).collect();
    let noted = pick(&|r| {
        r.polarity == 0 && !timed.contains(r.id.as_str()) && !r.tags.iter().any(|t| t == "timing")
            && !cancels.contains(r.id.as_str()) && !is_debt(r)
    });
    let title_of = |id: &str| report.results.iter().find(|r| r.id == id).map(|r| r.title.clone()).unwrap_or_else(|| id.to_string());
    let eased: Vec<String> = report
        .cancelled
        .iter()
        .filter(|(id, _)| results.iter().any(|r| &r.id == id))
        .map(|(id, by)| format!("\"{}\" would apply, but it is cancelled because {}.", title_of(id), title_of(by).to_lowercase()))
        .collect();
    let unknown: Vec<String> = results
        .iter()
        .filter(|r| r.outcome == Truth::Unknown)
        .map(|r| format!("\"{}\" could not be judged: {}.", r.title, r.trace.iter().find(|t| t.value == Truth::Unknown).map(|t| t.facts.as_str()).unwrap_or("a needed fact is missing")))
        .collect();

    for (kind, heading, points, lead) in [
        (SectionKind::Supporting, "What supports", supporting, "These classical indications apply to your chart and support this area:"),
        (SectionKind::Care, "What calls for care", caring, "These classical indications apply and call for care or effort:"),
        (SectionKind::Noted, "Also noted", noted, "Descriptive indications, not scored:"),
    ] {
        if !points.is_empty() {
            sections.push(Section { kind, heading: heading.into(), paragraphs: vec![lead.into()], points });
        }
    }
    if !debts.is_empty() {
        sections.push(Section {
            kind: SectionKind::Debts,
            heading: "Debts carried forward".into(),
            paragraphs: vec![
                "The tradition calls these rina, debts, and shapa. They are accounts left open rather than faults to be answered for, and each is stated with what it asks of you - in how you conduct yourself, never in anything to be performed.".into(),
            ],
            points: debts,
        });
    }
    if !eased.is_empty() {
        sections.push(Section { kind: SectionKind::Eased, heading: "What eases it".into(), paragraphs: eased, points: Vec::new() });
    }
    if !unknown.is_empty() {
        sections.push(Section { kind: SectionKind::Unknown, heading: "Could not be judged".into(), paragraphs: unknown, points: Vec::new() });
    }

    if focus.bridge {
        sections.extend(bridge_section(corpus, f, &report.topic, report.mode));
    }

    let tz = f.chart.birth.moment.utc_offset_hours;
    let windows: Vec<&crate::timing::Window> = report.timing.iter().filter(|w| results.iter().any(|r| r.id == w.rule)).collect();
    if !windows.is_empty() {
        let date = |jd: f64| {
            let d = jd_to_civil(jd, tz);
            format!("{:04}-{:02}", d.year, d.month)
        };
        // Each timing lord with the reason it times this area, e.g. "Venus
        // (Shukra), the lord of the 10th in D-10".
        let mut lords: Vec<(Graha, String)> = Vec::new();
        for r in results.iter().filter(|r| windows.iter().any(|w| w.rule == r.id)) {
            if let Some(rule) = corpus.rules.iter().find(|x| x.id == r.id) {
                for t in &rule.timing {
                    if let Some(g) = f.resolve(*t) {
                        let why = crate::explain::graha(t);
                        // A graha can time an area twice over: once as a house
                        // lord, once as the karaka, which is itself. The second
                        // reason is its own name, and printing it beside the
                        // name already shown read "Venus (Shukra), the lord of
                        // the 7th and Shukra". A real role always wins.
                        let own = g.name();
                        match lords.iter_mut().find(|(x, _)| *x == g) {
                            Some(_) if why == own => {}
                            Some((_, w)) if w.as_str() == own => *w = why,
                            Some((_, w)) if !w.contains(&why) => { w.push_str(" and "); w.push_str(&why); }
                            Some(_) => {}
                            None => lords.push((g, why)),
                        }
                    }
                }
            }
        }
        lords.sort_by_key(|(g, _)| *g);
        let lords: Vec<String> = lords
            .into_iter()
            .map(|(g, why)| if why == g.name() { graha_name(g) } else { format!("{}, {}", graha_name(g), why) })
            .collect();
        let mut paragraphs = vec![format!(
            "Matters of this area tend to come forward in the dasha periods of {}. Between ages {} and {} these run:",
            lords.join("; "), ages.0, ages.1
        )];
        let shown: Vec<String> = windows
            .iter()
            .take(12)
            .map(|w| format!("{} to {}: {} mahadasha, {} bhukti", date(w.start_jd), date(w.end_jd), w.maha.name(), w.antar.name()))
            .collect();
        paragraphs.extend(shown);
        if windows.len() > 12 {
            // The count is of what this list leaves out, so it has to say what
            // the table holds, or it reads as the total (defect D3).
            let more = windows.len() - 12;
            paragraphs.push(format!(
                "...and {more} more {}; the table under Details lists all {}.",
                if more == 1 { "period" } else { "periods" },
                windows.len()
            ));
        }
        sections.push(Section { kind: SectionKind::Timing, heading: "Timing".into(), paragraphs, points: Vec::new() });
    } else {
        // Health and parents carry no reviewed timing rule, so those topics
        // never show a timing table at all. A section that silently vanishes
        // looks like an unfinished feature; saying which case this is costs a
        // sentence (defect D6).
        let topic_is_timed = corpus.rules.iter().any(|r| r.topic == meta.id && !r.timing.is_empty());
        let text = if topic_is_timed {
            format!(
                "No dasha period between ages {} and {} is picked out for this area by the timing rules.",
                ages.0, ages.1
            )
        } else {
            "This engine does not time this area by dasha: no classical timing rule for it has been \
             reviewed. What is written above describes the standing condition of the chart, not when \
             it comes forward."
                .to_string()
        };
        sections.push(Section { kind: SectionKind::Timing, heading: "Timing".into(), paragraphs: vec![text], points: Vec::new() });
    }

    WriteUp { summary: summary(meta, report, &results), sections }
}

/// The write-up for a topic, using the topic's own catalogue focus.
pub fn topic_write_up(corpus: &Corpus, report: &TopicReport, f: &FactBase, ages: (f64, f64)) -> Option<WriteUp> {
    let meta = corpus.topics.iter().find(|t| t.id == report.topic)?;
    Some(write_up(corpus, meta, &meta.focus, report, f, ages, |_| true))
}
