//! `lagn` - validation CLI for the kernel (Phases 1 and 2).
//!
//! Exists to be diffed against JHora and swetest, not to be pretty. Every
//! number it prints should be reproducible by hand from a classical text.

mod phase2;
mod phase3;
mod phase6;
mod square;

use clap::{Parser, Subcommand, ValueEnum};
use lagn_core::{
    dasha::DashaPeriod, jd_to_civil, Ayanamsa, BirthData, BirthMoment, Calendar, Chart,
    Ashtakavarga, ChartSettings, Ephemeris, HouseSystem, NodeType, Rasi, Varga, Vimshottari,
    YearLength,
};

#[derive(Parser)]
#[command(
    name = "lagn",
    about = "Deterministic Vedic astrology kernel: charts, vargas, dashas, ashtakavarga",
    version
)]
struct Cli {
    /// Directory holding the .se1 ephemeris files.
    #[arg(long, global = true, env = "LAGN_EPHE_PATH")]
    ephe: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compute a birth chart.
    Chart(ChartArgs),
    /// Print the Vimshottari dasha tree.
    Dasha(DashaArgs),
    /// Every graha's sign in all sixteen vargas.
    Vargas(DerivedArgs),
    /// Dignity, relationships, aspects, avasthas, combustion, graha yuddha.
    Details(DerivedArgs),
    /// Bhinnashtakavarga and Sarvashtakavarga.
    Ashtakavarga(DerivedArgs),
    /// Rule corpus maintenance.
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },
    /// Evaluate a topic's rules against a chart (e.g. `topic marriage`).
    Topic(TopicArgs),
    /// Ten-porutham marriage matching between two charts.
    Match(MatchArgs),
    /// Generate the astrologer review sheet for a topic (Markdown).
    ReviewSheet(ReviewSheetArgs),
    /// Saturn, Jupiter, Rahu and Ketu transits from the natal Moon.
    Transits(AgeArgs),
    /// Period rules evaluated for every antardasha.
    Periods(AgeArgs),
    /// A day's panchanga and its Rahu kalam, Yamagandam, Kuligai and Abhijit.
    Day(DayArgs),
}

/// A day at a place. No birth chart: these timings belong to the day and the
/// place, not to a person.
#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct DayArgs {
    /// Date, YYYY-MM-DD.
    #[arg(long)]
    date: String,
    /// Latitude, degrees north-positive.
    #[arg(long)]
    lat: f64,
    /// Longitude, degrees east-positive.
    #[arg(long)]
    lon: f64,
    /// UTC offset in hours in force at that place on that date.
    #[arg(long, default_value_t = 5.5)]
    tz: f64,
    /// How many days to print, starting at --date.
    #[arg(long, default_value_t = 1)]
    days: u32,
    #[arg(long)]
    json: bool,
}

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct AgeArgs {
    /// Skip the gochara (transit) overlay. Much faster over long ranges; the
    /// reading then carries no transit lines and no transit pariharams.
    #[arg(long)]
    no_transits: bool,
    #[command(flatten)]
    birth: BirthArgs,
    #[command(flatten)]
    derivation: phase2::DerivationArgs,
    #[arg(long, default_value_t = 0.0)]
    from_age: f64,
    #[arg(long, default_value_t = 80.0)]
    to_age: f64,
    #[arg(long)]
    review: bool,
    #[arg(long, value_enum)]
    sex: Option<SexArg>,
    #[arg(long)]
    corpus: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum RulesAction {
    /// Load and validate the corpus.
    Validate {
        #[arg(long)]
        corpus: Option<String>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum SexArg { Female, Male }

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct TopicArgs {
    /// Topic name, e.g. marriage.
    name: String,
    #[command(flatten)]
    birth: BirthArgs,
    #[command(flatten)]
    derivation: phase2::DerivationArgs,
    /// Include draft rules, clearly marked. Default shows approved rules only.
    #[arg(long)]
    review: bool,
    /// The native's sex, needed by rules that the classical texts condition on it.
    #[arg(long, value_enum)]
    sex: Option<SexArg>,
    /// Start of the age range for timing windows.
    #[arg(long, default_value_t = 18.0)]
    from_age: f64,
    /// End of the age range for timing windows.
    #[arg(long, default_value_t = 45.0)]
    to_age: f64,
    #[arg(long)]
    corpus: Option<String>,
    #[arg(long)]
    json: bool,
    /// Print the written interpretation (DESIGN section 4a).
    #[arg(long)]
    writeup: bool,
}

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct MatchArgs {
    /// Bride: "YYYY-MM-DD HH:MM[:SS] LAT LON TZ".
    #[arg(long, required_unless_present = "table")]
    bride: Option<String>,
    /// Groom: "YYYY-MM-DD HH:MM[:SS] LAT LON TZ".
    #[arg(long, required_unless_present = "table")]
    groom: Option<String>,
    /// Dump the ungated procedure for all 11,664 pada pairs as JSON (for QA).
    #[arg(long)]
    table: bool,
    #[arg(long)]
    review: bool,
    #[arg(long)]
    corpus: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser)]
struct ReviewSheetArgs {
    #[arg(long, default_value = "marriage")]
    topic: String,
    /// Sample births used for firing-rate statistics.
    #[arg(long, default_value_t = 1000)]
    samples: usize,
    #[arg(long)]
    corpus: Option<String>,
}

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct DerivedArgs {
    #[command(flatten)]
    birth: BirthArgs,
    #[command(flatten)]
    derivation: phase2::DerivationArgs,
    #[arg(long)]
    json: bool,
}

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct ChartArgs {
    #[command(flatten)]
    birth: BirthArgs,
    /// Emit JSON instead of a formatted chart.
    #[arg(long)]
    json: bool,
    /// Also draw the D-9 navamsa square chart (shorthand for --varga d9).
    #[arg(long)]
    navamsa: bool,
    /// Also draw this divisional chart.
    #[arg(long, value_enum)]
    varga: Option<phase2::VargaArg>,
}

#[derive(Parser)]
#[command(allow_negative_numbers = true)]
struct DashaArgs {
    #[command(flatten)]
    birth: BirthArgs,
    /// How many levels to print: 1 maha, 2 +antar, 3 +pratyantar.
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=3))]
    levels: u8,
    /// Show only the dasha running on this date (YYYY-MM-DD).
    #[arg(long)]
    on: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Clone)]
