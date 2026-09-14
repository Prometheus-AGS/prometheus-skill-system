#!/usr/bin/env python3
"""score-sources.py — stage 05 source credibility with available-signal weight
renormalisation and rank sensitivity (change-rah-010, analysis D-11).

Scores every source in a stage 04 registry on the five documented rubric
dimensions (agents/source-verifier.md), but only where the registry carries
evidence for a dimension. A dimension with no evidence is EXCLUDED from that
source's weight denominator instead of scoring zero, and the renormalised
vector is recorded per source as `applied_weights` (sums to 1 over the
dimensions that were present). This is Feynman CLI's `combineSignals`
(companion-inc/feynman, MIT, src/rank/paper-rank.ts) applied to web sources.

The same sources are then re-ranked under four alternate weight vectors and
each rank is classified `stable` (rank never moves), `sensitive` (moves by at
most two places), or `volatile` (moves further), following Feynman CLI's
`generateRankSensitivity`. Rank order is an audit signal, not a verdict: a
volatile rank should be inspected before the order is treated as decisive.

Usage:
  score-sources.py --registry sources/registry.json [--chunks-dir sources]
                   [--threshold 40] [--penalties penalties.json] [--now ISO8601]
                   [--out credibility.json] [--sensitivity sensitivity.json]
                   [--weights weights.json] [--profiles profiles.json]
  score-sources.py --urls-from -            # legacy: one URL per stdin line

Inputs the registry MAY carry per source (all optional except url):
  url, entity_id, claims[] (strings or {text,label,evidence}), chunk_ids[],
  author | authors[], author_credentials | affiliation, published | date,
  references | reference_count | citation_count, title.
Chunk files (chunk-<n>.json with {url, chunk_id, text}) supply reference and
verifiability evidence when the registry does not.

Outputs:
  credibility.json  {verified_sources[], filtered_sources[], credibility_scores{},
                     threshold, weights, scored_at, scorer}
  sensitivity.json  {generated_at, profiles[], sources[], summary{}, basis[]}

Exit codes: 0 ok · 1 usage or unreadable input · 2 no sources.
Claims keep the label they arrive with; a bare string becomes
{text, label: "unverified", evidence: null}. Only the verifier agent (stage 05)
may assign `verified`, after re-reading the chunk.
"""
from __future__ import annotations

import argparse
import datetime as dt
import glob
import hashlib
import json
import os
import re
import sys
from urllib.parse import urlparse

DIMENSIONS = (
    "domain_authority",
    "author_expertise",
    "citation_depth",
    "publication_recency",
    "factual_verifiability",
)

# The documented rubric weights (stage-05 SKILL.md). Renormalised per source over
# the dimensions that carry evidence.
DEFAULT_WEIGHTS = {
    "domain_authority": 0.25,
    "author_expertise": 0.20,
    "citation_depth": 0.20,
    "publication_recency": 0.20,
    "factual_verifiability": 0.15,
}

# Four alternate weight vectors (spec open question, default answer). Each is a
# different reasonable reading of "credible"; a rank that survives all of them
# is robust to the weighting choice.
DEFAULT_PROFILES = [
    {
        "id": "balanced",
        "label": "Balanced",
        "description": "Every dimension weighted equally.",
        "weights": {k: 0.20 for k in DIMENSIONS},
    },
    {
        "id": "authority_heavy",
        "label": "Authority heavy",
        "description": "Stresses domain authority and author expertise over freshness.",
        "weights": {
            "domain_authority": 0.35,
            "author_expertise": 0.30,
            "citation_depth": 0.15,
            "publication_recency": 0.10,
            "factual_verifiability": 0.10,
        },
    },
    {
        "id": "recency_heavy",
        "label": "Recency heavy",
        "description": "Stresses publication recency for fast-moving topics.",
        "weights": {
            "domain_authority": 0.15,
            "author_expertise": 0.10,
            "citation_depth": 0.15,
            "publication_recency": 0.45,
            "factual_verifiability": 0.15,
        },
    },
    {
        "id": "methodology_heavy",
        "label": "Methodology heavy",
        "description": "Stresses citation depth and factual verifiability over popularity.",
        "weights": {
            "domain_authority": 0.10,
            "author_expertise": 0.10,
            "citation_depth": 0.35,
            "publication_recency": 0.10,
            "factual_verifiability": 0.35,
        },
    },
]

