#!/usr/bin/env python3
"""Generate a golden fixture from birth data.

A fixture records our current output for a chart so that future changes cannot
silently alter it. Freshly generated fixtures are *regression locks*, not
proof of correctness: `verified_against` is null until a human has checked the
chart in JHora and filled it in. See tests/golden/README.md.

Usage:
    python3 scripts/make_golden.py --id chennai-1985 \
        --date 1985-06-21 --time 14:30 --lat 13.0827 --lon 80.2707 --tz 5.5 \
        --place Chennai --description "baseline South Indian birth"
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = ROOT / "target" / "release" / "lagn"
GOLDEN = ROOT / "tests" / "golden"


def lagn(*args: str) -> dict:
    out = subprocess.run([str(LAGN), *args], cwd=ROOT, capture_output=True, text=True)
    if out.returncode != 0:
        raise SystemExit(f"lagn failed: {out.stderr.strip()}")
    return json.loads(out.stdout)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--id", required=True)
    ap.add_argument("--date", required=True)
    ap.add_argument("--time", required=True)
    ap.add_argument("--lat", required=True)
    ap.add_argument("--lon", required=True)
    ap.add_argument("--tz", required=True)
    ap.add_argument("--place", default="")
    ap.add_argument("--description", default="")
    ap.add_argument("--ayanamsa", default="lahiri")
    ap.add_argument("--node", default="mean")
    ap.add_argument("--year-length", default="julian")
    a = ap.parse_args()

    common = [
        "--date", a.date, "--time", a.time, "--lat", a.lat, "--lon", a.lon,
        "--tz", a.tz, "--place", a.place, "--ayanamsa", a.ayanamsa,
        "--node", a.node, "--year-length", a.year_length,
    ]
    chart = lagn("chart", *common, "--json")
    dasha = lagn("dasha", *common, "--json")

    grahas = {}
    for p in chart["placements"]:
        grahas[p["graha"]] = {
            "rasi": p["rasi"],
            "degrees_in_rasi": round(p["degrees_in_rasi"], 6),
            "nakshatra": p["nakshatra"]["nakshatra"],
            "pada": p["nakshatra"]["pada"],
            "house": p["house"],
            "navamsa": p["navamsa"],
            "retrograde": p["retrograde"],
        }

    fixture = {
        "id": a.id,
        "description": a.description,
        # Fill these in by hand once the chart has been checked in JHora.
        "verified_against": None,
        "verified_by": None,
        "verified_on": None,
        "notes": "",
        "input": {
            "date": a.date, "time": a.time,
            "latitude": float(a.lat), "longitude": float(a.lon),
            "utc_offset_hours": float(a.tz), "place": a.place,
        },
        "settings": {
            "ayanamsa": a.ayanamsa, "node": a.node,
            "house_system": "whole_sign", "year_length": a.year_length,
        },
        "expected": {
            "jd_ut": chart["jd_ut"],
            "ayanamsa_degrees": round(chart["ayanamsa_value"], 6),
            "lagna": {
                "rasi": chart["lagna"]["rasi"],
                "degrees_in_rasi": round(chart["lagna"]["degrees_in_rasi"], 6),
                "nakshatra": chart["lagna"]["nakshatra"]["nakshatra"],
                "pada": chart["lagna"]["nakshatra"]["pada"],
                "navamsa": chart["lagna"]["navamsa"],
            },
            "grahas": grahas,
            "vimshottari": {
                "janma_nakshatra": dasha["janma_nakshatra"],
                "birth_lord": dasha["birth_lord"],
                "balance_years": round(dasha["balance_years"], 6),
                "mahadashas": [
                    {"lord": m["lord"], "start_jd": round(m["start_jd"], 6),
                     "end_jd": round(m["end_jd"], 6)}
                    for m in dasha["mahadashas"]
                ],
            },
        },
    }

    GOLDEN.mkdir(parents=True, exist_ok=True)
    path = GOLDEN / f"{a.id}.json"
    path.write_text(json.dumps(fixture, indent=2) + "\n")
    print(f"wrote {path.relative_to(ROOT)}  (verified_against: null - check in JHora)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