#[command(allow_negative_numbers = true)]
struct BirthArgs {
    /// Date of birth, YYYY-MM-DD.
    #[arg(long)]
    date: String,
    /// Time of birth, HH:MM or HH:MM:SS, local wall clock.
    #[arg(long)]
    time: String,
    /// Latitude, degrees north-positive.
    #[arg(long)]
    lat: f64,
    /// Longitude, degrees east-positive.
    #[arg(long)]
    lon: f64,
    /// UTC offset in hours in force at that place on that date. India today is
    /// 5.5. For historical births use the birth record; the IANA tz database
    /// gives Madras Mean Time as +5:21:10 (5.352778) for 1870-1906.
    #[arg(long, default_value_t = 5.5)]
    tz: f64,
    /// Place name, for the printout only.
    #[arg(long, default_value = "")]
    place: String,
    #[arg(long, value_enum, default_value_t = AyanamsaArg::Lahiri)]
    ayanamsa: AyanamsaArg,
    #[arg(long, value_enum, default_value_t = NodeArg::Mean)]
    node: NodeArg,
    #[arg(long, value_enum, default_value_t = YearArg::Julian)]
    year_length: YearArg,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum AyanamsaArg { Lahiri, LahiriIcrc, TrueChitra, Raman, Krishnamurti, TrueRevati, Yukteshwar }
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum NodeArg { Mean, True }
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum YearArg { Julian, Gregorian, Tropical, SiderealSolar, Savana }

impl From<AyanamsaArg> for Ayanamsa {
    fn from(a: AyanamsaArg) -> Self {
        match a {
            AyanamsaArg::Lahiri => Ayanamsa::Lahiri,
            AyanamsaArg::LahiriIcrc => Ayanamsa::LahiriIcrc,
            AyanamsaArg::TrueChitra => Ayanamsa::TrueChitra,
            AyanamsaArg::Raman => Ayanamsa::Raman,
            AyanamsaArg::Krishnamurti => Ayanamsa::Krishnamurti,
            AyanamsaArg::TrueRevati => Ayanamsa::TrueRevati,
            AyanamsaArg::Yukteshwar => Ayanamsa::Yukteshwar,
        }
    }
}
impl From<NodeArg> for NodeType {
    fn from(n: NodeArg) -> Self {
        match n { NodeArg::Mean => NodeType::Mean, NodeArg::True => NodeType::True }
    }
}
impl From<YearArg> for YearLength {
    fn from(y: YearArg) -> Self {
        match y {
            YearArg::Julian => YearLength::Julian,
            YearArg::Gregorian => YearLength::Gregorian,
            YearArg::Tropical => YearLength::Tropical,
            YearArg::SiderealSolar => YearLength::SiderealSolar,
            YearArg::Savana => YearLength::Savana,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    Ephemeris::set_ephemeris_path(resolve_ephe(cli.ephe.as_deref())?)?;

    match cli.command {
        Command::Chart(a) => {
            let chart = build_chart(&a.birth)?;
            if a.json {
                println!("{}", serde_json::to_string_pretty(&chart)?);
            } else {
                let extra = match (a.varga, a.navamsa) {
                    (Some(v), _) => Some(Varga::from(v)),
                    (None, true) => Some(Varga::D9),
                    (None, false) => None,
                };
                print_chart(&chart, extra);
            }
        }
        Command::Vargas(a) => {
            let chart = build_chart(&a.birth)?;
            if a.json {
                println!("{}", phase2::vargas_json(&chart, &a.derivation.settings()));
            } else {
                phase2::print_vargas(&chart);
            }
        }
        Command::Details(a) => {
            let chart = build_chart(&a.birth)?;
            let analysis = chart.analyse(&a.derivation.settings());
            if a.json {
                println!("{}", phase2::details_json(&analysis));
            } else {
                phase2::print_details(&chart, &analysis);
            }
        }
        Command::Rules { action: RulesAction::Validate { corpus } } => {
            let c = phase3::load(corpus.as_deref())?;
            phase3::print_validation(&c);
        }
        Command::Topic(a) => {
            if !(a.from_age >= 0.0 && a.to_age > a.from_age && a.to_age <= 120.0) {
                return Err("age range must satisfy 0 <= from-age < to-age <= 120".into());
            }
            let corpus = phase3::load(a.corpus.as_deref())?;
            if !corpus.rules.iter().any(|r| r.topic == a.name) {
                return Err(format!("no rules for topic {:?} in the corpus", a.name).into());
            }
            let chart = build_chart(&a.birth)?;
            let native = lagn_rules::NativeInfo {
                sex: a.sex.map(|s| match s {
                    SexArg::Female => lagn_rules::Sex::Female,
                    SexArg::Male => lagn_rules::Sex::Male,
                }),
            };
            // Mutable because judging the topic's windows binds the period
            // lords per window; the borrows never overlap.
            let mut facts = lagn_rules::FactBase::new(chart, a.derivation.settings(), native);
            let mode = if a.review { lagn_rules::Mode::Review } else { lagn_rules::Mode::Production };
            let report = lagn_rules::evaluate_topic(&a.name, &corpus.rules, &facts, mode, (a.from_age, a.to_age));
            if a.writeup {
                let w = lagn_rules::writeup::topic_write_up(&corpus, &report, &facts, (a.from_age, a.to_age))
                    .ok_or("this topic has no catalogue entry, so no write-up")?;
                if a.json {
                    println!("{}", serde_json::to_string_pretty(&w)?);
                } else {
                    phase3::print_write_up(&w);
                }
            } else if a.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                // The topic's own windows, each judged by the period rules for
                // the same stretch, so the printout says whether a window
                // supports this area rather than only that it is listed.
                let title = corpus.topics.iter().find(|t| t.id == a.name).map(|t| t.title.clone())
                    .unwrap_or_else(|| a.name.clone());
                let judged = lagn_rules::reading::topic_timing(
                    &corpus, &mut facts, mode, &title, &report.timing, (a.from_age, a.to_age),
                );
                phase3::print_topic(&report, &facts, &judged);
            }
        }
        Command::Match(a) => {
            if a.table {
                println!("{}", phase3::match_table_json());
            } else {
                let corpus = phase3::load(a.corpus.as_deref())?;
                // Whole charts, not just the two stars: papasamyam, dasa
                // sandhi and Chevvai dosha need them, and the CLI should print
                // what the app shows.
                let bride = phase3::parse_person(a.bride.as_deref().unwrap())?;
                let groom = phase3::parse_person(a.groom.as_deref().unwrap())?;
                let mode = if a.review { lagn_rules::Mode::Review } else { lagn_rules::Mode::Production };
                let m = lagn_rules::report::match_report_full(&corpus, &bride, &groom, mode);
                if a.json {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                } else {
                    phase3::print_match(
                        &m,
                        bride.birth.moment.utc_offset_hours,
                        groom.birth.moment.utc_offset_hours,
                    );
                }
            }
        }
        Command::ReviewSheet(a) => {
            let corpus = phase3::load(a.corpus.as_deref())?;
            if !corpus.rules.iter().any(|r| r.topic == a.topic) {
                return Err(format!("no rules for topic {:?}", a.topic).into());
            }
            print!("{}", phase3::review_sheet(&corpus, &a.topic, a.samples.max(1)));
        }
        Command::Day(a) => {
            let d: Vec<&str> = a.date.split('-').collect();
            if d.len() != 3 {
                return Err("date must be YYYY-MM-DD".into());
            }
            let (y, m, dd): (i32, u32, u32) = (d[0].parse()?, d[1].parse()?, d[2].parse()?);
            if !(1..=366).contains(&a.days) {
                return Err("--days must be 1..=366".into());
            }
            let eph = lagn_core::Ephemeris::new(lagn_core::Ayanamsa::Lahiri, lagn_core::NodeType::Mean);
            let mut out = Vec::new();
            for i in 0..a.days {
                // Start the search just before local midnight, so the first
                // sunrise found is the one belonging to this date.
                let midnight = lagn_core::julian_day_ut(y, m, dd, 0.0, lagn_core::Calendar::Gregorian)?
                    - a.tz / 24.0 + i as f64;
                let sun = eph.sun_day(midnight, a.lat, a.lon)?;
                let t = lagn_core::day_timings(sun.sunrise_jd, sun.sunset_jd);
                let s = eph.position(sun.sunrise_jd, lagn_core::Graha::Sun)?;
                let mo = eph.position(sun.sunrise_jd, lagn_core::Graha::Moon)?;
                let p = lagn_core::panchanga(s.longitude, mo.longitude, sun.sunrise_jd);
                out.push((t, p));
            }
            if a.json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                phase3::print_days(&out, a.tz);
            }
        }
        Command::Transits(a) => {
            if !(a.from_age >= 0.0 && a.to_age > a.from_age && a.to_age <= 120.0) {
                return Err("age range must satisfy 0 <= from-age < to-age <= 120".into());
            }
            let chart = build_chart(&a.birth)?;
            let r = phase6::transits(&chart, a.from_age, a.to_age)?;
            if a.json { println!("{}", serde_json::to_string_pretty(&r)?); } else { phase6::print_transits(&chart, &r); }
        }
        Command::Periods(a) => {
            if !(a.from_age >= 0.0 && a.to_age > a.from_age && a.to_age <= 120.0) {
                return Err("age range must satisfy 0 <= from-age < to-age <= 120".into());
            }
            let corpus = phase3::load(a.corpus.as_deref())?;
            let chart = build_chart(&a.birth)?;
            let native = lagn_rules::NativeInfo {
                sex: a.sex.map(|s| match s { SexArg::Female => lagn_rules::Sex::Female, SexArg::Male => lagn_rules::Sex::Male }),
            };
            let mut facts = lagn_rules::FactBase::new(chart.clone(), a.derivation.settings(), native);
            let mode = if a.review { lagn_rules::Mode::Review } else { lagn_rules::Mode::Production };
            let windows = if a.no_transits {
                Vec::new()
            } else {
                phase6::transits(&chart, a.from_age, a.to_age)?.windows
            };
            let r = lagn_rules::reading::sensitive_periods(&corpus, &mut facts, mode, (a.from_age, a.to_age), &windows);
            if a.json { println!("{}", serde_json::to_string_pretty(&r)?); } else { phase6::print_periods(&chart, &r); }
        }
        Command::Ashtakavarga(a) => {
            let chart = build_chart(&a.birth)?;
            let av = Ashtakavarga::compute(&chart);
            if a.json {
                println!("{}", serde_json::to_string_pretty(&av)?);
            } else {
                phase2::print_ashtakavarga(&av);
            }
        }
        Command::Dasha(a) => {
            let chart = build_chart(&a.birth)?;
            let v = Vimshottari::compute(&chart);
            if a.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else if let Some(on) = a.on {
                print_dasha_on(&chart, &v, &on)?;
            } else {
                print_dasha(&chart, &v, a.levels);
            }
        }
    }
    Ok(())
}

/// Locate the ephemeris directory: explicit flag, then CWD, then the repo root
/// baked in at compile time (which is what makes `cargo run` just work).
fn resolve_ephe(explicit: Option<&str>) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let candidates: Vec<std::path::PathBuf> = match explicit {
        Some(p) => vec![p.into()],
        None => vec![
            "ephe".into(),
            "./ephe".into(),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ephe"),
        ],
    };
    for c in &candidates {
        if c.is_dir() {
            return Ok(c.canonicalize()?);
        }
    }
    Err(format!(
        "no ephemeris directory found (tried {}); pass --ephe or set LAGN_EPHE_PATH",
        candidates.iter().map(|c| c.display().to_string()).collect::<Vec<_>>().join(", ")
    )
    .into())
}