HIGH_AUTHORITY = (
    ".edu", ".gov", "arxiv.org", "nature.com", "science.org", "acm.org", "ieee.org",
    "nih.gov", "ncbi.nlm.nih.gov", "pubmed.ncbi.nlm.nih.gov", "scholar.google.com",
    "who.int", "europa.eu", "rfc-editor.org", "w3.org",
)
ESTABLISHED = (
    "wikipedia.org", "github.com", "docs.", "developer.", "reuters.com", "apnews.com",
    "bbc.co.uk", "nytimes.com", "theguardian.com", "economist.com", "ft.com",
)
LOW_QUALITY = (
    "reddit.com", "twitter.com", "x.com", "facebook.com", "ehow.com", "answers.com",
    "quora.com", "medium.com", "buzzfeed.com", "huffpost.com", "pinterest.com",
    "tiktok.com",
)

NUMBER_RE = re.compile(r"\d")
SPECIFIC_RE = re.compile(r"\d|\b(?:19|20)\d{2}\b|%|\bv\d|\bversion\b|\bbenchmark|\bmeasured|\bp\s*[<=]", re.I)
REFERENCE_RE = re.compile(r"doi\.org/|arxiv\.org/|\[\d+\]|\bet al\.|\(\d{4}\)|references\b|bibliography\b", re.I)


def die(msg: str, code: int = 1) -> None:
    print(f"[score-sources] {msg}", file=sys.stderr)
    sys.exit(code)


def load_json(path: str):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except OSError as e:
        die(f"cannot read {path}: {e}")
    except ValueError as e:
        die(f"{path} is not valid JSON: {e}")


def domain_of(url: str) -> str:
    try:
        host = urlparse(url).hostname or ""
    except ValueError:
        host = ""
    return host.lower()


def parse_date(value) -> dt.date | None:
    """Accept ISO datetimes, dates, year-month, and bare years. Year-month and
    year resolve to the first day so recency is computed, not silently lost."""
    if value is None:
        return None
    s = str(value).strip()
    if not s:
        return None
    # (format, rendered length) pairs: strptime needs the exact rendered width,
    # which is not len(fmt) — "%Y-%m" renders as 7 characters, "%Y" as 4.
    for fmt, width in (("%Y-%m-%dT%H:%M:%S", 19), ("%Y-%m-%d", 10), ("%Y-%m", 7), ("%Y", 4)):
        head = s[:width]
        if len(head) != width:
            continue
        try:
            return dt.datetime.strptime(head, fmt).date()
        except ValueError:
            continue
    return None


# ---------------------------------------------------------------------------
# Signals: value in [0, 1], available, explanation
# ---------------------------------------------------------------------------

def signal(value: float, available: bool, explanation: str) -> dict:
    return {"value": round(max(0.0, min(1.0, value)), 3), "available": available, "explanation": explanation}


def sig_domain_authority(src: dict) -> dict:
    url = src.get("url", "")
    host = domain_of(url)
    if not host:
        return signal(0.0, False, "no parseable URL host")
    flags = []
    if any(host.endswith(d) or d in host for d in HIGH_AUTHORITY):
        value, why = 1.0, "high-authority domain"
        flags.append("high_authority_domain")
    elif any(d in host for d in ESTABLISHED):
        value, why = 0.7, "established publication or primary documentation"
    elif any(host.endswith(d) or d in host for d in LOW_QUALITY):
        value, why = 0.15, "low-quality or social domain"
        flags.append("low_quality_domain")
    else:
        value, why = 0.5, "unlisted domain, neutral prior"
    if not url.startswith("https://"):
        value -= 0.1
        why += "; not https"
        flags.append("no_https")
    s = signal(value, True, why)
    s["flags"] = flags
    return s


def sig_author_expertise(src: dict) -> dict:
    author = src.get("author") or (src.get("authors") or [None])[0]
    if not author:
        return signal(0.0, False, "no author recorded")
    cred = src.get("author_credentials") or src.get("affiliation")
    if cred:
        return signal(1.0, True, f"named author with credentials: {author} ({cred})")
    return signal(0.6, True, f"named author, credentials not recorded: {author}")


