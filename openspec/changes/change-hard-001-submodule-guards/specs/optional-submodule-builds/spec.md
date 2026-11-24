## ADDED Requirements

### Requirement: Check manifests before optional builds

The managed installer SHALL enter optional Prometheus Knowledge, Liter LLM, Surreal Memory, Sycophancy Correction, and Template Forge builds only when the corresponding Cargo.toml exists, and continue to later components otherwise.

#### Scenario: Install with uninitialized optional submodules

- **WHEN** the optional submodule directories exist without manifests
- **THEN** the installer skips their build steps and still reaches the DSG install section
