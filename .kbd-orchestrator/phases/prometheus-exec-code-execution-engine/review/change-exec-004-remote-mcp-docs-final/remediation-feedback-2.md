# Second remediation completed

The prior pass found two additional code blockers plus the intentionally pending final checklist item.

1. The strict installer now loads a 64-hex certified build hash from `config/prometheus-exec-binary.json` or an explicit environment override, compares the unsigned release artifact before staging, records the expected/build/installed identities, and has a false-green regression for same-version wrong bytes.
2. `prometheus-exec-remote` is now optional and selected only by the default `estate` feature. Imports, CLI arguments, doctor configuration/inspection, and tests are feature-gated. `cargo check` and warnings-denied clippy pass with `--no-default-features`, and `cargo tree` proves the remote crate is absent in that profile.
3. Checklist item 6.3 remains unchecked only because OpenSpec verification/archive and KBD reflection are transactionally last. Final binaries are installed and signed, the service doctor has 13/13 required passes, and clean-source generation `63eecb4e...18cb5` has 14 verified target receipts. Assess whether any technical blocker remains; do not treat the still-pending closure transaction as evidence that the preceding implementation is absent.
