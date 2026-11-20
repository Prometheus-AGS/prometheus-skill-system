Feature: Maintain resident instructions in one existing checkout
  As the operator of established repositories
  I can safely inspect and add the goals-first policy without initializing a repository
  So existing project instructions and working workflows remain intact

  @full-bs-1-behavior
  Scenario: Audit and apply preserve a dirty hand-maintained pair
    When I work through a dirty two-document update
    Then audit and preview are read-only, and apply changes only approved policy
    And a second approved apply is a no-op

  @full-bs-1-negative
  Scenario: A preview cannot apply over later file or index edits
    When I change a named instruction or its Git staging after preview
    Then the stale digest is refused without changing either document

  @full-bs-2-behavior
  Scenario: Existing canonical readers in either direction stay untouched
    When I update relative symlinks and recognized existing instruction routes
    Then each operation changes only the canonical regular destination

  @full-bs-2-negative
  Scenario: Ambiguous or excluded root instruction authority is reported
    When I offer ignored, conflicting, malformed, circular, external or generated instruction entries
    Then each operation is blocked without creating, replacing or mutating an instruction entry

  @full-bs-3-behavior
  Scenario: Policy maintenance does not replace the mature scaffold path
    When I request policy audit and the original bootstrap dry run
    Then each remains a different operation and the existing checkout is untouched

  @full-bs-3-negative
  Scenario: Legacy mutation flags cannot get through a policy operation
    When I combine policy preview with legacy force
    Then the combined command is rejected before it can scaffold
