#!/usr/bin/env python3
"""Phase 13B N-version oracle: Chara dasha.

An independent implementation of docs/phase13/CHARA-DASHA.md sections 3, 4
and 5, diffed against the Rust kernel's `lagn chara --json` over random
charts.

Two things are *parsed out of the specification at run time* rather than
retyped here: which signs count as odd and therefore run direct (section 3's
direction table), and the three worked cases of the length rule (section 4's
case table). That checks the Rust transcription against the specification
text instead of against a second hand-copy that could share an error.

Implementation choices deliberately differ from the Rust. Sign order is built
by repeated stepping rather than by `nth_from`; the count is derived by
walking sign to sign until the lord's sign is reached rather than by modular
arithmetic; period boundaries are accumulated from birth rather than carried
in a running variable, so a drift bug in either shows up as a disagreement.

What this cannot catch: a misreading of the Jaimini rule shared by the
specification and both implementations. That is what the variant register in
section 6 exists to collect, and no variant in it is signed off.

Usage:
    python3 scripts/qa_chara_oracle.py [--charts N] [--seed S]
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
SPEC = ROOT / "docs" / "phase13" / "CHARA-DASHA.md"

RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula",
        "vrischika", "dhanus", "makara", "kumbha", "meena"]
# The classical seven-lord scheme, which is V-13-12's default.
LORD = ["mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars",
        "jupiter", "saturn", "saturn", "jupiter"]
YEAR_DAYS = {"julian": 365.25, "gregorian": 365.2425, "tropical": 365.242190,
             "sidereal_solar": 365.256363, "savana": 360.0}
# A second, in days. Boundaries closer than this are the same boundary.
SECOND = 1.0 / 86400.0


# ---------------------------------------------------------------------------
# Parsed from the specification
# ---------------------------------------------------------------------------

def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_odd_signs(text: str) -> set[int]:
    """Section 3's direction table: the signs it names as odd run direct."""
    row = re.search(r"^\|\s*odd \(([^)]+)\)\s*\|\s*zodiacal", text, re.M)
    if not row:
        sys.exit("could not find section 3's odd-sign row")
    names = [n.strip().lower() for n in row.group(1).split(",")]
    idx = set()
    for n in names:
        if n not in RASI:
            sys.exit(f"section 3 names a sign the oracle does not know: {n}")
        idx.add(RASI.index(n))
    if len(idx) != 6:
        sys.exit(f"expected six odd signs, parsed {sorted(idx)}")
    return idx


def parse_length_cases(text: str) -> dict[int, float]:
    """Section 4's case table: count -> years, for the cases it states."""
    rows = re.findall(r"^\|\s*lord ([^|]+?)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|", text, re.M)
    cases = {int(c): float(y) for _, c, y in rows}
    if cases.get(1) != 12.0:
        sys.exit(f"section 4 must state that a count of 1 gives 12 years; parsed {cases}")
    return cases


# ---------------------------------------------------------------------------
# The oracle
# ---------------------------------------------------------------------------

def step(i: int, direct: bool) -> int:
    return (i + 1) % 12 if direct else (i - 1) % 12


def sequence(lagna: int, direct: bool) -> list[int]:
    """The twelve signs in visiting order, by repeated stepping."""
    out, i = [lagna], lagna
    for _ in range(11):
        i = step(i, direct)
        out.append(i)
    return out


def count_to(start: int, target: int, direct: bool) -> int:
    """Walk one sign at a time until the target is reached, counting start as 1."""
    n, i = 1, start
    while i != target:
        i = step(i, direct)
        n += 1
    return n


def years_for(sign: int, where: dict[str, int], odd: set[int], cases: dict[int, float]) -> tuple[int, float]:
    lord_sign = where[LORD[sign]]
    direct = sign in odd           # the sign's own parity, not the lagna's
    n = count_to(sign, lord_sign, direct)
    years = cases[1] if n == 1 else float(n - 1)
    return n, years