fn build_chart(a: &BirthArgs) -> Result<Chart, Box<dyn std::error::Error>> {
    let (year, month, day) = parse_date(&a.date)?;
    let (hour, minute, second) = parse_time(&a.time)?;
    let birth = BirthData {
        moment: BirthMoment { year, month, day, hour, minute, second, utc_offset_hours: a.tz },
        latitude: a.lat,
        longitude: a.lon,
        place_name: a.place.clone(),
    };
    let settings = ChartSettings {
        ayanamsa: a.ayanamsa.into(),
        node_type: a.node.into(),
        house_system: HouseSystem::WholeSign,
        year_length: a.year_length.into(),
    };
    Ok(Chart::compute(birth, settings)?)
}

fn parse_date(s: &str) -> Result<(i32, u32, u32), Box<dyn std::error::Error>> {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 {
        return Err(format!("date must be YYYY-MM-DD, got {s:?}").into());
    }
    Ok((p[0].parse()?, p[1].parse()?, p[2].parse()?))
}

fn parse_time(s: &str) -> Result<(u32, u32, f64), Box<dyn std::error::Error>> {
    let p: Vec<&str> = s.split(':').collect();
    if p.len() < 2 || p.len() > 3 {
        return Err(format!("time must be HH:MM or HH:MM:SS, got {s:?}").into());
    }
    Ok((
        p[0].parse()?,
        p[1].parse()?,
        if p.len() == 3 { p[2].parse()? } else { 0.0 },
    ))
}

