# Tasks

scope: skills/testing/, skills/process/kbd-process-orchestrator/, skill registration/generator inputs, docs/, site/docs/

Depends on kbd-195-vlbv-policy-propagation; serialize shared KBD stage edits. Final generated-output reconciliation: kbd-194-196-delivery-qualification. Tasks below deliver production procedures, not executable acceptance tests.

## 1. Scenario workflow production

- [x] 1.1 Implement and register value-scenario-discovery with finite calibration, proposal status and Assess/Spec/Plan routing; delivered procedure covers SCN-001, inspected statically until final qualification.
- [x] 1.2 Add explicit author-only paths to existing JS/Rust BDD skills with stable ID/assertion/oracle linkage and policy-sensitive authoring; delivered guidance covers SCN-002 without invoking a runner or import.
- [x] 1.3 Implement and register thin value-scenario-verify procedure reusing runners and canonical writer, observation/negative-control evidence, dispositions and repair lineage; delivered procedure covers VER-001–003, with execution deferred.
- [x] 1.4 Replace conflicting BDD lifecycle and KBD integration procedures and document invocation/limitations; delivered docs preserve #195 authority and trace SCN-001–002 and VER-001–003.

## Workflow follow-up

- Test authoring and all runtime checks occur only after every parent production task, under kbd-194-196-delivery-qualification.
- No automatic contract adoption, successor, publication, global install or mini-repository change.

## Model assignment reference

Use `.kbd-orchestrator/phases/issues-194-195-196/plan.md` → Task model assignments. This change’s exact scoped keys are `issues-194-195-196 / kbd-196-value-scenario-workflow / 1`, `/ 2`, `/ 3`, `/ 4`; numbered heading labels are not backend IDs. Preserve task titles and checkbox syntax.
