//! Phase 2 output: vargas, details (dignity, relationships, aspects,
//! conditions) and Ashtakavarga.

use clap::{Parser, ValueEnum};
use lagn_core::{
    Analysis, Ashtakavarga, Chart, DerivationSettings, Dignity, Graha, NodeAspects, Rasi,
    RelationBasis, TemporarySource, Varga, VargaMoolatrikona,
};
use serde::Serialize;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum VargaArg { D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60 }

impl From<VargaArg> for Varga {
    fn from(v: VargaArg) -> Varga {
        match v {
            VargaArg::D1 => Varga::D1, VargaArg::D2 => Varga::D2, VargaArg::D3 => Varga::D3,
            VargaArg::D4 => Varga::D4, VargaArg::D7 => Varga::D7, VargaArg::D9 => Varga::D9,
            VargaArg::D10 => Varga::D10, VargaArg::D12 => Varga::D12, VargaArg::D16 => Varga::D16,
            VargaArg::D20 => Varga::D20, VargaArg::D24 => Varga::D24, VargaArg::D27 => Varga::D27,
            VargaArg::D30 => Varga::D30, VargaArg::D40 => Varga::D40, VargaArg::D45 => Varga::D45,
            VargaArg::D60 => Varga::D60,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum NodeAspectArg { None, Seventh, FiveSevenNine }
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum VargaMtArg { AsMoolatrikona, AsOwnSign }
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum TemporaryArg { Rasi, SameVarga }
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum BasisArg { Compound, Natural }

/// The switchable variants from DESIGN.md section 8.
#[derive(Parser, Clone)]
pub struct DerivationArgs {
    /// V-3: graha drishti cast by Rahu and Ketu.
    #[arg(long, value_enum, default_value_t = NodeAspectArg::None)]
    pub node_aspects: NodeAspectArg,
    /// V-4: moolatrikona in divisional charts.
    #[arg(long, value_enum, default_value_t = VargaMtArg::AsMoolatrikona)]
    pub varga_mt: VargaMtArg,
    /// V-5: chart supplying temporary relationships for varga dignity.
    #[arg(long, value_enum, default_value_t = TemporaryArg::Rasi)]
    pub temporary: TemporaryArg,
    /// V-6: relationship Jagradadi reads.
    #[arg(long, value_enum, default_value_t = BasisArg::Compound)]
    pub jagradadi: BasisArg,
}

impl DerivationArgs {
    pub fn settings(&self) -> DerivationSettings {
        DerivationSettings {
            node_aspects: match self.node_aspects {
                NodeAspectArg::None => NodeAspects::None,
                NodeAspectArg::Seventh => NodeAspects::Seventh,
                NodeAspectArg::FiveSevenNine => NodeAspects::FiveSevenNine,
            },
            varga_moolatrikona: match self.varga_mt {
                VargaMtArg::AsMoolatrikona => VargaMoolatrikona::AsMoolatrikona,
                VargaMtArg::AsOwnSign => VargaMoolatrikona::AsOwnSign,
            },
            temporary_source: match self.temporary {
                TemporaryArg::Rasi => TemporarySource::Rasi,
                TemporaryArg::SameVarga => TemporarySource::SameVarga,
            },
            jagradadi_basis: match self.jagradadi {
                BasisArg::Compound => RelationBasis::Compound,
                BasisArg::Natural => RelationBasis::Natural,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// vargas
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct VargaCell {
    sign: Rasi,
    house: u8,
    dignity: Option<Dignity>,
}

#[derive(Serialize)]
struct VargaBlock {
    varga: Varga,
    lagna: Rasi,
    grahas: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct VargasJson {
    settings: DerivationSettings,
    vargas: Vec<VargaBlock>,
}

fn key(g: Graha) -> String {
    serde_json::to_value(g).unwrap().as_str().unwrap().to_string()
}

pub fn vargas_json(c: &Chart, s: &DerivationSettings) -> String {
    let vargas = Varga::ALL
        .into_iter()
        .map(|v| {
            let vc = c.varga(v);
            let mut grahas = serde_json::Map::new();
            for g in Graha::ALL {
                let cell = VargaCell {
                    sign: vc.sign_of(g),
                    house: vc.house_of(g),
                    dignity: c.dignity(g, v, s),
                };
                grahas.insert(key(g), serde_json::to_value(cell).unwrap());
            }
            VargaBlock { varga: v, lagna: vc.lagna, grahas }
        })
        .collect();
    serde_json::to_string_pretty(&VargasJson { settings: *s, vargas }).unwrap()
}

fn short(r: Rasi) -> &'static str {
    &r.name()[..3]
}

pub fn print_vargas(c: &Chart) {
    print!("\n  {:<6}", "");
    for v in Varga::ALL {
        print!("{:>5}", format!("D{}", v.number()));
    }
    println!();
    print!("  {:<6}", "Asc");
    for v in Varga::ALL {
        print!("{:>5}", short(v.sign_of(c.lagna.longitude)));
    }
    println!();
    for g in Graha::ALL {
        print!("  {:<6}", g.abbrev());
        for v in Varga::ALL {
            print!("{:>5}", short(v.sign_of(c.placement(g).longitude)));
        }
        println!();
    }
    println!();
}

// ---------------------------------------------------------------------------
// details
// ---------------------------------------------------------------------------

pub fn details_json(a: &Analysis) -> String {
    serde_json::to_string_pretty(a).unwrap()
}

fn fmt_dignity(d: Option<Dignity>) -> &'static str {
    match d {
        None => "-",
        Some(Dignity::Exalted) => "Exalted",
        Some(Dignity::Moolatrikona) => "Moolatrikona",
        Some(Dignity::OwnSign) => "Own sign",
        Some(Dignity::GreatFriend) => "Great friend",
        Some(Dignity::Friend) => "Friend",
        Some(Dignity::Neutral) => "Neutral",
        Some(Dignity::Enemy) => "Enemy",
        Some(Dignity::GreatEnemy) => "Great enemy",
        Some(Dignity::Debilitated) => "Debilitated",
    }
}

pub fn print_details(c: &Chart, a: &Analysis) {
    println!(
        "\n  {:<8} {:<10} {:<13} {:<8} {:<9} {:<7} {:<16} War",
        "Graha", "Rasi", "Dignity", "Baladi", "Jagradadi", "Asta", "Aspects"
    );
    println!("  {}", "-".repeat(86));
    for k in &a.conditions {
        let aspects: Vec<&str> = k.aspects.iter().map(|r| short(*r)).collect();
        let war: Vec<&str> = k.at_war_with.iter().map(|g| g.abbrev()).collect();
        println!(
            "  {:<8} {:<10} {:<13} {:<8} {:<9} {:<7} {:<16} {}",
            k.graha.name(),
            c.placement(k.graha).rasi.name(),
            fmt_dignity(k.dignity),
            format!("{:?}", k.baladi),
            k.jagradadi.map(|j| format!("{j:?}")).unwrap_or_else(|| "-".into()),
            if k.combust { "combust" } else { "" },
            aspects.join(" "),
            war.join(" "),
        );
    }
    println!("\n  Compound relationships (row regards column)\n");
    print!("  {:<4}", "");
    for g in lagn_core::relationship::SEVEN {
        print!("{:>5}", g.abbrev());
    }
    println!();
    for of in lagn_core::relationship::SEVEN {
        print!("  {:<4}", of.abbrev());
        for toward in lagn_core::relationship::SEVEN {
            let cell = a
                .relations
                .iter()
                .find(|r| r.of == of && r.toward == toward)
                .map(|r| match r.compound {
                    lagn_core::CompoundRelation::GreatFriend => "GF",
                    lagn_core::CompoundRelation::Friend => "F",
                    lagn_core::CompoundRelation::Neutral => "N",
                    lagn_core::CompoundRelation::Enemy => "E",
                    lagn_core::CompoundRelation::GreatEnemy => "GE",
                })
                .unwrap_or("-");
            print!("{:>5}", cell);
        }
        println!();
    }
    println!("\n  Variants: {:?}\n", a.settings);
}

// ---------------------------------------------------------------------------
// ashtakavarga
// ---------------------------------------------------------------------------

pub fn print_ashtakavarga(av: &Ashtakavarga) {
    print!("\n  {:<6}", "");
    for r in Rasi::ALL {
        print!("{:>5}", short(r));
    }
    println!("{:>7}", "Total");
    for (i, g) in lagn_core::relationship::SEVEN.iter().enumerate() {
        print!("  {:<6}", g.abbrev());
        for s in 0..12 {
            print!("{:>5}", av.bav[i][s]);
        }
        println!("{:>7}", av.bav[i].iter().map(|&x| x as u32).sum::<u32>());
    }
    print!("  {:<6}", "SAV");
    for s in 0..12 {
        print!("{:>5}", av.sav[s]);
    }
    println!("{:>7}", av.sav.iter().map(|&x| x as u32).sum::<u32>());
    print!("\n  {:<6}", "House");
    for h in 1..=12u8 {
        print!("{:>5}", h);
    }
    println!();
    print!("  {:<6}", "SAV");
    for h in 1..=12u8 {
        print!("{:>5}", av.sav_in_house(h));
    }
    println!("\n");
}

/// Jaimini core: the eight chara karakas, the twelve arudha padas, and the
/// argala acting on each sign. Specification: `docs/phase13/DESIGN.md`.
pub fn print_jaimini(j: &lagn_core::jaimini::Jaimini) {
    println!("\n  Chara karakas");
    println!("  {:<5} {:<12} {:<10} {:<12} Advanced", "", "Karaka", "Graha", "Rasi");
    for a in &j.karakas.assigned {
        println!(
            "  {:<5} {:<12} {:<10} {:<12} {:>7.3}",
            a.karaka.abbrev(), a.karaka.name(), a.graha.name(), a.rasi.name(), a.advancement,
        );
    }

    println!("\n  Arudha padas");
    println!("  {:<5} {:<12} {:<9} {:<12} {:<6} Pada rasi", "Pada", "Bhava", "Lord", "Lord in", "Count");
    for p in &j.padas {
        println!(
            "  {:<5} {:<12} {:<9} {:<12} {:<6} {}{}",
            p.label(), p.bhava_rasi.name(), p.lord.name(), p.lord_rasi.name(),
            p.count, p.rasi.name(),
            if p.adjusted { "  (10th taken)" } else { "" },
        );
    }

    println!("\n  Argala");
    println!("  {:<12} {:<22} {:<22} gain (11/3)", "Sign", "wealth (2/12)", "home (4/10)");
    for a in &j.argala {
        print!("  {:<12}", a.rasi.name());
        for p in &a.pairs {
            let cell = format!("{} {}v{}", verdict(p.verdict), p.argala_grahas.len(), p.counter_grahas.len());
            print!(" {cell:<21}");
        }
        println!();
    }

    println!("\n  Variants in force (unsigned-off; see docs/phase13/DESIGN.md section 7)");
    for v in &j.variants {
        println!("  {:<8} {:<40} {}", v.id, v.question, v.chosen);
    }
    println!();
}

fn verdict(v: lagn_core::jaimini::ArgalaVerdict) -> &'static str {
    use lagn_core::jaimini::ArgalaVerdict as V;
    match v {
        V::Stands => "stands     ",
        V::Neutralised => "neutralised",
        V::Overcome => "overcome   ",
        V::None => "-          ",
    }
}

/// Chara dasha: the Jaimini rasi dasha.
/// Specification: `docs/phase13/CHARA-DASHA.md`.
pub fn print_chara(c: &Chart, d: &lagn_core::chara::CharaDasha, levels: u8) {
    use lagn_core::chara::Direction;
    println!(
        "\n  Lagna {} ({}), sequence runs {}",
        d.lagna.name(),
        if d.lagna.is_odd() { "odd" } else { "even" },
        if d.direction == Direction::Direct { "zodiacally" } else { "anti-zodiacally" },
    );
    println!("  Cycle of {:.0} years\n", d.cycle_years());

    println!("  {:<12} {:<9} {:<12} {:<7} Years", "Rasi", "Lord", "Lord in", "Count");
    for l in &d.lengths {
        println!(
            "  {:<12} {:<9} {:<12} {:<7} {:>5.0}{}",
            l.rasi.name(), l.lord.name(), l.lord_rasi.name(), l.count, l.years,
            if l.lord_at_home { "  (lord at home)" } else { "" },
        );
    }

    println!("\n  Periods");
    for p in &d.periods {
        let (from, to) = (civil(c, p.start_jd), civil(c, p.end_jd));
        println!("  {:<12} {}  to  {}   cycle {}", p.rasi.name(), from, to, p.cycle);
        if levels > 1 {
            for ch in &p.children {
                println!("      {:<10} {}  to  {}", ch.rasi.name(), civil(c, ch.start_jd), civil(c, ch.end_jd));
            }
        }
    }

    println!("\n  Variants in force (unsigned-off; see docs/phase13/CHARA-DASHA.md section 6)");
    for v in &d.variants {
        println!("  {:<9} {:<46} {}", v.id, v.question, v.chosen);
    }
    println!();
}

/// A date in the birth place's own wall clock, which is the only frame a
/// reader of this printout has.
///
/// `jd_to_civil` takes the offset itself and picks the calendar the date
/// belongs to, so a pre-1582 period is printed in the calendar its birth was
/// read in rather than shifted nine days by a hard-coded Gregorian.
fn civil(c: &Chart, jd: f64) -> String {
    let d = lagn_core::jd_to_civil(jd, c.birth.moment.utc_offset_hours);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}
