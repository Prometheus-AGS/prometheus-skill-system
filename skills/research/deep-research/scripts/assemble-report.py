#!/usr/bin/env python3
"""assemble-report.py — concatenate report sections and enforce the claim-set invariant.

Invoked by assemble-report.sh as: assemble-report.py <package_dir> <post_edit:0|1>
Kept as a sibling file rather than a heredoc inside the wrapper: the heredoc
form hangs indefinitely on some hosts (observed 2026-09-09 — this exact
program blocked past 30s inline and ran in under a second from a file).
Same convention as score-sources.py beside verify-sources.sh.
"""

import json, os, re, sys

pkg, post_edit = sys.argv[1], sys.argv[2] == "1"
report = os.path.join(pkg, "report")

def die(msg):
    sys.stderr.write("assemble-report: FAIL: " + msg + "\n")
    sys.exit(2)

def read_json(path, default=None):
    if not os.path.isfile(path):
        return default
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)

outline = read_json(os.path.join(report, "outline.json"))
graph   = read_json(os.path.join(pkg, "graph.json"), {}) or {}
citemap = read_json(os.path.join(pkg, "citation-map.json"), {}) or {}

# Claim id -> label. graph.json carries either `claims` or legacy `nodes`.
claims = {c["id"]: c for c in (graph.get("claims") or graph.get("nodes") or []) if c.get("id")}

# Claim id -> global citation number, resolved from the merge's map. The merge is
# the single numbering authority; nothing here assigns a number.
number_of = {}
for row in (citemap.get("citations") or []):
    if row.get("entity_id"):
        number_of[row["entity_id"]] = row.get("citation_number")

MARKER = re.compile(r"\[(claim-[A-Za-z0-9]+)\]")

# The label check reads the sentence a marker sits in — not a fixed character
# window. A window spanning a sentence boundary taints the NEXT claim with the
# previous sentence's wording: "Research confirms X [verified]. The quota may be
# raised [inferred]." would fail on the inferred marker for a word that belongs
# to the sentence before it. Sentence scope is what the rule actually means.
# Words that assert a claim as established fact.
STRONG = ("confirms", "confirmed", "proves", "proven", "demonstrates",
          "establishes", "verified that", "shows that", "it is established")

sections = outline.get("sections") or []
if not sections:
    die("outline has no sections")

assembled = []
cited_all = set()

for sec in sections:
    sid = sec["section_id"]
    path = os.path.join(report, "sections", sid + ".md")
    if not os.path.isfile(path):
        die(f"section {sid} has no file at report/sections/{sid}.md")
    with open(path, "r", encoding="utf-8") as fh:
        text = fh.read()

    assigned = set(sec.get("claim_ids") or [])
    cited = set(MARKER.findall(text))

    # --- claim-set drift, per section ---
    outside = sorted(cited - assigned)
    if outside:
        die(
            f"section {sid} cites {', '.join(outside)}, which its outline entry does "
            f"not assign. A section may cite only what it was given — otherwise the "
            f"report's evidence base is whatever each writer reached for, and nobody "
            f"can check it afterwards."
        )

    # --- missing reference ---
    for cid in sorted(cited):
        if cid not in claims:
            die(f"section {sid} cites {cid}, which resolves to no claim in graph.json")

    # --- label misuse ---
    for m in MARKER.finditer(text):
        cid = m.group(1)
        label = (claims.get(cid) or {}).get("label")
        if label in ("verified", None):
            continue
        # Back up to the start of the sentence containing this marker.
        head = text[:m.start()]
        cut = max(head.rfind(". "), head.rfind(".\n"), head.rfind("\n\n"))
        window = head[cut + 1:].lower() if cut >= 0 else head.lower()
        for word in STRONG:
            if word in window:
                die(
                    f"section {sid} describes {cid} with \"{word}\", but its label is "
                    f"'{label}'. Only a verified claim may be written as established."
                )

    cited_all |= cited
    assembled.append(text.rstrip() + "\n")

# --- orphan markers: assigned but never cited is fine; cited-but-unassigned was
# caught per section above. What remains is a marker in the assembled draft that
# no section owns, which can only appear if something wrote outside a section.
draft_body = "\n".join(assembled)
for cid in sorted(set(MARKER.findall(draft_body))):
    if cid not in cited_all:
        die(f"orphan marker {cid} in the assembled draft belongs to no section")

# --- the editor may only remove ---
prev_path = os.path.join(report, "cited-claims.json")
if post_edit:
    prev = read_json(prev_path)
    if prev is None:
        die("--post-edit requires report/cited-claims.json from the first pass")
    before = set(prev.get("cited") or [])
    added = sorted(cited_all - before)
    if added:
        die(
            f"the coherence edit ADDED {', '.join(added)}. The editor may remove "
            f"claims but never introduce them: an added citation is unevidenced "
            f"prose in an evidenced report, at exactly the point where it reads "
            f"most fluently."
        )
    removed = sorted(before - cited_all)
    if removed:
        # A removal is allowed, but must be recorded — an unlogged removal is
        # indistinguishable from evidence quietly going missing.
        plan = os.path.join(pkg, "plan.md")
        logged = ""
        if os.path.isfile(plan):
            with open(plan, "r", encoding="utf-8") as fh:
                logged = fh.read()
        unlogged = [c for c in removed if c not in logged]
        if unlogged:
            die(
                f"the coherence edit removed {', '.join(unlogged)} without logging "
                f"it in plan.md's decision log"
            )
        sys.stderr.write(f"assemble-report: editor removed {len(removed)} claim(s), all logged\n")

# --- emit ---
os.makedirs(report, exist_ok=True)

# Resolve each marker to its global citation number. Unresolvable ids keep the
# id visible rather than silently vanishing.
def resolve(m):
    cid = m.group(1)
    src = (claims.get(cid) or {}).get("source_id")
    n = number_of.get(src) if src else None
    return f"[{n}]" if n else f"[{cid}]"

draft = MARKER.sub(resolve, draft_body)

with open(os.path.join(report, "draft.md"), "w", encoding="utf-8") as fh:
    fh.write(draft)

with open(prev_path, "w", encoding="utf-8") as fh:
    json.dump({"schema_version": "1.0.0", "cited": sorted(cited_all)}, fh, indent=2)
    fh.write("\n")

sys.stderr.write(
    f"assemble-report: {len(sections)} sections, {len(cited_all)} distinct claims cited"
    + (" (post-edit)" if post_edit else "") + "\n"
)
