## ADDED Requirements

### Requirement: Hooks are dispatched without a host shell
Generated hook configuration SHALL invoke a real executable with an explicit argument vector. No hook entry SHALL rely on a host shell to tokenize a command string or to interpret a substituted path.

#### Scenario: Path containing shell-significant characters
- **WHEN** a substituted plugin root contains backslashes, `$`, or backticks
- **THEN** the dispatcher receives the value unmodified because no shell parses it

#### Scenario: Script shim as the executable
- **WHEN** a hook would resolve to a batch file, command file, or shell script
- **THEN** configuration generation fails, because such a target cannot be spawned without a shell on every supported host

### Requirement: Execution eligibility comes from the manifest
The runtime SHALL determine whether a payload entry is executable from the recorded manifest and SHALL launch it by explicit interpreter argument vector. It SHALL NOT gate execution on a filesystem executable bit.

#### Scenario: Host without an executable bit
- **WHEN** the dispatcher is materialized on a host that cannot represent an executable bit
- **THEN** it is launched successfully from its recorded manifest entry

### Requirement: Concurrency control survives abnormal termination
Mutual exclusion during activation SHALL use a lock the operating system releases when the holding process ends.

#### Scenario: Holder is terminated abruptly
- **WHEN** the process holding the activation lock is killed without running its exit path
- **THEN** the lock is released by the operating system and the next invocation proceeds without stale-holder detection

### Requirement: Cold-path shell scripts declare their dependency
Scripts that remain shell SHALL be gated on probed shell availability and SHALL report an explicit message when no shell is present.

#### Scenario: Windows host without a POSIX shell
- **WHEN** a cold-path script is invoked and no shell was detected
- **THEN** the caller receives a message naming the missing dependency rather than an interpreter error
