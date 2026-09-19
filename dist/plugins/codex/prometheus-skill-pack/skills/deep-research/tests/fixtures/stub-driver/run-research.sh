#!/usr/bin/env bash
# Stub driver fixture for installed-service-smoke.sh.
#
# Reproduces defect D-B: the shape of the pre-contract driver that the installed
# plugin generation shipped (1920 bytes, dated before the phase that built the
# stage contract). It emits plausible progress lines and exits 0 while producing
# no package at all — a green result over an empty directory, which is strictly
# worse than an honest failure.
#
# This file must NOT name any of the four contract markers the daemon's
# self-check greps for, not even in a comment, or the check would call it
# healthy. That is the point of the fixture: it looks like a driver and is not
# one. (An earlier draft listed them here and self-defeated.)
set -euo pipefail
for i in 01 02 03 04 05 09 10; do
  printf '{"stage":"%s","status":"started"}\n' "$i"
  printf '{"stage":"%s","status":"completed"}\n' "$i"
done
exit 0
