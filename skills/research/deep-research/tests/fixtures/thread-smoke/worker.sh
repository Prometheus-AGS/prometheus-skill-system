#!/usr/bin/env bash
# Fixture worker for dispatch-smoke.sh. Stands in for a research worker without
# calling a model: it records when it ran and what it was told, so the smoke
# test can decide two things — did dispatches overlap in time, and did each
# worker receive only its own brief.
#
# Isolation here means CONTEXT isolation, not filesystem isolation. All threads
# share a package directory by design (the merge reads them all), so "can this
# worker open a sibling's file" is the wrong question — it always can. The right
# question is whether a worker is *given* anything beyond its own brief, which
# is what the environment and argument surface record below.
#
# Args: <thread_id> <threads_dir> <hold_seconds>
# bash 3.2 compatible (constraint C-05).
set -euo pipefail

TID="${1:?thread id required}"
THREADS_DIR="${2:?threads dir required}"
HOLD="${3:-2}"

DIR="${THREADS_DIR}/${TID}"
mkdir -p "$DIR"

# Wall-clock window. Seconds suffice: the hold is seconds-scale and `date +%s`
# is portable where GNU `date -Ins` is not available on macOS.
date -u +%s > "${DIR}/started_at"

# What this worker was actually handed. The smoke test asserts the brief it sees
# is its own and that no sibling's task text reached it.
{
  printf 'thread_id=%s\n' "$TID"
  printf 'brief_task=%s\n' "${THREAD_BRIEF_TASK:-<unset>}"
  printf 'sibling_env_leak=%s\n' "${SIBLING_TASK:-none}"
} > "${DIR}/received.txt"

# This worker's own scratch marker. The smoke test asserts each file holds only
# its owner's marker.
printf 'marker-%s\n' "$TID" > "${DIR}/scratch.txt"

# Hold the slot so concurrent dispatches genuinely overlap.
sleep "$HOLD"

# Scratch-state probe, required by the spec scenario. The point is NOT that a
# sibling's file is unreadable — all threads share a package directory by
# design, so it always is. The point is that a worker given only its own brief
# has no reason to know a sibling exists, so it must not be able to NAME one.
# THREAD_PEERS is deliberately never exported by the dispatcher; a worker that
# can resolve a peer id got it from somewhere it should not have.
PEER="${THREAD_PEERS:-}"
if [ -n "$PEER" ]; then
  printf 'leaked:%s\n' "$PEER" > "${DIR}/peer_probe"
else
  printf 'none\n' > "${DIR}/peer_probe"
fi

date -u +%s > "${DIR}/ended_at"
printf 'complete\n' > "${DIR}/status"
