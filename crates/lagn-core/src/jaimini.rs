//! Jaimini core: chara karakas, arudha padas, argala.
//!
//! Specification: `docs/phase13/DESIGN.md` sections 4, 5 and 7.
//!
//! Nothing here reads a clock or an ephemeris. Every value is a function of
//! positions the chart already holds, which is why this module takes a
//! `&Chart` and returns plain data: the same chart always yields the same
//! karakas, the same twelve padas and the same argala.
//!
//! Jaimini has more live disagreement in it than Parashara does. Where a
//! choice exists it is made once, here, and named on the output by its
//! variant ID so a practitioner can see which scheme produced what they are
//! reading. The IDs are defined in section 7 of the design.

use std::borrow::Cow;

use crate::chart::Chart;
use crate::rasi::Rasi;
use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

/// The eight chara karakas, in rank order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Karaka {
    Atma,
    Amatya,
    Bhratri,
    Matri,
    Pitri,
    Putra,
    Gnati,
    Dara,
}

impl Karaka {
    /// All eight, highest advancement first.
    pub const ALL: [Karaka; 8] = [
        Karaka::Atma,
        Karaka::Amatya,
        Karaka::Bhratri,
        Karaka::Matri,
        Karaka::Pitri,
        Karaka::Putra,
        Karaka::Gnati,
        Karaka::Dara,
    ];

    /// The abbreviation practice actually uses: AK, AmK, BK and so on.
    pub fn abbrev(self) -> &'static str {
        match self {
            Karaka::Atma => "AK",
            Karaka::Amatya => "AmK",
            Karaka::Bhratri => "BK",
            Karaka::Matri => "MK",
            Karaka::Pitri => "PiK",
            Karaka::Putra => "PuK",
            Karaka::Gnati => "GK",
            Karaka::Dara => "DK",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Karaka::Atma => "Atmakaraka",
            Karaka::Amatya => "Amatyakaraka",
            Karaka::Bhratri => "Bhratrikaraka",
            Karaka::Matri => "Matrikaraka",
            Karaka::Pitri => "Pitrikaraka",
            Karaka::Putra => "Putrakaraka",
            Karaka::Gnati => "Gnatikaraka",
            Karaka::Dara => "Darakaraka",
        }
    }

    /// What the karaka stands for. Stated flatly; this module interprets
    /// nothing, and the corpus is where meaning is argued.
    pub fn signifies(self) -> &'static str {
        match self {
            Karaka::Atma => "the self",
            Karaka::Amatya => "career, the minister",
            Karaka::Bhratri => "siblings, the guru",
            Karaka::Matri => "mother",
            Karaka::Pitri => "father",
            Karaka::Putra => "children",
            Karaka::Gnati => "obstacles, illness, cousins",
            Karaka::Dara => "spouse",
        }
    }
}

/// The eight candidates, in the natural order that also breaks ties (V-13-4).
///
/// Ketu is not among them: no classical scheme lists nine karakas (V-13-3).
pub const CANDIDATES: [Graha; 8] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
];

/// How far a graha has advanced through its sign, for ranking.
///
/// Rahu is reversed (V-13-2): it moves anti-zodiacally, so the part of the
/// sign it has already travelled is the part above it, not below.
pub fn advancement(graha: Graha, degrees_in_rasi: f64) -> f64 {
    if graha == Graha::Rahu {
        30.0 - degrees_in_rasi
    } else {
        degrees_in_rasi
    }
}

/// One karaka and the graha holding it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KarakaAssignment {
    pub karaka: Karaka,
    pub graha: Graha,
    pub rasi: Rasi,
    /// The ranking value from [`advancement`], in degrees.
    pub advancement: f64,
}

/// The eight chara karakas of a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharaKarakas {
    /// Ranked, Atmakaraka first.
    pub assigned: Vec<KarakaAssignment>,
}

impl CharaKarakas {
    /// Rank the eight candidates by advancement, descending.
    ///
    /// An exact tie is possible in principle and must still be deterministic,
    /// so ties fall back to the natural graha order (V-13-4). `sort_by` is
    /// stable and `CANDIDATES` is already in that order, so comparing only
    /// the advancement gives the tie-break for free.
    pub fn compute(chart: &Chart) -> CharaKarakas {
        let mut ranked: Vec<(Graha, Rasi, f64)> = CANDIDATES
            .iter()
            .map(|&g| {
                let p = chart.placement(g);
                (g, p.rasi, advancement(g, p.degrees_in_rasi))
            })
            .collect();
        // Descending. Advancement is finite for every placement, so the
        // partial_cmp cannot be None; Equal keeps the stable order.
        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let assigned = ranked
            .into_iter()
            .zip(Karaka::ALL)
            .map(|((graha, rasi, advancement), karaka)| KarakaAssignment {
                karaka,
                graha,
                rasi,
                advancement,
            })
            .collect();
        CharaKarakas { assigned }
    }

