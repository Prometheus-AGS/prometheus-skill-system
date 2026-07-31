# KnowMe plugin host — evidence, captured 2026-07-31
## know-me-system @ 28c0e10f854ef2b999884bb2a1b0cd06b592c30b
Absolute root: /Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host

### src/host.rs (sha256 9b28fb1be8501f3d596314efdece0de87e929eae22a80c45d824dd20f393aeaa)
```rust
    async fn load(&self, bytes: &[u8]) -> Result<LoadedComponent, SandboxError>;

    /// Instantiate a loaded component for a declared world and capability
    /// set. A component importing a host interface outside the set fails
    /// with `SandboxError::MissingCapability` before any guest code runs.
    /// The guest's `lifecycle.init` export is called before this returns.
    async fn instantiate(
        &self,
        component: &LoadedComponent,
        config: InstanceConfig,
    ) -> Result<InstanceId, SandboxError>;

    /// `hook.handle-event`.
    async fn invoke_hook(
        &self,
        id: InstanceId,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), SandboxError>;
```
### src/sandbox/e2e.rs (sha256 391175f3c9efa00ea308a1701f728aca848f75d69879c4f98834c6cbd8265e62)
test count: 10
instantiate() call sites: 11
### src/sandbox/bindings.rs exists: yes

## Full absolute paths (round 2: bare paths were unresolvable)

Bare `src/host.rs` / `src/sandbox/e2e.rs` collide with or are missing from
THIS repo. The files are these, and only these:

| Absolute path | SHA-256 | Exists |
| `/Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host/src/host.rs` | `9b28fb1be8501f3d596314efdece0de87e929eae22a80c45d824dd20f393aeaa` | yes |
| `/Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host/src/sandbox/e2e.rs` | `391175f3c9efa00ea308a1701f728aca848f75d69879c4f98834c6cbd8265e62` | yes |
| `/Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host/src/sandbox/bindings.rs` | `560be9eae8163f94eec66d4b2bfa19ad7afd22d911e36347a7f327cae00d2214` | yes |

Reproduce:

```bash
shasum -a 256 /Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host/src/sandbox/e2e.rs
grep -c '#\[tokio::test\]\|#\[test\]' /Users/gqadonis/Projects/know-me/know-me-system/rust/crates/knowme_plugin_host/src/sandbox/e2e.rs   # => 10
```

**These paths are NOT in this repository and cannot be resolved from the review
packet.** They are cross-repo facts, evidenced by hash — not verifiable by a
reviewer scoped to this repo alone.
