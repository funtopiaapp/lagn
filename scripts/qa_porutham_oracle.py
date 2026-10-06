#!/usr/bin/env python3
"""Exhaustive porutham oracle.

Parses every porutham table (gana, yoni, yoni enemies, rajju, vedha, vasya,
naadi, varna) out of docs/phase3/DESIGN.md section 8 and checks the kernel's
output for all 108 x 108 = 11,664 bride/groom pada combinations - the complete
input space.

The tables come from the spec rather than from the kernel's source, so this is
an independent check and not a restatement of the code under test.
"""
import json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
T = (ROOT / "docs/phase3/DESIGN.md").read_text()
NAMES = ["Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra", "Punarvasu", "Pushya",
         "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha",
         "Anuradha", "Jyeshtha", "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishta",
         "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati"]
KEY = {n: n.lower().replace(" ", "_") for n in NAMES}
RASI = ["mesha", "vrishabha", "mithuna", "karka", "simha", "kanya", "tula", "vrischika",
        "dhanus", "makara", "kumbha", "meena"]
LORD = ["mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars",
        "jupiter", "saturn", "saturn", "jupiter"]

def section(a, b):
    return T[T.index(a) + len(a):T.index(b)]

def two_col(a, b):
    out = {}
    for line in section(a, b).splitlines():
        c = [x.strip() for x in line.strip().strip("|").split("|")]
        if len(c) == 2 and all(x.strip() in NAMES for x in c[1].split(",")):
            for n in c[1].split(","):
                out[KEY[n.strip()]] = c[0].lower()
    assert len(out) == 27, (a, len(out))
    return out

def pairs(a, b, keyfn):
    body = section(a, b).replace("\n", " ").strip().rstrip(".")
    return {frozenset(keyfn(p.strip()) for p in x.split(" and ")) for x in body.split(";")}

GANA = two_col("**Gana:**", "**Yoni:**")
YONI = two_col("**Yoni:**", "**Yoni enemy pairs:**")
RAJJU = two_col("**Rajju:**", "**Vedha pairs:**")

def rasi_col(a, b, expect):
    """A table of label -> rasis, as lowercase rasi keys."""
    out = {}
    for line in section(a, b).splitlines():
        c = [x.strip() for x in line.strip().strip("|").split("|")]
        if len(c) == 2 and all(x.strip().lower() in RASI for x in c[1].split(",")):
            out[c[0].lower()] = [x.strip().lower() for x in c[1].split(",")]
    assert len(out) == expect, (a, len(out))
    return out

def nak_col(a, b, expect):
    """A table of label -> nakshatras, as nakshatra keys."""
    out = {}
    for line in section(a, b).splitlines():
        c = [x.strip() for x in line.strip().strip("|").split("|")]
        if len(c) == 2 and all(x.strip() in NAMES for x in c[1].split(",")):
            for n in c[1].split(","):
                out[KEY[n.strip()]] = c[0].lower()
    assert len(out) == expect, (a, len(out))
    return out

# Vasya: each rasi and the rasis it holds sway over.
VASYA = rasi_col("**Vasya:**", "**Naadi:**", 12)
# Naadi: nakshatra -> adi / madhya / antya.
NAADI = nak_col("**Naadi:**", "**Varna:**", 27)
# Varna: rasi -> varna, with the ranking the spec states.
_VARNA_ROWS = rasi_col("**Varna:**", "**Self-checks QA must verify:**", 4)
VARNA = {r: v for v, rs in _VARNA_ROWS.items() for r in rs}
VARNA_RANK = {"shudra": 1, "vaishya": 2, "kshatriya": 3, "brahmin": 4}
assert len(VARNA) == 12, len(VARNA)
assert set(_VARNA_ROWS) == set(VARNA_RANK), set(_VARNA_ROWS)
YONI_ENEMY = pairs("**Yoni enemy pairs:**", "**Rajju:**", str.lower)
VEDHA = pairs("**Vedha pairs:**", "**Vasya:**", lambda n: KEY[n])

# Natural relationships, reused from the Phase 2 oracle (which parses them from its spec).
import importlib.util
s = importlib.util.spec_from_file_location("p2", ROOT / "scripts/qa_phase2_oracle.py")
P2 = importlib.util.module_from_spec(s); s.loader.exec_module(P2)

def expected(b, g):
    bn, gn = list(KEY.values()).index(b["nakshatra"]), list(KEY.values()).index(g["nakshatra"])
    n = (gn - bn) % 27 + 1
    r = (RASI.index(g["rasi"]) - RASI.index(b["rasi"])) % 12 + 1
    gb, gg = GANA[b["nakshatra"]], GANA[g["nakshatra"]]
    lb, lg = LORD[RASI.index(b["rasi"])], LORD[RASI.index(g["rasi"])]
    adh = lb == lg or (P2.NATURAL[lb][lg] != "enemy" and P2.NATURAL[lg][lb] != "enemy")
    v = lambda ok: "matching" if ok else "not_matching"
    return {
        "dina": v(n % 9 in (0, 2, 4, 6, 8)),
        "gana": v(gb == gg or {gb, gg} == {"deva", "manushya"}),
        "mahendra": v(n in (4, 7, 10, 13, 16, 19, 22, 25)),
        "stree_deergha": v(n > 13),
        "yoni": v(frozenset((YONI[b["nakshatra"]], YONI[g["nakshatra"]])) not in YONI_ENEMY),
        "rasi": v(r not in (2, 6, 8, 12)),
        "rasi_adhipati": v(adh),
        # Directional: the bride's rasi under the groom's sway.
        "vasya": v(b["rasi"] in VASYA[g["rasi"]]),
        "rajju": v(RAJJU[b["nakshatra"]] != RAJJU[g["nakshatra"]]),
        "vedha": v(frozenset((b["nakshatra"], g["nakshatra"])) not in VEDHA
                   or b["nakshatra"] == g["nakshatra"]),
        "naadi": v(NAADI[b["nakshatra"]] != NAADI[g["nakshatra"]]),
        "varna": v(VARNA_RANK[VARNA[g["rasi"]]] >= VARNA_RANK[VARNA[b["rasi"]]]),
    }

rows = json.loads(subprocess.run([str(ROOT / "target/release/lagn"), "match", "--table"],
                                 capture_output=True, text=True, check=True).stdout)
assert len(rows) == 11664, len(rows)
bad = checks = 0
for row in rows:
    want = expected(row["bride"], row["groom"])
    for res in row["results"]:
        checks += 1
        crit = res["kind"] in ("rajju", "vedha", "naadi") and res["verdict"] == "not_matching"
        if res["verdict"] != want[res["kind"]] or res["critical"] != crit:
            bad += 1
            if bad <= 10:
                print("MISMATCH", row["bride"], row["groom"], res, "expected", want[res["kind"]])
# 108 padas collapse onto 36 distinct (nakshatra, rasi) pairs: each nakshatra
# lies in one sign, or straddles two (9 of them do), so 27 + 9 = 36. The
# distinct input space is therefore 36 x 36 = 1,296 pairs, all covered here.
padas = {(r["bride"]["nakshatra"], r["bride"]["rasi"]) for r in rows}
assert len(padas) == 36, len(padas)
distinct = {(r["bride"]["nakshatra"], r["bride"]["rasi"], r["groom"]["nakshatra"], r["groom"]["rasi"]) for r in rows}
assert len(distinct) == 1296, len(distinct)
print(f"{len(rows)} pairs, {checks:,} porutham verdicts checked, {bad} mismatches")
sys.exit(1 if bad else 0)
