#!/usr/bin/env bash
# canonical-url.sh — one canonical URL rule for the whole pipeline.
#
# Stage 02's dedupe and the thread merge must agree on when two URLs are the
# same document, or the merge emits two registry entries and two citation
# numbers for one page. Two copies of a normalisation rule is exactly how they
# diverge, so the rule lives here and both callers source it (analysis: the
# open question on change-drt-004, resolved to "extract").
#
# Sourced, not executed. bash 3.2 compatible (C-05): no mapfile, no declare -A.

# Emit the canonical form of a URL on stdout.
#
# The rule, in order:
#   1. lowercase the scheme and host (they are case-insensitive per RFC 3986;
#      the path is NOT, so it is left alone)
#   2. drop a default port (:80 on http, :443 on https)
#   3. drop a trailing "/" on the path, but never turn "https://host/" into
#      "https://host" — the empty path is not the same string to a fetcher
#   4. drop the fragment: "#section" never changes which document was fetched
#   5. drop tracking parameters, then sort the rest, so ?b=2&a=1 and ?a=1&b=2
#      are one document
#
# Deliberately NOT done: stripping "www.", collapsing http to https, or
# following redirects. Each of those can change which document you get, and a
# merge that silently unions two different pages is worse than one that keeps
# them apart.
canonical_url() {
    printf '%s' "$1" | python3 -c '
import sys
from urllib.parse import urlsplit, urlunsplit, parse_qsl, urlencode

raw = sys.stdin.read().strip()
if not raw:
    sys.stdout.write("")
    sys.exit(0)

try:
    parts = urlsplit(raw)
except ValueError:
    # A URL we cannot parse is passed through unchanged rather than mangled:
    # the merge will treat it as its own document, which is the safe failure.
    sys.stdout.write(raw)
    sys.exit(0)

scheme = parts.scheme.lower()
host = parts.hostname or ""
host = host.lower()

port = parts.port
if port is not None and not ((scheme == "http" and port == 80) or (scheme == "https" and port == 443)):
    host = f"{host}:{port}"

# Credentials are part of access, not identity.
netloc = host

path = parts.path
if len(path) > 1 and path.endswith("/"):
    path = path[:-1]

# Tracking parameters identify a campaign, never a document.
DROP_EXACT = {"gclid", "fbclid", "msclkid", "mc_cid", "mc_eid", "igshid", "ref", "ref_src"}
kept = [
    (k, v) for (k, v) in parse_qsl(parts.query, keep_blank_values=True)
    if not (k.lower().startswith("utm_") or k.lower() in DROP_EXACT)
]
kept.sort()
query = urlencode(kept)

sys.stdout.write(urlunsplit((scheme, netloc, path, query, "")))
'
}

# Stable short id for a canonical URL, for use as a source entity id.
canonical_url_id() {
    printf 'src-%s' "$(canonical_url "$1" | shasum -a 256 | cut -c1-12)"
}
