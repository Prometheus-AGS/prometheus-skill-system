#!/usr/bin/env python3
"""merge-threads.py — fold thread artifacts into stage 02/03/04, deterministically.

Invoked by merge-threads.sh as: merge-threads.py <package_dir> <canonical-url-lib>
Kept as a sibling file rather than a heredoc inside the wrapper: the heredoc
form hangs indefinitely on some hosts (observed 2026-09-09 — this exact
program blocked past 90s inline and ran in under a second from a file).
Same convention as score-sources.py beside verify-sources.sh.
"""

import hashlib, json, os, subprocess, sys

pkg, lib = sys.argv[1], sys.argv[2]
threads_dir = os.path.join(pkg, "threads")

def die_critical(msg):
    sys.stderr.write("merge-threads: CRITICAL: " + msg + "\n")
    sys.exit(2)

def read_json(path, default=None):
    if not os.path.isfile(path):
        return default
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)

# --- canonical URL -----------------------------------------------------------
# Shell out to the shared library rather than reimplementing the rule here.
# Two copies of a normalisation rule is exactly how they diverge; the library's
# header says so and this is the caller that would otherwise drift.
_canon_cache = {}
def canonical(url):
    if url in _canon_cache:
        return _canon_cache[url]
    out = subprocess.run(
        ["bash", "-c", '. "$1"; canonical_url "$2"', "_", lib, url],
        capture_output=True, text=True, check=True,
    ).stdout
    _canon_cache[url] = out
    return out

def source_entity_id(canon):
    return "src-" + hashlib.sha256(canon.encode("utf-8")).hexdigest()[:12]

# Sorted so the walk order of the filesystem cannot change the output.
tids = sorted(
    d for d in os.listdir(threads_dir)
    if os.path.isdir(os.path.join(threads_dir, d))
)

# --- pass 1: sources, unioned by canonical URL -------------------------------
# Insertion order is thread order, which is sorted, so numbering is stable.
sources = {}          # canon -> record
local_to_canon = {}   # (tid, local_source_id) -> canon

for tid in tids:
    tdir = os.path.join(threads_dir, tid)
    for s in read_json(os.path.join(tdir, "sources.json"), []) or []:
        url = s.get("url")
        if not url:
            die_critical(f"thread {tid} has a source with no url")
        canon = canonical(url)
        local_to_canon[(tid, s.get("id"))] = canon
        rec = sources.get(canon)
        if rec is None:
            sources[canon] = {
                "url": canon,
                "entity_id": source_entity_id(canon),
                "title": s.get("title") or "",
                # Provenance: every thread that independently found this page.
                "threads": [tid],
                "chunk_ids": list(s.get("chunk_ids") or []),
                "claims": [],
            }
        else:
            if tid not in rec["threads"]:
                rec["threads"].append(tid)
            for c in (s.get("chunk_ids") or []):
                if c not in rec["chunk_ids"]:
                    rec["chunk_ids"].append(c)
            if not rec["title"]:
                rec["title"] = s.get("title") or ""

ordered_canons = list(sources.keys())

# --- the no-search rule ------------------------------------------------------
# A dossier may only cite sources its own thread fetched. A citation to anything
# else means a source entered the package outside a worker: a director that
# searched, or a worker that sub-dispatched. Both are CRITICAL.
for tid in tids:
    tdir = os.path.join(threads_dir, tid)
    own = set()
    for s in read_json(os.path.join(tdir, "sources.json"), []) or []:
        if s.get("url"):
            own.add(canonical(s["url"]))

    dossier_path = os.path.join(tdir, "dossier.md")
    if not os.path.isfile(dossier_path):
        continue
    with open(dossier_path, "r", encoding="utf-8") as fh:
        dossier = fh.read()

    # Inline citations in a dossier are bare URLs; find them without a regex
    # dialect argument by scanning tokens.
    for tok in dossier.replace("(", " ").replace(")", " ").replace("<", " ").replace(">", " ").split():
        tok = tok.strip().rstrip(".,;:")
        if not (tok.startswith("http://") or tok.startswith("https://")):
            continue
        if canonical(tok) not in own:
            die_critical(
                f"thread {tid}: dossier.md cites {tok}, which is absent from "
                f"threads/{tid}/sources.json. A source may only enter the package "
                f"through the worker that fetched it (the director cannot search, "
                f"and a worker cannot sub-dispatch)."
            )

