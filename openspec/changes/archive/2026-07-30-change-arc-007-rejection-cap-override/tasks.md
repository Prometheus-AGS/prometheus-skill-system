# Tasks

- [x] Read PROMETHEUS_ADV_REJECT_CAP, defaulting to 2 (same pattern as STRICTNESS)
- [x] Hard ceiling (5); values above it error rather than being honoured silently
- [x] Record cap_overridden: true and the value used in the findings artifact
- [x] Prompt once in a TTY, defaulting to accept; never block in CI or hooks
- [x] Changes ONLY the sycophancy-screen cap, not the 004/005 retry-loop cap
- [x] Ships its own test; 006 Group C deliberately does not cover it
