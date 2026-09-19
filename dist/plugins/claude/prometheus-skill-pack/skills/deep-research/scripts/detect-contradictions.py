#!/usr/bin/env python3
"""detect-contradictions.py — stage 06 contradiction detection.

Invoked by detect-contradictions.sh, which passes CRED, REGISTRY, PKG_ID,
SEMANTIC, MAX_PAIRS, OUT and WORK through the environment. Kept as a sibling
file rather than a heredoc inside the wrapper: the heredoc form hangs
indefinitely on some hosts (observed 2026-09-09), while the same program from
a file runs immediately. Same convention as score-sources.py beside
verify-sources.sh.
"""

import hashlib, itertools, json, os, re, subprocess, sys

def load(path):
    if not path or not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        return json.load(f)

# --- claim id: identical in build-graph.sh -----------------------------------
def normalise(text):
    t = text.lower().strip()
    t = re.sub(r"\s+", " ", t)
    return t.rstrip(" .;:,!")

def claim_id(scope, text):
    return "claim-" + hashlib.sha256(f"{scope}:{normalise(text)}".encode("utf-8")).hexdigest()[:16]
# -----------------------------------------------------------------------------

scope = os.environ["PKG_ID"]
cred = load(os.environ.get("CRED")) or {}
registry = load(os.environ.get("REGISTRY"))

# Claims with their source and credibility. credibility.json is preferred (it
# carries labels and scores); the registry fills in sources it lacks.
claims = []   # {text, source, credibility, label}
seen = set()
def add(text, source, score, label):
    key = (source, normalise(text))
    if key in seen or not text.strip():
        return
    seen.add(key)
    claims.append({"text": text.strip(), "source": source, "credibility": score, "label": label})

for s in (cred.get("verified_sources") or []) + (cred.get("filtered_sources") or []):
    for c in s.get("claims") or []:
        t = c.get("text") if isinstance(c, dict) else c
        if isinstance(t, str):
            add(t, s.get("url"), s.get("credibility_score"), (c.get("label") if isinstance(c, dict) else None) or "unverified")
scores = cred.get("credibility_scores") or {}
sources = registry.get("sources") if isinstance(registry, dict) else (registry or [])
for s in sources or []:
    for c in s.get("claims") or []:
        t = c.get("text") if isinstance(c, dict) else c
        if isinstance(t, str):
            add(t, s.get("url"), scores.get(s.get("url")), "unverified")

entries = []
def entry(topic, a, b, strategy, resolved, resolution, confidence, label, audit):
    return {
        "id": f"contra-{len(entries) + 1:03d}",
        "topic": topic,
        "claim_a": {"id": claim_id(scope, a["text"]), "text": a["text"], "source": a["source"], "credibility": a["credibility"]},
        "claim_b": {"id": claim_id(scope, b["text"]), "text": b["text"], "source": b["source"], "credibility": b["credibility"]},
        "strategy_tried": strategy,
        "resolved": resolved,
        "resolution": resolution,
        "confidence": confidence,
        "label": label,
        "audit_trail": audit,
    }

# ---- numeric path --------------------------------------------------------------
PATTERNS = [
    (r"(\d+(?:\.\d+)?)\s*%", "percentage"),
    (r"(\d+(?:\.\d+)?)\s*(?:ms|milliseconds?)\b", "latency_ms"),
    (r"(\d+(?:[,\d]+)?)\s*(?:k\s*)?(?:qps|queries?\s*per\s*second)", "throughput_qps"),
    (r"(\d+(?:\.\d+)?)\s*(?:gb|mb|tb)\b", "storage"),
]
by_topic = {}
for c in claims:
    for pat, topic in PATTERNS:
        m = re.findall(pat, c["text"], re.IGNORECASE)
        if m:
            by_topic.setdefault(topic, []).append((c, m[0]))
numeric_pairs = set()
for topic, items in by_topic.items():
    for (a, va), (b, vb) in itertools.combinations(items, 2):
        if a["source"] == b["source"]:
            continue
        try:
            fa, fb = float(va.replace(",", "")), float(vb.replace(",", ""))
        except ValueError:
            continue
        ratio = max(fa, fb) / max(min(fa, fb), 0.001)
        if ratio > 2.0:
            numeric_pairs.add((claim_id(scope, a["text"]), claim_id(scope, b["text"])))
            entries.append(entry(topic, a, b, "numeric", False, None, round(min(0.95, 0.5 + (ratio - 2) / 20), 2), "inferred",
                                 f"same measurable topic {topic}: {va} vs {vb} (ratio {ratio:.2f} > 2); detected mechanically, resolution left to stage 06"))

