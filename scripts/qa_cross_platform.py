#!/usr/bin/env python3
"""Cross-platform agreement: the same 500 births on two platforms.

  --emit DIR        write every CLI output for the fixed births to DIR
  --compare A B     compare two emitted directories

A difference in anything discrete - a sign, nakshatra, pada, house, dasha lord,
dignity, verdict, bindu count, rule outcome - is a FAILURE. Floating-point
values are compared exactly; any that differ are reported with the largest
deviation, so "bit-identical" or "agrees to 1e-N" is measured, not assumed.
"""
import argparse, json, random, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = str(ROOT / "target" / "release" / "lagn")

def births(n=500, seed=2026):
    rng = random.Random(seed)
    out = []
    while len(out) < n:
        y, m, d = rng.randint(1300, 2390), rng.randint(1, 12), rng.randint(1, 28)
        if (y, m) == (1582, 10) and 5 <= d <= 14:
            continue
        out.append(["--date", f"{y:04d}-{m:02d}-{d:02d}", "--time",
                    f"{rng.randint(0,23):02d}:{rng.randint(0,59):02d}:{rng.randint(0,59):02d}",
                    "--lat", f"{rng.uniform(-85,85):.5f}", "--lon", f"{rng.uniform(-180,180):.5f}",
                    "--tz", rng.choice(["5.5", "0", "-5", "9", "5.352778"])])
    return out

def run(*a):
    return json.loads(subprocess.run([LAGN, *a], capture_output=True, text=True, check=True).stdout)

def emit(d):
    d = Path(d); d.mkdir(parents=True, exist_ok=True)
    for i, b in enumerate(births()):
        rec = {c: run(c, *b, "--json") for c in ("chart", "dasha", "vargas", "details", "ashtakavarga")}
        rec["topic"] = run("topic", "marriage", *b, "--review", "--sex", "female", "--json")
        (d / f"{i:03d}.json").write_text(json.dumps(rec, sort_keys=True))
    print(f"emitted 500 births to {d}")

def walk(a, b, path, floats, discrete):
    if isinstance(a, dict) and isinstance(b, dict):
        if set(a) != set(b):
            discrete.append((path, "keys differ")); return
        for k in a:
            walk(a[k], b[k], f"{path}.{k}", floats, discrete)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            discrete.append((path, f"length {len(a)} vs {len(b)}")); return
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, f"{path}[{i}]", floats, discrete)
    elif isinstance(a, float) or isinstance(b, float):
        if a != b:
            floats.append((path, abs(a - b)))
    elif a != b:
        discrete.append((path, f"{a!r} vs {b!r}"))

def compare(pa, pb):
    fa, fb = sorted(Path(pa).glob("*.json")), sorted(Path(pb).glob("*.json"))
    assert len(fa) == len(fb) == 500, (len(fa), len(fb))
    floats, discrete = [], []
    for x, y in zip(fa, fb):
        walk(json.loads(x.read_text()), json.loads(y.read_text()), x.stem, floats, discrete)
    print(f"compared {len(fa)} births: {pa} vs {pb}")
    print(f"  discrete differences: {len(discrete)}")
    for p, why in discrete[:10]:
        print(f"    {p}: {why}")
    if floats:
        worst = max(floats, key=lambda t: t[1])
        lon = [d for p, d in floats if p.endswith("longitude") or "degrees" in p]
        print(f"  floating-point values differing: {len(floats)}; largest {worst[1]:.3e} at {worst[0]}")
        if lon:
            print(f"  largest longitude difference: {max(lon) * 3600:.3e} arcsec")
    else:
        print("  floating-point values: bit-identical")
    return 1 if discrete else 0

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--emit"); g.add_argument("--compare", nargs=2)
    a = ap.parse_args()
    sys.exit(emit(a.emit) if a.emit else compare(*a.compare))
