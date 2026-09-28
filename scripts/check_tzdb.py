#!/usr/bin/env python3
"""Release gate: the tz database the server uses must be IANA's latest.

IANA corrects historical offsets several times a year; a stale copy gives
wrong birth offsets. Fails (exit 1) if the database in use is older than
IANA's latest release, telling you to run scripts/update_tzdb.sh.

Usage: python3 scripts/check_tzdb.py --url http://127.0.0.1:8080
       python3 scripts/check_tzdb.py            # checks data/zoneinfo only
"""
import argparse, json, sys, urllib.request
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--url")
a = ap.parse_args()
with urllib.request.urlopen("https://data.iana.org/time-zones/tzdb/version", timeout=20) as r:
    latest = r.read().decode().strip()
if a.url:
    with urllib.request.urlopen(f"{a.url}/api/version", timeout=10) as r:
        in_use = json.load(r)["tzdb"].split()[0]
    what = "server"
else:
    in_use = (Path(__file__).resolve().parent.parent / "data/zoneinfo/+VERSION").read_text().strip()
    what = "data/zoneinfo"
print(f"IANA latest: {latest}   {what}: {in_use}")
if in_use < latest:
    print(f"STALE: run scripts/update_tzdb.sh (IANA {latest} corrects offsets the {in_use} data gets wrong)", file=sys.stderr)
    sys.exit(1)
print("tz database is current")
