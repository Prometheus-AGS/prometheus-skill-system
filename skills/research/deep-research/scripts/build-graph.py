#!/usr/bin/env python3
"""build-graph.py — stage 07 graph builder for the deep-research pipeline.

Invoked by build-graph.sh, which passes REGISTRY, CRED, CONTRA, PKG_ID,
CRITICAL and OUT through the environment. Kept as a sibling file rather than
a heredoc inside the wrapper: the heredoc form hangs indefinitely on some
hosts (observed 2026-09-09 — this exact program blocked past 60s inline and
ran in under a second from a file), and a stage that hangs is worse than one
that fails. Same convention as score-sources.py beside verify-sources.sh.
"""

import hashlib, json, os, re, sys
from urllib.parse import urlparse

def load(path):
    if not path or not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        return json.load(f)

# --- claim id: identical in detect-contradictions.sh ------------------------
def normalise(text):
    t = text.lower().strip()
    t = re.sub(r"\s+", " ", t)
    return t.rstrip(" .;:,!")

def claim_id(scope, text):
    return "claim-" + hashlib.sha256(f"{scope}:{normalise(text)}".encode("utf-8")).hexdigest()[:16]
# -----------------------------------------------------------------------------

LABEL_RANK = {"blocked": 0, "unverified": 1, "inferred": 2, "verified": 3}
scope = os.environ["PKG_ID"]

registry = load(os.environ["REGISTRY"])
sources = registry.get("sources") if isinstance(registry, dict) else registry
if not isinstance(sources, list):
    sys.exit("[build-graph] registry has no sources array")
cred = load(os.environ.get("CRED")) or {}
contra = load(os.environ.get("CONTRA")) or {}
critical_texts = set()
if os.environ.get("CRITICAL") and os.path.exists(os.environ["CRITICAL"]):
    with open(os.environ["CRITICAL"], encoding="utf-8") as f:
        critical_texts = {normalise(l) for l in f if l.strip()}

# Source id: registry entity_id, else the URL.
def source_id(src):
    return src.get("entity_id") or src.get("url")

score_by_url = {}
label_by_url_text = {}
evidence_by_url_text = {}
for s in (cred.get("verified_sources") or []) + (cred.get("filtered_sources") or []):
    score_by_url[s.get("url")] = s.get("credibility_score")
    for c in s.get("claims") or []:
        if isinstance(c, dict) and c.get("text"):
            key = (s.get("url"), normalise(c["text"]))
            label_by_url_text[key] = c.get("label") if c.get("label") in LABEL_RANK else "unverified"
            evidence_by_url_text[key] = c.get("evidence")
for u, sc in (cred.get("credibility_scores") or {}).items():
    score_by_url.setdefault(u, sc)

claims = {}   # id -> claim dict (merged)
order = []
def upsert(text, label, source, confidence, evidence, critical):
    cid = claim_id(scope, text)
    if cid not in claims:
        claims[cid] = {"id": cid, "text": text.strip(), "label": label, "critical": bool(critical),
                       "confidence": confidence, "sources": [], "contradicts": [], "evidence": evidence}
        order.append(cid)
    c = claims[cid]
    # Merge: the higher label wins, sources union, evidence from the winning label.
    if LABEL_RANK[label] > LABEL_RANK[c["label"]]:
        c["label"] = label
        c["evidence"] = evidence
    elif LABEL_RANK[label] == LABEL_RANK[c["label"]] and not c.get("evidence") and evidence:
        c["evidence"] = evidence
    if source and source not in c["sources"]:
        c["sources"].append(source)
    c["confidence"] = round(max(c["confidence"], confidence), 3)
    c["critical"] = c["critical"] or bool(critical)
    return cid

