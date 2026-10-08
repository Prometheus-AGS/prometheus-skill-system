# Acceptance — change-ldd-13-source-convergence

Status: pending. Executed only by the consolidated change-12 final boundary after all production changes.

- AC-1: The disposition accounts for 100% of discovered full/mini and relevant dependency source items, with final integration ancestry/tree evidence or an explicit exclusion reason.

- AC-2: Fresh clone/scratch package gates exercise the integrated source; no local path dependency, unpublished-only pin or unrecorded patch is needed. User merges still remain separate from integration.

Evidence records exact source identity, command, local result, isolation roots and actual collaborator. Exit 2 is BLOCKED. Supplemental structural checks never replace production integration. See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md`.
