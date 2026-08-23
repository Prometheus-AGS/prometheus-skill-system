# Prometheus Knowledge Semantic-Port Table

Run: `20260823T175006Z`

The current authoritative `main` tip is `cea7b9063bd0c8b2fe4c2a59f04e5e1eee87d844`.
The four requested source commits were replayed in order on that tip. Each replay
was empty after conflict resolution because merge commit `4a62bef615b1c210a94f2f97e59757be21eead94`
already contains their behavior, with newer main fixes retained.

| Source | Hunk accounting | Disposition | Acceptance check |
| --- | --- | --- | --- |
| `e5cb0dde759204beadb07583b3593b3592da9a59` | All 27 touched paths appear in `4a62bef`. `Cargo.toml`/`Cargo.lock` retain the newer coordinated `1.7.0` versions. The doctor implementation retains the newer non-mutating schema. The worker retains `cea7b90`'s local-worker recovery. | `merged-or-patch-equivalent` | Four-way replay was empty; `cargo check --workspace`; doctor contract. |
| `fe2b2ba9dfa61c5810dce4b81211fc68af583e0b` | `pk-cli/tests/doctor.rs` is present on main; the only replay conflict was formatting. | `merged-or-patch-equivalent` | Replay was empty; doctor contract verifies no filesystem mutation. |
| `89eecc11acb8c714107ad7c1d43cf96b494904bc` | Modern doctor code and its JSON/exit-status contract are present through `4a62bef`; main's formatting and version fixes win. | `merged-or-patch-equivalent` | Replay was empty; doctor contract. |
| `b2f796c7450df0b922798db950442168e1068205` | The deterministic runtime documentation is incorporated by the later `4a62bef` README rewrite. | `merged-or-patch-equivalent` | Replay was empty; README terminology check. |

Newer-main preservation:

- `4a62bef615b1c210a94f2f97e59757be21eead94` remains an ancestor of the selected tip and owns the `1.7.0` binary/version contract.
- `cea7b9063bd0c8b2fe4c2a59f04e5e1eee87d844` remains the selected tip and owns the local worker recovery behavior.
