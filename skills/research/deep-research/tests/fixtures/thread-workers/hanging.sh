#!/bin/sh
# Never returns on its own: the `timeout` path. The scheduler must kill it.
# It writes a marker first so the test can prove the process really started,
# and a second marker it can only reach if it was NOT killed — the negative
# control for the kill assertion.
: > "${RESEARCH_PACKAGE_DIR:-.}/hanging-started-${RESEARCH_THREAD_ID:-x}"
sleep 300
: > "${RESEARCH_PACKAGE_DIR:-.}/hanging-SURVIVED-${RESEARCH_THREAD_ID:-x}"
