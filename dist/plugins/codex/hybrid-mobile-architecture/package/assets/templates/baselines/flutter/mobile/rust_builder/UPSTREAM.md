# SDK integration provenance

Platform integration and cargokit/ originate from flutter_rust_bridge_codegen 2.12.0 integrate. Cargokit contains upstream shell/cmd/PowerShell launchers required by its SDK build integration; these are vendor internals, not first-party orchestration. First-party setup and verification run through Node .mjs tools. Gradle wrappers in mobile/android originate from Flutter SDK generation. These vendor files must be included as vendor exclusions in portability scans. Do not claim Windows application execution from their presence.

Local vendor patches: exact Rust1.97.1 toolchain selection, static archive deduplication from scripts/patch-cargokit-ios.mjs, and removal of environment-variable dumping from build_pod.sh.

Gradle9 compatibility: replace removed Project.exec with injected ExecOperations (https://docs.gradle.org/current/userguide/service_injection.html#execoperations). Respect Flutter-requested architectures instead of unconditionally appending obsolete Android x86 debug targets.

Android plugin compile SDK36, minimum SDK29 and Java17 align the FFI plugin with the current Flutter AndroidX dependencies and package platform floor.
