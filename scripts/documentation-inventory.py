#!/usr/bin/env python3
"""Inventory a Git revision's public guides, release records and README entry points."""
import argparse
import csv
import io
import posixpath
import re
import subprocess
import sys
from pathlib import PurePosixPath
from urllib.parse import unquote, urlsplit


def git(*args):
    return subprocess.check_output(["git", *args])


def channel(path):
    root = path.parts[0]
    if path.name.lower() == "readme.md":
        if root == "dist" or root in {".agents", ".opencode"}:
            return "projected README"
        if root.startswith(".") or root == "openspec":
            return "process record"
    if root in {"docs", "site"} and path.suffix.lower() in {".md", ".mdx", ".html"}:
        if root == "site":
            return "Pages"
        if len(path.parts) > 2 and path.parts[1] in {"guide", "learn"}:
            return "guide / Pages" if path.suffix != ".html" else "HTML supplement"
        if path.suffix == ".html":
            return "HTML supplement"
        if path.parts[1] == "generated":
            return "generated reference"
        if path.parts[1] in {"audits", "decisions", "evidence", "fixes", "future-work", "plans",
                             "releases", "reports", "research", "superpowers"}:
            return "historical record"
        return "repository docs"
    if len(path.parts) == 1 and path.suffix.lower() == ".md":
        return "repository entry"
    if path.name.lower() == "readme.md":
        return "component README"
    return None


def document_title(text):
    match = re.search(r"(?m)^#\s+(.+?)\s*$", text)
    if match:
        return match.group(1).strip()
    match = re.search(r"(?mi)^title:\s*(.+)$|<title>([^<]+)</title>", text)
    return (match.group(1) or match.group(2)).strip() if match else ""


def local_links(path, text, tree, directories):
    # Only literal Markdown links to repository paths are covered. Docusaurus routes,
    # anchors, external URLs and code examples have separate validation boundaries.
    body = re.sub(r"(?ms)^[ \t]*(`{3,}|~{3,})[^\n]*\n.*?^[ \t]*\1[ \t]*$", "", text)
    ok = set()
    gaps = set()
    for raw in re.findall(r"\[[^\]\n]+\]\(([^()\s]+)\)", body):
        url = urlsplit(raw.strip("<>"))
        if url.scheme or url.netloc or not url.path or url.path.startswith("/"):
            continue
        dest = posixpath.normpath(posixpath.join(str(path.parent), unquote(url.path)))
        route_source = str(path).startswith(("site/docs/", "docs/guide/", "docs/learn/")) and not posixpath.splitext(dest)[1]
        if dest in tree or dest in directories or (route_source and (dest + ".md" in tree or dest + ".mdx" in tree)):
            ok.add(url.path)
        else:
            gaps.add(url.path)
    return len(ok), ", ".join(sorted(gaps))


