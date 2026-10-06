//! Corpus loading and validation.
//!
//! A corpus that fails validation is never evaluated. Every check here has a
//! negative test in `tests/corpus.rs`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{Condition, GrahaRef, GrahaSet, Review, ReviewStatus, Rule, Scope};
use crate::porutham::PoruthamKind;

#[derive(Debug, thiserror::Error)]
pub enum CorpusError {
    #[error("cannot read {path}: {source}")]
    Io { path: String, source: std::io::Error },
    #[error("{path}: {source}")]
    Parse { path: String, source: serde_json::Error },
    #[error("corpus is invalid:\n  {}", .0.join("\n  "))]
    Invalid(Vec<String>),
}

/// Review status for each porutham (`porutham.review.json`).
pub type PoruthamReviews = BTreeMap<PoruthamKind, Review>;

#[derive(Debug, Clone, Default)]
pub struct Corpus {
    pub rules: Vec<Rule>,
    /// What Ketu's house and sign say about the life carried forward.
    pub karma: Option<crate::catalogue::Karma>,
    pub porutham: Option<PoruthamReviews>,
    /// What each porutham measures, and what its verdict is read to mean.
    pub porutham_explain: Option<crate::catalogue::PoruthamExplainer>,
    /// The matters a reader can ask about, each mapped to a topic.
    pub questions: Option<crate::catalogue::Questions>,
    pub topics: Vec<crate::catalogue::TopicMeta>,
    pub bhavas: Option<crate::catalogue::Bhavas>,
    pub pariharams: Vec<crate::catalogue::Pariharam>,
}

