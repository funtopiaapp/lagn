//! Phase 3: the rule engine and the porutham procedure.
//!
//! Specification: `docs/phase3/DESIGN.md`.
//!
//! The engine proves a rule does what it says. Whether what it says is true
//! is decided by astrologer review, enforced by the gate in [`resolve`]: in
//! production mode nothing unapproved is ever evaluated or shown.

pub mod catalogue;
pub mod corpus;
pub mod eval;
pub mod explain;
pub mod facts;
pub mod family;
pub mod model;
pub mod porutham;
pub mod reading;
pub mod report;
pub mod resolve;
pub mod timing;
pub mod writeup;

pub use corpus::{load, validate, Corpus, CorpusError};
pub use eval::{eval, justify, TraceEntry, Truth};
pub use facts::{FactBase, NativeInfo};
pub use model::{Condition, GrahaRef, GrahaSet, PeriodLevel, ReviewStatus, Rule, Scope, Sex};
pub use porutham::{match_stars, PoruthamKind, PoruthamResult, StarPos, Verdict};
pub use resolve::{evaluate_periods, evaluate_topic, Label, Mode, PeriodReport, PeriodWindow, RuleResult, TopicReport};
pub use timing::Window;
