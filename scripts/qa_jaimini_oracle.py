#!/usr/bin/env python3
"""Phase 13A N-version oracle: chara karakas, arudha padas, argala.

An independent implementation of docs/phase13/DESIGN.md sections 4 and 5,
diffed against the Rust kernel's `lagn jaimini --json` over random charts.

Three tables are *parsed out of DESIGN.md at run time* rather than retyped
here: the karaka rank order (section 4), the advancement rule for Rahu
(section 4) and the three argala pairs (section 5.2). That checks the Rust
transcription against the specification text itself rather than against a
second hand-copy that could share an error.

Implementation choices deliberately differ from the Rust where the
specification allows. Positions come from each graha's absolute longitude,
not from the kernel's `degrees_in_rasi`. Arudhas are computed by stepping
sign by sign in a loop rather than by modular arithmetic, and the exception
is tested by comparing against the bhava and its seventh directly rather
than by inspecting the count. Occupancy is built as a bucket map rather than
by filtering per sign. A random chart essentially never lands within an ulp
of a sign boundary, so these differences are invisible to correct code and
fatal to incorrect code.

What this cannot catch: a misreading of the Jaimini sutras shared by the
specification and both implementations. That needs astrologer review, which
is what the variant register in section 7 exists to collect.

Usage:
    python3 scripts/qa_jaimini_oracle.py [--charts N] [--seed S]
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
DESIGN = ROOT / "docs" / "phase13" / "DESIGN.md"

RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula",
        "vrischika", "dhanus", "makara", "kumbha", "meena"]
# Classical seven-lord scheme, which is V-13-6's default: no co-lordship, so
# Vrischika is Mars's and Kumbha is Saturn's.
LORD = ["mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars",
        "jupiter", "saturn", "saturn", "jupiter"]
ALL_GRAHA = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu"]

# ---------------------------------------------------------------------------
# Tables parsed from DESIGN.md
# ---------------------------------------------------------------------------

def design_text() -> str:
    if not DESIGN.exists():
        sys.exit(f"missing specification: {DESIGN}")
    return DESIGN.read_text()


def parse_karaka_order(text: str) -> list[str]:
    """Section 4's rank table, in rank order, as the kernel's own JSON keys.

    Rows look like: | 1 | AK | Atmakaraka | the self; ... |

    The third column is the one taken, not the abbreviation, because the
    kernel names each karaka by its stem: "Atmakaraka" serialises as "atma".
    Deriving the key from the full name ties this oracle to the word the
    specification uses rather than to a separate list of keys.
    """
    rows = re.findall(r"^\|\s*(\d)\s*\|\s*([A-Za-z]+)\s*\|\s*(\w+)karaka\s*\|", text, re.M)
    by_rank = {int(r): stem.lower() for r, _, stem in rows}
    order = [by_rank[i] for i in range(1, 9)]
    if len(order) != 8:
        sys.exit(f"could not parse eight karakas from section 4, got {order}")
    return order


def parse_rahu_is_reversed(text: str) -> bool:
    """Section 4's advancement table: does Rahu use `30 - degrees_in_rasi`?"""
    row = re.search(r"^\|\s*Rahu\s*\|\s*`([^`]+)`\s*\|", text, re.M)
    if not row:
        sys.exit("could not find Rahu's advancement row in section 4")
    return "30" in row.group(1) and "-" in row.group(1)


def parse_argala_pairs(text: str) -> list[tuple[str, int, int]]:
    """Section 5.2's table of (pair name, argala house, counter house).

    Rows look like: | 2nd | 12th | wealth |
    """
    rows = re.findall(r"^\|\s*(\d+)(?:st|nd|rd|th)\s*\|\s*(\d+)(?:st|nd|rd|th)\s*\|\s*(\w+)\s*\|", text, re.M)
    pairs = [(name, int(a), int(c)) for a, c, name in rows]
    if len(pairs) != 3:
        sys.exit(f"expected three argala pairs in section 5.2, parsed {pairs}")
    return pairs


# ---------------------------------------------------------------------------
# The oracle itself
# ---------------------------------------------------------------------------