    /// The graha holding a given karaka.
    pub fn of(&self, karaka: Karaka) -> &KarakaAssignment {
        self.assigned
            .iter()
            .find(|a| a.karaka == karaka)
            .expect("every karaka is assigned exactly once")
    }

    /// The karaka a graha holds, if it is a candidate at all.
    pub fn karaka_of(&self, graha: Graha) -> Option<Karaka> {
        self.assigned.iter().find(|a| a.graha == graha).map(|a| a.karaka)
    }
}

/// One bhava's arudha pada, with the working shown.
///
/// The intermediate fields are not decoration: an arudha is the one Jaimini
/// quantity a practitioner most often recomputes by hand, and showing `count`
/// and `raw` is what lets them check this against their own arithmetic rather
/// than take it on trust.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArudhaPada {
    /// 1 to 12. Bhava 1 is the Arudha Lagna, bhava 12 the Upapada.
    pub bhava: u8,
    pub bhava_rasi: Rasi,
    pub lord: Graha,
    pub lord_rasi: Rasi,
    /// Signs from the bhava to its lord, counting the bhava as 1.
    pub count: u8,
    /// Where the count lands before the exception is applied.
    pub raw: Rasi,
    /// The pada itself.
    pub rasi: Rasi,
    /// Whether the exception moved it: true when `raw` fell on the bhava or
    /// the 7th from it and the 10th was taken instead (V-13-8).
    pub adjusted: bool,
}

impl ArudhaPada {
    /// The name practice uses: A1 is "AL", A12 is "UL", the rest are "A7".
    pub fn label(&self) -> String {
        match self.bhava {
            1 => "AL".to_string(),
            12 => "UL".to_string(),
            n => format!("A{n}"),
        }
    }
}

/// The arudha pada of one bhava.
///
/// Count from the bhava to its lord, then the same count on from the lord.
/// If that lands on the bhava itself or opposite it, take the 10th from there
/// instead - a pada may not sit on its own bhava or its 7th.
pub fn arudha_pada(chart: &Chart, bhava: u8) -> ArudhaPada {
    assert!((1..=12).contains(&bhava), "bhava {bhava} out of range");
    let bhava_rasi = chart.rasi_of_house(bhava);
    let lord = bhava_rasi.lord();
    let lord_rasi = chart.placement(lord).rasi;

    let count = bhava_rasi.houses_to(lord_rasi);
    let raw = Rasi::from_index(lord_rasi.index() as i32 + count as i32 - 1);

    // Lands on the bhava when the count is 1 or 7, and on the 7th from it
    // when the count is 4 or 10.
    let from_bhava = bhava_rasi.houses_to(raw);
    let adjusted = from_bhava == 1 || from_bhava == 7;
    let rasi = if adjusted { Rasi::from_index(raw.index() as i32 + 9) } else { raw };

    ArudhaPada { bhava, bhava_rasi, lord, lord_rasi, count, raw, rasi, adjusted }
}

/// All twelve padas, A1 through A12.
pub fn arudha_padas(chart: &Chart) -> Vec<ArudhaPada> {
    (1..=12).map(|b| arudha_pada(chart, b)).collect()
}

/// Which of the three argala pairs an intervention belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgalaKind {
    /// 2nd, countered by the 12th.
    Wealth,
    /// 4th, countered by the 10th.
    Home,
    /// 11th, countered by the 3rd.
    Gain,
}

impl ArgalaKind {
    pub const ALL: [ArgalaKind; 3] = [ArgalaKind::Wealth, ArgalaKind::Home, ArgalaKind::Gain];

    /// (argala house, counter house), counted from the sign in question.
    pub const fn houses(self) -> (u8, u8) {
        match self {
            ArgalaKind::Wealth => (2, 12),
            ArgalaKind::Home => (4, 10),
            ArgalaKind::Gain => (11, 3),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ArgalaKind::Wealth => "wealth",
            ArgalaKind::Home => "home",
            ArgalaKind::Gain => "gain",
        }
    }
}

/// Whether an argala survives its counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgalaVerdict {
    /// More grahas intervening than countering.
    Stands,
    /// Equal numbers on both sides.
    Neutralised,
    /// More countering than intervening.
    Overcome,
    /// Nothing in the intervening sign, so there is no intervention to weigh.
    None,
}

