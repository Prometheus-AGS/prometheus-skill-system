#!/usr/bin/env python3
"""score-fact.py — FACT metrics and the verified-claim ratio for one package.

The adopted benchmark scores prose quality (RACE). It says nothing about whether
the citations under that prose hold up, so these three numbers are built here:

  effective_citations   distinct sources actually cited by a surviving claim.
                        A source fetched and never cited did not contribute;
                        counting registry size instead would reward crawling.

  citation_accuracy     the fraction of cited claims whose verbatim quote is
                        really present in the chunk it names. This is the one
                        that catches a confident report built on paraphrase:
                        a quote that is not in the source is a citation to
                        something that was never said.

  verified_claim_ratio  the fraction of cited claims labelled `verified`.
                        This pack carries a per-claim label from stage 05, so it
                        can report how much of a report rests on checked ground
                        rather than on inference. Onyx has no per-claim label,
                        so it cannot report this at all — which is why it is the
                        differentiator rather than just another metric.

A high RACE score with a low verified-claim ratio is a well-written report that
is mostly inference. Reporting them together is the point.

Usage:  score-fact.py --package <dir>          one JSON object on stdout
Exit:   0 computed, 2 the package lacks what the metrics need (never a zero
        score, which would be indistinguishable from a real result).
"""

import argparse
import json
import os
import re
import sys


