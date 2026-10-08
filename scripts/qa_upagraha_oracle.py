#!/usr/bin/env python3
"""Phase 13C N-version oracle: upagrahas and the time lagnas.

An independent implementation of docs/phase13/UPAGRAHA.md sections 3, 4 and 5,
diffed against the Rust kernel's `lagn upagraha --json`.

Three things are *parsed out of the specification at run time* rather than
retyped here: the Sun-offset table (section 3), the upagraha-to-ruler mapping
(section 4), and the rate of each time lagna (section 5). That checks the Rust
transcription against the specification text rather than against a second
hand-copy that could share an error.

Implementation choices deliberately differ from the Rust. The Sun offsets are
collapsed into a single closed-form expression per point, derived from the
chain algebraically, where the Rust walks the chain step by step - so a
mistranscribed intermediate shows up as a disagreement rather than cancelling
out. The part-of-the-day index is found by rotating a list rather than by
modular arithmetic. The time lagnas are recomputed from the kernel's own
reported sunrise Sun and elapsed hours, which isolates the rate from the
sunrise search.

What this cannot catch: a misreading shared by the specification and both
implementations, or an error in the sunrise itself - which has its own oracle
against swetest in crates/lagn-core/tests/day.rs.

Usage:
    python3 scripts/qa_upagraha_oracle.py [--charts N] [--seed S]
"""
from __future__ import annotations

import argparse
import json
import random
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = ROOT / "target" / "release" / "lagn"
SPEC = ROOT / "docs" / "phase13" / "UPAGRAHA.md"

RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula",
        "vrischika", "dhanus", "makara", "kumbha", "meena"]
# The weekday lords, in the order the parts follow. Sanskrit names as the
# kernel prints them.
ORDER = ["Surya", "Chandra", "Kuja", "Budha", "Guru", "Shukra", "Shani"]
# An arcsecond, in degrees.
ARCSEC = 1.0 / 3600.0


# ---------------------------------------------------------------------------
# Parsed from the specification
# ---------------------------------------------------------------------------

def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_dm(text: str) -> float:
    """"133:20" -> 133.3333..."""
    d, m = text.split(":")
    return float(d) + float(m) / 60.0


def parse_sun_offsets(text: str) -> dict[str, str]:
    """Section 3's table: upagraha -> the formula as written."""
    rows = re.findall(r"^\|\s*(Dhuma|Vyatipata|Parivesha|Indrachapa|Upaketu)\s*\|\s*`([^`]+)`\s*\|", text, re.M)
    table = {name: formula.strip() for name, formula in rows}
    if len(table) != 5:
        sys.exit(f"expected five Sun offsets in section 3, parsed {sorted(table)}")
    return table


def parse_rulers(text: str) -> dict[str, str]:
    """Section 4's table: upagraha -> ruling graha."""
    rows = re.findall(r"^\|\s*(Kaala|Mrityu|Ardhaprahara|Yamaghantaka|Gulika)\s*\|\s*(\w+)\s*\|", text, re.M)
    table = dict(rows)
    if len(table) != 5:
        sys.exit(f"expected five day-division upagrahas in section 4, parsed {sorted(table)}")
    return table


def parse_rates(text: str) -> dict[str, float]:
    """Section 5's table: lagna -> degrees per hour."""
    rows = re.findall(r"^\|\s*(Bhava|Hora|Ghati)\s*\|[^|]+\|\s*([\d.]+)\s*\|", text, re.M)
    table = {f"{n} lagna": float(v) for n, v in rows}
    if len(table) != 3:
        sys.exit(f"expected three time lagnas in section 5, parsed {sorted(table)}")
    return table


# ---------------------------------------------------------------------------
# The oracle
# ---------------------------------------------------------------------------

