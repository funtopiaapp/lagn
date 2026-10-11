#!/usr/bin/env python3
"""Phase 13D N-version oracle: Yogi, Avayogi and the Yoga sphuta.

An independent implementation of docs/phase13/YOGI.md section 2, diffed
against `lagn yogi --json`.

The offset and the Avayogi count are parsed out of the specification at run
time rather than retyped.

Implementation choices deliberately differ. The offset is built here from
degrees and arcminutes (93 + 20/60) where the kernel builds it as seven
nakshatra spans, so the identity that the two agree is tested rather than
assumed. The nakshatra of a longitude is found by integer division on an
exact Fraction, not by the kernel's floor.

The invariant this oracle also asserts - that the Avayogi's lord is five
places past the Yogi's in the nine-lord cycle - needs no reference output and
holds for every chart, which is why this group was safe to build.

Usage:
    python3 scripts/qa_yogi_oracle.py [--charts N] [--seed S]
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
SPEC = ROOT / "docs" / "phase13" / "YOGI.md"

NAK = Fraction(40, 3)
CYCLE = ["Ketu", "Shukra", "Surya", "Chandra", "Kuja", "Rahu", "Guru", "Shani", "Budha"]
SERDE = {"sun": "Surya", "moon": "Chandra", "mars": "Kuja", "mercury": "Budha",
         "jupiter": "Guru", "venus": "Shukra", "saturn": "Shani",
         "rahu": "Rahu", "ketu": "Ketu"}
ARCSEC = 1.0 / 3600.0


def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_offset(text: str) -> Fraction:
    """Section 2's offset, as degrees and arcminutes."""
    m = re.search(r"`Sun \+ Moon \+ (\d+):(\d+)`", text)
    if not m:
        sys.exit("section 2's Yoga sphuta offset is not in the form this oracle reads")
    off = Fraction(int(m.group(1))) + Fraction(int(m.group(2)), 60)
    # The identity the specification rests on: the offset is a whole number of
    # nakshatras. Checked before anything is compared.
    if off / NAK != 7:
        sys.exit(f"the offset {off} is not seven nakshatras; the spec's own identity fails")
    return off


def parse_avayogi_count(text: str) -> int:
    m = re.search(r"the (\w+)th nakshatra from the Yogi's", text)
    words = {"six": 6, "sixt": 6, "eigh": 8}
    if not m:
        sys.exit("section 2's Avayogi rule is not in the form this oracle reads")
    w = m.group(1).lower()
    for k, v in words.items():
        if w.startswith(k):
            return v
    sys.exit(f"section 2 names an Avayogi count this oracle does not know: {w!r}")


def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], off: Fraction, count: int, report: list[str]) -> tuple[int, int]:
    chart = lagn("chart", *birth, "--json")
    rust = lagn("yogi", *birth, "--json")
    tag = " ".join(birth)
    bad = checked = 0

    sun = next(p["longitude"] for p in chart["placements"] if p["graha"] == "sun")
    moon = next(p["longitude"] for p in chart["placements"] if p["graha"] == "moon")

    lon = (Fraction(sun).limit_denominator(10**12)
           + Fraction(moon).limit_denominator(10**12) + off) % 360
    k = int(lon // NAK)

    checked += 1
    if abs(float(lon) - rust["sphuta"]["longitude"]) > ARCSEC:
        report.append(f"{tag}: sphuta {rust['sphuta']['longitude']} vs {float(lon)}")
        bad += 1

    checked += 1
    yogi = CYCLE[k % 9]
    if SERDE.get(rust["yogi"], rust["yogi"]) != yogi:
        report.append(f"{tag}: Yogi {rust['yogi']} vs {yogi}")
        bad += 1

    checked += 1
    avy_nak = (k + count - 1) % 27
    avayogi = CYCLE[avy_nak % 9]
    if SERDE.get(rust["avayogi"], rust["avayogi"]) != avayogi:
        report.append(f"{tag}: Avayogi {rust['avayogi']} vs {avayogi}")
        bad += 1

    # The invariant, asserted here too: five places on in the nine-cycle.
    checked += 1
    shift = (CYCLE.index(avayogi) - CYCLE.index(yogi)) % 9
    if shift != 5:
        report.append(f"{tag}: the cycle shift is {shift}, not 5")
        bad += 1

    return bad, checked


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=150)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = spec_text()
    off = parse_offset(text)
    count = parse_avayogi_count(text)
    print(f"parsed from YOGI.md: offset {off} degrees = {off / NAK} nakshatras; Avayogi is the {count}th")

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
        ]
        b, c = compare(birth, off, count, report)
        bad += b
        checked += c

    print(f"{args.charts} charts, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