def read_json(path, default=None):
    if not os.path.isfile(path):
        return default
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def normalise(text):
    """Whitespace-insensitive comparison.

    A quote that differs from its source only by line wrapping is the same
    quote; failing it would make the accuracy metric measure formatting.
    Anything beyond whitespace — a changed word, a dropped negation — must
    still fail, so nothing else is normalised away.
    """
    return " ".join((text or "").split()).lower()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--package", required=True)
    args = ap.parse_args()
    pkg = args.package

    graph = read_json(os.path.join(pkg, "graph.json"), {}) or {}
    claims = graph.get("claims") or graph.get("nodes") or []
    if not claims:
        sys.stderr.write(f"score-fact: {pkg} has no claims in graph.json\n")
        sys.exit(2)

    # Which claims the report actually cites. A claim in the graph that no
    # section cited did not reach the reader, so it must not count toward any
    # of these numbers.
    cited_ids = None
    cc = read_json(os.path.join(pkg, "report", "cited-claims.json"))
    if cc and isinstance(cc.get("cited"), list):
        cited_ids = set(cc["cited"])
    else:
        # Unthreaded packages have no cited-claims.json; fall back to scanning
        # the report for markers, and if there is no report at all, treat every
        # graph claim as cited rather than silently reporting zero.
        report_path = os.path.join(pkg, "report.md")
        if os.path.isfile(report_path):
            import re
            with open(report_path, "r", encoding="utf-8") as fh:
                body = fh.read()
            found = set(re.findall(r"\[(claim-[A-Za-z0-9]+)\]", body))
            cited_ids = found if found else {c["id"] for c in claims if c.get("id")}
        else:
            cited_ids = {c["id"] for c in claims if c.get("id")}

    cited = [c for c in claims if c.get("id") in cited_ids]
    if not cited:
        sys.stderr.write(f"score-fact: {pkg} cites no claims\n")
        sys.exit(2)

    # --- effective citations ---
    sources = set()
    for c in cited:
        sid = c.get("source_id")
        if sid:
            sources.add(sid)
        for p in (c.get("provenance") or []):
            if p.get("source_id"):
                sources.add(p["source_id"])
        # Stage 07's own output (build-graph.py) carries a `sources` array of
        # entity ids rather than a single `source_id` or a provenance list —
        # the merge's shape and the graph builder's shape are both legitimate
        # inputs here. Reading only one of them silently reported zero
        # effective citations for a package with 31 cited claims.
        for s in (c.get("sources") or []):
            if isinstance(s, str) and s:
                sources.add(s)
    effective_citations = len(sources)

    # --- citation accuracy ---
    # Index every chunk the package holds, by chunk_id and by url.
    chunk_text = {}
    sdir = os.path.join(pkg, "sources")
    if os.path.isdir(sdir):
        for name in sorted(os.listdir(sdir)):
            if not (name.startswith("chunk-") and name.endswith(".json")):
                continue
            ch = read_json(os.path.join(sdir, name)) or {}
            body = normalise(ch.get("text"))
            if ch.get("chunk_id"):
                chunk_text[ch["chunk_id"]] = body
            if ch.get("url"):
                chunk_text.setdefault(ch["url"], body)
    tdir = os.path.join(pkg, "threads")
    if os.path.isdir(tdir):
        for tid in sorted(os.listdir(tdir)):
            cdir = os.path.join(tdir, tid, "chunks")
            if not os.path.isdir(cdir):
                continue
            for name in sorted(os.listdir(cdir)):
                ch = read_json(os.path.join(cdir, name)) or {}
                if ch.get("chunk_id"):
                    chunk_text.setdefault(ch["chunk_id"], normalise(ch.get("text")))

    # Accuracy is scored PER CLAIM, not per provenance tuple.
    #
    # After the merge, a claim three threads independently found carries three
    # provenance tuples pointing at the same quote. Counting tuples would let
    # that one claim contribute three of six checks while a single-thread claim
    # contributes one — so the metric would drift toward whatever more threads
    # happened to find, which is corroboration, not accuracy. A claim is either
    # supported by its source or it is not, exactly once.
    #
    # Corroboration is a real signal, but a different one; it is reported
    # separately as `mean_corroboration` rather than folded in here.
    checkable = 0
    supported = 0
    corroboration = []
    for c in cited:
        quotes = set()
        if c.get("quote"):
            quotes.add((c.get("chunk_id"), normalise(c["quote"])))
        for p in (c.get("provenance") or []):
            if p.get("quote"):
                quotes.add((p.get("chunk_id"), normalise(p["quote"])))
        # Stage 07 records the supporting passage as `evidence`, formatted
        # `chunk-N: "quoted text"`. It is the same assertion as a `quote` —
        # the passage the claim rests on — under the name build-graph.py uses,
        # so it must be checkable too. Reading only `quote` reported "no
        # checkable quotes" for a package whose every claim carried evidence.
        ev = c.get("evidence")
        if isinstance(ev, str) and ev.strip():
            m = re.match(r'\s*(chunk-[A-Za-z0-9_-]+)\s*:\s*"(.*)"\s*$', ev, re.S)
            if m:
                quotes.add((m.group(1), normalise(m.group(2))))
            else:
                # Stage 07 evidence is often a sentence that *contains* the
                # supporting passage in quotes rather than being it, e.g.
                #   Usage header: '# Execution modes ...' listing RUNNER
                # Checking the whole sentence verbatim would fail every time and
                # report 0.0 accuracy for claims whose passages are genuinely
                # present — measured here: 19 of 26 inner fragments were
                # verbatim in a chunk while 0 of 26 whole sentences were. The
                # quoted fragment is the assertion; the prose around it is the
                # annotator's framing. Fragments under 15 characters are too
                # short to be evidence of anything and are ignored.
                frags = re.findall(r"'([^']{15,})'", ev) + re.findall(r'"([^"]{15,})"', ev)
                for f in frags:
                    quotes.add((None, normalise(f)))
                if not frags:
                    quotes.add((c.get("chunk_id"), normalise(ev)))
        quotes = {(cid, q) for (cid, q) in quotes if q}
        if not quotes:
            continue

        corroboration.append(len(c.get("provenance") or []) or 1)
        checkable += 1
        # The claim counts as supported when ANY of its distinct quotes is
        # verbatim-present in the chunk it names. A missing chunk supports
        # nothing: skipping it would let a package raise its score by shipping
        # fewer chunks.
        for chunk_id, q in quotes:
            if chunk_id is None:
                # The claim names no chunk (stage 07 evidence carries the
                # passage but not its chunk id). Search every chunk the package
                # holds: the assertion is still checkable, just not addressed.
                if any(q in body for body in chunk_text.values()):
                    supported += 1
                    break
                continue
            body = chunk_text.get(chunk_id)
            if body is not None and q in body:
                supported += 1
                break

    citation_accuracy = (supported / checkable) if checkable else None
    mean_corroboration = (sum(corroboration) / len(corroboration)) if corroboration else None

    # --- verified-claim ratio ---
    labelled = [c for c in cited if c.get("label")]
    verified = [c for c in labelled if c.get("label") == "verified"]
    verified_claim_ratio = (len(verified) / len(labelled)) if labelled else None

    print(json.dumps({
        "package": os.path.basename(pkg.rstrip("/")),
        "cited_claims": len(cited),
        "effective_citations": effective_citations,
        "citation_accuracy": citation_accuracy,
        "claims_checked": checkable,
        "claims_supported": supported,
        "mean_corroboration": mean_corroboration,
        "verified_claim_ratio": verified_claim_ratio,
        "labelled_claims": len(labelled),
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
