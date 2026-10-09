# Design

The existing shell entry point dispatches reconciliation to Node before hook and backend initialization. Filesystem readers inspect task artifacts without native-task migration or OpenSpec initialization. Canonical state is replayed by the installed runtime in a temporary copy containing only the selected project's authority and registry entry. Source hashes are checked before and after reading, and changes make the scan incomplete.

The scanner preserves JSON fields `phase`, `clean`, `drifted`, `drift` and adds `errors`. Exit 0 means clean, 1 means drift, and 2 means invalid input or incomplete inspection. Missing and ambiguous archives fail explicitly.

Repair requires the selected phase to remain active. It uses existing canonical task transitions and boundary handling, skips backend writes for already-complete tasks, preserves cancellations and archived history, stops on failure and rescans. Only legacy progress counters may be written directly; runtime projections remain runtime-owned.
