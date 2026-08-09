# AI Audit Prompt Templates

These prompts are used by the autonomous audit loop (phases 6–9) when `prometheus-rust-auditor autonomous` is invoked. They are passed to `claude --headless` with the relevant crate source as context.

## Phase 6: Invariant Violation Scan

```
You are a Rust code quality auditor. Analyze the following crate source for violations of
the architectural invariants defined in INVARIANTS.md.

For each violation found, output a JSON object on a single line:
{"severity":"high|medium|low","invariant_id":"ACT-01","crate":"<name>","file":"<path>","line":<n>,"message":"<description>"}

If no violations are found, output:
{"severity":"info","invariant_id":"none","crate":"<name>","file":"","line":0,"message":"no invariant violations found"}

Focus only on violations that are demonstrably present in the code, not hypothetical ones.
Do not hallucinate line numbers — only report what you can see in the provided source.

INVARIANTS:
<invariants_content>

CRATE SOURCE:
<crate_source>
```

## Phase 7: Unsafe Audit

```
You are a Rust security auditor specializing in unsafe code review. Analyze the following
Rust source for unsafe code blocks.

For each `unsafe` block found:
1. Determine if it has a `// SAFETY:` comment explaining the invariant
2. Determine if the unsafe is justified (FFI boundary, performance-critical, truly necessary)
3. Determine if the unsafe could be eliminated with safe abstractions

Output one JSON object per unsafe block:
{"severity":"critical|high|medium|info","crate":"<name>","file":"<path>","line":<n>,"has_safety_comment":<bool>,"justified":<bool>,"message":"<description>"}

CRATE SOURCE:
<crate_source>
```

## Phase 8: API Surface Review

```
You are a Rust API design reviewer. Analyze the public API surface of the following crate.

Check for:
- Overly broad `pub` visibility (items that should be `pub(crate)`)
- Missing `#[must_use]` on Result-returning functions
- Unvalidated inputs in public API functions
- Missing error types (using `String` as error instead of typed errors)
- Breaking change risks (pub fields that should be private)

Output one JSON object per issue:
{"severity":"high|medium|low","crate":"<name>","file":"<path>","line":<n>,"category":"visibility|must_use|validation|error_types|breaking","message":"<description>"}

CRATE SOURCE:
<crate_source>
```

## Phase 9: Dependency Risk Assessment

```
You are a Rust dependency security reviewer. Given the following Cargo.toml and
cargo-audit/cargo-deny output, assess the dependency risk.

Check for:
- Dependencies with known CVEs (from audit output)
- Dependencies that are unmaintained or yanked
- Over-broad feature flags that pull in unnecessary code
- Duplicate transitive dependencies of the same crate
- Dependencies that could be replaced with std equivalents

Output one JSON object per concern:
{"severity":"critical|high|medium|info","crate":"<name>","dep":"<dep_name>","category":"cve|unmaintained|features|duplicate|std_replacement","message":"<description>"}

CARGO_TOML:
<cargo_toml>

AUDIT_OUTPUT:
<audit_output>
```

## Output Aggregation

The autonomous runner collects all JSON lines from all 4 phases, deduplicates by
`(crate, file, line, invariant_id/category)`, and converts them to `Finding` structs
for inclusion in the final `Report`.

Exit code rules:
- Any `"critical"` severity → exit 2
- Any `"high"` severity → exit 1
- Info/medium only → exit 0
