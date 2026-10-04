#!/usr/bin/env python3
"""Regenerate tests/validation13.json from the built CLI.

The thirteen subjects come from the independent validation pass of
2026-10-01/02 (tests/QA-VALIDATION-1.0.md). They are real public figures with
published birth data, so only the anonymised subject codes from that report
appear here - no names, in this file or the fixture. The code map, if one is
needed, stays outside the repository.

What the fixture is for: every subject below reproduced exactly on 1.0, and
128 of the 130 topic scores in the report matched the engine cell for cell.
That makes these thirteen charts the one corpus where engine output is tied to
externally researched lives, so a change in any of these numbers is a change
worth noticing. The fixture locks the numbers; it does not bless them. A score
here being "expected" means only that 1.0 produced it - several are wrong
about the documented life, which is what the recalibration work addresses.

    cargo build --release -p lagn-cli
    python3 scripts/make_validation13.py

Rerun it when a deliberate change moves these scores, and say in the commit
which subjects moved and why.
"""

import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CLI = ROOT / "target" / "release" / "lagn"
OUT = ROOT / "tests" / "validation13.json"

# Age ranges are the ones each topic declares in corpus/topics.json. The web
# app sends no override, so these are what production actually evaluates - the
# CLI's own 18/45 default would pin windows no user ever sees.
AGES = {
    "marriage": (18, 45), "career": (18, 65), "wealth": (18, 80),
    "education": (4, 30), "health": (0, 90), "vitality": (0, 90),
    "progeny": (20, 45), "parents": (0, 60), "later_life": (55, 85),
    "past_life": (0, 90),
}

# Inputs exactly as the validation report's Appendix A entered them, including
# the places it had to substitute and the noon defaults it had to assume.
# `time_confidence` is that report's provenance grade, not a claim of truth:
# A certificate, B published, C quoted/disputed, D unknown (noon default).
SUBJECTS = [
    # code, descriptor,                              date,         time,  lat,       lon,       tz,  sex,      conf
    ("01", "M team-sport athlete, b. 1973",          "1973-04-24", "13:14", 19.07283, 72.88261, 5.5, "male",   "B"),
    ("02", "M film actor turned politician, b. 1974", "1974-06-22", "12:00", 13.08784, 80.27847, 5.5, "male",   "D"),
    ("03", "M veteran film actor, b. 1942",          "1942-10-11", "16:00", 25.44478, 81.84322, 5.5, "male",   "B"),
    ("04", "M film actor, b. 1965",                  "1965-11-02", "06:35", 28.61,    77.21,    5.5, "male",   "C"),
    ("05", "M politician, b. 1950",                  "1950-09-17", "11:00", 23.78593, 72.63893, 5.5, "male",   "C+"),
    ("06", "M team-sport athlete, b. 1988",          "1988-11-05", "07:30", 28.65195, 77.23149, 5.5, "male",   "C-"),
    ("07", "M team-sport athlete, b. 1981",          "1981-07-07", "11:00", 23.34316, 85.30940, 5.5, "male",   "C"),
    ("08", "M film actor, b. 1954",                  "1954-11-07", "12:00",  9.37158, 78.83077, 5.5, "male",   "D"),
    ("09", "M film actor, b. 1974",                  "1974-01-10", "06:00", 19.07283, 72.88261, 5.5, "male",   "C"),
    ("10", "F film actress, b. 1986",                "1986-01-05", "12:00", 55.67594, 12.56553, 1.0, "female", "D"),
    ("11", "M team-sport athlete, b. 1997",          "1997-10-04", "12:00", 29.86632, 77.89118, 5.5, "male",   "D"),
    ("12", "F racket-sport athlete, b. 1986",        "1986-11-15", "12:00", 19.07283, 72.88261, 5.5, "female", "D"),
    ("13", "M politician, b. 1968",                  "1968-08-16", "12:00", 28.90909, 75.61469, 5.5, "male",   "D"),
]

# Subject-08's birth town is Paramakudi. The report could not find it in the
# place search and substituted the nearest selectable town, so that is what
# these coordinates are. The town is in the data as "Paramagudi" (9.54633,
# 78.59070); the search could not reach it. Kept as-entered so the fixture
# matches what was actually graded - see defect D8.


def run(args):
    env = dict(os.environ, LAGN_EPHE_PATH=str(ROOT / "ephe"))
    r = subprocess.run([str(CLI), *args], capture_output=True, text=True, env=env)
    if r.returncode != 0:
        sys.exit(f"lagn {' '.join(args)} failed:\n{r.stderr}")
    return json.loads(r.stdout)


def main():
    if not CLI.exists():
        sys.exit("build the CLI first: cargo build --release -p lagn-cli")

    subjects = []
    for code, desc, date, time, lat, lon, tz, sex, conf in SUBJECTS:
        birth = ["--date", date, "--time", time, "--lat", str(lat), "--lon", str(lon), "--tz", str(tz)]
        chart = run(["chart", *birth, "--json"])
        topics = {}
        for topic, (from_age, to_age) in AGES.items():
            rep = run(["topic", topic, *birth, "--sex", sex, "--json",
                       "--from-age", str(from_age), "--to-age", str(to_age)])
            topics[topic] = {
                "ages": [from_age, to_age],
                "score": rep["score"],
                "label": rep["label"],
                "supporting": len(rep["supporting"]),
                "afflicting": len(rep["afflicting"]),
                "windows": len(rep.get("timing") or []),
            }
        subjects.append({
            "code": code,
            "descriptor": desc,
            "time_confidence": conf,
            "birth": {"date": date, "time": time, "lat": lat, "lon": lon, "tz": tz, "sex": sex},
            "lagna": {
                "rasi": chart["lagna"]["rasi"],
                "degrees_in_rasi": chart["lagna"]["degrees_in_rasi"],
                "nakshatra": chart["lagna"]["nakshatra"]["nakshatra"],
                "pada": chart["lagna"]["nakshatra"]["pada"],
            },
            "topics": topics,
        })
        print(f"  {code}  {chart['lagna']['rasi']:9s} "
              f"{chart['lagna']['degrees_in_rasi']:6.2f}  "
              + " ".join(f"{t[:4]}={topics[t]['score']:+d}" for t in AGES))

    doc = {
        "description": "The thirteen charts of the independent 1.0 validation pass, "
                       "with every topic score as 1.0 produced it.",
        "provenance": "tests/QA-VALIDATION-1.0.md - independent QA, 2026-10-01/02. "
                      "Anonymised: subject codes only, no names.",
        "regenerate": "python3 scripts/make_validation13.py",
        "locks_not_blesses": "A score here is what 1.0 computes, not what is correct. "
                             "Several conflict with the documented life; see the report's "
                             "failure patterns P1 and P2.",
        "ayanamsa": "lahiri", "node": "mean", "houses": "whole-sign",
        "subjects": subjects,
    }
    OUT.write_text(json.dumps(doc, indent=2) + "\n")
    print(f"\nwrote {OUT.relative_to(ROOT)}: {len(subjects)} subjects x {len(AGES)} topics")


if __name__ == "__main__":
    main()