/// Load every `*.json` under `dir`. `porutham.review.json` holds porutham
/// review statuses; every other file is an array of rules. Files are read in
/// sorted order so loading is deterministic.
pub fn load(dir: &Path) -> Result<Corpus, CorpusError> {
    let mut files = Vec::new();
    collect(dir, &mut files)?;
    files.sort();

    let mut corpus = Corpus::default();
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CorpusError::Io { path: path.display().to_string(), source: e })?;
        let parse_err = |e| CorpusError::Parse { path: path.display().to_string(), source: e };
        match path.file_name().and_then(|n| n.to_str()) {
            Some("porutham.review.json") => {
                corpus.porutham = Some(serde_json::from_str(&text).map_err(parse_err)?);
                continue;
            }
            Some("porutham.explain.json") => {
                corpus.porutham_explain = Some(serde_json::from_str(&text).map_err(parse_err)?);
                continue;
            }
            Some("questions.json") => {
                corpus.questions = Some(serde_json::from_str(&text).map_err(parse_err)?);
                continue;
            }
            Some("topics.json") => {
                corpus.topics = serde_json::from_str(&text).map_err(parse_err)?;
                continue;
            }
            Some("karma.json") => {
                corpus.karma = Some(serde_json::from_str(&text).map_err(parse_err)?);
                continue;
            }
            Some("bhava.json") => {
                corpus.bhavas = Some(serde_json::from_str(&text).map_err(parse_err)?);
                continue;
            }
            Some("pariharam.json") => {
                corpus.pariharams = serde_json::from_str(&text).map_err(parse_err)?;
                continue;
            }
            _ => {}
        }
        {
            let mut rules: Vec<Rule> = serde_json::from_str(&text)
                .map_err(|e| CorpusError::Parse { path: path.display().to_string(), source: e })?;
            corpus.rules.append(&mut rules);
        }
    }
    let problems = validate(&corpus);
    if problems.is_empty() {
        Ok(corpus)
    } else {
        Err(CorpusError::Invalid(problems))
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), CorpusError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| CorpusError::Io { path: dir.display().to_string(), source: e })?;
    for e in entries {
        let p = e.map_err(|e| CorpusError::Io { path: dir.display().to_string(), source: e })?.path();
        if p.is_dir() {
            collect(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    Ok(())
}

fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && s.chars().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && s[5..7].parse::<u8>().is_ok_and(|m| (1..=12).contains(&m))
        && s[8..10].parse::<u8>().is_ok_and(|d| (1..=31).contains(&d))
}

fn check_review(where_: &str, r: &Review, problems: &mut Vec<String>) {
    if r.status == ReviewStatus::Approved {
        if r.reviewer.as_deref().is_none_or(|s| s.trim().is_empty()) {
            problems.push(format!("{where_}: approved without a reviewer"));
        }
        match r.date.as_deref() {
            None => problems.push(format!("{where_}: approved without a date")),
            Some(d) if !valid_date(d) => problems.push(format!("{where_}: review date {d:?} is not YYYY-MM-DD")),
            _ => {}
        }
    }
}

fn check_ref(id: &str, r: &GrahaRef, problems: &mut Vec<String>) {
    if let GrahaRef::LordOf { house, .. } = r {
        if !(1..=12).contains(house) {
            problems.push(format!("{id}: lord_of house {house} is not 1-12"));
        }
    }
}

fn check_houses(id: &str, what: &str, hs: &[u8], problems: &mut Vec<String>) {
    if hs.is_empty() {
        problems.push(format!("{id}: {what} has an empty house list"));
    }
    for &h in hs {
        if !(1..=12).contains(&h) {
            problems.push(format!("{id}: {what} house {h} is not 1-12"));
        }
    }
    let unique: HashSet<_> = hs.iter().collect();
    if unique.len() != hs.len() {
        problems.push(format!("{id}: {what} lists a house twice"));
    }
}

fn check_set(id: &str, s: &GrahaSet, min: u8, problems: &mut Vec<String>) {
    if let GrahaSet::Explicit(e) = s {
        if e.grahas.is_empty() {
            problems.push(format!("{id}: empty graha set"));
        }
        for r in &e.grahas {
            check_ref(id, r, problems);
        }
    }
    if !(1..=9).contains(&min) {
        problems.push(format!("{id}: min {min} is not 1-9"));
    }
}

fn check_condition(id: &str, c: &Condition, problems: &mut Vec<String>) {
    c.walk(&mut |node| match node {
        Condition::All(v) | Condition::Any(v) if v.is_empty() => {
            problems.push(format!("{id}: empty all/any"));
        }
        Condition::GrahaInHouse { graha, houses, .. } => {
            check_ref(id, graha, problems);
            check_houses(id, "graha_in_house", houses, problems);
        }
        Condition::GrahaInSign { graha, signs, .. } => {
            check_ref(id, graha, problems);
            if signs.is_empty() {
                problems.push(format!("{id}: graha_in_sign has an empty sign list"));
            }
        }
        Condition::GrahaFrom { graha, from, houses } => {
            check_ref(id, graha, problems);
            if let crate::model::FromRef::Graha(r) = from {
                check_ref(id, r, problems);
            }
            check_houses(id, "graha_from", houses, problems);
        }
        Condition::Dignity { graha, any_of, .. } => {
            check_ref(id, graha, problems);
            if any_of.is_empty() {
                problems.push(format!("{id}: dignity has an empty list"));
            }
        }
        Condition::HouseOccupied { house, by, min, .. } | Condition::HouseAspected { house, by, min } => {
            check_houses(id, "house test", &[*house], problems);
            check_set(id, by, *min, problems);
        }
        Condition::Conjunct { a, b } => {
            check_ref(id, a, problems);
            check_ref(id, b, problems);
        }
        Condition::AspectsGraha { from, to } => {
            check_ref(id, from, problems);
            check_ref(id, to, problems);
        }
        Condition::Combust { graha } | Condition::Retrograde { graha } | Condition::NeechaBhanga { graha } => {
            check_ref(id, graha, problems)
        }
        Condition::Functional { graha, is } => {
            check_ref(id, graha, problems);
            if is.is_empty() {
                problems.push(format!("{id}: functional has an empty list"));
            }
        }
        Condition::RulesHouse { graha, houses } => {
            check_ref(id, graha, problems);
            check_houses(id, "rules_house", houses, problems);
        }
        Condition::Sambandha { a, b, kinds } => {
            check_ref(id, a, problems);
            check_ref(id, b, problems);
            let unique: HashSet<_> = kinds.iter().collect();
            if unique.len() != kinds.len() {
                problems.push(format!("{id}: sambandha lists a kind twice"));
            }
        }
        Condition::Sav { house, min, max } => {
            check_houses(id, "sav", &[*house], problems);
            if min.is_none() && max.is_none() {
                problems.push(format!("{id}: sav needs min or max"));
            }
            // A house's SAV is at most 7 grahas x 8 contributors.
            for v in [min, max].into_iter().flatten() {
                if *v > 56 {
                    problems.push(format!("{id}: sav bound {v} exceeds 56"));
                }
            }
            if let (Some(a), Some(b)) = (min, max) {
                if a > b {
                    problems.push(format!("{id}: sav min {a} > max {b}"));
                }
            }
        }
        _ => {}
    });
}

/// All validation problems, or an empty list.
pub fn validate(corpus: &Corpus) -> Vec<String> {
    let mut problems = Vec::new();
    let mut by_id: HashMap<&str, &Rule> = HashMap::new();

    for r in &corpus.rules {
        if by_id.insert(r.id.as_str(), r).is_some() {
            problems.push(format!("duplicate rule id {}", r.id));
        }
        if !r.id.starts_with(&format!("{}.", r.topic)) {
            problems.push(format!("{}: id must start with its topic '{}.'", r.id, r.topic));
        }
        if !(-3..=3).contains(&r.polarity) {
            problems.push(format!("{}: polarity {} outside -3..=3", r.id, r.polarity));
        }
        if r.title.trim().is_empty() || r.text.en.trim().is_empty() {
            problems.push(format!("{}: title and text.en are required", r.id));
        }
        // Every rule says what it means for the reader (DESIGN section 4b).
        match &r.text.impact {
            None => problems.push(format!("{}: text.impact is required", r.id)),
            Some(i) if i.trim().len() < 20 => problems.push(format!("{}: text.impact is too short to explain anything", r.id)),
            Some(i) if i.trim() == r.text.en.trim() => problems.push(format!("{}: text.impact must say something the technical text does not", r.id)),
            Some(_) => {}
        }
        if r.source.note.trim().is_empty() {
            problems.push(format!("{}: source.note is required", r.id));
        }
        check_review(&r.id, &r.review, &mut problems);
        check_condition(&r.id, &r.when, &mut problems);
        // Period-lord references exist only while a period is being evaluated.
        if let Some(sub) = &r.subject {
            check_ref(&r.id, sub, &mut problems);
        }
        let uses_period = r.when.refs().iter().chain(r.timing.iter()).chain(r.subject.iter()).any(|g| matches!(g, GrahaRef::Period(_)));
        if uses_period && r.scope != Scope::Period {
            problems.push(format!("{}: uses a period-lord reference but is not a period rule", r.id));
        }
        if r.scope == Scope::Period && !r.timing.is_empty() {
            problems.push(format!("{}: period rules are already timed; remove `timing`", r.id));
        }
        for t in &r.timing {
            check_ref(&r.id, t, &mut problems);
        }
    }

    // Override graph: targets exist, same topic, no self-edges, no cycles.
    for r in &corpus.rules {
        for t in &r.overrides {
            match by_id.get(t.as_str()) {
                None => problems.push(format!("{}: overrides unknown rule {t}", r.id)),
                Some(target) if target.topic != r.topic => {
                    problems.push(format!("{}: overrides {t} in a different topic", r.id))
                }
                Some(_) if *t == r.id => problems.push(format!("{}: overrides itself", r.id)),
                _ => {}
            }
        }
    }
    if let Some(cycle) = find_cycle(&corpus.rules) {
        problems.push(format!("override cycle: {}", cycle.join(" -> ")));
    }

    validate_catalogue(corpus, &mut problems);

    if let Some(q) = &corpus.questions {
        check_review("questions.json", &q.review, &mut problems);
        let mut ids = HashSet::new();
        for x in &q.questions {
            if !ids.insert(x.id.as_str()) {
                problems.push(format!("questions.json: duplicate id {}", x.id));
            }
            if !x.question.trim_end().ends_with('?') {
                problems.push(format!("questions.json {}: a question must end in a question mark", x.id));
            }
            if x.keywords.is_empty() {
                problems.push(format!("questions.json {}: no keywords, so it can never be found", x.id));
            }
            // A question that names a topic the corpus cannot read would offer
            // an answer and then fail to give one.
            if !corpus.topics.iter().any(|t| t.id == x.topic) {
                problems.push(format!("questions.json {}: topic {} is not in topics.json", x.id, x.topic));
            }
        }
    }
    if let Some(e) = &corpus.porutham_explain {
        check_review("porutham.explain.json", &e.review, &mut problems);
        if e.source.note.trim().is_empty() {
            problems.push("porutham.explain.json: source.note is required".into());
        }
        for k in PoruthamKind::ALL {
            match e.get(k) {
                None => problems.push(format!("porutham.explain.json: missing {k:?}")),
                Some(x) => {
                    for (what, text) in [("measures", &x.measures), ("when_matching", &x.when_matching), ("when_not", &x.when_not), ("touches", &x.touches)] {
                        if text.trim().len() < 8 {
                            problems.push(format!("porutham.explain.json {k:?}: {what} is too short to explain anything"));
                        }
                    }
                }
            }
        }
    }
    if let Some(p) = &corpus.porutham {
        for k in PoruthamKind::ALL {
            match p.get(&k) {
                None => problems.push(format!("porutham.review.json: missing {k:?}")),
                Some(r) => check_review(&format!("porutham {k:?}"), r, &mut problems),
            }
        }
    }
    problems
}

/// Depth-first search for a cycle in the override graph.
fn find_cycle(rules: &[Rule]) -> Option<Vec<String>> {
    let edges: HashMap<&str, Vec<&str>> = rules
        .iter()
        .map(|r| (r.id.as_str(), r.overrides.iter().map(String::as_str).collect()))
        .collect();
    let mut ids: Vec<&str> = edges.keys().copied().collect();
    ids.sort();
    // 0 = unvisited, 1 = on stack, 2 = done
    let mut state: HashMap<&str, u8> = HashMap::new();
    let mut stack: Vec<&str> = Vec::new();

    fn dfs<'a>(
        n: &'a str,
        edges: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        state.insert(n, 1);
        stack.push(n);
        for &m in edges.get(n).map(Vec::as_slice).unwrap_or(&[]) {
            match state.get(m).copied().unwrap_or(0) {
                1 => {
                    let start = stack.iter().position(|&x| x == m).unwrap();
                    let mut cyc: Vec<String> = stack[start..].iter().map(|s| s.to_string()).collect();
                    cyc.push(m.to_string());
                    return Some(cyc);
                }
                0 => {
                    if let Some(c) = dfs(m, edges, state, stack) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
        stack.pop();
        state.insert(n, 2);
        None
    }

    for id in ids {
        if state.get(id).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(id, &edges, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
}

/// Summary counts for `rules validate`.
#[derive(Debug, Serialize, Deserialize)]
pub struct CorpusSummary {
    pub rules: usize,
    pub by_topic: BTreeMap<String, usize>,
    pub by_status: BTreeMap<String, usize>,
}

pub fn summary(c: &Corpus) -> CorpusSummary {
    let mut by_topic = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    for r in &c.rules {
        *by_topic.entry(r.topic.clone()).or_default() += 1;
        *by_status.entry(format!("{:?}", r.review.status).to_lowercase()).or_default() += 1;
    }
    CorpusSummary { rules: c.rules.len(), by_topic, by_status }
}

/// Topics, bhavas and pariharams. Skipped entirely for a corpus that has no
/// topic catalogue (e.g. a synthetic test corpus).
fn validate_catalogue(c: &Corpus, problems: &mut Vec<String>) {
    if c.topics.is_empty() {
        return;
    }
    let mut ids = HashSet::new();
    for t in &c.topics {
        if !ids.insert(t.id.as_str()) {
            problems.push(format!("topics.json: duplicate topic {}", t.id));
        }
        if t.focus.houses.is_empty() && t.focus.karakas.is_empty() {
            problems.push(format!("topic {}: focus needs houses or karakas for the write-up", t.id));
        }
        for h in t.focus.houses.iter().chain(t.focus.vargas.iter().map(|v| &v.house)) {
            if !(1..=12).contains(h) {
                problems.push(format!("topic {}: focus house {h} is not 1..12", t.id));
            }
        }
        for k in &t.focus.karakas {
            if k.role.trim().is_empty() {
                problems.push(format!("topic {}: karaka {:?} needs a role", t.id, k.graha));
            }
        }
        if t.title.trim().is_empty() || t.summary.trim().is_empty() {
            problems.push(format!("topic {}: title and summary are required", t.id));
        }
        if !(t.ages[0] >= 0.0 && t.ages[1] > t.ages[0] && t.ages[1] <= 120.0) {
            problems.push(format!("topic {}: bad age range {:?}", t.id, t.ages));
        }
        // Content policy (DESIGN 6, section 2): health and vitality always
        // carry a disclaimer that points to a doctor.
        if ["health", "vitality"].contains(&t.id.as_str()) {
            let ok = t.disclaimer.as_deref().is_some_and(|d| {
                let d = d.to_lowercase();
                d.contains("doctor") || d.contains("medical")
            });
            if !ok {
                problems.push(format!("topic {}: needs a disclaimer pointing to medical advice", t.id));
            }
        }
        check_review(&format!("topic {}", t.id), &t.review, problems);
    }
    for r in &c.rules {
        if r.scope == Scope::Natal && !ids.contains(r.topic.as_str()) {
            problems.push(format!("{}: topic {} is not in topics.json", r.id, r.topic));
        }
    }
    for t in &c.topics {
        if !c.rules.iter().any(|r| r.topic == t.id) {
            problems.push(format!("topic {} has no rules", t.id));
        }
    }
    if let Some(b) = &c.bhavas {
        let hs: Vec<u8> = b.houses.iter().map(|h| h.house).collect();
        if hs != (1..=12).collect::<Vec<u8>>() {
            problems.push("bhava.json: must list houses 1 to 12 in order".into());
        }
        if b.houses.iter().any(|h| h.significations.is_empty() || h.name.trim().is_empty()) {
            problems.push("bhava.json: every house needs a name and significations".into());
        }
        check_review("bhava.json", &b.review, problems);
    }
    if let Some(k) = &c.karma {
        let hs: Vec<u8> = k.ketu_house.iter().map(|x| x.house).collect();
        if hs != (1..=12).collect::<Vec<u8>>() {
            problems.push("karma.json: ketu_house must list houses 1 to 12 in order".into());
        }
        let ss: Vec<u8> = k.ketu_sign.iter().map(|x| x.sign).collect();
        if ss != (1..=12).collect::<Vec<u8>>() {
            problems.push("karma.json: ketu_sign must list signs 1 to 12 in order".into());
        }
        if k.ketu_house.iter().any(|x| x.station.trim().len() < 20) {
            problems.push("karma.json: every house needs a station".into());
        }
        if k.ketu_sign.iter().any(|x| x.temperament.trim().len() < 20 || x.name.trim().is_empty()) {
            problems.push("karma.json: every sign needs a name and a temperament".into());
        }
        if k.dispositor.strong.trim().len() < 20 || k.dispositor.weak.trim().len() < 20 {
            problems.push("karma.json: both dispositor readings are required".into());
        }
        check_review("karma.json", &k.review, problems);
    }
    let rule_tags: HashSet<&str> = c.rules.iter().flat_map(|r| r.tags.iter().map(String::as_str)).collect();
    let mut pids = HashSet::new();
    for p in &c.pariharams {
        if !pids.insert(p.id.as_str()) {
            problems.push(format!("pariharam.json: duplicate id {}", p.id));
        }
        if p.triggers.is_empty() || p.practices.is_empty() {
            problems.push(format!("pariharam {}: needs triggers and practices", p.id));
        }
        for t in &p.triggers {
            // graha:<name> is produced from rule subjects at run time.
            let graha_tag = t.strip_prefix("graha:").is_some_and(|g| {
                serde_json::from_value::<lagn_core::Graha>(serde_json::Value::String(g.to_string())).is_ok()
            });
            if !graha_tag && !rule_tags.contains(t.as_str()) && !crate::catalogue::TRANSIT_TAGS.contains(&t.as_str()) {
                problems.push(format!("pariharam {}: trigger {t} is used by no rule and is not a transit tag", p.id));
            }
        }
        check_review(&format!("pariharam {}", p.id), &p.review, problems);
    }
}
