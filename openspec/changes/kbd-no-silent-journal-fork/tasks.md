# Tasks

- [x] 1. CLI: `ensure_runtime` refuses to sign a new genesis over a runtime-generated waypoint (revision > 0) unless `PROMETHEUS_KBD_NEW_JOURNAL=1`
- [x] 2. Gate: integration scenario — A writes the plan; B with an empty data dir is refused and no journal is created; B with `PROMETHEUS_KBD_NEW_JOURNAL=1` starts one; a legacy waypoint still bootstraps
