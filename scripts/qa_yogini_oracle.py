#!/usr/bin/env python3
"""Phase 13D N-version oracle: Yogini dasha.

An independent implementation of docs/phase13/YOGINI.md sections 3 to 5,
diffed against `lagn yogini --json`.

Two tables are parsed out of the specification at run time rather than
retyped: the eight yoginis with their lords and years (section 3), and the
starting rule (section 4).

Implementation choices deliberately differ from the Rust. The sequence is
built by rotating a list rather than by modular arithmetic on an ordinal;
boundaries are accumulated as exact Fractions of a year and converted to
Julian Days once, where the kernel accumulates f64 years; and the balance is
derived from the kernel's own reported nakshatra fraction rather than
recomputed, so a disagreement isolates the dasha arithmetic from the
nakshatra arithmetic (which has its own tests).

What this cannot catch: a misreading of the Yogini rule shared by the
specification and both implementations. The guard against that is the
structure - the periods are the consecutive integers 1 to 8 and sum to 36,
which this oracle asserts from the parsed table before comparing anything.

Usage:
    python3 scripts/qa_yogini_oracle.py [--charts N] [--seed S]
"""
from __future__ import annotations

import argparse
import json
import random
import re
import subprocess
import sys
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = ROOT / "target" / "release" / "lagn"
SPEC = ROOT / "docs" / "phase13" / "YOGINI.md"

YEAR_DAYS = {"julian": 365.25, "gregorian": 365.2425, "tropical": 365.242190,
             "sidereal_solar": 365.256363, "savana": 360.0}
# A second, in days.
SECOND = 1.0 / 86400.0
# The kernel serialises Graha in snake_case English; the spec names them in
# Sanskrit.
SERDE = {"sun": "Surya", "moon": "Chandra", "mars": "Kuja", "mercury": "Budha",
         "jupiter": "Guru", "venus": "Shukra", "saturn": "Shani",
         "rahu": "Rahu", "ketu": "Ketu"}


def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_yoginis(text: str) -> list[tuple[str, str, int]]:
    """Section 3's table: (yogini, lord, years), in sequence."""
    start = text.index("| # | Yogini | Lord | Years |")
    end = text.index("Total 36 years", start)
    rows = re.findall(r"^\|\s*([1-8])\s*\|\s*(\w+)\s*\|\s*(\w+)\s*\|\s*(\d+)\s*\|",
                      text[start:end], re.M)
    by_n = {int(n): (y, lord, int(yrs)) for n, y, lord, yrs in rows}
    if len(by_n) != 8:
        sys.exit(f"expected eight yoginis in section 3, parsed {sorted(by_n)}")
    table = [by_n[i] for i in range(1, 9)]

    # The structural check, before anything is compared: consecutive 1..8
    # summing to 36. A mistranscribed period cannot pass both.
    years = [y for _, _, y in table]
    if years != list(range(1, 9)):
        sys.exit(f"section 3's periods are not the consecutive integers 1 to 8: {years}")
    if sum(years) != 36:
        sys.exit(f"section 3's periods sum to {sum(years)}, not 36")
    return table


def parse_start_offset(text: str) -> int:
    """Section 4's starting rule: the number added before taking mod 8."""
    m = re.search(r"start = \(janma nakshatra number \+ (\d+)\) mod 8", text)
    if not m:
        sys.exit("section 4's starting rule is not the one this oracle implements")
    return int(m.group(1))


