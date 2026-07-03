# Tasks: change-credibility-004-path-confinement

- [ ] In `forge_enrich` handler, add `Path::canonicalize()` call on `task_path_str`
- [ ] Add prefix check: reject if canonical path does not start_with project_root_canonical
- [ ] Return `Err(anyhow!(...))` with clear message on traversal attempt
- [ ] Run `cargo build --workspace` — verify clean
- [ ] Test traversal rejection: `task_path = "../../etc/passwd"` returns error, not file read
- [ ] Test valid path within project root still works
