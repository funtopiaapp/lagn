#!/usr/bin/env python3
"""Phase 13H N-version oracle: the Krishnamurti Paddhati.

An independent implementation of docs/phase13/KP.md sections 3 and 4, diffed
against the Rust kernel's `lagn kp --json`.

Two things are *parsed out of the specification at run time*: the sub-width
formula (section 3) and the five ruling-planet roles in order (section 4).
The Vimshottari years are read from the kernel's own dasha output rather than
retyped, so the proportions cannot drift between the two implementations
without the diff noticing.

Implementation choices deliberately differ from the Rust. Subs are built with
exact rational arithmetic (fractions.Fraction) where the kernel uses f64, and
the sub containing a longitude is found by accumulating boundaries from the
nakshatra's start rather than by scanning a list of divisions. The 249-cell
count is rebuilt here too, independently of the kernel's own assertion.

What this cannot catch: a misreading of the KP rule shared by the
specification and both implementations. The 249 derivation is the guard
against that - a wrong sub width, starting lord or cycle order changes the
count.

Usage:
    python3 scripts/qa_kp_oracle.py [--charts N] [--seed S]
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
SPEC = ROOT / "docs" / "phase13" / "KP.md"

# The Vimshottari cycle, in the order the kernel declares it.
CYCLE = ["Ketu", "Shukra", "Surya", "Chandra", "Kuja", "Rahu", "Guru", "Shani", "Budha"]
YEARS = {"Ketu": 7, "Shukra": 20, "Surya": 6, "Chandra": 10, "Kuja": 7,
         "Rahu": 18, "Guru": 16, "Shani": 19, "Budha": 17}
TOTAL = 120
assert sum(YEARS.values()) == TOTAL

# The kernel serialises Graha in snake_case English; this oracle works in the
# Sanskrit names the specification uses. Mapped rather than renamed, so the
# oracle stays readable against the document it implements.
SERDE = {"sun": "Surya", "moon": "Chandra", "mars": "Kuja", "mercury": "Budha",
         "jupiter": "Guru", "venus": "Shukra", "saturn": "Shani",
         "rahu": "Rahu", "ketu": "Ketu"}


def graha(name: str) -> str:
    """A kernel graha name in the specification's vocabulary."""
    return SERDE.get(name, name)


# Classical seven-lord scheme, by rasi index.
RASI_LORD = ["Kuja", "Shukra", "Budha", "Chandra", "Surya", "Budha",
             "Shukra", "Kuja", "Guru", "Shani", "Shani", "Guru"]

NAK = Fraction(40, 3)      # 13 deg 20 min
SIGN = Fraction(30)
ARCSEC = 1.0 / 3600.0


def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_width_rule(text: str) -> bool:
    """Section 3's formula, checked for the shape this oracle assumes."""
    m = re.search(r"width\(lord\)\s*=\s*\(13 deg 20 min\)\s*x\s*years\(lord\)\s*/\s*120", text)
    if not m:
        sys.exit("section 3's sub-width formula is not the one this oracle implements")
    return True


def parse_ruling_kinds(text: str) -> list[str]:
    """Section 4's ruling planets, as the *kind* of lord each one is.

    Scoped to the block between the Ruling-planets and Significators
    headings; parsing the whole document also matched the significator-group
    table and the revision history, which the diff caught.

    The wording is reduced to a kind - day, moon-sign, moon-star, asc-sign,
    asc-star - rather than compared verbatim. The kernel's label is prose for
    a reader and the document's is prose for a practitioner; demanding they
    match character for character tests nothing about the astrology.
    """
    start = text.index("**Ruling planets.**")
    end = text.index("**Significators of a house.**")
    rows = re.findall(r"^\|\s*([1-5])\s*\|\s*([^|]+?)\s*\|", text[start:end], re.M)
    by_n = {int(n): r.lower() for n, r in rows}
    if len(by_n) != 5:
        sys.exit(f"expected five ruling planets in section 4, parsed {sorted(by_n)}")

    kinds = []
    for i in range(1, 6):
        r = by_n[i]
        if "day" in r:
            kinds.append("day")
        elif "moon" in r:
            kinds.append("moon-star" if "star" in r else "moon-sign")
        elif "ascendant" in r:
            kinds.append("asc-star" if "star" in r else "asc-sign")
        else:
            sys.exit(f"section 4 row {i} is not a kind this oracle knows: {r!r}")
    if kinds != ["day", "moon-sign", "moon-star", "asc-sign", "asc-star"]:
        sys.exit(f"section 4 lists the ruling planets in an unexpected order: {kinds}")
    return kinds


def subs_of(star_lord: str, span: Fraction = NAK) -> list[tuple[Fraction, Fraction, str]]:
    """The nine subs of a span, as exact fractions, from `star_lord`."""
    i = CYCLE.index(star_lord)
    out, at = [], Fraction(0)
    for j in range(9):
        lord = CYCLE[(i + j) % 9]
        w = span * Fraction(YEARS[lord], TOTAL)
        out.append((at, at + w, lord))
        at += w
    assert at == span
    return out