# ---- semantic path -------------------------------------------------------------
semantic = {"attempted": False, "status": "skipped", "pairs_checked": 0, "reason": "--semantic not requested"}
if os.environ.get("SEMANTIC") == "1":
    STOP = {"the", "a", "an", "is", "are", "of", "in", "on", "to", "and", "for", "with", "by", "as", "at", "that", "this", "it", "its", "be", "or", "from", "now", "than"}
    def words(t):
        return {w for w in re.findall(r"[a-z0-9]+", t.lower()) if w not in STOP and len(w) > 2}
    candidates = []
    for a, b in itertools.combinations(claims, 2):
        if a["source"] == b["source"]:
            continue
        pair = (claim_id(scope, a["text"]), claim_id(scope, b["text"]))
        if pair in numeric_pairs or (pair[1], pair[0]) in numeric_pairs:
            continue
        shared = words(a["text"]) & words(b["text"])
        if len(shared) >= 2:
            candidates.append((a, b, sorted(shared)))
    candidates = candidates[: int(os.environ.get("MAX_PAIRS") or 20)]
    semantic = {"attempted": True, "status": "inferred", "pairs_checked": 0, "reason": None}
    gateway_down = None
    work = os.environ["WORK"]
    for i, (a, b, shared) in enumerate(candidates):
        if gateway_down:
            entries.append(entry(" ".join(shared[:3]), a, b, "semantic", False, None, 0.0, "blocked",
                                 f"semantic check not run: {gateway_down}"))
            continue
        prompt = (
            "Do these two statements contradict each other? They come from different sources.\n"
            f"A: {a['text']}\nB: {b['text']}\n"
            'Reply with JSON only: {"contradicts": true|false, "topic": "<two or three words>", "confidence": 0.0-1.0, "reason": "<one sentence>"}'
        )
        pf = os.path.join(work, f"prompt-{i}.txt")
        with open(pf, "w", encoding="utf-8") as f:
            f.write(prompt)
        proc = subprocess.run(["bash", "-c", 'semantic_judge "$1"', "semantic_judge", pf],
                              capture_output=True, text=True)
        semantic["pairs_checked"] += 1
        if proc.returncode == 3:
            gateway_down = "no OpenAI-compatible gateway reachable (kbd_complete exit 3): " + (proc.stderr.strip().splitlines()[-1] if proc.stderr.strip() else "no detail")
            semantic["status"] = "blocked"
            semantic["reason"] = gateway_down
            entries.append(entry(" ".join(shared[:3]), a, b, "semantic", False, None, 0.0, "blocked", f"semantic check not run: {gateway_down}"))
            continue
        if proc.returncode != 0:
            reason = "judge call failed (exit %d): %s" % (proc.returncode, (proc.stderr.strip().splitlines() or ["no detail"])[-1])
            entries.append(entry(" ".join(shared[:3]), a, b, "semantic", False, None, 0.0, "blocked", reason))
            semantic["status"] = "blocked"
            semantic["reason"] = reason
            continue
        m = re.search(r"\{.*\}", proc.stdout, re.S)
        verdict = None
        if m:
            try:
                verdict = json.loads(m.group(0))
            except ValueError:
                verdict = None
        if not isinstance(verdict, dict) or not isinstance(verdict.get("contradicts"), bool):
            # The judge answered but not in a form that decides anything: the
            # pair stays visible as blocked, never as "no contradiction".
            snippet = (proc.stdout.strip() or "<empty>")[:120].replace("\n", " ")
            entries.append(entry(" ".join(shared[:3]), a, b, "semantic", False, None, 0.0, "blocked",
                                 f"judge reply could not be parsed as a verdict: {snippet}"))
            semantic["status"] = "blocked"
            semantic["reason"] = "one or more judge replies could not be parsed as a verdict"
            continue
        if verdict.get("contradicts") is True:
            entries.append(entry(verdict.get("topic") or " ".join(shared[:3]), a, b, "semantic", False, None,
                                 float(verdict.get("confidence") or 0.5), "inferred",
                                 "judged contradictory by the critic model: " + str(verdict.get("reason") or "no reason given")))

result = {"contradictions": entries, "detection": {"numeric_pairs": len(numeric_pairs), "semantic": semantic, "claims_considered": len(claims), "scope": scope}}
text = json.dumps(result, indent=2) + "\n"
out = os.environ.get("OUT")
if out:
    with open(out, "w", encoding="utf-8") as f:
        f.write(text)
else:
    sys.stdout.write(text)