# --- pass 2: claims, unioned by the pack's content-addressed id --------------
# rah-010 already gives build-graph.sh and detect-contradictions.sh a shared
# claim id. Keying on it here means the merge invents no scheme of its own, and
# every (thread_id, source_id, quote) provenance tuple is preserved: three
# threads asserting the same sentence is evidence, not duplication.
claims = {}
for tid in tids:
    tdir = os.path.join(threads_dir, tid)
    for c in read_json(os.path.join(tdir, "claims.json"), []) or []:
        cid = c.get("id")
        if not cid:
            die_critical(f"thread {tid} has a claim with no id")
        canon = local_to_canon.get((tid, c.get("source_id")))
        if canon is None:
            die_critical(
                f"thread {tid}: claim {cid} cites source_id {c.get('source_id')!r}, "
                f"which is not in that thread's sources.json"
            )
        rec = claims.get(cid)
        if rec is None:
            claims[cid] = {
                "id": cid,
                "text": c.get("text", ""),
                "provenance": [],
            }
            rec = claims[cid]
        rec["provenance"].append({
            "thread_id": tid,
            "source_id": sources[canon]["entity_id"],
            "url": canon,
            "quote": c.get("quote", ""),
            "chunk_id": c.get("chunk_id"),
        })
        if canon in sources and c.get("text") and c["text"] not in sources[canon]["claims"]:
            sources[canon]["claims"].append(c["text"])

for rec in claims.values():
    rec["provenance"].sort(key=lambda p: (p["thread_id"], p["source_id"], p["quote"]))

# --- citation map: one document, one number ----------------------------------
# Numbers follow the sorted-thread insertion order above, so they are stable
# across runs. Each thread's local marker for a URL resolves to the same global
# number, which is the property `citation_utils.py::collapse_citations` has and
# the reason it cannot drift: no model is involved.
citation_map = {}
for i, canon in enumerate(ordered_canons, start=1):
    citation_map[canon] = i

local_markers = []
for (tid, local_id), canon in sorted(local_to_canon.items(), key=lambda kv: (kv[0][0], str(kv[0][1]))):
    local_markers.append({
        "thread_id": tid,
        "local_source_id": local_id,
        "url": canon,
        "citation_number": citation_map[canon],
    })

# --- emit, in the EXISTING stage shapes --------------------------------------
# Stage numbers and validators do not change; only how these files are produced.
def write_json(rel, obj):
    path = os.path.join(pkg, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(obj, fh, indent=2, sort_keys=False, ensure_ascii=False)
        fh.write("\n")
    return path

# stage 02
write_json("sources/url-list.json", {
    "source_urls": ordered_canons,
    "search_metadata": [],
    "total_found": len(ordered_canons),
    "filtered_count": 0,
})

# stage 04
write_json("sources/registry.json", {
    "sources": [
        {
            "url": sources[c]["url"],
            "entity_id": sources[c]["entity_id"],
            "title": sources[c]["title"],
            "threads": sources[c]["threads"],
            "claims": sources[c]["claims"],
            "chunk_ids": sources[c]["chunk_ids"],
        }
        for c in ordered_canons
    ]
})

# stage 03 — chunks copied into the flat package shape the validator expects
# (url/chunk_id/text). Numbering follows source order then chunk order.
n = 0
seen_chunks = set()
for canon in ordered_canons:
    for tid in sources[canon]["threads"]:
        cdir = os.path.join(threads_dir, tid, "chunks")
        if not os.path.isdir(cdir):
            continue
        for name in sorted(os.listdir(cdir)):
            if not name.endswith(".json"):
                continue
            chunk = read_json(os.path.join(cdir, name))
            if not chunk or canonical(chunk.get("url", "")) != canon:
                continue
            key = (canon, chunk.get("chunk_id"))
            if key in seen_chunks:
                continue
            seen_chunks.add(key)
            n += 1
            write_json(f"sources/chunk-{n}.json", {
                "url": canon,
                "chunk_id": chunk.get("chunk_id", f"chunk-{n}"),
                "text": chunk.get("text", ""),
                "thread_id": tid,
            })

write_json("citation-map.json", {
    "schema_version": "1.0.0",
    "citations": [
        {"citation_number": citation_map[c], "url": c, "entity_id": sources[c]["entity_id"]}
        for c in ordered_canons
    ],
    "local_markers": local_markers,
})

write_json("claims.json", {
    "schema_version": "1.0.0",
    "claims": [claims[k] for k in sorted(claims.keys())],
})

sys.stderr.write(
    f"merge-threads: {len(tids)} threads -> {len(ordered_canons)} sources, "
    f"{len(claims)} claims, {n} chunks\n"
)
