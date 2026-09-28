#!/usr/bin/env python3
"""Phase 3 N-version oracle: an independent rule evaluator.

Implements docs/phase3/DESIGN.md sections 3-7 from the specification text and
compares it against `lagn topic --json` rule by rule, over:

  * the real marriage corpus, in review mode and under random approvals in
    production mode; and
  * random synthetic corpora - random condition trees covering every condition
    type, random acyclic cancellation graphs, random review statuses - so the
    engine is exercised far beyond what the draft corpus happens to use.

Chart facts come from the Phase 2 oracle (scripts/qa_phase2_oracle.py), which
itself works from raw longitudes. Nothing here reads the kernel's derived
facts, so the chain from longitude to verdict is independent end to end.

Usage: python3 scripts/qa_phase3_oracle.py [--charts N] [--seed S]
"""
from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import random
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAGN = ROOT / "target" / "release" / "lagn"
CORPUS = ROOT / "corpus"

spec = importlib.util.spec_from_file_location("p2", ROOT / "scripts" / "qa_phase2_oracle.py")
P2 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P2)

GRAHA, SEVEN, RASI, LORD = P2.GRAHA, P2.SEVEN, P2.RASI, P2.LORD
VARGAS = ["d1", "d2", "d3", "d4", "d7", "d9", "d10", "d12", "d16", "d20", "d24", "d27", "d30", "d40", "d45", "d60"]
DIGNITIES = ["exalted", "moolatrikona", "own_sign", "great_friend", "friend", "neutral",
             "enemy", "great_enemy", "debilitated"]

# ---------------------------------------------------------------------------
# Facts
# ---------------------------------------------------------------------------