def sign_of(longitude: float) -> int:
    """Sign index from an absolute sidereal longitude."""
    return int((longitude % 360.0) // 30.0)


def within(longitude: float) -> float:
    return (longitude % 360.0) - 30.0 * sign_of(longitude)


def nth_from(start: int, n: int) -> int:
    """The nth sign from `start`, counting `start` itself as 1.

    Deliberately a loop rather than modular arithmetic: the Rust does the
    arithmetic, so doing it differently here is the point of an N-version
    check.
    """
    i = start
    for _ in range(n - 1):
        i = (i + 1) % 12
    return i


def count_to(start: int, target: int) -> int:
    """Signs from `start` to `target`, counting `start` as 1, so 1..12."""
    n = 1
    i = start
    while i != target:
        i = (i + 1) % 12
        n += 1
    return n


def karakas(chart: dict, order: list[str], rahu_reversed: bool) -> list[dict]:
    candidates = []
    for p in chart["placements"]:
        g = p["graha"]
        if g == "ketu":
            continue  # V-13-3
        deg = within(p["longitude"])
        adv = (30.0 - deg) if (g == "rahu" and rahu_reversed) else deg
        candidates.append((g, sign_of(p["longitude"]), adv))

    # Descending advancement; ties by the natural order, which is the order
    # the chart's own placements come in (V-13-4).
    natural = {g: i for i, g in enumerate(ALL_GRAHA)}
    candidates.sort(key=lambda c: (-c[2], natural[c[0]]))
    return [
        {"karaka": order[i], "graha": g, "rasi": RASI[s], "advancement": adv}
        for i, (g, s, adv) in enumerate(candidates)
    ]


def padas(chart: dict) -> list[dict]:
    lagna_sign = sign_of(chart["lagna"]["longitude"])
    where = {p["graha"]: sign_of(p["longitude"]) for p in chart["placements"]}

    out = []
    for bhava in range(1, 13):
        # Whole-sign houses: the nth house is the nth sign from the lagna.
        bhava_sign = nth_from(lagna_sign, bhava)
        lord = LORD[bhava_sign]
        lord_sign = where[lord]

        n = count_to(bhava_sign, lord_sign)
        raw = nth_from(lord_sign, n)

        # The exception, tested against the bhava and its seventh directly
        # rather than by looking at the count.
        seventh = nth_from(bhava_sign, 7)
        adjusted = raw == bhava_sign or raw == seventh
        pada = nth_from(raw, 10) if adjusted else raw

        out.append({
            "bhava": bhava, "bhava_rasi": RASI[bhava_sign], "lord": lord,
            "lord_rasi": RASI[lord_sign], "count": n, "raw": RASI[raw],
            "rasi": RASI[pada], "adjusted": adjusted,
        })
    return out


def argala(chart: dict, pairs: list[tuple[str, int, int]]) -> list[dict]:
    # A bucket map, built once, rather than a filter per sign.
    buckets: dict[int, list[str]] = {i: [] for i in range(12)}
    for g in ALL_GRAHA:  # V-13-5: Ketu included
        p = next(x for x in chart["placements"] if x["graha"] == g)
        buckets[sign_of(p["longitude"])].append(g)

    out = []
    for s in range(12):
        entry = {"rasi": RASI[s], "pairs": []}
        for name, ah, ch in pairs:
            a_sign, c_sign = nth_from(s, ah), nth_from(s, ch)
            na, nc = len(buckets[a_sign]), len(buckets[c_sign])
            if na == 0:
                verdict = "none"
            elif na > nc:
                verdict = "stands"
            elif na == nc:
                verdict = "neutralised"
            else:
                verdict = "overcome"
            entry["pairs"].append({
                "kind": name, "argala_rasi": RASI[a_sign], "counter_rasi": RASI[c_sign],
                "argala_grahas": buckets[a_sign], "counter_grahas": buckets[c_sign],
                "verdict": verdict,
            })
        out.append(entry)
    return out


# ---------------------------------------------------------------------------
# Diffing
# ---------------------------------------------------------------------------

def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], order, rahu_reversed, pairs, report: list[str]) -> int:
    chart = lagn("chart", *birth, "--json")
    rust = lagn("jaimini", *birth, "--json")
    bad = 0
    tag = " ".join(birth)

    mine = karakas(chart, order, rahu_reversed)
    theirs = rust["karakas"]["assigned"]
    for a, b in zip(mine, theirs):
        if a["karaka"] != b["karaka"] or a["graha"] != b["graha"]:
            report.append(f"{tag}: karaka {a['karaka']} is {a['graha']} here, {b['graha']} ({b['karaka']}) there")
            bad += 1
        elif abs(a["advancement"] - b["advancement"]) > 1e-9:
            report.append(f"{tag}: {a['graha']} advancement {a['advancement']} vs {b['advancement']}")
            bad += 1

    for a, b in zip(padas(chart), rust["padas"]):
        for k in ("bhava", "bhava_rasi", "lord", "lord_rasi", "count", "raw", "rasi", "adjusted"):
            if a[k] != b[k]:
                report.append(f"{tag}: pada A{a['bhava']} {k}: {a[k]} vs {b[k]}")
                bad += 1

    for a, b in zip(argala(chart, pairs), rust["argala"]):
        if a["rasi"] != b["rasi"]:
            report.append(f"{tag}: argala sign {a['rasi']} vs {b['rasi']}")
            bad += 1
            continue
        for pa, pb in zip(a["pairs"], b["pairs"]):
            for k in ("kind", "argala_rasi", "counter_rasi", "verdict"):
                if pa[k] != pb[k]:
                    report.append(f"{tag}: argala {a['rasi']} {pa['kind']} {k}: {pa[k]} vs {pb[k]}")
                    bad += 1
            if sorted(pa["argala_grahas"]) != sorted(pb["argala_grahas"]):
                report.append(f"{tag}: argala {a['rasi']} {pa['kind']} occupants differ")
                bad += 1
    return bad


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=120)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = design_text()
    order = parse_karaka_order(text)
    rahu_reversed = parse_rahu_is_reversed(text)
    pairs = parse_argala_pairs(text)
    print(f"parsed from DESIGN.md: karakas {order}")
    print(f"parsed from DESIGN.md: Rahu reversed = {rahu_reversed}; argala pairs {pairs}")

    rng = random.Random(args.seed)
    report: list[str] = []
    bad = 0
    for _ in range(args.charts):
        birth = [
            "--date", f"{rng.randint(1350, 2380)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
            "--time", f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}",
            "--tz", "5.5",
            "--lat", f"{rng.uniform(-65, 65):.4f}",
            "--lon", f"{rng.uniform(-180, 180):.4f}",
        ]
        bad += compare(birth, order, rahu_reversed, pairs, report)

    cells = args.charts * (8 + 12 * 8 + 12 * 3 * 5)
    print(f"{args.charts} charts, {cells} compared values")
    print(f"agree: {cells - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