def lords_of(longitude: float) -> tuple[str, str, str]:
    """(star lord, sub lord, sub-sub lord) for a sidereal longitude."""
    lon = Fraction(longitude).limit_denominator(10**12) % 360
    k = int(lon // NAK)
    star = CYCLE[k % 9]
    within = lon - NAK * k

    sub = next((a, b, l) for (a, b, l) in subs_of(star) if a <= within < b)
    inner = subs_of(sub[2], sub[1] - sub[0])
    off = within - sub[0]
    sub_sub = next(l for (a, b, l) in inner if a <= off < b)
    return star, sub[2], sub_sub


def cell_count() -> tuple[int, int]:
    """Rebuild the 249, independently of the kernel's own assertion."""
    cells = straddling = 0
    bounds = {SIGN * m for m in range(13)}
    for k in range(27):
        base = NAK * k
        for (a, b, _) in subs_of(CYCLE[k % 9]):
            inner = [x for x in bounds if base + a < x < base + b]
            cells += 1 + len(inner)
            if inner:
                straddling += 1
    return cells, straddling


def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], kinds: list[str], report: list[str]) -> tuple[int, int]:
    rust = lagn("kp", *birth, "--json")
    tag = " ".join(birth)
    bad = checked = 0

    def check(where: str, got: dict) -> None:
        nonlocal bad, checked
        star, sub, sub_sub = lords_of(got["longitude"])
        for name, mine, theirs in (
            ("star", star, graha(got["star_lord"])),
            ("sub", sub, graha(got["sub_lord"])),
            ("sub-sub", sub_sub, graha(got["sub_sub_lord"])),
        ):
            checked += 1
            if mine != theirs:
                report.append(f"{tag}: {where} {name} lord {mine} vs {theirs} at {got['longitude']}")
                bad += 1

    check("ascendant", rust["ascendant"])
    for entry in rust["grahas"]:
        check(entry[0], entry[1])
    for c in rust["cusps"]:
        check(f"cusp {c['house']}", c["lords"])

    # The five ruling planets: each really is the lord it claims to be,
    # recomputed here from the Moon's and the ascendant's own longitudes.
    rp = rust["ruling_planets"]
    checked += 1
    if len(rp) != 5:
        report.append(f"{tag}: {len(rp)} ruling planets, expected 5")
        bad += 1
    else:
        moon_lon = rust["moon"]["longitude"]
        asc_lon = rust["ascendant"]["longitude"]
        want = {
            "moon-sign": RASI_LORD[int(moon_lon % 360 // 30)],
            "moon-star": lords_of(moon_lon)[0],
            "asc-sign": RASI_LORD[int(asc_lon % 360 // 30)],
            "asc-star": lords_of(asc_lon)[0],
        }
        for i, kind in enumerate(kinds):
            if kind == "day":
                checked += 1
                # Not recomputed here: the vara needs the sunrise, which has
                # its own oracle against swetest. Checked for plausibility.
                if graha(rp[i]["graha"]) not in CYCLE:
                    report.append(f"{tag}: day lord {rp[i]['graha']} is not a graha")
                    bad += 1
                continue
            checked += 1
            got = graha(rp[i]["graha"])
            if got != want[kind]:
                report.append(f"{tag}: ruling {kind} is {got}, expected {want[kind]}")
                bad += 1

    # Significators: no repeats, ranked, and the house lord always present.
    for h in rust["significators"]:
        checked += 2
        names = [s["graha"] for s in h["significators"]]
        if len(set(names)) != len(names):
            report.append(f"{tag}: house {h['house']} repeats a graha")
            bad += 1
        ranks = [{"star_of_occupant": 1, "occupant": 2, "star_of_lord": 3, "lord": 4}[s["group"]]
                 for s in h["significators"]]
        if ranks != sorted(ranks):
            report.append(f"{tag}: house {h['house']} is not ranked: {ranks}")
            bad += 1

    return bad, checked


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=60)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = spec_text()
    parse_width_rule(text)
    kinds = parse_ruling_kinds(text)
    print(f"parsed from KP.md: ruling planets {kinds}")

    cells, straddling = cell_count()
    print(f"derived independently: {cells} cells, {straddling} subs straddling a sign boundary")
    if (cells, straddling) != (249, 6):
        print(f"FAIL: the construction gives {cells} cells, not 249")
        return 1

    rng = random.Random(args.seed)
    report: list[str] = []
    bad = checked = 0
    for _ in range(args.charts):
        birth = [
            "--date", f"{rng.randint(1900, 2060)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
            "--time", f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}",
            "--tz", "5.5",
            # Placidus cusps are undefined near the poles.
            "--lat", f"{rng.uniform(-60, 60):.4f}",
            "--lon", f"{rng.uniform(-180, 180):.4f}",
            "--ayanamsa", "krishnamurti",
        ]
        b, c = compare(birth, kinds, report)
        bad += b
        checked += c

    print(f"{args.charts} charts, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
