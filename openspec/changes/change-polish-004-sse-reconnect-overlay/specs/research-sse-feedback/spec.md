## ADDED Requirements

### Requirement: Show recovery state during interruption

The research UI SHALL show a reconnecting overlay after an SSE error, clear it on reconnect or terminal job state, and respect reduced-motion preferences.

#### Scenario: Lose and regain the live stream

- **WHEN** the established SSE connection errors and later reopens
- **THEN** the progress view shows reconnecting while interrupted and clears the overlay when the connection is restored
