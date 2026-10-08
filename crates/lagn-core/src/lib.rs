//! Layers 2 and 3 - chart derivation and time systems.
//!
//! Pure computation over the numbers `lagn-ephem` produces. No I/O, no globals,
//! no platform dependencies, so this compiles unchanged to wasm32, iOS and
//! Android.
//!
//! Phase 1: rasi chart, whole-sign bhavas, Vimshottari dasha to three levels.
//! Phase 2A: sixteen vargas, dignity, relationships, drishti, avasthas,
//! combustion, graha yuddha. Phase 2B: Ashtakavarga.
//! Specification: `docs/phase2/DESIGN.md`.

pub mod analysis;
pub mod ashtakavarga;
pub mod chara;
pub mod chart;
pub mod condition;
pub mod day;
pub mod dasha;
pub mod dignity;
pub mod drishti;
pub mod format;
pub mod functional;
pub mod jaimini;
pub mod nakshatra;
pub mod rasi;
pub mod relationship;
pub mod transit;
pub mod upagraha;
pub mod varga;

pub use chart::{BirthData, Chart, ChartSettings, Lagna, LagnaWindow, Placement, VargaChart};
pub use day::{day_timings, panchanga, DaySegment, DayTimings, Panchanga, Vara};
pub use dasha::{DashaChain, DashaPeriod, Vimshottari, YearLength};
pub use nakshatra::{Nakshatra, NakshatraPosition};
pub use rasi::{Element, Mobility, Rasi};
pub use varga::{navamsa_sign, trimsamsa_sign, Varga};
pub use analysis::{Analysis, DerivationSettings, GrahaCondition, RelationBasis, TemporarySource};
pub use ashtakavarga::{Ashtakavarga, Contributor};
pub use condition::{BaladiAvastha, JagradadiAvastha};
pub use dignity::{Dignity, VargaMoolatrikona};
pub use drishti::NodeAspects;
pub use functional::{FunctionalNature, SambandhaKind};
pub use chara::{CharaChain, CharaDasha, CharaLength, CharaPeriod, Direction};
pub use jaimini::{
    arudha_pada, arudha_padas, argala, Argala, ArgalaKind, ArgalaPair, ArgalaVerdict, ArudhaPada,
    CharaKarakas, Jaimini, Karaka, KarakaAssignment, VariantChoice,
};
pub use transit::{Ingress, SignStay, TransitKind, TransitWindow};
pub use upagraha::{DayPart, DayPartPoint, Point, SunOffset, TimeLagna, Upagrahas};
pub use relationship::{CompoundRelation, NaturalRelation, TemporaryRelation};

// Re-exported so downstream crates need only depend on lagn-core.
pub use lagn_ephem::{
    days_in_month, is_leap_year, jd_to_calendar, jd_to_civil, julian_day_ut, validate_civil_date, validate_date, Angles, Ayanamsa,
    BirthMoment, Calendar, CalendarDateTime, Ephemeris, EphemError, Graha, HouseSystem, NodeType,
    Position, GREGORIAN_REFORM_JD, MAX_YEAR, MIN_YEAR,
};
