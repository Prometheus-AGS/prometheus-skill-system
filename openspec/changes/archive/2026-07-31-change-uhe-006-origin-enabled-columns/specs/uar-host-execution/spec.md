## ADDED Requirements

### Requirement: Skill origin is expressible in a database constraint

This change SHALL first probe whether a constraint can target definition->>'origin'. It SHALL complete in either branch — delivering that expression, or adding real columns — so that dependent changes never dangle.

#### Scenario: Either branch completes the change

- **GIVEN** the probe finds a constraint can target the JSONB field
- **WHEN** the change finishes
- **THEN** it delivers that expression and adds no columns, and dependent ordering is unchanged