def expected(chart: dict, odd: set[int], cases: dict[int, float], horizon: float) -> dict:
    lagna = int(chart["lagna"]["longitude"] % 360.0 // 30.0)
    direct = lagna in odd
    where = {p["graha"]: int(p["longitude"] % 360.0 // 30.0) for p in chart["placements"]}
    year_days = YEAR_DAYS[chart["settings"]["year_length"]]
    birth = chart["jd_ut"]

    order = sequence(lagna, direct)
    lengths = [(s, *years_for(s, where, odd, cases)) for s in order]

    # Boundaries accumulated from birth rather than from a running cursor, so
    # a drift bug in either implementation surfaces as a disagreement.
    periods, elapsed, cycle = [], 0.0, 1
    while birth + elapsed * year_days < birth + horizon * year_days:
        for sign, _n, y in lengths:
            start = birth + elapsed * year_days
            end = birth + (elapsed + y) * year_days
            children = []
            for k in range(12):
                cs = start + (end - start) * k / 12.0
                ce = end if k == 11 else start + (end - start) * (k + 1) / 12.0
                ci = sign
                for _ in range(k):
                    ci = step(ci, direct)
                children.append({"rasi": RASI[ci], "start_jd": cs, "end_jd": ce})
            periods.append({"rasi": RASI[sign], "start_jd": start, "end_jd": end,
                            "cycle": cycle, "children": children})
            elapsed += y
        cycle += 1

    return {
        "lagna": RASI[lagna],
        "direction": "direct" if direct else "reverse",
        "lengths": [{"rasi": RASI[s], "count": n, "years": y} for s, n, y in lengths],
        "periods": periods,
    }


# ---------------------------------------------------------------------------
# Diffing
# ---------------------------------------------------------------------------

def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], odd, cases, report: list[str]) -> tuple[int, int]:
    chart = lagn("chart", *birth, "--json")
    rust = lagn("chara", *birth, "--json")
    mine = expected(chart, odd, cases, 120.0)
    tag = " ".join(birth)
    bad = checked = 0

    for k in ("lagna", "direction"):
        checked += 1
        if mine[k] != rust[k]:
            report.append(f"{tag}: {k}: {mine[k]} vs {rust[k]}")
            bad += 1

    for a, b in zip(mine["lengths"], rust["lengths"]):
        for k in ("rasi", "count", "years"):
            checked += 1
            if a[k] != b[k]:
                report.append(f"{tag}: {a['rasi']} {k}: {a[k]} vs {b[k]}")
                bad += 1

    checked += 1
    if len(mine["periods"]) != len(rust["periods"]):
        report.append(f"{tag}: {len(mine['periods'])} periods vs {len(rust['periods'])}")
        bad += 1
        return bad, checked

    for a, b in zip(mine["periods"], rust["periods"]):
        checked += 4
        if a["rasi"] != b["rasi"]:
            report.append(f"{tag}: period sign {a['rasi']} vs {b['rasi']}")
            bad += 1
        if a["cycle"] != b["cycle"]:
            report.append(f"{tag}: {a['rasi']} cycle {a['cycle']} vs {b['cycle']}")
            bad += 1
        for k in ("start_jd", "end_jd"):
            if abs(a[k] - b[k]) > SECOND:
                report.append(f"{tag}: {a['rasi']} {k} differs by {(a[k] - b[k]) * 86400:.3f} s")
                bad += 1
        for ca, cb in zip(a["children"], b["children"]):
            checked += 3
            if ca["rasi"] != cb["rasi"]:
                report.append(f"{tag}: {a['rasi']} antar {ca['rasi']} vs {cb['rasi']}")
                bad += 1
            for k in ("start_jd", "end_jd"):
                if abs(ca[k] - cb[k]) > SECOND:
                    report.append(f"{tag}: {a['rasi']}/{ca['rasi']} {k} differs by {(ca[k] - cb[k]) * 86400:.3f} s")
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
    odd, cases = parse_odd_signs(text), parse_length_cases(text)
    print(f"parsed from CHARA-DASHA.md: odd signs {sorted(RASI[i] for i in odd)}")
    print(f"parsed from CHARA-DASHA.md: length cases {cases}")

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
        b, c = compare(birth, odd, cases, report)
        bad += b
        checked += c

    print(f"{args.charts} charts, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
