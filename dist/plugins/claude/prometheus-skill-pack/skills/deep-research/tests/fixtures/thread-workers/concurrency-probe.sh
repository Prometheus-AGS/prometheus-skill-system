#!/bin/sh
# Records its own start/stop so the harness can reconstruct the true peak
# concurrency from the outside, rather than trusting the scheduler's own count.
D="${RESEARCH_PACKAGE_DIR:-.}/concurrency"
mkdir -p "$D"
echo "start $(date +%s%N)" >> "$D/${RESEARCH_THREAD_ID:-x}.log"
# Long enough that all tasks overlap if the cap were not enforced.
sleep 0.6
echo "stop $(date +%s%N)" >> "$D/${RESEARCH_THREAD_ID:-x}.log"
exit 0
