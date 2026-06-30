Feature: forge validate
  As a developer
  I want to validate a source file against a constitution
  So that style violations are caught before review

  Scenario: No constitution returns informative message with exit 0
    Given a Rust source file "clean.rs" with content "fn main() {}"
    And no constitution for language "rust"
    When I run forge validate on "clean.rs" with language "rust"
    Then the exit code is 0
    And the output contains "No constitution found"

  Scenario: Clean file with no violations exits 0
    Given a Rust source file "clean.rs" with content "fn main() {}"
    And a rust constitution that forbids "unwrap"
    When I run forge validate on "clean.rs" with language "rust"
    Then the exit code is 0
    And the output contains "passed"

  Scenario: File with violation exits 1
    Given a Rust source file "bad.rs" with content "let x = foo().unwrap();"
    And a rust constitution that forbids "unwrap"
    When I run forge validate on "bad.rs" with language "rust"
    Then the exit code is 1
    And the output contains "unwrap"

  Scenario: Unknown language returns error
    Given a Rust source file "file.xyz" with content "content"
    When I run forge validate on "file.xyz" with language "cobol"
    Then the exit code is non-zero
    And the output contains "unknown language"