/// One argala and its counter, weighed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArgalaPair {
    pub kind: ArgalaKind,
    pub argala_rasi: Rasi,
    pub counter_rasi: Rasi,
    pub argala_grahas: Vec<Graha>,
    pub counter_grahas: Vec<Graha>,
    pub verdict: ArgalaVerdict,
}

/// The intervention acting on one sign, across all three pairs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argala {
    pub rasi: Rasi,
    pub pairs: Vec<ArgalaPair>,
}

/// Grahas occupying a sign. Ketu counts as an occupant (V-13-5), as it does
/// everywhere else in this engine.
fn occupants(chart: &Chart, rasi: Rasi) -> Vec<Graha> {
    Graha::ALL
        .iter()
        .copied()
        .filter(|&g| chart.placement(g).rasi == rasi)
        .collect()
}

/// Argala and virodhargala for one sign.
pub fn argala(chart: &Chart, rasi: Rasi) -> Argala {
    let pairs = ArgalaKind::ALL
        .iter()
        .map(|&kind| {
            let (a, c) = kind.houses();
            // "nth from" counts the sign itself as 1, so n steps on is n - 1.
            let argala_rasi = Rasi::from_index(rasi.index() as i32 + a as i32 - 1);
            let counter_rasi = Rasi::from_index(rasi.index() as i32 + c as i32 - 1);
            let argala_grahas = occupants(chart, argala_rasi);
            let counter_grahas = occupants(chart, counter_rasi);
            let verdict = if argala_grahas.is_empty() {
                ArgalaVerdict::None
            } else if argala_grahas.len() > counter_grahas.len() {
                ArgalaVerdict::Stands
            } else if argala_grahas.len() == counter_grahas.len() {
                ArgalaVerdict::Neutralised
            } else {
                ArgalaVerdict::Overcome
            };
            ArgalaPair { kind, argala_rasi, counter_rasi, argala_grahas, counter_grahas, verdict }
        })
        .collect();
    Argala { rasi, pairs }
}

/// A variant choice, named on the output so the reader knows the scheme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantChoice {
    /// The ID from `docs/phase13/DESIGN.md` section 7.
    pub id: Cow<'static, str>,
    pub question: Cow<'static, str>,
    /// What this engine does.
    pub chosen: Cow<'static, str>,
}

/// The defaults in force. Until an astrologer signs these off they are
/// recorded on every Jaimini result rather than assumed.
pub const VARIANTS: [VariantChoice; 8] = [
    VariantChoice { id: Cow::Borrowed("V-13-1"), question: Cow::Borrowed("number of chara karakas"), chosen: Cow::Borrowed("8, Rahu included") },
    VariantChoice { id: Cow::Borrowed("V-13-2"), question: Cow::Borrowed("Rahu's advancement"), chosen: Cow::Borrowed("reversed (30 - degrees)") },
    VariantChoice { id: Cow::Borrowed("V-13-3"), question: Cow::Borrowed("Ketu as a karaka candidate"), chosen: Cow::Borrowed("no") },
    VariantChoice { id: Cow::Borrowed("V-13-4"), question: Cow::Borrowed("tie-break on equal advancement"), chosen: Cow::Borrowed("natural graha order") },
    VariantChoice { id: Cow::Borrowed("V-13-5"), question: Cow::Borrowed("Ketu counted when weighing argala"), chosen: Cow::Borrowed("yes") },
    VariantChoice { id: Cow::Borrowed("V-13-6"), question: Cow::Borrowed("lord of a dual-lorded sign"), chosen: Cow::Borrowed("sole traditional lord (Kuja, Shani)") },
    VariantChoice { id: Cow::Borrowed("V-13-7"), question: Cow::Borrowed("argala from the 5th"), chosen: Cow::Borrowed("not applied") },
    VariantChoice { id: Cow::Borrowed("V-13-8"), question: Cow::Borrowed("arudha exception"), chosen: Cow::Borrowed("10th from the computed sign") },
];

/// Everything Jaimini core computes for a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Jaimini {
    pub karakas: CharaKarakas,
    pub padas: Vec<ArudhaPada>,
    /// One entry per sign, in zodiacal order.
    pub argala: Vec<Argala>,
    pub variants: Vec<VariantChoice>,
}

impl Jaimini {
    pub fn compute(chart: &Chart) -> Jaimini {
        Jaimini {
            karakas: CharaKarakas::compute(chart),
            padas: arudha_padas(chart),
            argala: Rasi::ALL.iter().map(|&r| argala(chart, r)).collect(),
            variants: VARIANTS.to_vec(),
        }
    }
}
