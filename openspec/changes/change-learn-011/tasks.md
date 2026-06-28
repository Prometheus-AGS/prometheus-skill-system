# Tasks — change-learn-011

- [ ] Write `skills/learn/learn-plan/SKILL.md` with invocation contract, inputs (survey-result.json, goal), and output schema
- [ ] Query concept DAG from surreal-memory (`expand_neighbors`, `find_path`) to identify prerequisites for target concept
- [ ] Produce `curriculum.json`: ordered phases with prerequisite gates, estimated sessions, target mastery per phase
- [ ] Implement `--replan` mode: triggered when live mastery diverges > 0.2 from planned mastery; re-runs DAG query and reorders remaining phases
- [ ] Render curriculum via ui-surface: Tier 0 ordered list, Tier 2 mindmap (when available); never block on Tier 2