def sig_citation_depth(src: dict, chunk_text: str) -> dict:
    for key in ("references", "reference_count", "citation_count"):
        v = src.get(key)
        if isinstance(v, list):
            n = len(v)
        elif isinstance(v, (int, float)):
            n = int(v)
        else:
            continue
        return signal(min(n / 5.0, 1.0), True, f"{n} reference(s) recorded in {key}")
    if chunk_text:
        hits = len(REFERENCE_RE.findall(chunk_text))
        if hits:
            return signal(min(hits / 5.0, 1.0), True, f"{hits} reference marker(s) in fetched text")
        return signal(0.1, True, "fetched text carries no reference markers")
    return signal(0.0, False, "no reference metadata and no fetched text")


def sig_publication_recency(src: dict, now: dt.date) -> dict:
    published = parse_date(src.get("published") or src.get("date") or src.get("published_at"))
    if published is None:
        return signal(0.0, False, "no publication date recorded")
    age_days = (now - published).days
    if age_days < 0:
        return signal(1.0, True, f"published {published.isoformat()} (future-dated relative to --now)")
    if age_days <= 365:
        value = 1.0
    elif age_days <= 3 * 365:
        value = 1.0 - 0.8 * (age_days - 365) / (2 * 365)
    else:
        value = 0.1
    return signal(value, True, f"published {published.isoformat()}, {age_days} days before scoring")


def claim_texts(src: dict) -> list[str]:
    out = []
    for c in src.get("claims") or []:
        if isinstance(c, dict):
            t = c.get("text")
        else:
            t = c
        if isinstance(t, str) and t.strip():
            out.append(t.strip())
    return out


def sig_factual_verifiability(src: dict) -> dict:
    texts = claim_texts(src)
    if not texts:
        return signal(0.0, False, "no claims extracted")
    specific = sum(1 for t in texts if SPECIFIC_RE.search(t))
    return signal(specific / len(texts), True, f"{specific} of {len(texts)} claim(s) carry a number, date, or measurable term")


# ---------------------------------------------------------------------------
# Combination (Feynman CLI combineSignals): renormalise over available signals
# ---------------------------------------------------------------------------

def combine(signals: dict, weights: dict) -> tuple[float, dict]:
    available = [(k, s) for k, s in signals.items() if s["available"]]
    denominator = sum(weights[k] for k, _ in available)
    if denominator == 0:
        return 0.0, {}
    applied = {}
    total = 0.0
    for k, s in available:
        w = weights[k] / denominator
        applied[k] = round(w, 4)
        total += s["value"] * w
    return total, applied


def rank(scored: list[dict], key: str = "credibility_score") -> list[dict]:
    return sorted(scored, key=lambda s: (-s[key], s["url"]))


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def normalise_claims(src: dict) -> list[dict]:
    out = []
    for c in src.get("claims") or []:
        if isinstance(c, dict):
            text = str(c.get("text", "")).strip()
            if not text:
                continue
            label = c.get("label") if c.get("label") in ("verified", "unverified", "blocked", "inferred") else "unverified"
            out.append({"text": text, "label": label, "evidence": c.get("evidence")})
        elif isinstance(c, str) and c.strip():
            out.append({"text": c.strip(), "label": "unverified", "evidence": None})
    return out