fn dms(deg: f64) -> String {
    lagn_core::format::dms(deg)
}

fn print_chart(c: &Chart, extra: Option<Varga>) {
    let b = &c.birth;
    println!("\n{}", "=".repeat(72));
    println!(
        "  {}  {:04}-{:02}-{:02}  {:02}:{:02}:{:04.1}  UTC{:+}",
        if b.place_name.is_empty() { "Chart" } else { &b.place_name },
        b.moment.year, b.moment.month, b.moment.day,
        b.moment.hour, b.moment.minute, b.moment.second, b.moment.utc_offset_hours
    );
    println!(
        "  {:.4}°{}  {:.4}°{}   JD(UT) {:.6}",
        b.latitude.abs(), if b.latitude >= 0.0 { "N" } else { "S" },
        b.longitude.abs(), if b.longitude >= 0.0 { "E" } else { "W" },
        c.jd_ut
    );
    println!(
        "  Ayanamsa {:?} = {}   Node {:?}   Houses {:?}",
        c.settings.ayanamsa, dms(c.ayanamsa_value), c.settings.node_type, c.settings.house_system
    );
    println!("{}", "=".repeat(72));

    let jn = c.janma_nakshatra();
    println!(
        "\n  Lagna        {:<11} {}   {} pada {}",
        c.lagna.rasi.name(), dms(c.lagna.degrees_in_rasi),
        c.lagna.nakshatra.nakshatra.name(), c.lagna.nakshatra.pada
    );
    println!(
        "  Janma rasi   {:<11} ({})",
        c.janma_rasi().name(), c.janma_rasi().tamil_name()
    );
    println!(
        "  Janma nak.   {:<11} ({}) pada {}",
        jn.nakshatra.name(), jn.nakshatra.tamil_name(), jn.pada
    );

    println!("\n  {:<9} {:<11} {:>12}  {:<3} {:<18} {:<4} {:<11}",
        "Graha", "Rasi", "Degree", "Bh", "Nakshatra", "Pada", "Navamsa");
    println!("  {}", "-".repeat(70));
    for p in &c.placements {
        println!(
            "  {:<9} {:<11} {:>12}  {:<3} {:<18} {:<4} {:<11}{}",
            p.graha.name(),
            p.rasi.name(),
            dms(p.degrees_in_rasi),
            p.house,
            p.nakshatra.nakshatra.name(),
            p.nakshatra.pada,
            p.navamsa.name(),
            if p.retrograde { "  R" } else { "" }
        );
    }

    println!("\n  Rasi chart (D-1), South Indian\n");
    print!("{}", square_for(c, Varga::D1));

    if let Some(v) = extra.filter(|v| *v != Varga::D1) {
        println!("\n  {} chart, South Indian\n", v.label());
        print!("{}", square_for(c, v));
    }
    println!();
}