class Ctx:
    def __init__(self, chart, dasha, settings, sex):
        self.lon = {p["graha"]: p["longitude"] for p in chart["placements"]}
        self.retro = {p["graha"]: p["retrograde"] for p in chart["placements"]}
        self.asc = chart["lagna"]["longitude"]
        self.settings = settings
        self.sex = sex
        self.dasha = dasha
        sign_of = {g: int(self.lon[g] // 30) for g in SEVEN}
        sign_of["lagna"] = int(self.asc // 30)
        _, self.sav = P2.ashtakavarga(sign_of)

    def vn(self, v):
        return int(v[1:])

    def lagna(self, v):
        return P2.varga_sign(self.vn(v), self.asc)

    def sign(self, g, v="d1"):
        return P2.varga_sign(self.vn(v), self.lon[g])

    def house(self, g, v="d1"):
        return (self.sign(g, v) - self.lagna(v)) % 12 + 1

    period = None   # (maha, antar) while evaluating period rules

    def resolve(self, ref):
        if isinstance(ref, str):
            return ref
        if "period" in ref:
            return None if self.period is None else self.period[0 if ref["period"] == "maha" else 1]
        v = ref.get("varga", "d1")
        return LORD[(self.lagna(v) + ref["lord_of"] - 1) % 12]

    def houses_ruled(self, g):
        l = int(self.asc // 30)
        return [h for h in range(1, 13) if LORD[(l + h - 1) % 12] == g]

    def functional(self, g):
        if g in ("rahu", "ketu"):
            return None
        hs = self.houses_ruled(g)
        has = lambda xs: any(h in xs for h in hs)
        if has({4, 7, 10}) and has({5, 9}): return "yogakaraka"
        if has({1, 5, 9}): return "benefic"
        if has({3, 6, 8, 11, 12}): return "malefic"
        return "neutral"

    def sambandha(self, a, b):
        if a == b: return []
        sa, sb = self.sign(a), self.sign(b)
        out = []
        if sa == sb: out.append("conjunct")
        if self.aspects(a, sb) and self.aspects(b, sa): out.append("mutual_aspect")
        if a not in ("rahu", "ketu") and b not in ("rahu", "ketu") and LORD[sa] == b and LORD[sb] == a:
            out.append("exchange")
        return out

    def neecha_bhanga(self, g):
        if g in ("rahu", "ketu"): return None
        if self.dignity(g, "d1") != "debilitated": return False
        l, moon = int(self.asc // 30), self.sign("moon")
        kendra = lambda x: ((self.sign(x) - l) % 12 + 1) in (1, 4, 7, 10) or ((self.sign(x) - moon) % 12 + 1) in (1, 4, 7, 10)
        deb = (P2.EXALT[g] + 6) % 12
        deb_lord, ex_lord = LORD[deb], LORD[P2.EXALT[g]]
        c3 = deb_lord != g and (self.sign(deb_lord) == self.sign(g) or self.aspects(deb_lord, self.sign(g)))
        c4 = P2.varga_sign(9, self.lon[g]) == P2.EXALT[g]
        return kendra(deb_lord) or kendra(ex_lord) or c3 or c4

    def benefic(self, g):
        if g in ("jupiter", "venus", "mercury"):
            return True
        if g == "moon":
            return (self.lon["moon"] - self.lon["sun"]) % 360.0 < 180.0
        return False

    def members(self, s):
        if s == "any":
            out = list(GRAHA)
        elif s == "benefics":
            out = [g for g in GRAHA if self.benefic(g)]
        elif s == "malefics":
            out = [g for g in GRAHA if not self.benefic(g)]
        else:
            out = [g for g in (self.resolve(r) for r in s["grahas"]) if g is not None]
        return sorted(set(out), key=GRAHA.index)

    def aspects(self, g, target_sign):
        h = (target_sign - self.sign(g)) % 12 + 1
        return h in P2.aspect_houses(g, self.settings["node_aspects"])

    def dignity(self, g, v):
        return P2.dignity(g, self.vn(v), self.lon, self.settings)

    def sav_house(self, h):
        return self.sav[(int(self.asc // 30) + h - 1) % 12]

# ---------------------------------------------------------------------------
# Section 4: Kleene evaluation
# ---------------------------------------------------------------------------

def k_all(vs):
    return "F" if "F" in vs else ("U" if "U" in vs else "T")

def k_any(vs):
    return "T" if "T" in vs else ("U" if "U" in vs else "F")

def tf(b):
    return "T" if b else "F"

class Unresolved(Exception):
    pass

def R(x, ref):
    g = x.resolve(ref)
    if g is None:
        raise Unresolved()
    return g

def ev(c, x):
    try:
        return ev_inner(c, x)
    except Unresolved:
        return "U"

def ev_inner(c, x):
    (kind, a), = c.items()
    if kind == "all":
        return k_all([ev(y, x) for y in a])
    if kind == "any":
        return k_any([ev(y, x) for y in a])
    if kind == "not":
        return {"T": "F", "F": "T", "U": "U"}[ev(a, x)]
    if kind == "always":
        return "T"
    if kind == "graha_in_house":
        return tf(x.house(R(x, a["graha"]), a.get("varga", "d1")) in a["houses"])
    if kind == "graha_in_sign":
        return tf(RASI[x.sign(R(x, a["graha"]), a.get("varga", "d1"))] in a["signs"])
    if kind == "graha_from":
        g = R(x, a["graha"])
        origin = int(x.asc // 30) if a["from"] == "lagna" else x.sign(R(x, a["from"]))
        return tf((x.sign(g) - origin) % 12 + 1 in a["houses"])
    if kind == "dignity":
        d = x.dignity(R(x, a["graha"]), a.get("varga", "d1"))
        return "U" if d is None else tf(d in a["in"])
    if kind == "house_occupied":
        v = a.get("varga", "d1")
        n = sum(1 for g in x.members(a["by"]) if x.house(g, v) == a["house"])
        return tf(n >= a.get("min", 1))
    if kind == "house_aspected":
        target = (int(x.asc // 30) + a["house"] - 1) % 12
        n = sum(1 for g in x.members(a["by"]) if x.aspects(g, target))
        return tf(n >= a.get("min", 1))
    if kind == "conjunct":
        ga, gb = R(x, a["a"]), R(x, a["b"])
        return "F" if ga == gb else tf(x.sign(ga) == x.sign(gb))
    if kind == "aspects_graha":
        return tf(x.aspects(R(x, a["from"]), x.sign(R(x, a["to"]))))
    if kind == "combust":
        g = R(x, a["graha"])
        return tf(P2.combust(g, x.lon[g], x.retro[g], x.lon["sun"]))
    if kind == "retrograde":
        return tf(x.retro[R(x, a["graha"])])
    if kind == "sav":
        v = x.sav_house(a["house"])
        return tf((a.get("min") is None or v >= a["min"]) and (a.get("max") is None or v <= a["max"]))
    if kind == "native":
        return "U" if x.sex is None else tf(x.sex == a["sex"])
    if kind == "functional":
        n = x.functional(R(x, a["graha"]))
        return "U" if n is None else tf(n in a["is"])
    if kind == "rules_house":
        return tf(any(h in a["houses"] for h in x.houses_ruled(R(x, a["graha"]))))
    if kind == "sambandha":
        ga, gb = R(x, a["a"]), R(x, a["b"])
        found = x.sambandha(ga, gb)
        kinds = a.get("kinds") or []
        return tf(any(not kinds or k in kinds for k in found))
    if kind == "neecha_bhanga":
        v = x.neecha_bhanga(R(x, a["graha"]))
        return "U" if v is None else tf(v)
    raise ValueError(kind)

# ---------------------------------------------------------------------------
# Sections 5-7
# ---------------------------------------------------------------------------

def admits(mode, status):
    return status == "approved" or (mode == "review" and status == "draft")

def topic_report(rules, topic, x, mode, ages):
    admitted, withheld = [], {}
    for r in rules:
        if r["topic"] != topic or r.get("scope", "natal") != "natal":
            continue
        if admits(mode, r["review"]["status"]):
            admitted.append(r)
        else:
            withheld[r["review"]["status"]] = withheld.get(r["review"]["status"], 0) + 1
    admitted.sort(key=lambda r: r["id"])
    out = {r["id"]: ev(r["when"], x) for r in admitted}
    cancellers = {}
    for r in admitted:
        for t in r.get("overrides", []):
            cancellers.setdefault(t, []).append(r["id"])
    memo = {}

    def eff(i):
        if i in memo:
            return memo[i]
        fired = out.get(i) == "T"
        by = None
        if fired:
            for c in sorted(cancellers.get(i, [])):
                if eff(c)[0]:
                    by = c
                    break
        memo[i] = (fired and by is None, by)
        return memo[i]

    results, sup, aff, canc, unk, timed = [], [], [], [], [], []
    score = pos = neg = 0
    for r in admitted:
        e, by = eff(r["id"])
        if e:
            p = r["polarity"]
            score += p
            if p > 0:
                sup.append(r["id"]); pos += p
            elif p < 0:
                aff.append(r["id"]); neg += p
            if r.get("timing"):
                timed.append(r)
        if by:
            canc.append([r["id"], by])
        if out[r["id"]] == "U":
            unk.append(r["id"])
        results.append({"id": r["id"], "outcome": out[r["id"]], "effective": e, "cancelled_by": by,
                        "polarity": r["polarity"], "status": r["review"]["status"],
                        "subject": x.resolve(r["subject"]) if r.get("subject") is not None else None,
                        "tags": r.get("tags", []), "impact": r["text"].get("impact")})
    label = ("mixed" if pos > 0 and neg < 0 else "supportive" if pos > 0 else
             "afflicted" if neg < 0 else "neutral")

    timing = []
    dpy = 365.25
    lo = x.dasha["birth_jd"] + ages[0] * dpy
    hi = x.dasha["birth_jd"] + ages[1] * dpy
    for r in timed:
        lords = sorted(set(x.resolve(t) for t in r["timing"]), key=GRAHA.index)
        for m in x.dasha["mahadashas"]:
            for a in m["children"]:
                if a["end_jd"] <= lo or a["start_jd"] >= hi:
                    continue
                matched = [g for g in lords if g in (m["lord"], a["lord"])]
                if matched:
                    timing.append([r["id"], max(a["start_jd"], lo), min(a["end_jd"], hi),
                                   m["lord"], a["lord"], matched])
    return {"results": results, "withheld": withheld, "supporting": sup, "afflicting": aff,
            "cancelled": canc, "unknown": unk, "score": score, "label": label, "timing": timing}

def period_report(rules, x, mode, ages):
    prules = [dict(r, scope="natal", topic="__period") for r in rules if r.get("scope") == "period"]
    withheld = {}
    for r in rules:
        if r.get("scope") == "period" and not admits(mode, r["review"]["status"]):
            withheld[r["review"]["status"]] = withheld.get(r["review"]["status"], 0) + 1
    dpy = 365.25
    lo = x.dasha["birth_jd"] + ages[0] * dpy
    hi = x.dasha["birth_jd"] + ages[1] * dpy
    windows = []
    for m in x.dasha["mahadashas"]:
        for a in m["children"]:
            if a["end_jd"] <= lo or a["start_jd"] >= hi:
                continue
            x.period = (m["lord"], a["lord"])
            rep = topic_report(prules, "__period", x, mode, (0, 0))
            eff = {r["id"]: r for r in rep["results"] if r["effective"]}
            pol = {r["id"]: r["polarity"] for r in rep["results"]}
            windows.append({
                "maha": m["lord"], "antar": a["lord"],
                "start_jd": max(a["start_jd"], lo), "end_jd": min(a["end_jd"], hi),
                "score": rep["score"], "sensitive": rep["score"] <= -2,
                "amplifiers": sorted(i for i in eff if pol[i] < 0),
                "negators": sorted(i for i in eff if pol[i] > 0),
                "noted": sorted(i for i in eff if pol[i] == 0),
                "cancelled": rep["cancelled"],
            })
    x.period = None
    return {"withheld": withheld, "windows": windows}

# ---------------------------------------------------------------------------
# Synthetic corpora
# ---------------------------------------------------------------------------

def rand_ref(rng, period=False):
    if period and rng.random() < 0.35:
        return {"period": rng.choice(["maha", "antar"])}
    if rng.random() < 0.6:
        return rng.choice(GRAHA)
    ref = {"lord_of": rng.randint(1, 12)}
    if rng.random() < 0.3:
        ref["varga"] = rng.choice(VARGAS)
    return ref

def rand_set(rng):
    r = rng.random()
    if r < 0.6:
        return rng.choice(["benefics", "malefics", "any"])
    return {"grahas": [rand_ref(rng) for _ in range(rng.randint(1, 4))]}

def rand_houses(rng):
    return sorted(rng.sample(range(1, 13), rng.randint(1, 5)))

def rand_cond(rng, depth=0, period=False):
    rand_ref_p = lambda r: rand_ref(r, period)
    kinds = ["graha_in_house", "graha_in_sign", "graha_from", "dignity", "house_occupied",
             "house_aspected", "conjunct", "aspects_graha", "combust", "retrograde", "sav", "native",
             "functional", "rules_house", "sambandha", "neecha_bhanga"]
    if depth < 3 and rng.random() < 0.35:
        kind = rng.choice(["all", "any", "not"])
        if kind == "not":
            return {"not": rand_cond(rng, depth + 1, period)}
        return {kind: [rand_cond(rng, depth + 1, period) for _ in range(rng.randint(1, 4))]}
    k = rng.choice(kinds)
    maybe_varga = lambda d: ({**d, "varga": rng.choice(VARGAS)} if rng.random() < 0.4 else d)
    if k == "graha_in_house":
        return {k: maybe_varga({"graha": rand_ref_p(rng), "houses": rand_houses(rng)})}
    if k == "graha_in_sign":
        return {k: maybe_varga({"graha": rand_ref_p(rng), "signs": rng.sample(RASI, rng.randint(1, 5))})}
    if k == "graha_from":
        frm = "lagna" if rng.random() < 0.4 else rand_ref_p(rng)
        return {k: {"graha": rand_ref_p(rng), "from": frm, "houses": rand_houses(rng)}}
    if k == "dignity":
        return {k: maybe_varga({"graha": rand_ref_p(rng), "in": rng.sample(DIGNITIES, rng.randint(1, 4))})}
    if k == "house_occupied":
        return {k: maybe_varga({"house": rng.randint(1, 12), "by": rand_set(rng), "min": rng.randint(1, 3)})}
    if k == "house_aspected":
        return {k: {"house": rng.randint(1, 12), "by": rand_set(rng), "min": rng.randint(1, 3)}}
    if k == "conjunct":
        return {k: {"a": rand_ref_p(rng), "b": rand_ref_p(rng)}}
    if k == "aspects_graha":
        return {k: {"from": rand_ref_p(rng), "to": rand_ref_p(rng)}}
    if k in ("combust", "retrograde"):
        return {k: {"graha": rand_ref_p(rng)}}
    if k == "sav":
        lo = rng.randint(15, 35)
        choice = rng.random()
        if choice < 0.33:
            return {k: {"house": rng.randint(1, 12), "min": lo}}
        if choice < 0.66:
            return {k: {"house": rng.randint(1, 12), "max": lo}}
        return {k: {"house": rng.randint(1, 12), "min": lo, "max": lo + rng.randint(0, 10)}}
    if k == "native":
        return {k: {"sex": rng.choice(["female", "male"])}}
    if k == "functional":
        return {k: {"graha": rand_ref_p(rng), "is": rng.sample(["yogakaraka", "benefic", "malefic", "neutral"], rng.randint(1, 3))}}
    if k == "rules_house":
        return {k: {"graha": rand_ref_p(rng), "houses": rand_houses(rng)}}
    if k == "sambandha":
        d = {"a": rand_ref_p(rng), "b": rand_ref_p(rng)}
        if rng.random() < 0.5:
            d["kinds"] = rng.sample(["conjunct", "mutual_aspect", "exchange"], rng.randint(1, 3))
        return {k: d}
    return {k: {"graha": rand_ref_p(rng)}}

def rand_review(rng):
    s = rng.choices(["draft", "approved", "rejected"], [0.4, 0.45, 0.15])[0]
    if s == "approved":
        return {"status": s, "reviewer": "QA synthetic", "date": "2026-09-21", "comment": None}
    return {"status": s, "reviewer": None, "date": None, "comment": None}

def synthetic_corpus(rng, n=40, scope="natal"):
    rules = []
    for i in range(n):
        rid = f"qa.r{i:03d}"
        overrides = sorted({f"qa.r{j:03d}" for j in rng.sample(range(i), min(i, rng.randint(0, 3)))}) if i and rng.random() < 0.4 else []
        r = {"id": rid, "topic": "qa", "title": f"synthetic {i}", "tradition": "parashari",
             "when": rand_cond(rng, period=(scope == "period")), "polarity": rng.randint(-3, 3)}
        if scope == "period":
            r["scope"] = "period"
        if overrides:
            r["overrides"] = overrides
        if scope == "natal" and rng.random() < 0.25:
            r["timing"] = [rand_ref(rng) for _ in range(rng.randint(1, 3))]
        if rng.random() < 0.4:
            r["subject"] = rand_ref(rng, scope == "period")
        if rng.random() < 0.3:
            r["tags"] = sorted(rng.sample(["qa:a", "qa:b", "dosha:x"], rng.randint(1, 2)))
        r["text"] = {"en": "synthetic rule", "impact": f"what synthetic rule {i} means for the reader in plain words"}
        r["source"] = {"reference": None, "note": "QA synthetic corpus"}
        r["review"] = rand_review(rng)
        rules.append(r)
    return rules

# ---------------------------------------------------------------------------
# Harness
# ---------------------------------------------------------------------------

def lagn(*args):
    o = subprocess.run([str(LAGN), *args], capture_output=True, text=True)
    if o.returncode != 0:
        raise RuntimeError(o.stderr.strip())
    return json.loads(o.stdout)

def compare(mine, theirs, errs):
    tr = {r["id"]: r for r in theirs["results"]}
    if [r["id"] for r in mine["results"]] != [r["id"] for r in theirs["results"]]:
        errs.append(f"admitted rule ids differ: oracle {len(mine['results'])} vs kernel {len(theirs['results'])}")
        return
    code = {"true": "T", "false": "F", "unknown": "U"}
    for r in mine["results"]:
        k = tr[r["id"]]
        got = (code[k["outcome"]], k["effective"], k["cancelled_by"], k["polarity"], k["status"], k.get("subject"), k.get("tags", []), k.get("impact"))
        want = (r["outcome"], r["effective"], r["cancelled_by"], r["polarity"], r["status"], r["subject"], r["tags"], r["impact"])
        if got != want:
            errs.append(f"{r['id']}: kernel {got} oracle {want}")
    for f in ("withheld", "supporting", "afflicting", "cancelled", "unknown", "score", "label"):
        if mine[f] != theirs[f]:
            errs.append(f"{f}: kernel {theirs[f]!r} oracle {mine[f]!r}")
    kt = [[w["rule"], w["start_jd"], w["end_jd"], w["maha"], w["antar"], w["matched"]] for w in theirs["timing"]]
    if len(kt) != len(mine["timing"]):
        errs.append(f"timing windows: kernel {len(kt)} oracle {len(mine['timing'])}")
    else:
        for a, b in zip(kt, mine["timing"]):
            if a[0] != b[0] or a[3:] != b[3:] or abs(a[1] - b[1]) > 1e-6 or abs(a[2] - b[2]) > 1e-6:
                errs.append(f"timing window: kernel {a} oracle {b}")
                break

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--charts", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=3)
    a = ap.parse_args()
    rng = random.Random(a.seed)

    # Every shipped rule file (catalogue files at the corpus root are not rules).
    real_rules = []
    for f in sorted(CORPUS.glob("*/*.json")):
        if f.name != "porutham.review.json":
            real_rules += json.loads(f.read_text())
    real_topics = sorted({r["topic"] for r in real_rules if r.get("scope", "natal") == "natal"})
    real_period = [r for r in real_rules if r.get("scope") == "period"]
    print(f"real corpus: {len(real_rules)} rules, topics {real_topics}, {len(real_period)} period rules")
    tmp = Path(tempfile.mkdtemp(prefix="lagn-qa3-"))
    stats = {"rules_checked": 0, "outcomes": {"T": 0, "F": 0, "U": 0}, "cancelled": 0,
             "windows": 0, "modes": {"review": 0, "production": 0}, "kinds": {}}
    failures = 0
    try:
        for i in range(a.charts):
            birth = P2.random_birth(rng)
            settings = {k: rng.choice(list(m.keys())) for k, (_, m) in P2.VARIANT_FLAGS.items()} \
                if rng.random() < 0.3 else dict(P2.DEFAULTS)
            flags = []
            for k, v in settings.items():
                flag, mapping = P2.VARIANT_FLAGS[k]
                flags += [flag, mapping[v]]
            sex = rng.choice([None, "female", "male"])
            ages = sorted(rng.sample(range(0, 100), 2))
            mode = rng.choice(["review", "production"])

            # Alternate: real corpus (possibly with random approvals) and synthetic corpora.
            kind = ["real", "real_approved", "synthetic"][i % 3]
            cdir = tmp / f"c{i % 5}"
            if cdir.exists():
                shutil.rmtree(cdir)
            cdir.mkdir()
            if kind == "real":
                rules, topic = real_rules, rng.choice(real_topics)
            elif kind == "real_approved":
                rules = copy.deepcopy(real_rules)
                for r in rules:
                    r["review"] = rand_review(rng)
                topic = rng.choice(real_topics)
            else:
                rules, topic = synthetic_corpus(rng), "qa"
            (cdir / "rules.json").write_text(json.dumps(rules))

            args = ["topic", topic, *birth, *flags, "--corpus", str(cdir),
                    "--from-age", str(ages[0]), "--to-age", str(ages[1]), "--json"]
            if mode == "review":
                args.append("--review")
            if sex:
                args += ["--sex", sex]
            try:
                chart = lagn("chart", *birth, "--json")
                dasha = lagn("dasha", *birth, "--json")
                theirs = lagn(*args)
            except RuntimeError as e:
                failures += 1
                print(f"  ERROR chart {i} ({kind}): {e}")
                continue
            x = Ctx(chart, dasha, settings, sex)
            mine = topic_report(rules, topic, x, mode, ages)
            errs = []
            compare(mine, theirs, errs)

            # Period rules: a synthetic period corpus through `lagn periods`.
            if i % 2 == 0:
                # Alternate the shipped period rules with a synthetic period corpus.
                prules = copy.deepcopy(real_period) if i % 4 == 0 else synthetic_corpus(rng, n=25, scope="period")
                (cdir / "rules.json").write_text(json.dumps(prules))
                pargs = ["periods", *birth, *flags, "--corpus", str(cdir),
                         "--from-age", str(ages[0]), "--to-age", str(min(ages[1], ages[0] + 30)), "--json"]
                if mode == "review": pargs.append("--review")
                if sex: pargs += ["--sex", sex]
                ptheirs = lagn(*pargs)
                pmine = period_report(prules, x, mode, (ages[0], min(ages[1], ages[0] + 30)))
                if ptheirs["withheld"] != pmine["withheld"]:
                    errs.append(f"period withheld: kernel {ptheirs['withheld']} oracle {pmine['withheld']}")
                if len(ptheirs["windows"]) != len(pmine["windows"]):
                    errs.append(f"period windows: kernel {len(ptheirs['windows'])} oracle {len(pmine['windows'])}")
                for kw, mw in zip(ptheirs["windows"], pmine["windows"]):
                    got = (kw["maha"], kw["antar"], kw["score"], kw["sensitive"],
                           sorted(r["id"] for r in kw["amplifiers"]), sorted(r["id"] for r in kw["negators"]),
                           sorted(r["id"] for r in kw["noted"]), kw["cancelled"])
                    want = (mw["maha"], mw["antar"], mw["score"], mw["sensitive"], mw["amplifiers"],
                            mw["negators"], mw["noted"], mw["cancelled"])
                    if got != want or abs(kw["start_jd"] - mw["start_jd"]) > 1e-6 or abs(kw["end_jd"] - mw["end_jd"]) > 1e-6:
                        errs.append(f"period window {kw['maha']}/{kw['antar']}: kernel {got} oracle {want}")
                        break
                stats["period_windows"] = stats.get("period_windows", 0) + len(pmine["windows"])

            # Gate property: production never shows anything unapproved.
            if mode == "production" and any(r["status"] != "approved" for r in theirs["results"]):
                errs.append("GATE LEAK: unapproved rule in production output")

            stats["modes"][mode] += 1
            stats["kinds"][kind] = stats["kinds"].get(kind, 0) + 1
            if kind != "synthetic":
                stats.setdefault("topics", {})[topic] = stats.setdefault("topics", {}).get(topic, 0) + 1
            stats["rules_checked"] += len(mine["results"])
            for r in mine["results"]:
                stats["outcomes"][r["outcome"]] += 1
            stats["cancelled"] += len(mine["cancelled"])
            stats["windows"] += len(mine["timing"])
            if errs:
                failures += 1
                print(f"  FAIL chart {i} ({kind}, {mode}): {' '.join(birth)}")
                for e in errs[:6]:
                    print(f"       {e}")
            if (i + 1) % 500 == 0:
                print(f"  ... {i + 1} charts, {stats['rules_checked']:,} rule evaluations, {failures} failing")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    print(f"\n{a.charts} charts: {stats['kinds']}, modes {stats['modes']}")
    print(f"real topics exercised: {stats.get('topics', {})}")
    print(f"period windows compared: {stats.get('period_windows', 0):,}")
    print(f"{stats['rules_checked']:,} rule evaluations; outcomes {stats['outcomes']}; "
          f"{stats['cancelled']:,} cancellations; {stats['windows']:,} timing windows")
    if failures:
        print(f"\n{failures} FAILING CHARTS")
        return 1
    print("\nkernel and oracle agree on every rule, report field and timing window")
    return 0

if __name__ == "__main__":
    sys.exit(main())