def read_chunks(chunks_dir: str | None) -> dict:
    """url -> concatenated fetched text, from chunk-<n>.json files."""
    texts: dict[str, list[str]] = {}
    if not chunks_dir or not os.path.isdir(chunks_dir):
        return {}
    for p in sorted(glob.glob(os.path.join(chunks_dir, "chunk-*.json"))):
        try:
            with open(p, encoding="utf-8") as f:
                c = json.load(f)
        except (OSError, ValueError):
            continue
        url = c.get("url")
        if isinstance(url, str) and isinstance(c.get("text"), str):
            texts.setdefault(url, []).append(c["text"])
    return {u: "\n".join(t) for u, t in texts.items()}


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="score-sources.py", description=__doc__.split("\n\n")[0])
    ap.add_argument("--registry", help="stage 04 sources/registry.json ({sources:[...]} or a bare array)")
    ap.add_argument("--urls-from", help="legacy: file of URLs, one per line ('-' for stdin)")
    ap.add_argument("--chunks-dir", help="directory holding chunk-<n>.json (defaults to the registry's directory)")
    ap.add_argument("--threshold", type=float, default=40.0, help="minimum score to retain (default 40)")
    ap.add_argument("--penalties", help="JSON {url: penalty_points} from the sycophancy check (subtracted, floor 0, cap 30)")
    ap.add_argument("--now", help="ISO date used for recency (default: today, UTC)")
    ap.add_argument("--weights", help="JSON weight vector overriding the rubric defaults")
    ap.add_argument("--profiles", help="JSON list of alternate weight profiles for sensitivity")
    ap.add_argument("--out", help="write credibility.json here (default stdout)")
    ap.add_argument("--sensitivity", help="write sensitivity.json here")
    ap.add_argument("--legacy-array", action="store_true", help="print the pre-rah-010 [{url, credibility_score, flags}] array")
    args = ap.parse_args(argv)

    if not args.registry and not args.urls_from:
        ap.error("--registry or --urls-from is required")

    if args.urls_from:
        fh = sys.stdin if args.urls_from == "-" else open(args.urls_from, encoding="utf-8")
        with fh:
            urls = [line.strip() for line in fh if line.strip()]
        sources = [{"url": u} for u in urls]
        chunks_dir = args.chunks_dir
    else:
        reg = load_json(args.registry)
        sources = reg.get("sources") if isinstance(reg, dict) else reg
        if not isinstance(sources, list):
            die("registry has no sources array")
        chunks_dir = args.chunks_dir or os.path.dirname(os.path.abspath(args.registry))
    sources = [s for s in sources if isinstance(s, dict) and s.get("url")]
    if not sources:
        die("no sources to score", 2)

    weights = dict(DEFAULT_WEIGHTS)
    if args.weights:
        w = load_json(args.weights)
        if set(w) != set(DIMENSIONS):
            die(f"--weights must name exactly {list(DIMENSIONS)}")
        weights = {k: float(w[k]) for k in DIMENSIONS}
    profiles = DEFAULT_PROFILES
    if args.profiles:
        profiles = load_json(args.profiles)
        for p in profiles:
            if set(p.get("weights", {})) != set(DIMENSIONS):
                die(f"profile {p.get('id')} must weight exactly {list(DIMENSIONS)}")
    penalties = load_json(args.penalties) if args.penalties else {}
    now = parse_date(args.now) if args.now else dt.datetime.now(dt.timezone.utc).date()
    if now is None:
        die(f"--now {args.now!r} is not an ISO date")
    chunk_text = read_chunks(chunks_dir)

    scored = []
    for src in sources:
        url = src["url"]
        signals = {
            "domain_authority": sig_domain_authority(src),
            "author_expertise": sig_author_expertise(src),
            "citation_depth": sig_citation_depth(src, chunk_text.get(url, "")),
            "publication_recency": sig_publication_recency(src, now),
            "factual_verifiability": sig_factual_verifiability(src),
        }
        flags = list(signals["domain_authority"].pop("flags", []))
        value, applied = combine(signals, weights)
        penalty = 0.0
        if isinstance(penalties, dict) and url in penalties:
            try:
                penalty = min(max(float(penalties[url]), 0.0), 30.0)
            except (TypeError, ValueError):
                penalty = 0.0
            if penalty:
                flags.append("sycophancy_penalty")
        score = max(0, round(value * 100 - penalty))
        missing = [k for k, s in signals.items() if not s["available"]]
        if missing:
            flags.append("partial_evidence:" + ",".join(missing))
        scored.append({
            "url": url,
            "entity_id": src.get("entity_id"),
            "credibility_score": score,
            "applied_weights": applied,
            "signals": signals,
            "sycophancy_penalty": penalty,
            "flags": flags,
            "claims": normalise_claims(src),
            "chunk_ids": src.get("chunk_ids") or [],
        })

    ranked = rank(scored)
    for i, s in enumerate(ranked, 1):
        s["rank"] = i
    retained = [s for s in ranked if s["credibility_score"] >= args.threshold]
    filtered = [s for s in ranked if s["credibility_score"] < args.threshold]
    for s in filtered:
        s["filter_reason"] = f"score {s['credibility_score']} below threshold {args.threshold:g}"

    scored_at = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    credibility = {
        "verified_sources": retained,
        "filtered_sources": filtered,
        "credibility_scores": {s["url"]: s["credibility_score"] for s in ranked},
        "threshold": args.threshold,
        "weights": weights,
        "scored_at": scored_at,
        "scorer": "score-sources.py (renormalised over available signals; see sensitivity.json)",
    }

    # Sensitivity: rerun the same signals under each profile, same per-source
    # renormalisation, and classify every rank by how far it moves.
    base_rank = {s["url"]: s["rank"] for s in ranked}
    base_score = {s["url"]: s["credibility_score"] for s in ranked}
    profile_ranks: dict[str, list[dict]] = {s["url"]: [] for s in ranked}
    for p in profiles:
        alt = []
        for s in ranked:
            v, applied = combine(s["signals"], p["weights"])
            alt.append({"url": s["url"], "credibility_score": max(0, round(v * 100 - s["sycophancy_penalty"])), "applied_weights": applied})
        for i, a in enumerate(rank(alt), 1):
            profile_ranks[a["url"]].append({"profile_id": p["id"], "rank": i, "score": a["credibility_score"], "applied_weights": a["applied_weights"]})
    sens_sources = []
    for s in ranked:
        ranks = [base_rank[s["url"]]] + [r["rank"] for r in profile_ranks[s["url"]]]
        scores = [base_score[s["url"]]] + [r["score"] for r in profile_ranks[s["url"]]]
        rank_range = max(ranks) - min(ranks)
        stability = "stable" if rank_range == 0 else ("sensitive" if rank_range <= 2 else "volatile")
        drivers = []
        missing = [k for k, sig in s["signals"].items() if not sig["available"]]
        if missing:
            drivers.append("missing evidence for " + ", ".join(missing) + "; the remaining weights are renormalised, so profiles that lean on them swing this rank")
        strong = [k for k, sig in s["signals"].items() if sig["available"] and sig["value"] >= 0.8]
        weak = [k for k, sig in s["signals"].items() if sig["available"] and sig["value"] <= 0.2]
        if strong:
            drivers.append("strong on " + ", ".join(strong))
        if weak:
            drivers.append("weak on " + ", ".join(weak))
        sens_sources.append({
            "url": s["url"],
            "base_rank": base_rank[s["url"]],
            "base_score": base_score[s["url"]],
            "rank_range": rank_range,
            "score_range": max(scores) - min(scores),
            "stability": stability,
            "profile_ranks": profile_ranks[s["url"]],
            "drivers": drivers,
        })
    counts = {k: sum(1 for x in sens_sources if x["stability"] == k) for k in ("stable", "sensitive", "volatile")}
    top = next((x for x in sens_sources if x["base_rank"] == 1), None)
    sensitivity = {
        "generated_at": scored_at,
        "default_weights": weights,
        "profiles": profiles,
        "sources": sens_sources,
        "summary": {
            "stable": counts["stable"],
            "sensitive": counts["sensitive"],
            "volatile": counts["volatile"],
            "top_source": top["url"] if top else None,
            "top_source_stable": bool(top and top["stability"] == "stable"),
        },
        "basis": [
            "Each profile reruns the same dimension signals with a different weight vector and the same per-source missing-evidence renormalisation.",
            "stable: the rank never moves across profiles; sensitive: it moves by at most two places; volatile: it moves further and should be inspected before the order is treated as decisive.",
            "Sycophancy penalties are preserved across profiles.",
        ],
    }

    if args.legacy_array:
        legacy = [{"url": s["url"], "credibility_score": s["credibility_score"], "flags": s["flags"]} for s in ranked]
        print(json.dumps(legacy, indent=2))
    elif args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump(credibility, f, indent=2)
            f.write("\n")
    else:
        print(json.dumps(credibility, indent=2))
    if args.sensitivity:
        with open(args.sensitivity, "w", encoding="utf-8") as f:
            json.dump(sensitivity, f, indent=2)
            f.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
