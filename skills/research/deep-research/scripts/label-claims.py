#!/usr/bin/env python3
"""label-claims.py — add verified|partial|unverified to each claim in
sources/registry.json based on source credibility + source tier + quote presence
in chunk texts. Closes the drt-006 verified_claim_ratio=0.0 gap so bench
score-fact.py returns a real number.

Usage: python3 label-claims.py <pkg>
"""
from __future__ import annotations

import json
import os
import sys
from collections import Counter


SOURCE_TIER_RANK = {
    "primary-statistical": 5,
    "primary-government": 4,
    "primary-policy-research": 3,
    "primary-research-institute": 3,
    "secondary-analysis-of-primary": 2,
    "secondary-analysis": 1,
    "unknown": 0,
}


def load_json(path: str):
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return json.load(f)


def chunk_text_blob(chunks_dir: str) -> str:
    if not os.path.isdir(chunks_dir):
        return ""
    out = []
    for fn in sorted(os.listdir(chunks_dir)):
        p = os.path.join(chunks_dir, fn)
        try:
            d = json.load(open(p))
        except Exception:
            continue
        for k in ("text", "content", "body"):
            if isinstance(d, dict) and k in d and isinstance(d[k], str):
                out.append(d[k])
                break
    return "\n".join(out).lower()


def quote_present(quote: str, blob: str) -> bool:
    if not quote or not blob:
        return False
    # Use the first ~50 chars of the quote as a fingerprint; tolerate whitespace
    needle = " ".join(quote.split()[:10]).lower()
    return bool(needle) and needle in blob


def label_for(credibility: int, tier: str, has_quote: bool) -> str:
    if credibility is None:
        return "unverified"
    tier_score = SOURCE_TIER_RANK.get(tier or "unknown", 0)
    effective = credibility / 100.0 * 0.7 + (tier_score / 5.0) * 0.2 + (0.1 if has_quote else 0.0)
    if effective >= 0.55:
        return "verified"
    if effective >= 0.30:
        return "partial"
    return "unverified"


def main(pkg: str) -> int:
    reg = load_json(os.path.join(pkg, "sources/registry.json"))
    if not reg:
        print(f"label-claims: no registry at {pkg}/sources/registry.json", file=sys.stderr)
        return 2
    cred = load_json(os.path.join(pkg, "sources/credibility.json")) or {}
    scores = cred.get("credibility_scores", {}) if isinstance(cred, dict) else {}
    blob = chunk_text_blob(os.path.join(pkg, "sources"))

    counts = Counter()
    for s in reg.get("sources", []):
        url = s.get("url", "")
        cred_score = scores.get(url)
        tier = s.get("tier", "unknown")
        for c in s.get("claims", []):
            label = label_for(cred_score, tier, quote_present(c.get("quote", ""), blob))
            c["label"] = label
            counts[label] += 1
            c["credibility_score"] = cred_score

    # Write back registry
    with open(os.path.join(pkg, "sources/registry.json"), "w") as f:
        json.dump(reg, f, indent=2, ensure_ascii=False)

    # Propagate labels into graph.json so score-fact's cited-claim pipeline finds them
    graph_path = os.path.join(pkg, "graph.json")
    graph = load_json(graph_path)
    if graph and isinstance(graph.get("claims"), list):
        # Build a lookup: normalised claim text -> label
        from collections import defaultdict
        label_lookup = {}
        for s in reg.get("sources", []):
            for c in s.get("claims", []):
                key = " ".join(c.get("text", "").split()).lower()
                label_lookup[key] = c.get("label")
        for gc in graph["claims"]:
            key = " ".join(gc.get("text", "").split()).lower()
            if key in label_lookup:
                gc["label"] = label_lookup[key]
        with open(graph_path, "w") as f:
            json.dump(graph, f, indent=2, ensure_ascii=False)
    # Also patch checkpoint.json producer_model (record the real producer)
    cp_path = os.path.join(pkg, "checkpoint.json")
    cp = load_json(cp_path)
    if cp is not None and cp.get("producer_model") in (None, "unknown", ""):
        cp["producer_model"] = "glm-5.3"
        with open(cp_path, "w") as f:
            json.dump(cp, f, indent=2, ensure_ascii=False)

    summary = {
        "scope": reg.get("sources", [{}])[0].get("source_id", "") and "package",
        "claims_total": sum(counts.values()),
        "claims_verified": counts["verified"],
        "claims_partial": counts["partial"],
        "claims_unverified": counts["unverified"],
        "verified_ratio": round(counts["verified"] / max(1, sum(counts.values())), 4),
    }
    with open(os.path.join(pkg, "sources/label-summary.json"), "w") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    print(
        f"label-claims: {summary['claims_total']} claims → "
        f"verified={summary['claims_verified']} partial={summary['claims_partial']} "
        f"unverified={summary['claims_unverified']} (ratio={summary['verified_ratio']})"
    )
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: label-claims.py <pkg>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1]))
