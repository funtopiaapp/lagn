#!/usr/bin/env python3
"""Cross-validate the lagn kernel against swetest.

swetest is the reference front end that ships with Swiss Ephemeris. It drives
the same C library our FFI calls, through completely separate code, so
agreement confirms that our flags, sidereal mode, node choice and coordinate
conventions are right - it is not circular.

Scope: layer 1 only (planetary longitudes and the ascendant). Layers 2-3
(rasi, navamsa, dasha) are validated by unit tests against classical rules and
by golden fixtures checked against JHora.

Usage:
    python3 scripts/crossvalidate.py            # run the built-in case set
    python3 scripts/crossvalidate.py --verbose  # show every case
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SWETEST = ROOT / "tools" / "swetest" / "swetest"
EPHE = ROOT / "ephe"

# swetest body codes -> our graha names
BODY_MAP = {
    "Sun": "sun", "Moon": "moon", "Mars": "mars", "Mercury": "mercury",
    "Jupiter": "jupiter", "Venus": "venus", "Saturn": "saturn",
    "mean Node": "rahu",
}

# Arcseconds. Anything above this means a genuine disagreement, not rounding.
TOLERANCE_ARCSEC = 0.01


@dataclass
class Case:
    name: str
    date: str          # YYYY-MM-DD
    time: str          # HH:MM:SS
    lat: float
    lon: float
    tz: float
    note: str = ""


def build_cases() -> list[Case]:
    """The Phase 1 edge-case set.

    Chosen to exercise the places this kind of code actually breaks: timezone
    history, calendar transitions, ephemeris-file boundaries, and latitudes
    where house systems degenerate.
    """
    cases: list[Case] = []

    # --- ordinary South Indian births, spread across the year -------------
    cities = [
        ("Chennai", 13.0827, 80.2707),
        ("Madurai", 9.9252, 78.1198),
        ("Thiruvananthapuram", 8.5241, 76.9366),
        ("Kochi", 9.9312, 76.2673),
        ("Coimbatore", 11.0168, 76.9558),
        ("Thanjavur", 10.7870, 79.1378),
    ]
    for i, (city, lat, lon) in enumerate(cities):
        cases.append(Case(
            f"{city} ordinary", f"19{70+i:02d}-{(i*2)+1:02d}-15",
            f"{(i*3+6) % 24:02d}:{(i*7) % 60:02d}:00", lat, lon, 5.5,
            "baseline",
        ))

    # --- Indian timezone history ------------------------------------------
    # Madras Local Mean Time, used until 1906: longitude 80.2707E / 15 = 5.3514h
    cases.append(Case("Madras LMT 1890", "1890-03-14", "04:22:00",
                      13.0827, 80.2707, 5.3514, "pre-standard-time local mean time"))
    cases.append(Case("Madras LMT 1905", "1905-11-02", "21:47:30",
                      13.0827, 80.2707, 5.3514, "last year before IST"))
    # IST adopted 1906-01-01 at +5:30.
    cases.append(Case("IST first day", "1906-01-01", "00:00:01",
                      13.0827, 80.2707, 5.5, "IST adoption"))
    # Bombay kept its own time (+4:51) until 1955.
    cases.append(Case("Bombay Time 1940", "1940-07-19", "11:11:11",
                      19.0760, 72.8777, 4.85, "Bombay Time, not IST"))
    # Wartime +6:30 was in force 1942-09-01 to 1945-10-15.
    cases.append(Case("Indian wartime 1943", "1943-05-05", "14:00:00",
                      13.0827, 80.2707, 6.5, "WWII +6:30"))
    cases.append(Case("Independence midnight", "1947-08-15", "00:00:00",
                      28.6139, 77.2090, 5.5, "pre-1947 boundary"))
    cases.append(Case("Pre-independence 1920", "1920-12-25", "18:30:00",
                      13.0827, 80.2707, 5.5, "pre-1947"))
    cases.append(Case("Pre-independence 1900", "1900-06-11", "09:15:00",
                      9.9252, 78.1198, 5.3514, "pre-1947"))

    # --- ephemeris file boundaries (.se1 files break at 1800 and 2400) -----
    cases.append(Case("Just before 1800", "1799-12-31", "23:59:00",
                      13.0827, 80.2707, 5.3514, "sepl_12 -> sepl_18 boundary"))
    cases.append(Case("Just after 1800", "1800-01-01", "00:01:00",
                      13.0827, 80.2707, 5.3514, "sepl_12 -> sepl_18 boundary"))
    cases.append(Case("Deep past 1300", "1300-04-04", "12:00:00",
                      13.0827, 80.2707, 5.3514, "sepl_12 coverage"))
    cases.append(Case("Future 2380", "2380-09-09", "06:00:00",
                      13.0827, 80.2707, 5.5, "sepl_24 coverage"))

    # --- latitude extremes -------------------------------------------------
    for name, lat, lon in [
        ("Tromso", 69.6496, 18.9560),
        ("Longyearbyen", 78.2232, 15.6267),
        ("Near north pole", 89.5, 0.0),
        ("Near south pole", -89.5, 0.0),
        ("Antarctic", -77.8463, 166.6683),
        ("Equator", 0.0, 0.0),
    ]:
        cases.append(Case(f"{name}", "1985-06-21", "12:00:00", lat, lon, 0.0,
                          "latitude extreme"))

    # --- longitude / date-line extremes ------------------------------------
    cases.append(Case("Date line east", "1990-01-01", "00:30:00",
                      -18.1416, 178.4419, 12.0, "Fiji, +12"))
    cases.append(Case("Date line west", "1990-01-01", "00:30:00",
                      21.3069, -157.8583, -10.0, "Honolulu, -10"))
    cases.append(Case("Southern hemisphere", "1978-11-03", "03:20:00",
                      -33.8688, 151.2093, 11.0, "Sydney"))
    cases.append(Case("Western hemisphere", "1965-02-14", "22:45:00",
                      40.7128, -74.0060, -5.0, "New York"))

    # --- calendar and clock edges -----------------------------------------
    cases.append(Case("Leap day 2000", "2000-02-29", "12:00:00",
                      13.0827, 80.2707, 5.5, "leap day in a century year"))
    cases.append(Case("Leap day 1996", "1996-02-29", "23:59:59",
                      13.0827, 80.2707, 5.5, "leap day"))
    cases.append(Case("Non-leap century", "1900-02-28", "12:00:00",
                      13.0827, 80.2707, 5.3514, "1900 is not a leap year"))
    cases.append(Case("Midnight exact", "1990-01-01", "00:00:00",
                      13.0827, 80.2707, 5.5, "midnight"))
    cases.append(Case("Noon exact", "1990-01-01", "12:00:00",
                      13.0827, 80.2707, 5.5, "noon"))
    cases.append(Case("Second before midnight", "1990-12-31", "23:59:59",
                      13.0827, 80.2707, 5.5, "day rollover"))
    cases.append(Case("Offset crosses date back", "1990-01-01", "02:00:00",
                      13.0827, 80.2707, 5.5, "local date != UT date"))
    cases.append(Case("Offset crosses date fwd", "1990-01-01", "20:00:00",
                      21.3069, -157.8583, -10.0, "local date != UT date"))

    # --- a spread of years to catch anything era-dependent -----------------
    for y in range(1850, 2030, 20):
        cases.append(Case(f"Decadal {y}", f"{y}-07-04", "17:17:17",
                          13.0827, 80.2707, 5.5 if y >= 1906 else 5.3514,
                          "decadal sweep"))

    return cases


LAGN = ROOT / "target" / "release" / "lagn"


def run_lagn(case: Case) -> dict:
    out = subprocess.run(
        [str(LAGN), "chart",
         "--date", case.date, "--time", case.time,
         "--lat", str(case.lat), "--lon", str(case.lon), "--tz", str(case.tz),
         "--json"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if out.returncode != 0:
        raise RuntimeError(f"lagn failed for {case.name}: {out.stderr.strip()}")
    return json.loads(out.stdout)


def run_swetest(case: Case, jd_ut: float) -> dict:
    """Query swetest at the exact Julian Day our kernel computed.

    Feeding it the JD rather than a date/time removes our timezone arithmetic
    from the comparison, so a mismatch can only mean a disagreement about the
    astronomy. The JD itself is checked separately by `jd_roundtrips_through
    _the_calendar` and the BirthMoment tests.
    """
    # -ut is load-bearing twice over: without it swetest reads the JD as
    # TT rather than UT (a delta-T sized error, ~5 arcmin on the Moon in
    # medieval dates), and it refuses to compute houses at all.
    args = [
        str(SWETEST), f"-j{jd_ut:.9f}", "-ut", "-p0123456m", "-sid1",
        "-fPl", "-eswe", f"-edir{EPHE}",
        f"-house{case.lon},{case.lat},W",
    ]
    out = subprocess.run(args, capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(f"swetest failed for {case.name}: {out.stderr.strip()}")

    result: dict = {"bodies": {}, "ascendant": None}
    for line in out.stdout.splitlines():
        line = line.rstrip()
        for label, key in BODY_MAP.items():
            if line.startswith(label):
                tail = line[len(label):].strip().split()
                if tail:
                    try:
                        result["bodies"][key] = float(tail[0])
                    except ValueError:
                        pass
        if line.startswith("Ascendant"):
            try:
                result["ascendant"] = float(line.split()[1])
            except (ValueError, IndexError):
                pass
    return result


def angdiff(a: float, b: float) -> float:
    """Smallest absolute separation between two angles, in degrees."""
    d = abs(a - b) % 360.0
    return min(d, 360.0 - d)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    if not SWETEST.exists():
        print("swetest not built. Run: bash tools/swetest/build.sh", file=sys.stderr)
        return 2
    if not LAGN.exists():
        print("lagn not built. Run: cargo build --release -p lagn-cli", file=sys.stderr)
        return 2

    cases = build_cases()
    print(f"Cross-validating {len(cases)} charts against swetest "
          f"(tolerance {TOLERANCE_ARCSEC} arcsec)\n")

    failures = []
    worst = (0.0, "", "")
    checked = 0

    for case in cases:
        try:
            ours = run_lagn(case)
            theirs = run_swetest(case, ours["jd_ut"])
        except RuntimeError as e:
            failures.append((case.name, str(e)))
            print(f"  ERROR  {case.name}: {e}")
            continue

        bad = []
        for p in ours["placements"]:
            key = p["graha"]
            if key not in theirs["bodies"]:
                continue
            diff_as = angdiff(p["longitude"], theirs["bodies"][key]) * 3600.0
            checked += 1
            if diff_as > worst[0]:
                worst = (diff_as, case.name, key)
            if diff_as > TOLERANCE_ARCSEC:
                bad.append(f"{key}: {diff_as:.4f}\"")

        if theirs["ascendant"] is not None:
            diff_as = angdiff(ours["lagna"]["longitude"], theirs["ascendant"]) * 3600.0
            checked += 1
            if diff_as > worst[0]:
                worst = (diff_as, case.name, "ascendant")
            if diff_as > TOLERANCE_ARCSEC:
                bad.append(f"ascendant: {diff_as:.4f}\"")

        status = "FAIL" if bad else "ok"
        if bad:
            failures.append((case.name, ", ".join(bad)))
        if bad or args.verbose:
            note = f"  ({case.note})" if case.note else ""
            print(f"  {status:5} {case.name:28} {case.date} {case.time}{note}")
            for b in bad:
                print(f"         {b}")

    print(f"\n{checked} values compared across {len(cases)} charts")
    print(f"worst deviation: {worst[0]:.6f} arcsec ({worst[2]} in {worst[1]!r})")
    if failures:
        print(f"\n{len(failures)} FAILING CHARTS")
        return 1
    print("\nall charts agree with swetest")
    return 0


if __name__ == "__main__":
    sys.exit(main())