fn square_for(c: &Chart, varga: Varga) -> String {
    let v = c.varga(varga);
    let occupants: Vec<(Rasi, Vec<String>)> = Rasi::ALL
        .into_iter()
        .map(|r| {
            let labels = c
                .placements
                .iter()
                .filter(|p| v.sign_of(p.graha) == r)
                .map(|p| {
                    // Only the rasi chart carries meaningful degrees.
                    if varga == Varga::D1 {
                        square::graha_label(p.graha, p.degrees_in_rasi, p.retrograde)
                    } else {
                        p.graha.abbrev().to_string()
                    }
                })
                .collect();
            (r, labels)
        })
        .collect();

    let jn = c.janma_nakshatra();
    let caption = vec![
        varga.label(),
        String::new(),
        format!("Lagna {}", v.lagna.name()),
        format!("{} p{}", jn.nakshatra.name(), jn.pada),
    ];
    square::render(v.lagna, &occupants, &caption)
}

fn fmt_jd(jd: f64, tz: f64) -> String {
    // Print in the same calendar the birth date was read in.
    let d = jd_to_civil(jd, tz);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

fn print_dasha(c: &Chart, v: &Vimshottari, levels: u8) {
    let tz = c.birth.moment.utc_offset_hours;
    println!("\n  Vimshottari dasha  -  year length {:?} ({} days)",
        v.year_length, v.year_length.days());
    println!("  Janma nakshatra {} ({}), lord {}",
        v.janma_nakshatra.name(), v.janma_nakshatra.tamil_name(), v.birth_lord.name());

    let years = v.balance_years.floor();
    let rem_months = (v.balance_years - years) * 12.0;
    println!(
        "  Balance at birth: {} {:.0}y {:.0}m {:.0}d",
        v.birth_lord.name(),
        years, rem_months.floor(), (rem_months - rem_months.floor()) * 30.0
    );
    if let Some(chain) = v.at_birth() {
        println!("  Running at birth: {chain}");
    }
    println!();

    for maha in v.mahadashas_from_birth() {
        print_period(maha, tz, levels, 0, v.birth_jd);
    }
    println!();
}

fn print_period(p: &DashaPeriod, tz: f64, levels: u8, depth: usize, birth_jd: f64) {
    if p.level > levels {
        return;
    }
    // Suppress sub-periods that ended before the native was born.
    if p.end_jd <= birth_jd {
        return;
    }
    let indent = "  ".repeat(depth + 1);
    let start = p.start_jd.max(birth_jd);
    let marker = if p.start_jd < birth_jd { "*" } else { " " };
    println!(
        "{indent}{marker}{:<9} {}  ->  {}",
        p.lord.name(),
        fmt_jd(start, tz),
        fmt_jd(p.end_jd, tz)
    );
    for child in &p.children {
        print_period(child, tz, levels, depth + 1, birth_jd);
    }
}

fn print_dasha_on(
    c: &Chart,
    v: &Vimshottari,
    on: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (y, m, d) = parse_date(on)?;
    let jd = lagn_core::julian_day_ut(y, m, d, 12.0, Calendar::for_date(y, m, d))?
        - c.birth.moment.utc_offset_hours / 24.0;
    match v.at(jd) {
        Some(chain) => println!("  {on}:  {chain}"),
        None => println!("  {on}: outside the computed 120-year tree"),
    }
    Ok(())
}
