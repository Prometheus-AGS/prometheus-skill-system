# Phase Goals — kbd-execute-spec-wrapping-and-nesting

1. **Fix the plan→execute→spec seam.** The Execute phase must *wrap* the spec
   backend (OpenSpec today; GitHub Spec Kit / others later) and drive it
   task-by-task, instead of handing the turn off to a bare `/opsx:apply` that
   runs outside KBD. KBD stays the source of truth throughout.

2. **Guarantee position reporting on every turn.** "Starting/Completing
   <phase|task> <name>, <i> of <n>" — for both the outer phase chain and any
   active child loop — must be emitted reliably each turn, through a mechanism
   that does not depend on the model remembering to source a shell hook, and
   must remain user-overridable.

3. **Confirm / harden nesting commands.** `/kbd-next-phase`, `/kbd-new-phase`,
   `/kbd-new-child`, `/kbd-next-child` already exist; verify they compose with
   the wrapped execute loop and the position reporter.

4. **Backend-agnostic spec wrapping.** Support OpenSpec and Spec Kit behind one
   driver interface; do not fork a parallel implementation per tool.