def closed_form(offsets: dict[str, str], sun: float) -> dict[str, float]:
    """The five Sun offsets, each as one expression rather than a chain.

    The chain in section 3 defines each point from the previous one. Solving
    it algebraically gives a direct formula per point, which is what this
    computes - deliberately a different route to the same numbers.
    """
    d = parse_dm(offsets["Dhuma"].split("+")[1].strip())
    u = parse_dm(offsets["Upaketu"].split("+")[1].strip())
    return {
        "Dhuma": (sun + d) % 360.0,
        "Vyatipata": (-sun - d) % 360.0,
        "Parivesha": (180.0 - sun - d) % 360.0,
        "Indrachapa": (sun + d - 180.0) % 360.0,
        "Upaketu": (sun + d - 180.0 + u) % 360.0,
    }


def part_index(first: str, graha: str) -> int:
    """Which eighth `graha` rules, 1-based, by rotating the list."""
    rotated = ORDER[ORDER.index(first):] + ORDER[:ORDER.index(first)]
    return rotated.index(graha) + 1


def night_first(vara_lord: str) -> str:
    """The fifth lord from the weekday lord, counting it as the first."""
    rotated = ORDER[ORDER.index(vara_lord):] + ORDER[:ORDER.index(vara_lord)]
    return rotated[4]


def apart(a: float, b: float) -> float:
    d = abs(a - b) % 360.0
    return 360.0 - d if d > 180.0 else d


# ---------------------------------------------------------------------------
# Diffing
# ---------------------------------------------------------------------------

def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], offsets, rulers, rates, report: list[str]) -> tuple[int, int]:
    chart = lagn("chart", *birth, "--json")
    rust = lagn("upagraha", *birth, "--json")
    tag = " ".join(birth)
    bad = checked = 0

    sun = next(p["longitude"] for p in chart["placements"] if p["graha"] == "sun")
    mine = closed_form(offsets, sun)
    for p in rust["sun_offsets"]:
        checked += 2
        want = mine[p["name"]]
        if apart(p["longitude"], want) > ARCSEC:
            report.append(f"{tag}: {p['name']} {p['longitude']} vs {want}")
            bad += 1
        if RASI[int(want // 30)] != p["rasi"].lower():
            report.append(f"{tag}: {p['name']} rasi {p['rasi']} vs {RASI[int(want // 30)]}")
            bad += 1

    # The day division: the part index, which is the part this oracle can
    # check without redoing the sunrise search.
    vara_lord = rust["vara"]
    lord_name = {"ravi": "Surya", "soma": "Chandra", "mangala": "Kuja", "budha": "Budha",
                 "guru": "Guru", "shukra": "Shukra", "shani": "Shani"}[vara_lord]
    first = night_first(lord_name) if rust["at_night"] else lord_name
    for d in rust["day_parts"]:
        checked += 1
        ruler = rulers[d["point"]["name"]]
        want = part_index(first, ruler)
        if d["part"] != want:
            report.append(f"{tag}: {d['point']['name']} part {d['part']} vs {want} (first {first})")
            bad += 1

    # The time lagnas, from the kernel's own sunrise Sun and elapsed hours, so
    # only the rate is under test.
    for p in rust["time_lagnas"]:
        checked += 1
        want = (rust["sunrise_sun"] + rates[p["name"]] * rust["hours_since_sunrise"]) % 360.0
        if apart(p["longitude"], want) > ARCSEC:
            report.append(f"{tag}: {p['name']} {p['longitude']} vs {want}")
            bad += 1

    return bad, checked


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=120)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = spec_text()
    offsets, rulers, rates = parse_sun_offsets(text), parse_rulers(text), parse_rates(text)
    print(f"parsed from UPAGRAHA.md: offsets {offsets}")
    print(f"parsed from UPAGRAHA.md: rulers {rulers}")
    print(f"parsed from UPAGRAHA.md: rates {rates}")

    rng = random.Random(args.seed)
    report: list[str] = []
    bad = checked = 0
    for _ in range(args.charts):
        birth = [
            "--date", f"{rng.randint(1900, 2060)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
            "--time", f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}",
            "--tz", "5.5",
            # Moderate latitudes: the division needs a real sunrise and sunset.
            "--lat", f"{rng.uniform(-50, 50):.4f}",
            "--lon", f"{rng.uniform(-180, 180):.4f}",
        ]
        b, c = compare(birth, offsets, rulers, rates, report)
        bad += b
        checked += c

    print(f"{args.charts} charts, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