def review(path, group):
    name = str(path)
    if name.startswith("dist/") and name.endswith(("shared/scripts/README.md", "shared/scripts/scheduled/README.md", "entity-graph-optimize/component/README.md")):
        return "Repair upstream repository links that break inside flattened distributions; regenerate both plugins."
    if name.startswith("site/docs/sovereign-sync/"):
        return "Update Companion ownership; compare with merged Companion 0.2.0 control API and implementation."
    changes = {
        "README.md": "Verify first install, profile and release links against the installer.",
        "CHANGELOG.md": "Restore omitted 1.2, 1.6 and 1.8 tagged-source entries; distinguish attached assets.",
        "docs/maintenance.md": "Reconcile Node floor and Companion ownership with package engines and Companion source.",
        "scripts/release/README.md": "Cross-check exec pin ownership with scripts/release/binaries.py.",
        "tools/prometheus-cli/README.md": "Add the Full project status/watch path alongside the existing application-managed path.",
        "docker/service-images/README.md": "Link installer provider selection separately from instructions for image publishers.",
        "docs/guide/20-updating.md": "Connect generation rollback and independent service recovery to the release migration guide.",
        "site/docs/operations/local-validation-and-docs-automation.md": "Reconcile allowed release publication with .github/workflows/binary-release.yml.",
        "docs/generated/runtime-reference.md": "Correct hosted-publishing claim in scripts/docs-sync.mjs and regenerate.",
        "site/docs/operations/generated-reference.md": "Correct hosted-publishing claim in scripts/docs-sync.mjs and regenerate.",
        "docs/SKILL_TEMPLATE.md": "Example reference paths belong to the skill being authored.",
        "docs/deep-research/research/google-okf-llm-wiki-standards-report.md": "Quoted wiki linking example; retain placeholder in the research record.",
        "docs/plans/2026-05-06-prometheus-rust-auditor-implementation.md": "Preserve illustrative paths relative to the proposed skill, not to this archived plan.",
        "docs/future-work/03-cross-cutting/PAGS-SPEC-PSP-IP-001-AUDIT.md": "Repair relative link to BDD STATUS without changing the historical finding.",
        "docs/releases/1.13.0/README.md": "Repair two historical links to changelog and phase-readiness contract.",
        "SKILLS.md": "Check generated skills index from repository manifests.",
    }
    if name in changes:
        return changes[name]
    if group == "projected README":
        return "Generated or harness projection; trace changes to producer before regenerating."
    if group in {"historical record", "process record"} or any(w in name.lower() for w in
            ("research-agent-hardening-evidence", "production-readiness", "incident", "-handoff",
             "handoff.md", "problem-report", "update_considerations", "ledger", "2026", "addendum")):
        return "Preserve as dated evidence, design or handoff; use current guides for operation."
    if group == "HTML supplement":
        return "Static rendered content; inspect rendering separately from Markdown links."
    if group == "component README":
        return "Component instructions belong to their enclosing source directory."
    if name.startswith("docs/guide/"):
        return "Authored chapter served directly by the guide plugin; route and commands are maintained in source."
    if name.startswith("docs/learn/"):
        return "Learning reference served directly by the learn-internals plugin."
    if group == "Pages":
        return "Authored Pages topic; validate routes using site config and sidebar."
    return "Repository source documentation; inspect the owning subsystem for behavior."


def inventory(ref, out):
    paths = [PurePosixPath(p) for p in git("ls-tree", "-r", "--name-only", "-z", ref).decode().split("\0") if p]
    docs = {p: group for p in paths if (group := channel(p))}
    files = {str(p) for p in paths}
    dirs = {str(d) for p in paths for d in p.parents}
    # First-parent changes include the merge through which each path reached main.
    log = git("log", "-m", "--first-parent", "--format=@@%H %as", "--name-only", ref).decode()
    changes = {}
    for line in log.splitlines():
        if line.startswith("@@"):
            commit, date = line[2:].split()
            last = f"{commit[:12]} {date}"
        elif line:
            changes.setdefault(line, last)
    writer = csv.writer(out, delimiter="\t", lineterminator="\n")
    writer.writerow(["path", "channel", "title", "last_main_change", "local_link_targets",
                     "missing_literal_targets_at_revision", "review_disposition"])
    # Batch-read the exact objects at ref even if the working tree has newer edits.
    batch = b"".join(f"{ref}:{p}\n".encode() for p in docs)
    blobs = io.BytesIO(subprocess.run(["git", "cat-file", "--batch"], input=batch,
                                      stdout=subprocess.PIPE, check=True).stdout)
    for p, group in docs.items():
        _, kind, size = blobs.readline().split()
        if kind != b"blob":
            raise ValueError(f"expected a documentation blob at {p}")
        text = blobs.read(int(size)).decode()
        blobs.read(1)
        ok, missing = ("", "") if p.suffix == ".html" else local_links(p, text, files, dirs)
        writer.writerow([p, group, document_title(text), changes.get(str(p), ""), ok, missing, review(p, group)])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ref", help="Committed Git revision to review")
    inventory(parser.parse_args().ref, sys.stdout)
