#!/usr/bin/env python3
"""Build data/places.tsv from GeoNames.

Source: https://download.geonames.org/export/dump/ (cities15000, admin1CodesASCII)
Licence: Creative Commons Attribution 4.0 - see data/PLACES-LICENSE.md.

Output columns (tab-separated, one place per line, sorted by geonameid):
  id  name  ascii  alternates  country  admin1  lat  lon  tz  population

`alternates` keeps ASCII-only alternate names (pipe-separated) so historical
names - Madras, Bombay, Calcutta, Trivandrum, Cochin - find the modern city.
"""
import csv, io, sys, urllib.request, zipfile
from pathlib import Path

BASE = "https://download.geonames.org/export/dump/"
OUT = Path(__file__).resolve().parent.parent / "data" / "places.tsv"

def fetch(name):
    with urllib.request.urlopen(BASE + name, timeout=120) as r:
        return r.read()

admin1 = {}
for line in fetch("admin1CodesASCII.txt").decode("utf-8").splitlines():
    code, name, ascii_name, _ = line.split("\t")
    admin1[code] = ascii_name

z = zipfile.ZipFile(io.BytesIO(fetch("cities15000.zip")))
text = z.read("cities15000.txt").decode("utf-8")
rows = []
for line in text.splitlines():
    f = line.split("\t")
    gid, name, ascii_name, alts = f[0], f[1], f[2], f[3]
    lat, lon, cc, a1, pop, tz = f[4], f[5], f[8], f[10], f[14], f[17]
    if not tz:
        continue
    seen, keep = {ascii_name.lower(), name.lower()}, []
    for a in alts.split(","):
        a = a.strip()
        # Skip airport and station codes (IATA/ICAO: 3-4 capitals, e.g. MAD,
        # MAA, BOM). Left in, "mad" matched Madrid exactly and outranked Madurai.
        is_code = a.isupper() and len(a) <= 4
        if a and a.isascii() and not is_code and a.replace(" ", "").replace("-", "").replace(".", "").isalpha() \
                and 3 <= len(a) <= 40 and a.lower() not in seen:
            seen.add(a.lower()); keep.append(a)
    rows.append([gid, name, ascii_name, "|".join(keep[:25]), cc,
                 admin1.get(f"{cc}.{a1}", ""), f"{float(lat):.5f}", f"{float(lon):.5f}", tz, pop])

rows.sort(key=lambda r: int(r[0]))
with OUT.open("w", newline="") as fh:
    w = csv.writer(fh, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE, escapechar="\\")
    w.writerows(rows)
print(f"wrote {len(rows)} places to {OUT} ({OUT.stat().st_size/1e6:.1f} MB)")
