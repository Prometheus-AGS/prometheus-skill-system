## ADDED Requirements

### Requirement: Single embedded execution surface
The mobile/desktop embedded boundary SHALL expose run, status/events, receipt, artifact, and offline-verify operations through one Rust API and SHALL reuse the process-global runtime owned by `gen_ui_core`.

#### Scenario: Embedded run round trip
- **WHEN** an authorized Tier W request is submitted through FRB or a Tauri command
- **THEN** the caller receives a run ID, ordered lifecycle events, terminal receipt, and resolvable artifacts from the same core service behavior

### Requirement: UI isolation
Flutter and React presentation code SHALL consume execution state through their existing Riverpod/Zustand adapters and SHALL NOT instantiate Wasmtime, access private device keys, or bypass the Rust approval boundary.

#### Scenario: Grant-pending event
- **WHEN** a component request requires trusted-host approval
- **THEN** the Rust layer emits a grant-pending event and execution cannot continue until an authenticated host decision is returned

### Requirement: Mobile backend and lifecycle limits
iOS SHALL execute Tier W only through Pulley. Android SHALL use Pulley unless an explicitly detected and certified Cranelift profile is available. Mobile execution SHALL be foreground-bounded and SHALL NOT claim background continuation.

#### Scenario: Application suspension
- **WHEN** the application is suspended during an in-flight component run
- **THEN** the run is interrupted or recovered according to durable state and no completed receipt is fabricated

### Requirement: Physical-device evidence boundary
Mobile runtime certification SHALL require returned-value and receipt-verification round trips on a physical iOS device and a physical Android device. Simulator, cross-build, and compile-only results SHALL remain separately labeled evidence.

#### Scenario: Device unavailable
- **WHEN** no physical device is connected for a release candidate
- **THEN** mobile runtime status remains pending evidence while desktop Tier W status may certify independently

### Requirement: Mobile binary budget
The measured `gen_ui_core` Tier W size delta SHALL be recorded per ABI and SHALL remain below 12 MiB before mobile Tier W is declared release-ready.

#### Scenario: ABI budget exceeded
- **WHEN** any supported mobile ABI grows by 12 MiB or more
- **THEN** the mobile release gate fails with the measured delta and desktop execution remains unaffected
