#!/usr/bin/env python3
"""Transit oracle: every ingress lagn reports, checked against swetest.

swetest (Swiss Ephemeris's own front end, separate code from ours) lists the
sidereal longitude of each graha every half day across the range. From that
series this script finds sign changes independently, then requires:

  * a one-to-one match between swetest's sign changes and lagn's ingresses;
  * each ingress within one minute: swetest must place the graha in the old
    sign one minute before and in the new sign one minute after;
  * from/to signs agree.
"""
import json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SWETEST = ROOT / "tools/swetest/swetest"
LAGN = ROOT / "target/release/lagn"
EPHE = ROOT / "ephe"
RASI = ["mesha","vrishabha","mithuna","karka","simha","kanya","tula","vrischika","dhanus","makara","kumbha","meena"]
BODY = {"saturn": "6", "jupiter": "5", "rahu": "m"}   # mean node, as lagn defaults

CHUNK = 30000   # swetest silently caps output at 36,525 rows; stay well under it

def swe_series(body, jd0, n, step):
    rows = []
    done = 0
    while done < n:
        k = min(CHUNK, n - done)
        rows.extend(_swe_series(body, jd0 + done * step, k, step))
        done += k
    return rows

def _swe_series(body, jd0, n, step):
    out = subprocess.run([str(SWETEST), f"-bj{jd0}", "-ut", f"-p{body}", "-sid1", "-fJl", "-head",
                          f"-n{n}", f"-s{step}", "-eswe", f"-edir{EPHE}"], capture_output=True, text=True, check=True).stdout
    rows = []
    for line in out.split("\n"):
        parts = line.split()
        if len(parts) >= 2:
            try: rows.append((float(parts[0]), float(parts[-1])))
            except ValueError: pass
    return rows

def swe_lon(body, jd):
    return _swe_series(body, jd, 1, 1)[0][1]

def sign(lon): return int((lon % 360) // 30)

def main():
    # Births 1950-01-01 12:00 UT at 0,0; ages 0 to 100 -> a century of transits.
    rep = json.loads(subprocess.run([str(LAGN), "transits", "--date", "1950-01-01", "--time", "12:00",
        "--lat", "0", "--lon", "0", "--tz", "0", "--from-age", "0", "--to-age", "100", "--json"],
        capture_output=True, text=True, check=True).stdout)
    start, end = rep["start_jd"], rep["end_jd"]
    failures = 0
    for g, body in BODY.items():
        mine = [i for i in rep["ingresses"] if i["graha"] == g]
        n = int((end - start) / 0.5) + 1
        series = swe_series(body, start, n, 0.5)
        assert series[-1][0] >= end - 0.5, f"swetest series ends at {series[-1][0]}, range ends {end}"
        changes = []
        for (t0, l0), (t1, l1) in zip(series, series[1:]):
            if t1 > end: break
            if sign(l0) != sign(l1):
                changes.append((t0, t1, sign(l0), sign(l1)))
        if len(changes) != len(mine):
            failures += 1
            print(f"  {g}: swetest finds {len(changes)} sign changes, lagn {len(mine)}")
        worst = 0.0
        for (t0, t1, a, b), m in zip(changes, mine):
            jd = m["jd_ut"]
            ok_window = t0 <= jd <= t1 + 1e-9
            before, after = sign(swe_lon(body, jd - 1/1440)), sign(swe_lon(body, jd + 1/1440))
            ok = ok_window and RASI[a] == m["from"] and RASI[b] == m["to"] and before == a and after == b
            if not ok:
                failures += 1
                print(f"  {g} ingress at JD {jd}: lagn {m['from']}->{m['to']}, swetest {RASI[a]}->{RASI[b]} (before {RASI[before]}, after {RASI[after]})")
        # Ketu is Rahu reflected: its ingresses must coincide with Rahu's.
        if g == "rahu":
            ketu = [i for i in rep["ingresses"] if i["graha"] == "ketu"]
            if [round(k["jd_ut"], 6) for k in ketu] != [round(r["jd_ut"], 6) for r in mine]:
                failures += 1; print("  ketu ingresses do not coincide with rahu's")
        print(f"  {g:8} {len(mine):4} ingresses over 100 years, all within 1 minute of swetest" if not failures else "")
    print(f"\n{'FAIL' if failures else 'all ingresses agree with swetest'} ({failures} problems)")
    return 1 if failures else 0

if __name__ == "__main__":
    sys.exit(main())
