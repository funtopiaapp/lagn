#!/usr/bin/env python3
"""Phase 13F/13G N-version oracle: the annual chart, Muntha and kaksha.

An independent implementation of docs/phase13/VARSHA-KAKSHA.md sections 2 to
4, diffed against `lagn varsha --json`.

Two things are parsed out of the specification at run time: the kaksha owner
order (section 4) and the Muntha rule (section 3).

Implementation choices deliberately differ. The solar return is not re-solved
here - that would be the same bisection twice - it is *verified* instead, by
asking the kernel for the Sun at the moment it returned and checking it
against the natal Sun. That is the stronger check anyway: a root-finder is
right exactly when its answer satisfies the equation. Muntha and the kaksha
index are recomputed from scratch.

Usage:
    python3 scripts/qa_varsha_oracle.py [--charts N] [--seed S]
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
SPEC = ROOT / "docs" / "phase13" / "VARSHA-KAKSHA.md"

RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula",
        "vrischika", "dhanus", "makara", "kumbha", "meena"]
# A milliarcsecond, in degrees.
MAS = 1.0 / 3_600_000.0


def spec_text() -> str:
    if not SPEC.exists():
        sys.exit(f"missing specification: {SPEC}")
    return SPEC.read_text()


def parse_kaksha_order(text: str) -> list[str]:
    """Section 4's kaksha table, owners in order from the start of the sign."""
    start = text.index("| Kaksha | Owner |")
    end = text.index("A transiting graha stands", start)
    rows = re.findall(r"^\|\s*([1-8])\s*\|\s*(\w+)\s*\|", text[start:end], re.M)
    by_n = {int(n): o for n, o in rows}
    if len(by_n) != 8:
        sys.exit(f"expected eight kakshas in section 4, parsed {sorted(by_n)}")
    return [by_n[i] for i in range(1, 9)]


def parse_muntha_rule(text: str) -> bool:
    if not re.search(r"Muntha sign for age n\s*=\s*natal lagna \+ n signs", text):
        sys.exit("section 3's Muntha rule is not the one this oracle implements")
    return True


# The kernel serialises Contributor in snake_case; the spec names grahas in
# Sanskrit. Mapped so the oracle can read the document it implements.
OWNER = {"Shani": "saturn", "Guru": "jupiter", "Kuja": "mars", "Surya": "sun",
         "Shukra": "venus", "Budha": "mercury", "Chandra": "moon", "Lagna": "lagna"}


def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def compare(birth: list[str], age: int, order: list[str], report: list[str]) -> tuple[int, int]:
    natal = lagn("chart", *birth, "--json")
    v = lagn("varsha", *birth, "--age", str(age), "--json")
    tag = f"{' '.join(birth)} age {age}"
    bad = checked = 0

    # The solar return, verified rather than re-solved: the Sun in the annual
    # chart must equal the natal Sun.
    natal_sun = next(p["longitude"] for p in natal["placements"] if p["graha"] == "sun")
    annual_sun = next(p["longitude"] for p in v["chart"]["placements"] if p["graha"] == "sun")
    checked += 1
    d = abs(annual_sun - natal_sun) % 360.0
    d = min(d, 360.0 - d)
    if d > MAS:
        report.append(f"{tag}: annual Sun is {d} degrees from the natal Sun")
        bad += 1

    # Muntha: one sign per completed year from the natal lagna.
    checked += 2
    natal_lagna = int(natal["lagna"]["longitude"] % 360.0 // 30.0)
    want = RASI[(natal_lagna + age) % 12]
    if v["muntha"].lower() != want:
        report.append(f"{tag}: Muntha {v['muntha']} vs {want}")
        bad += 1
    annual_lagna = int(v["chart"]["lagna"]["longitude"] % 360.0 // 30.0)
    want_house = ((RASI.index(v["muntha"].lower()) - annual_lagna) % 12) + 1
    if v["muntha_house"] != want_house:
        report.append(f"{tag}: Muntha house {v['muntha_house']} vs {want_house}")
        bad += 1

    # Every kaksha reading, recomputed: which eighth of the sign, who owns
    # it, and whether the natal Ashtakavarga backs it. The verdict is checked
    # against the kernel's own contributor list for that sign, obtained from
    # the ashtakavarga command - a different code path from the one that
    # produced the verdict.
    av = lagn("ashtakavarga", *birth, "--json")
    seven = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"]
    for k in v["kaksha"]:
        within = (k["longitude"] % 360.0) % 30.0
        i = min(int(within / 3.75), 7)
        checked += 3

        if k["kaksha"] != i + 1:
            report.append(f"{tag}: {k['graha']} kaksha {k['kaksha']} vs {i + 1}")
            bad += 1
        if k["owner"] != OWNER[order[i]]:
            report.append(f"{tag}: {k['graha']} owner {k['owner']} vs {OWNER[order[i]]}")
            bad += 1

        # The verdict: the owner's bit in the prastara for that sign.
        row = seven.index(k["graha"])
        sign = RASI.index(k["rasi"].lower())
        mask = av["prastara"][row][sign]
        # Contributor::ALL order, which is the bit order.
        bits = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "lagna"]
        want = bool(mask & (1 << bits.index(k["owner"])))
        if k["supported"] != want:
            report.append(f"{tag}: {k['graha']} supported={k['supported']} vs {want}")
            bad += 1

        checked += 1
        if k["bindus"] != av["bav"][row][sign]:
            report.append(f"{tag}: {k['graha']} bindus {k['bindus']} vs {av['bav'][row][sign]}")
            bad += 1

    checked += 1
    if len(v["kaksha"]) != 7:
        report.append(f"{tag}: {len(v['kaksha'])} kaksha readings, expected 7")
        bad += 1

    return bad, checked


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=40)
    ap.add_argument("--seed", type=int, default=13)
    args = ap.parse_args()

    if not LAGN.exists():
        sys.exit(f"missing {LAGN}; run: cargo build --release -p lagn-cli")

    text = spec_text()
    parse_muntha_rule(text)
    order = parse_kaksha_order(text)
    print(f"parsed from VARSHA-KAKSHA.md: kaksha owners {order}")

    rng = random.Random(args.seed)
    report: list[str] = []
    bad = checked = 0
    for _ in range(args.charts):
        birth = [
            "--date", f"{rng.randint(1920, 2010)}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}",
            "--time", f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}",
            "--tz", "5.5",
            "--lat", f"{rng.uniform(-55, 55):.4f}",
            "--lon", f"{rng.uniform(-180, 180):.4f}",
        ]
        for age in (0, rng.randint(1, 40), rng.randint(41, 85)):
            b, c = compare(birth, age, order, report)
            bad += b
            checked += c

    print(f"{args.charts} charts x 3 ages, {checked} compared values")
    print(f"agree: {checked - bad}   disagree: {bad}")
    for line in report[:40]:
        print("  " + line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
