# Restore the local health and specification checks

Two active KBD sessions are the primary user scenario. Finishing that work also requires closing the local checks that were already red. A working two-session demonstration establishes communication; the two cases below establish that users can install and diagnose the pack and use its specifications on the same machine.

**Scope.** Full pack installation and doctor, Full OpenSpec changes and capability specifications, preservation of the existing Companion strict check, and local regression evidence. The Full change owns both tasks. The shared local runner belongs to the live-sessions change. Full and Companion product repositories retain their respective code and specification histories.

## GREEN-001 · Diagnose the installed profile

**Owner:** GREEN-01. A user installs the actual skills profile in one isolated environment and the supported full profile in another, using the installer's normal entry point on this Mac. Running the unfiltered doctor from a newly registered consumer project SHALL diagnose the installed generation and dependencies, including when the consumer project contains neither the pack source tree nor a Companion socket. Both healthy installations SHALL give exit 0, zero failures and no unresolved baseline warning. Report availability per check and profile. A service or hook required by the selected installation must actually run or work for that check to pass.

The recorded skills-profile installation contained 218 skills including 28 KBD skills. CLI 1.15.0 and pack 1.17.0 verified signed standalone work with Companion absent. Its unfiltered doctor exited 1, with 9 failures, 1 warning, 6 passes and 8 skips. Preserve the identities of every check in the baseline and document the treatment of each one in each profile:

| Baseline ID | Investigation and required observable |
|---|---|
| `skills.directory` | Resolve the installed catalog and check its contents from the consumer checkout. |
| `binaries.manifest`, `execution.runtime` | Compare the selected installation to its binary manifest and validate the required installed executables. The old run found six binary and two execution-runtime defects. |
| `services.launch-agents`, `learning.worker` | Check the isolated user's real service definitions, registrations, executable and worker state for the selected profile. The old run found four service defects and a missing worker while reporting a loaded service. |
| `learning.snapshots`, `hooks.rotation` | Resolve the project, installation and per-user files correctly; verify the required snapshot, rotation configuration and owned service state. |
| `hooks.lifecycle`, `hooks.harness-adapters` | Validate the installed lifecycle generation and adapters from a consumer checkout. The recorded invocation looked for a pack script and release manifest relative to the consumer; it reported 47 hook-graph defects. |
| `skills.discovery-budget` | The old run had measurements for 0/4 harnesses and 0/4 configured discovery profiles. Produce current real measurements for every configured profile that requires one, or show from installation/configuration provenance why that profile does not apply. An empty measurement set for a configured harness still fails closure. |

On a skills-only installation, legitimately unselected services may be explicitly inapplicable, with a reason backed by the installation manifest. The full-profile run must exercise applicable service, catalog, binary, hook and discovery checks. A required check that cannot run is BLOCKED. The installed KBD projection check, signed standalone readback, pack-to-Companion continuity and disposable service restart must remain green. Historical counts identify the baseline; the selected release manifest defines expected installed contents.

## GREEN-002 · Restore strict validation in both repositories

**Owner:** GREEN-02. A user validating Full or Companion from its root SHALL get a successful strict, noninteractive validation of the whole workspace. Use OpenSpec 2.1.1, the version that produced the failed inventory, to reproduce and close it. Preserve the project's managed pinned check where its version differs, and record each actual executable's version.

Full had 158 results with 28 passing and 130 failing. The failures comprise 104 change records and 26 capability specifications. The immutable receipt keys each row by type and ID. Missing requirement deltas occurred in 102 records, placeholder purposes in 24 specifications, and seven overlong-requirement warnings occurred across six distinct records. These diagnoses overlap. Companion’s repaired workspace change passed (1/1).

Each original item SHALL remain valid under its original type/ID or have a one-to-one or explicitly enumerated semantic migration with auditable provenance, a valid successor, and preserved requirements and scenarios. Every one of the 130 failures, all 28 originally passing items, and every newly included item must be accounted for. Recover missing deltas from authoritative authored requirements, write capability-specific purposes tied to actual behavior, and preserve obligations and negative cases when splitting long requirements. A metadata exemption for a genuinely specification-free change requires item-specific source evidence and review; it cannot silently discharge a missing product requirement. Full inventory and per-item evidence must agree. Keep Companion’s existing passing workspace change in the target set.

**Phase constraint.** The recalled lead lesson distinguishes a completed task ledger from verified outcome evidence. The exit decision therefore reads the raw doctor and validator results from production entry points and independently reconciles baseline IDs. All cases are currently specified. Execute the complete adopted local boundary after implementation; preserve historical failures for comparison.

[Standalone baseline](/Users/gqadonis/Projects/companion-pack-live.wgtMGxnu/standalone/summary.json) · [Full strict inventory](/Users/gqadonis/Projects/companion-pack-live.wgtMGxnu/prometheus-skill-pack/strict.json) · [Companion strict baseline](/Users/gqadonis/Projects/companion-pack-live.wgtMGxnu/prometheus-companion/strict-after-mirror.json) · [Phase recall](../../phases/vlbv-r4-portability/children/intersession-communication/prior-context.md)
