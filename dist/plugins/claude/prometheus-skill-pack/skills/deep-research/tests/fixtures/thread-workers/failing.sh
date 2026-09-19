#!/bin/sh
# Exits non-zero: the `failed` path. The run must continue past it.
sleep 0.1
echo "worker ${RESEARCH_THREAD_ID:-?} broke" >&2
exit 3
