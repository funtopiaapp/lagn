#!/usr/bin/env python3
"""Phase 4 parity: the live HTTP API against the CLI.

The CLI's JSON is what the Phase 1-3 oracles verified. This checks that the
web layer delivers exactly that, byte-for-value, through a real server and
real HTTP, under concurrent load.

Usage: python3 scripts/qa_api_parity.py --url http://127.0.0.1:8789 --token-file F [--charts N]
"""
import argparse, json, random, subprocess, sys, urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = str(ROOT / "target" / "release" / "lagn")

def cli(*args):
    return json.loads(subprocess.run([LAGN, *args], capture_output=True, text=True, check=True).stdout)

def api(url, path, body, token=None):
    req = urllib.request.Request(url + path, data=json.dumps(body).encode(), method="POST",
                                 headers={"content-type": "application/json", **({"x-review-token": token} if token else {})})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)

def birth(rng):
    y, m, d = rng.randint(1300, 2390), rng.randint(1, 12), rng.randint(1, 28)
    if (y, m) == (1582, 10) and 5 <= d <= 14:
        d = 20
    return {"date": f"{y:04d}-{m:02d}-{d:02d}",
            "time": f"{rng.randint(0,23):02d}:{rng.randint(0,59):02d}:{rng.randint(0,59):02d}",
            "latitude": round(rng.uniform(-85, 85), 5), "longitude": round(rng.uniform(-180, 180), 5),
            "utc_offset_hours": rng.choice([5.5, 0.0, -5.0, 9.0, 5.352778])}

def flags(b):
    return ["--date", b["date"], "--time", b["time"], "--lat", str(b["latitude"]),
            "--lon", str(b["longitude"]), "--tz", str(b["utc_offset_hours"])]

def check(args):
    url, token, i, b, g, sex = args
    errs = []
    f = flags(b)
    r = api(url, "/api/chart", {"birth": b})
    raw = r["raw"]
    if raw["chart"] != cli("chart", *f, "--json"): errs.append("chart")
    if raw["vimshottari"] != cli("dasha", *f, "--json"): errs.append("dasha")
    if raw["ashtakavarga"] != cli("ashtakavarga", *f, "--json"): errs.append("ashtakavarga")
    if raw["analysis"] != cli("details", *f, "--json"): errs.append("details")
    vj = cli("vargas", *f, "--json")
    for mine, theirs in zip(r["vargas"], vj["vargas"]):
        for cell in mine["grahas"]:
            t = theirs["grahas"][cell["key"]]
            if cell["house"] != t["house"] or cell["sign"].lower() != t["sign"]:
                errs.append(f"varga {mine['varga']} {cell['key']}")
    t = api(url, "/api/topic/marriage", {"birth": b, "sex": sex, "mode": "review"}, token)
    if t["report"] != cli("topic", "marriage", *f, "--review", "--sex", sex, "--json"): errs.append("topic")
    ms = lambda x: f"{x['date']} {x['time']} {x['latitude']} {x['longitude']} {x['utc_offset_hours']}"
    m = api(url, "/api/match", {"bride": b, "groom": g, "mode": "review"}, token)
    if m != cli("match", "--bride", ms(b), "--groom", ms(g), "--review", "--json"): errs.append("match")
    return i, b, errs

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True); ap.add_argument("--token-file", required=True)
    ap.add_argument("--charts", type=int, default=300); ap.add_argument("--seed", type=int, default=5)
    a = ap.parse_args()
    token = Path(a.token_file).read_text().strip()
    rng = random.Random(a.seed)
    jobs = [(a.url, token, i, birth(rng), birth(rng), rng.choice(["female", "male"])) for i in range(a.charts)]
    failures = 0
    with ThreadPoolExecutor(max_workers=8) as pool:
        for i, b, errs in pool.map(check, jobs):
            if errs:
                failures += 1
                print(f"  FAIL {i} {b}: {errs}")
    print(f"{a.charts} births x 7 endpoint comparisons, 8 concurrent clients: {failures} failing")
    return 1 if failures else 0

if __name__ == "__main__":
    sys.exit(main())
