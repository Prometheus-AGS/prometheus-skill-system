Feature: forge enrich
  As a developer
  I want to enrich a task description with relevant skills and constitution context
  So that my agent has grounded implementation guidance

  Scenario: Enrich a task directory produces context file
    Given a task directory "my-task" with a tasks.md file containing "implement a REST API in Rust using axum"
    And a project root with a valid skill directory
    When I run forge enrich on the "my-task" directory
    Then the exit code is 0
    And the output contains "Enriched"

  Scenario: Path traversal attempt is rejected
    When I attempt to enrich with task_path "../../etc/passwd" via MCP
    Then the response contains "outside the project root"

  Scenario: Missing task directory returns error
    When I run forge enrich on a non-existent path "nonexistent-task"
    Then the exit code is non-zero
