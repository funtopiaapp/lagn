#!/usr/bin/env python3
"""Offset N-version oracle: lagn-server's /api/offset against Python zoneinfo.

zoneinfo reads the system tz database independently of chrono-tz, the crate
the server embeds. Cases are random (zone, date, time) triples plus times
placed deliberately around detected offset transitions - where DST gap and
overlap handling goes wrong. Any disagreement must be traced to a tz database
version difference, or it is a defect.

Usage: python3 scripts/qa_offset_oracle.py --url http://127.0.0.1:8788 [--random N] [--zones Z]
"""
import argparse, json, random, sys, urllib.parse, urllib.request, zoneinfo
from datetime import datetime, timedelta, timezone
from pathlib import Path
from zoneinfo import ZoneInfo, available_timezones

def classify(zone, local):
    """(kind, [offset seconds...], is_lmt) per zoneinfo."""
    z = ZoneInfo(zone)
    out = []
    for fold in (0, 1):
        d = local.replace(tzinfo=z, fold=fold)
        back = d.astimezone(timezone.utc).astimezone(z).replace(tzinfo=None)
        out.append((back == local, int(d.utcoffset().total_seconds()), d.tzname()))
    (e0, o0, n0), (e1, o1, n1) = out
    if e0 and e1 and o0 != o1:
        return "ambiguous", [o0, o1], False
    if e0 or e1:
        o, n = (o0, n0) if e0 else (o1, n1)
        return ("lmt" if n == "LMT" else "unique"), [o], n == "LMT"
    return "nonexistent", [], False

def secs(text):
    sign = -1 if text[0] == "-" else 1
    parts = [int(x) for x in text[1:].split(":")]
    h, m, s = (parts + [0, 0])[:3]
    return sign * (h * 3600 + m * 60 + s)

def query(url, zone, local):
    q = urllib.parse.urlencode({"tz": zone, "date": local.strftime("%Y-%m-%d"), "time": local.strftime("%H:%M:%S")})
    try:
        with urllib.request.urlopen(f"{url}/api/offset?{q}", timeout=10) as r:
            return 200, json.load(r)
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read() or b"{}")

def transitions(zone, start=1900, end=2037):
    """UTC instants where the zone's offset changes, found by bisection."""
    z = ZoneInfo(zone)
    t = datetime(start, 1, 1, tzinfo=timezone.utc)
    stop = datetime(end, 1, 1, tzinfo=timezone.utc)
    step = timedelta(days=15)
    prev = t.astimezone(z).utcoffset()
    found = []
    while t < stop:
        n = t + step
        cur = n.astimezone(z).utcoffset()
        if cur != prev:
            lo, hi = t, n
            while hi - lo > timedelta(seconds=1):
                mid = lo + (hi - lo) / 2
                if mid.astimezone(z).utcoffset() == prev:
                    lo = mid
                else:
                    hi = mid
            found.append(hi)
            prev = cur
        t = n
    return found

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default="http://127.0.0.1:8788")
    ap.add_argument("--random", type=int, default=3000)
    ap.add_argument("--zones", type=int, default=80)
    ap.add_argument("--seed", type=int, default=4)
    ap.add_argument("--tzpath", default=str(Path(__file__).resolve().parent.parent / "data" / "zoneinfo"),
                    help="zoneinfo directory to read; must be the database the server uses")
    a = ap.parse_args()
    # Read the same database files as the server, through an independent parser.
    zoneinfo.reset_tzpath([a.tzpath])
    ZoneInfo.clear_cache()
    with urllib.request.urlopen(f"{a.url}/api/version", timeout=10) as r:
        server_tz = json.load(r)["tzdb"]
    mine = (Path(a.tzpath) / "+VERSION").read_text().strip() if (Path(a.tzpath) / "+VERSION").exists() else "?"
    print(f"server tz database: {server_tz}; oracle reads: {mine} ({a.tzpath})")
    if not server_tz.startswith(mine):
        print("ERROR: oracle and server read different tz database versions", file=sys.stderr)
        return 2
    rng = random.Random(a.seed)
    zones = sorted(z for z in available_timezones() if "/" in z and not z.startswith(("Etc/", "SystemV/", "US/", "posix/", "right/")))

    cases = []
    for _ in range(a.random):
        zone = rng.choice(zones)
        local = datetime(rng.randint(1900, 2037), rng.randint(1, 12), rng.randint(1, 28),
                         rng.randint(0, 23), rng.randint(0, 59), rng.randint(0, 59))
        cases.append(("random", zone, local))
    for zone in rng.sample(zones, a.zones):
        z = ZoneInfo(zone)
        for tr in transitions(zone):
            before = (tr - timedelta(seconds=1)).astimezone(z).replace(tzinfo=None)
            after = tr.astimezone(z).replace(tzinfo=None)
            for base in (before, after):
                for dm in (-61, -31, -1, 0, 1, 31, 61):
                    cases.append(("transition", zone, (base + timedelta(minutes=dm)).replace(microsecond=0)))

    stats = {"agree": 0, "unknown_zone": 0, "by_kind": {}}
    mismatches = []
    for origin, zone, local in cases:
        if local.year < 1583:
            continue
        status, body = query(a.url, zone, local)
        if status == 400 and "unknown time zone" in body.get("error", ""):
            stats["unknown_zone"] += 1
            continue
        want_kind, want_offsets, is_lmt = classify(zone, local)
        if status != 200:
            mismatches.append((origin, zone, local, f"HTTP {status} {body}", want_kind))
            continue
        got_kind = {"unique": "unique", "ambiguous": "ambiguous", "nonexistent": "nonexistent", "local_mean_time": "lmt"}[body["kind"]]
        got_offsets = [secs(c["offset_text"]) for c in body["candidates"]]
        ok = got_kind == want_kind and got_offsets == want_offsets
        stats["by_kind"][want_kind] = stats["by_kind"].get(want_kind, 0) + 1
        if ok:
            stats["agree"] += 1
        else:
            mismatches.append((origin, zone, local, f"server {got_kind} {got_offsets}", f"zoneinfo {want_kind} {want_offsets}"))

    total = stats["agree"] + len(mismatches)
    print(f"{total} cases ({sum(1 for c in cases if c[0]=='random')} random, "
          f"{sum(1 for c in cases if c[0]=='transition')} around transitions); kinds {stats['by_kind']}")
    print(f"zones unknown to the server's tz database: {stats['unknown_zone']} cases skipped")
    print(f"agree: {stats['agree']}   disagree: {len(mismatches)}")
    by_zone = {}
    for m in mismatches:
        by_zone.setdefault(m[1], []).append(m)
    for zone, ms in sorted(by_zone.items()):
        years = sorted({m[2].year for m in ms})
        print(f"  {zone}: {len(ms)} disagreements, years {years[0]}-{years[-1]}; e.g. {ms[0][2]} {ms[0][3]} vs {ms[0][4]}")
    json.dump([[m[0], m[1], m[2].isoformat(), m[3], m[4]] for m in mismatches],
              open("/tmp/offset_mismatches.json", "w"), indent=1)
    return 1 if mismatches else 0

if __name__ == "__main__":
    sys.exit(main())
