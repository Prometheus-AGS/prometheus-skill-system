# Explicit author-only BDD route

Applies to both existing JS and Rust BDD skills. Invocation is a skill instruction,
not an invented cucumber/cargo flag:

`/bdd-cucumber-js author-only <contract-reference> <scenario-IDs>`

`/bdd-cucumber-rs author-only <contract-reference> <scenario-IDs>`

1. Read governing host policy, current selected method and latest authorized
   contract binding/supersession. Reuse reviewed calibration and permissions;
   never repeat approval already given for this scope. Scenario definitions
   precede generation; executable feature/step authoring follows host policy.
   In repositories requiring production-first test authoring, wait for the
   complete agreed production implementation before writing these artifacts.
2. Resolve only adopted stable scenario IDs and their owner tasks, real
   production entries, fixtures and required evidence. Refuse unknown IDs,
   stale identities or missing authoring authority without inventing scope.
3. Produce declarative features tagged with their stable IDs, real assertions,
   step definitions and configuration using the existing adapter guidance.
   Keep a scenario-to-assertion map with contract digest, requirement source,
   expected-truth source, real boundary and the plausible false positive the
   assertion excludes. Steps actually exercise the production operation; they
   do not merely return success or assert a process exit.
4. Stop after writing and statically inspecting the requested files. Do not
   execute, import/require support modules, compile/generate test code, install
   dependencies, run lint/typecheck, invoke runner help/discovery or dry-run.
   Cucumber dry-run loads support code and cannot establish this boundary.
   Do not modify protected scenarios without their existing signed approval.
5. Report exact files, mapped approved IDs, assumptions/missing fixtures and
   “authored only; NOT_EXERCISED.” This is not behavioral acceptance.

Execution is a different invocation at the complete authorized value-phase
boundary through `value-scenario-verify`, using the smallest adequate existing
runner, actual observations and safe oracle sensitivity controls. Missing
prerequisites block rather than trigger installation or early diagnostic runs.
