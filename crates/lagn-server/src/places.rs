//! Place search over the GeoNames-derived index (`data/places.tsv`).

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Place {
    pub id: u64,
    pub name: String,
    pub admin1: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub population: u64,
    /// Set when the query matched a historical or alternate name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_alias: Option<String>,
}

struct Entry {
    place: Place,
    name_key: String,
    ascii_key: String,
    alternates: Vec<(String, String)>, // (key, original)
}

pub struct PlaceIndex {
    entries: Vec<Entry>,
}

/// Case- and punctuation-insensitive key.
fn key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl PlaceIndex {
    pub fn load(path: &Path) -> Result<PlaceIndex, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        PlaceIndex::parse(&text)
    }

    pub fn parse(text: &str) -> Result<PlaceIndex, String> {
        let mut entries = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != 10 {
                return Err(format!("places line {}: expected 10 fields, got {}", n + 1, f.len()));
            }
            let parse_f = |s: &str, what: &str| {
                s.parse::<f64>().map_err(|_| format!("places line {}: bad {what} {s:?}", n + 1))
            };
            let place = Place {
                id: f[0].parse().map_err(|_| format!("places line {}: bad id", n + 1))?,
                name: f[1].to_string(),
                country: f[4].to_string(),
                admin1: f[5].to_string(),
                latitude: parse_f(f[6], "latitude")?,
                longitude: parse_f(f[7], "longitude")?,
                timezone: f[8].to_string(),
                population: f[9].parse().unwrap_or(0),
                matched_alias: None,
            };
            if !(-90.0..=90.0).contains(&place.latitude) || !(-180.0..=180.0).contains(&place.longitude) {
                return Err(format!("places line {}: coordinates out of range", n + 1));
            }
            let alternates = f[3]
                .split('|')
                .filter(|a| !a.is_empty())
                .map(|a| (key(a), a.to_string()))
                .collect();
            entries.push(Entry { name_key: key(f[1]), ascii_key: key(f[2]), place, alternates });
        }
        Ok(PlaceIndex { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Search by name. Two tiers, best first:
    /// 0 exact match on the current name or an alternate (e.g. "Madras"),
    /// 1 current name or an alternate starts with the query;
    /// within a tier by population (descending), then id - fully deterministic.
    ///
    /// Current and historical names share a tier on purpose: "Calcutta" must
    /// find Kolkata (4.6 million) before a small town that is named Calcutta
    /// today. The alias shown with each result makes the choice visible.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Place> {
        let q = key(query);
        if q.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<(u8, u64, u64, usize, Option<String>)> = Vec::new();
        for (i, e) in self.entries.iter().enumerate() {
            let rank = if e.name_key == q || e.ascii_key == q {
                Some((0, None))
            } else if let Some((_, orig)) = e.alternates.iter().find(|(k, _)| *k == q) {
                Some((0, Some(orig.clone())))
            } else if e.name_key.starts_with(&q) || e.ascii_key.starts_with(&q) {
                Some((1, None))
            } else {
                e.alternates.iter().find(|(k, _)| k.starts_with(&q)).map(|(_, o)| (1, Some(o.clone())))
            };
            if let Some((r, alias)) = rank {
                hits.push((r, u64::MAX - e.place.population, e.place.id, i, alias));
            }
        }
        hits.sort_by_key(|h| (h.0, h.1, h.2));
        hits.into_iter()
            .take(limit)
            .map(|(_, _, _, i, alias)| Place { matched_alias: alias, ..self.entries[i].place.clone() })
            .collect()
    }
}
