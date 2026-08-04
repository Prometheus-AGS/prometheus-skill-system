## ADDED Requirements

### Requirement: Hard auto-approval ceiling
The execution PEP SHALL auto-approve only network-free requests whose writable paths are confined to `outputs/` and whose environment passthrough is empty. Operator Cedar policy MAY tighten but MUST NOT broaden this ceiling.

#### Scenario: Network capability requested
- **WHEN** a request includes any network egress destination
- **THEN** Cedar auto-approval is unavailable and a trusted-host grant is required

### Requirement: Privileged grant evidence
An escalated run SHALL require either a valid SSH-signed canonical manifest under namespace `prometheus-exec-grant` or an interactive trusted-host decision. The receipt SHALL record the grant kind and canonical hash; invalid or absent grants fail before execution.

#### Scenario: Missing privileged grant
- **WHEN** a request needs a privileged capability and no valid grant is supplied
- **THEN** the service returns a grant-required state without spawning code
