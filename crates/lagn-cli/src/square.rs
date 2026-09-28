//! South Indian square chart rendering.
//!
//! The South Indian chart is a 4x4 grid with the rasis in *fixed* positions -
//! Meena at top-left, running clockwise - so the lagna moves around the grid
//! rather than the signs moving. This is the format used across Tamil Nadu and
//! Kerala, and it is what a jathagam printed at a temple looks like.
//!
//! (The North Indian diamond chart is the opposite convention: houses fixed,
//! signs moving. We do not render it.)

use lagn_core::{Graha, Rasi};

/// Grid position of each rasi, as `(row, col)` in a 4x4 layout.
///
/// ```text
///   Meena  | Mesha  | Vrisha | Mithuna
///   Kumbha |        |        | Karka
///   Makara |        |        | Simha
///   Dhanus | Vrisch | Tula   | Kanya
/// ```
const fn cell_of(rasi_index: u8) -> (usize, usize) {
    match rasi_index {
        0 => (0, 1),   // Mesha
        1 => (0, 2),   // Vrishabha
        2 => (0, 3),   // Mithuna
        3 => (1, 3),   // Karka
        4 => (2, 3),   // Simha
        5 => (3, 3),   // Kanya
        6 => (3, 2),   // Tula
        7 => (3, 1),   // Vrischika
        8 => (3, 0),   // Dhanus
        9 => (2, 0),   // Makara
        10 => (1, 0),  // Kumbha
        _ => (0, 0),   // Meena
    }
}

const CELL_W: usize = 17;
const CELL_H: usize = 4;

/// Render a square chart.
///
/// `occupants` maps a rasi index to the labels shown in that cell.
/// `lagna` gets an `Asc` marker. `caption` fills the empty 2x2 centre.
// Row/column indices are the geometry here, not incidental loop counters.
#[allow(clippy::needless_range_loop)]
pub fn render(
    lagna: Rasi,
    occupants: &[(Rasi, Vec<String>)],
    caption: &[String],
) -> String {
    let mut cells: [[Vec<String>; 4]; 4] = Default::default();

    for (rasi, labels) in occupants {
        let (r, c) = cell_of(rasi.index());
        cells[r][c].extend(labels.iter().cloned());
    }
    // The lagna marker goes first in its cell.
    let (lr, lc) = cell_of(lagna.index());
    cells[lr][lc].insert(0, "Asc".to_string());

    let mut out = String::new();
    let hline = format!("+{}+{}+{}+{}+\n", "-".repeat(CELL_W), "-".repeat(CELL_W), "-".repeat(CELL_W), "-".repeat(CELL_W));

    for r in 0..4 {
        out.push_str(&hline);
        for line in 0..CELL_H {
            out.push('|');
            for c in 0..4 {
                let is_centre = (1..=2).contains(&r) && (1..=2).contains(&c);
                let text = if is_centre {
                    // Centre 2x2: caption text, centred across the whole block.
                    centre_text(caption, r, c, line)
                } else if line == 0 {
                    // First line of every sign cell is the sign's short name.
                    let rasi = Rasi::ALL.iter().find(|x| cell_of(x.index()) == (r, c));
                    rasi.map(|x| format!("{:<width$}", short(*x), width = CELL_W - 1))
                        .unwrap_or_else(|| " ".repeat(CELL_W - 1))
                } else {
                    cells[r][c]
                        .get(line - 1)
                        .map(|s| format!("{:<width$}", s, width = CELL_W - 1))
                        .unwrap_or_else(|| " ".repeat(CELL_W - 1))
                };
                out.push(' ');
                out.push_str(&text);
                out.push('|');
            }
            out.push('\n');
        }
    }
    out.push_str(&hline);
    out
}

/// The centre 2x2 block is drawn as one 4-line region spanning two cells.
fn centre_text(caption: &[String], r: usize, c: usize, line: usize) -> String {
    let idx = (r - 1) * CELL_H + line;
    let text = caption.get(idx).cloned().unwrap_or_default();
    // Left cell carries the text; right cell stays blank so it reads as one box.
    if c == 1 {
        format!("{:<width$}", truncate(&text, CELL_W - 1), width = CELL_W - 1)
    } else {
        " ".repeat(CELL_W - 1)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect()
    }
}

fn short(r: Rasi) -> &'static str {
    match r {
        Rasi::Mesha => "Mesha",
        Rasi::Vrishabha => "Vrishabha",
        Rasi::Mithuna => "Mithuna",
        Rasi::Karka => "Karka",
        Rasi::Simha => "Simha",
        Rasi::Kanya => "Kanya",
        Rasi::Tula => "Tula",
        Rasi::Vrischika => "Vrischika",
        Rasi::Dhanus => "Dhanus",
        Rasi::Makara => "Makara",
        Rasi::Kumbha => "Kumbha",
        Rasi::Meena => "Meena",
    }
}

/// Label for a graha in a cell, e.g. `Ju(R) 12:34`.
pub fn graha_label(g: Graha, deg_in_rasi: f64, retro: bool) -> String {
    let d = deg_in_rasi.floor() as u32;
    let m = ((deg_in_rasi - d as f64) * 60.0).floor() as u32;
    format!("{}{} {:02}:{:02}", g.abbrev(), if retro { "(R)" } else { "" }, d, m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rasi_has_a_distinct_cell_outside_the_centre() {
        let mut seen = std::collections::HashSet::new();
        for r in Rasi::ALL {
            let cell = cell_of(r.index());
            assert!(seen.insert(cell), "{:?} collides at {cell:?}", r);
            let (row, col) = cell;
            let centre = (1..=2).contains(&row) && (1..=2).contains(&col);
            assert!(!centre, "{:?} landed in the centre block", r);
        }
        assert_eq!(seen.len(), 12);
    }

    #[test]
    fn signs_run_clockwise_from_meena() {
        // Walking Mesha..Meena must trace the ring in order.
        let ring: Vec<_> = (0..12).map(cell_of).collect();
        assert_eq!(ring[0], (0, 1)); // Mesha
        assert_eq!(ring[3], (1, 3)); // Karka
        assert_eq!(ring[6], (3, 2)); // Tula
        assert_eq!(ring[9], (2, 0)); // Makara
        assert_eq!(ring[11], (0, 0)); // Meena
    }
}