def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], table, offset: int, report: list[str]) -> tuple[int, int]:
    chart = lagn("chart", *birth, "--json")
    rust = lagn("yogini", *birth, "--json")
    tag = " ".join(birth)
    bad = checked = 0

    names = [y for y, _, _ in table]
    lords = {y: lord for y, lord, _ in table}
    years = {y: yrs for y, _, yrs in table}

    # Where it starts. The nakshatra serial comes from the chart's own Moon,
    # read independently of the dasha command.
    nak = chart["lagna"]  # placeholder; the janma nakshatra is the Moon's
    moon = next(p for p in chart["placements"] if p["graha"] == "moon")
    serial = int((moon["longitude"] % 360.0) // (40.0 / 3.0)) + 1
    r = (serial + offset) % 8
    want_start = names[(8 if r == 0 else r) - 1]
    _ = nak

    checked += 1
    if rust["birth_yogini"].lower() != want_start.lower():
        report.append(f"{tag}: starts in {rust['birth_yogini']}, expected {want_start}")
        bad += 1
        return bad, checked

    # The balance, from the kernel's own nakshatra fraction so a disagreement
    # isolates the dasha arithmetic.
    frac = Fraction(moon["nakshatra"]["fraction_traversed"]).limit_denominator(10**12)
    want_balance = years[want_start] * (1 - frac)
    checked += 1
    if abs(float(want_balance) - rust["balance_years"]) > 1e-9:
        report.append(f"{tag}: balance {rust['balance_years']} vs {float(want_balance)}")
        bad += 1

    # The sequence and every boundary, accumulated as exact fractions of a
    # year from the first period's origin.
    year_days = YEAR_DAYS[chart["settings"]["year_length"]]
    birth_jd = chart["jd_ut"]
    elapsed = -(Fraction(years[want_start]) - want_balance)
    i = names.index(want_start)

    for p in rust["periods"]:
        checked += 4
        expected_name = names[i % 8]
        if p["yogini"].lower() != expected_name.lower():
            report.append(f"{tag}: period {p['yogini']} vs {expected_name}")
            bad += 1
            break
        if SERDE.get(p["lord"], p["lord"]) != lords[expected_name]:
            report.append(f"{tag}: {expected_name} lord {p['lord']} vs {lords[expected_name]}")
            bad += 1

        start = birth_jd + float(elapsed) * year_days
        end = birth_jd + float(elapsed + years[expected_name]) * year_days
        if abs(p["start_jd"] - start) > SECOND:
            report.append(f"{tag}: {expected_name} start off by {(p['start_jd'] - start) * 86400:.3f} s")
            bad += 1
        if abs(p["end_jd"] - end) > SECOND:
            report.append(f"{tag}: {expected_name} end off by {(p['end_jd'] - end) * 86400:.3f} s")
            bad += 1

        # Eight antardashas, proportional, opening on the mahadasha's yogini.
        checked += 1
        if len(p["children"]) != 8:
            report.append(f"{tag}: {expected_name} has {len(p['children'])} antardashas")
            bad += 1
        else:
            inner = elapsed
            for k in range(8):
                sub = names[(i + k) % 8]
                checked += 2
                if p["children"][k]["yogini"].lower() != sub.lower():
                    report.append(f"{tag}: {expected_name}/{k} is {p['children'][k]['yogini']} vs {sub}")
                    bad += 1
                span = Fraction(years[expected_name] * years[sub], 36)
                cs = birth_jd + float(inner) * year_days
                if abs(p["children"][k]["start_jd"] - cs) > SECOND:
                    report.append(f"{tag}: {expected_name}/{sub} start off")
                    bad += 1
                inner += span

        elapsed += years[expected_name]
        i += 1

    return bad, checked


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=80)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = spec_text()
    table = parse_yoginis(text)
    offset = parse_start_offset(text)
    print(f"parsed from YOGINI.md: {[y for y, _, _ in table]}")
    print(f"parsed from YOGINI.md: periods {[y for _, _, y in table]} summing to 36; start offset +{offset}")

    rng = random.Random(args.seed)
    report: list[str] = []
    bad = checked = 0
    for _ in range(args.charts):
        birth = [
            "--date", f"{rng.randint(1400, 2300)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
            "--time", f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}",
            "--tz", "5.5",
            "--lat", f"{rng.uniform(-60, 60):.4f}",
            "--lon", f"{rng.uniform(-180, 180):.4f}",
            "--year-length", rng.choice(["julian", "savana", "tropical"]),
        ]
        b, c = compare(birth, table, offset, report)
        bad += b
        checked += c

    print(f"{args.charts} charts, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