for src in sources:
    url = src.get("url")
    sid = source_id(src)
    score = score_by_url.get(url)
    conf = round((score if isinstance(score, (int, float)) else 50) / 100.0, 3)
    for c in src.get("claims") or []:
        text = c.get("text") if isinstance(c, dict) else c
        if not isinstance(text, str) or not text.strip():
            continue
        key = (url, normalise(text))
        label = label_by_url_text.get(key)
        if label is None:
            label = c.get("label") if isinstance(c, dict) and c.get("label") in LABEL_RANK else "unverified"
        evidence = evidence_by_url_text.get(key) or (c.get("evidence") if isinstance(c, dict) else None)
        critical = (isinstance(c, dict) and bool(c.get("critical"))) or normalise(text) in critical_texts
        upsert(text, label, sid, conf if label != "unverified" else min(conf, 0.5), evidence, critical)

# A verified label is only as good as its evidence. One that arrived without a
# passage is DOWNGRADED to unverified (the honest label for "a source is cited,
# support not shown"), so the package-level derivation never counts it as
# verified; the note says why so the verifier can repair it. A blocked label
# without a recorded attempt keeps its label (it is already the lowest) and
# says the attempt was not recorded.
for c in claims.values():
    if c["label"] == "verified" and not c.get("evidence"):
        c["label"] = "unverified"
        c["confidence"] = min(c["confidence"], 0.5)
        c["evidence"] = "downgraded from verified: credibility.json carried the label without a supporting passage; re-read the source to restore it"
    elif c["label"] == "blocked" and not c.get("evidence"):
        c["evidence"] = "blocked without a recorded attempt"
    elif c["label"] not in ("verified", "blocked") and not c.get("evidence"):
        c.pop("evidence", None)

# Relations: cites, then contradicts from contradictions.json.
relations = []
for cid in order:
    for sid in claims[cid]["sources"]:
        relations.append({"from": cid, "to": sid, "type": "cites"})

topic_claims = {}   # topic name -> [claim ids]
for entry in contra.get("contradictions") or []:
    a, b = entry.get("claim_a") or {}, entry.get("claim_b") or {}
    ta, tb = a.get("text"), b.get("text")
    if not (isinstance(ta, str) and isinstance(tb, str)):
        continue
    # An entry's own id is honoured when the graph already has it; otherwise
    # the claim is (re)addressed by content, so a legacy `claim-NNN` id or an
    # id from another scope never dereferences a claim that does not exist.
    def resolve(side, text):
        supplied = side.get("id")
        if supplied in claims:
            return supplied
        return upsert(text, "unverified", side.get("source"), 0.5, None, False)
    ida = resolve(a, ta)
    idb = resolve(b, tb)
    if ida == idb:
        continue
    if idb not in claims[ida]["contradicts"]:
        claims[ida]["contradicts"].append(idb)
    if ida not in claims[idb]["contradicts"]:
        claims[idb]["contradicts"].append(ida)
    pair = {"from": ida, "to": idb, "type": "contradicts"}
    if pair not in relations:
        relations.append(pair)
    topic = entry.get("topic")
    if topic:
        topic_claims.setdefault(topic, [])
        for cid in (ida, idb):
            if cid not in topic_claims[topic]:
                topic_claims[topic].append(cid)

# Topics: contradiction topics first, then one per source domain for the rest.
assigned = {cid for ids in topic_claims.values() for cid in ids}
for cid in order:
    if cid in assigned:
        continue
    src = claims[cid]["sources"][0] if claims[cid]["sources"] else None
    host = ""
    if src:
        host = urlparse(src).hostname or src if "://" in src else src
    name = host or "unattributed"
    topic_claims.setdefault(name, []).append(cid)
topics = [{"id": f"topic-{i:03d}", "name": name, "claims": ids} for i, (name, ids) in enumerate(topic_claims.items(), 1)]

graph = {"topics": topics, "claims": [claims[c] for c in order], "relations": relations}
out = os.environ.get("OUT")
text = json.dumps(graph, indent=2) + "\n"
if out:
    with open(out, "w", encoding="utf-8") as f:
        f.write(text)
else:
    sys.stdout.write(text)
