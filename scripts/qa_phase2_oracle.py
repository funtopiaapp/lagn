#!/usr/bin/env python3
"""Phase 2 N-version oracle.

An independent implementation of docs/phase2/DESIGN.md sections 3-7, written
from the specification text alone, diffed against the Rust kernel's CLI output
over thousands of random charts.

Two tables are *parsed out of DESIGN.md at run time* rather than retyped here:
the Ashtakavarga table (section 7) and the natural-relationship table
(section 4.3). That checks the Rust transcription against the specification
text itself instead of against a second hand-copy that could share an error.

Implementation choices deliberately differ from the Rust where the spec
allows: vargas compute degrees-within-sign separately rather than via the
single-floor grid, dignity uses an explicit decision list, and so on. Random
longitudes essentially never land within an ulp of a boundary, so the
difference is invisible to correct code and fatal to incorrect code.

What this cannot catch: a misreading of BPHS shared by the spec and both
implementations. That needs JHora comparison and astrologer review.

Usage:
    python3 scripts/qa_phase2_oracle.py [--charts N] [--seed S]
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
DESIGN = ROOT / "docs" / "phase2" / "DESIGN.md"

RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula",
        "vrischika", "dhanus", "makara", "kumbha", "meena"]
GRAHA = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu"]
SEVEN = GRAHA[:7]
NAME_TO_KEY = {"Sun": "sun", "Moon": "moon", "Mars": "mars", "Mercury": "mercury",
               "Jupiter": "jupiter", "Venus": "venus", "Saturn": "saturn"}
LORD = ["mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars",
        "jupiter", "saturn", "saturn", "jupiter"]

# ---------------------------------------------------------------------------
# Tables parsed from DESIGN.md
# ---------------------------------------------------------------------------

def parse_design():
    text = DESIGN.read_text()

    # Section 4.3 natural relationships: | Sun | Moon, Mars, Jupiter | Mercury | Venus, Saturn |
    natural = {}
    sec = text[text.index("**Natural (naisargika)"):text.index("**Temporary (tatkalika)")]
    for line in sec.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 4 and cells[0] in NAME_TO_KEY:
            g = NAME_TO_KEY[cells[0]]
            natural[g] = {}
            for rel, cell in zip(("friend", "neutral", "enemy"), cells[1:]):
                if cell.lower() == "none":
                    continue
                for other in cell.split(","):
                    natural[g][NAME_TO_KEY[other.strip()]] = rel
    assert len(natural) == 7 and all(len(v) == 6 for v in natural.values()), natural

    # Section 7 Ashtakavarga: | Sun | 1 2 4 ... | ... | **48** |
    bav = {}
    sec = text[text.index("| BAV of |"):text.index("**Chart-independent invariants.**")]
    cols = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "lagna"]
    for line in sec.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 10 and cells[0] in NAME_TO_KEY:
            g = NAME_TO_KEY[cells[0]]
            bav[g] = {c: [int(x) for x in cells[i + 1].split()] for i, c in enumerate(cols)}
            declared = int(cells[9].strip("*"))
            assert sum(len(v) for v in bav[g].values()) == declared, (g, declared)
    assert len(bav) == 7, bav
    return natural, bav


NATURAL, BAV_TABLE = parse_design()

# ---------------------------------------------------------------------------
# Section 3: vargas
# ---------------------------------------------------------------------------

def varga_sign(n: int, lon: float) -> int:
    s = int(lon // 30.0)
    d = lon - 30.0 * s
    odd = (s % 2 == 0)          # Mesha is odd-numbered
    mob = s % 3                 # 0 chara, 1 sthira, 2 dvisvabhava
    elem = s % 4                # 0 agni, 1 prithvi, 2 vayu, 3 jala
    if n == 1:
        return s
    if n == 30:
        if odd:
            bands = [(5, 0), (10, 10), (18, 8), (25, 2), (30, 6)]
        else:
            bands = [(5, 1), (12, 5), (20, 11), (25, 9), (30, 7)]
        for upper, sign in bands:
            if d < upper:
                return sign
        return bands[-1][1]
    k = min(int(d // (30.0 / n)), n - 1)
    if n == 2:
        first, second = (4, 3) if odd else (3, 4)
        return first if k == 0 else second
    if n == 3:
        return (s + 4 * k) % 12
    if n == 4:
        return (s + 3 * k) % 12
    start = {
        7: s if odd else s + 6,
        9: [s, s + 8, s + 4][mob],
        10: s if odd else s + 8,
        12: s,
        16: [0, 4, 8][mob],
        20: [0, 8, 4][mob],
        24: 4 if odd else 3,
        27: [0, 3, 6, 9][elem],
        40: 0 if odd else 6,
        45: [0, 4, 8][mob],
        60: s,
    }[n]
    return (start + k) % 12


VARGAS = [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60]

# ---------------------------------------------------------------------------
# Section 4: dignity and relationships
# ---------------------------------------------------------------------------

EXALT = {"sun": 0, "moon": 1, "mars": 9, "mercury": 5, "jupiter": 3, "venus": 11, "saturn": 6}
OWN = {"sun": [4], "moon": [3], "mars": [0, 7], "mercury": [2, 5], "jupiter": [8, 11],
       "venus": [1, 6], "saturn": [9, 10]}
MT = {"sun": (4, 0, 20), "moon": (1, 3, 30), "mars": (0, 0, 12), "mercury": (5, 15, 20),
      "jupiter": (8, 0, 10), "venus": (6, 0, 15), "saturn": (10, 0, 20)}
EXALT_ZONE_END = {"moon": 3, "mercury": 15}


def temporary(sign_a: int, sign_b: int) -> str:
    h = (sign_b - sign_a) % 12 + 1
    return "friend" if h in (2, 3, 4, 10, 11, 12) else "enemy"


COMPOUND = {("friend", "friend"): "great_friend", ("friend", "enemy"): "neutral",
            ("neutral", "friend"): "friend", ("neutral", "enemy"): "enemy",
            ("enemy", "friend"): "neutral", ("enemy", "enemy"): "great_enemy"}


def compound(a, b, sign_a, sign_b):
    return COMPOUND[(NATURAL[a][b], temporary(sign_a, sign_b))]


def dignity(g, varga, lon_of, settings):
    if g not in SEVEN:
        return None
    lon = lon_of[g]
    if varga == 1:
        s = int(lon // 30.0)
        d = lon - 30.0 * s
        zone = None
        if s == EXALT[g] and d < EXALT_ZONE_END.get(g, 30):
            zone = "exalted"
        elif s == (EXALT[g] + 6) % 12:
            zone = "debilitated"
        elif s == MT[g][0] and MT[g][1] <= d < MT[g][2]:
            zone = "moolatrikona"
        elif s in OWN[g]:
            zone = "own_sign"
    else:
        s = varga_sign(varga, lon)
        zone = None
        if s == EXALT[g]:
            zone = "exalted"
        elif s == (EXALT[g] + 6) % 12:
            zone = "debilitated"
        elif s == MT[g][0]:
            zone = "moolatrikona" if settings["varga_moolatrikona"] == "as_moolatrikona" else "own_sign"
        elif s in OWN[g]:
            zone = "own_sign"
    if zone:
        return zone
    lord = LORD[s]
    tv = 1 if settings["temporary_source"] == "rasi" else varga
    return compound(g, lord, varga_sign(tv, lon), varga_sign(tv, lon_of[lord]))

# ---------------------------------------------------------------------------
# Sections 5-6
# ---------------------------------------------------------------------------

def aspect_houses(g, node_setting):
    if g in ("sun", "moon", "mercury", "venus"):
        return [7]
    if g == "mars":
        return [4, 7, 8]
    if g == "jupiter":
        return [5, 7, 9]
    if g == "saturn":
        return [3, 7, 10]
    return {"none": [], "seventh": [7], "five_seven_nine": [5, 7, 9]}[node_setting]


def baladi(lon):
    s = int(lon // 30.0)
    d = lon - 30.0 * s
    order = ["bala", "kumara", "yuva", "vriddha", "mrita"]
    band = min(int(d // 6.0), 4)
    return order[band] if s % 2 == 0 else order[4 - band]


def jagradadi(g, dig, lon_of, settings):
    if dig is None:
        return None
    if dig in ("exalted", "moolatrikona", "own_sign"):
        return "jagrat"
    if settings["jagradadi_basis"] == "compound":
        return "swapna" if dig in ("great_friend", "friend", "neutral") else "sushupti"
    if dig == "debilitated":
        return "sushupti"
    lord = LORD[int(lon_of[g] // 30.0)]
    return "sushupti" if NATURAL[g][lord] == "enemy" else "swapna"


ORB = {"moon": 12, "mars": 17, "jupiter": 11, "saturn": 15}


def arc(a, b):
    d = (a - b) % 360.0
    return 360.0 - d if d > 180.0 else d


def combust(g, lon, retro, sun):
    if g in ("sun", "rahu", "ketu"):
        return False
    if g == "mercury":
        orb = 12 if retro else 14
    elif g == "venus":
        orb = 8 if retro else 10
    else:
        orb = ORB[g]
    return arc(lon, sun) <= orb

# ---------------------------------------------------------------------------
# Section 7
# ---------------------------------------------------------------------------

def ashtakavarga(sign_of):
    bav = []
    for p in SEVEN:
        row = [0] * 12
        for c, houses in BAV_TABLE[p].items():
            for h in houses:
                row[(sign_of[c] + h - 1) % 12] += 1
        bav.append(row)
    sav = [sum(bav[p][s] for p in range(7)) for s in range(12)]
    return bav, sav

# ---------------------------------------------------------------------------
# Harness
# ---------------------------------------------------------------------------

def lagn(*args):
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(out.stderr.strip())
    return json.loads(out.stdout)


def random_birth(rng):
    while True:
        y = rng.randint(1300, 2390)
        m = rng.randint(1, 12)
        d = rng.randint(1, 28)
        if y == 1582 and m == 10 and 5 <= d <= 14:
            continue
        return [
            "--date", f"{y}-{m:02d}-{d:02d}",
            "--time", f"{rng.randint(0,23):02d}:{rng.randint(0,59):02d}:{rng.randint(0,59):02d}",
            "--lat", f"{rng.uniform(-89, 89):.5f}",
            "--lon", f"{rng.uniform(-180, 180):.5f}",
            "--tz", rng.choice(["5.5", "5.3514", "0", "-5", "9", "4.85", "6.5"]),
        ]


VARIANT_FLAGS = {
    "node_aspects": ("--node-aspects", {"none": "none", "seventh": "seventh", "five_seven_nine": "five-seven-nine"}),
    "varga_moolatrikona": ("--varga-mt", {"as_moolatrikona": "as-moolatrikona", "as_own_sign": "as-own-sign"}),
    "temporary_source": ("--temporary", {"rasi": "rasi", "same_varga": "same-varga"}),
    "jagradadi_basis": ("--jagradadi", {"compound": "compound", "natural": "natural"}),
}
DEFAULTS = {"node_aspects": "none", "varga_moolatrikona": "as_moolatrikona",
            "temporary_source": "rasi", "jagradadi_basis": "compound"}


def check_chart(birth, settings, stats):
    flags = []
    for k, v in settings.items():
        flag, mapping = VARIANT_FLAGS[k]
        flags += [flag, mapping[v]]

    chart = lagn("chart", *birth, "--json")
    vargas = lagn("vargas", *birth, *flags, "--json")
    details = lagn("details", *birth, *flags, "--json")
    av = lagn("ashtakavarga", *birth, "--json")

    lon_of = {p["graha"]: p["longitude"] for p in chart["placements"]}
    retro = {p["graha"]: p["retrograde"] for p in chart["placements"]}
    asc = chart["lagna"]["longitude"]
    errs = []

    def expect(what, got, want):
        stats["checks"] += 1
        if got != want:
            errs.append(f"{what}: kernel={got!r} oracle={want!r}")

    # --- recorded settings ---
    expect("vargas.settings", vargas["settings"], settings)
    expect("details.settings", details["settings"], settings)

    # --- section 3 + 4 per varga ---
    for block in vargas["vargas"]:
        n = int(block["varga"][1:])
        lagna_sign = varga_sign(n, asc)
        expect(f"D{n} lagna", block["lagna"], RASI[lagna_sign])
        for g in GRAHA:
            cell = block["grahas"][g]
            s = varga_sign(n, lon_of[g])
            expect(f"D{n} {g} sign", cell["sign"], RASI[s])
            expect(f"D{n} {g} house", cell["house"], (s - lagna_sign) % 12 + 1)
            expect(f"D{n} {g} dignity", cell["dignity"], dignity(g, n, lon_of, settings))

    # --- sections 4.3, 5, 6 ---
    sun = lon_of["sun"]
    by_graha = {c["graha"]: c for c in details["conditions"]}
    for g in GRAHA:
        c = by_graha[g]
        s = int(lon_of[g] // 30.0)
        dig = dignity(g, 1, lon_of, settings)
        expect(f"{g} D1 dignity", c["dignity"], dig)
        expect(f"{g} baladi", c["baladi"], baladi(lon_of[g]))
        expect(f"{g} jagradadi", c["jagradadi"], jagradadi(g, dig, lon_of, settings))
        expect(f"{g} combust", c["combust"], combust(g, lon_of[g], retro[g], sun))
        want_aspects = [RASI[(s + h - 1) % 12] for h in aspect_houses(g, settings["node_aspects"])]
        expect(f"{g} aspects", c["aspects"], want_aspects)
        if abs(c["distance_from_sun"] - arc(lon_of[g], sun)) > 1e-9:
            errs.append(f"{g} distance_from_sun {c['distance_from_sun']} vs {arc(lon_of[g], sun)}")

    for r in details["relations"]:
        a, b = r["of"], r["toward"]
        expect(f"natural {a}->{b}", r["natural"], NATURAL[a][b])
        expect(f"compound {a}->{b}", r["compound"],
               compound(a, b, int(lon_of[a] // 30), int(lon_of[b] // 30)))
    stats["checks"] += 1
    if len(details["relations"]) != 42:
        errs.append(f"expected 42 directed relations, got {len(details['relations'])}")

    war = ["mars", "mercury", "jupiter", "venus", "saturn"]
    want_wars = sorted(
        [(a, b) for i, a in enumerate(war) for b in war[i + 1:] if arc(lon_of[a], lon_of[b]) <= 1.0]
    )
    got_wars = sorted((w[0], w[1]) for w in details["wars"])
    expect("wars", got_wars, want_wars)
    stats["wars"] += len(want_wars)

    # --- section 7 ---
    sign_of = {g: int(lon_of[g] // 30.0) for g in SEVEN}
    sign_of["lagna"] = int(asc // 30.0)
    bav, sav = ashtakavarga(sign_of)
    expect("BAV", av["bav"], bav)
    expect("SAV", av["sav"], sav)
    expect("SAV total", sum(av["sav"]), 337)
    for p in range(7):
        for s in range(12):
            expect(f"prastara popcount {SEVEN[p]} {RASI[s]}",
                   bin(av["prastara"][p][s]).count("1"), av["bav"][p][s])

    for gg in GRAHA:
        d = by_graha[gg]["dignity"]
        if d:
            stats["dignities"][d] = stats["dignities"].get(d, 0) + 1
    stats["combust"] += sum(1 for c in details["conditions"] if c["combust"])
    return errs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=20260920)
    a = ap.parse_args()
    if not LAGN.exists():
        print("build first: cargo build --release -p lagn-cli", file=sys.stderr)
        return 2

    rng = random.Random(a.seed)
    stats = {"checks": 0, "wars": 0, "combust": 0, "dignities": {}}
    failures = 0
    variant_runs = 0
    for i in range(a.charts):
        birth = random_birth(rng)
        # One chart in four runs under randomly chosen non-default variants.
        if i % 4 == 3:
            settings = {k: rng.choice(list(m.keys())) for k, (_, m) in VARIANT_FLAGS.items()}
            variant_runs += 1
        else:
            settings = dict(DEFAULTS)
        try:
            errs = check_chart(birth, settings, stats)
        except RuntimeError as e:
            print(f"  ERROR chart {i} {' '.join(birth)}: {e}")
            failures += 1
            continue
        if errs:
            failures += 1
            print(f"  FAIL chart {i}: {' '.join(birth)}  settings={settings}")
            for e in errs[:8]:
                print(f"       {e}")
        if (i + 1) % 500 == 0:
            print(f"  ... {i + 1} charts, {stats['checks']:,} checks, {failures} failing")

    print(f"\n{a.charts} charts ({variant_runs} under non-default variants), "
          f"{stats['checks']:,} individual checks")
    print(f"coverage: {stats['combust']} combust grahas, {stats['wars']} graha yuddhas, "
          f"dignities seen: {dict(sorted(stats['dignities'].items()))}")
    if failures:
        print(f"\n{failures} FAILING CHARTS")
        return 1
    print("\nkernel and oracle agree on every check")
    return 0


if __name__ == "__main__":
    sys.exit(main())
