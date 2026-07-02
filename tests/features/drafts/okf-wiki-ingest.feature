# DRAFT — Open Knowledge Format (OKF) v0.1 wiki ingest round-trip.
#
# Placed under tests/features/drafts/ per the BDD Immutable-Tests Rule
# (BDD-006): new features are drafted here with matching new step
# definitions; existing steps/features are never edited to make code pass.
#
# Verifies the behavior shipped in phase-okf-llm-wiki-adoption: pk (and the
# pk-cherry MCP server that shares its engine) ingests a source into an
# OKF-conformant wiki page and maintains the two reserved bundle files.
# Steps are not yet implemented — this documents the intended contract.

Feature: OKF wiki ingest round-trip
  As an agent maintaining a Karpathy-style LLM wiki
  I want ingested sources compiled into OKF v0.1 pages with index/log upkeep
  So that the knowledge base stays conformant, navigable, and interlinked

  Background:
    Given an empty pk knowledge base

  Scenario: Ingesting a source produces an OKF-conformant page
    When I ingest the source "Axum is an async Rust web framework built on Tower."
    Then a wiki page is created
    And its frontmatter has a non-empty "type" field
    And its frontmatter has a "timestamp" in ISO 8601 form

  Scenario: Ingest maintains the reserved index.md and log.md files
    When I ingest the source "Tower provides composable middleware layers."
    Then the wiki root contains "index.md"
    And "index.md" lists the new page under its type heading
    And the wiki root contains "log.md"
    And "log.md" has a "## <YYYY-MM-DD>" heading with a Creation entry for the new page

  Scenario: Cross-references become bundle-relative body links
    Given a wiki page with concept id "tower-middleware"
    When I ingest a source that references the Tower middleware page
    Then the new page body contains a bundle-relative link to "/tower-middleware.md"
    And the page link graph includes an edge to "tower-middleware"

  Scenario: A source with citations gets a Citations section
    When I ingest the source "See the axum docs." with source id "axum-notes.md"
    Then the new page body ends with a "# Citations" section
    And the Citations section references source id "axum-notes.md"

  Scenario: Lint reports a missing type as an auto-fixable error
    Given a wiki page whose frontmatter has no "type" field
    When I run the OKF conformance lint
    Then a lint error reports the missing "type"
    And that lint error is marked auto-fixable
    And lint with --fix assigns the default type without an LLM call

  Scenario: Lint tolerates broken cross-links as warnings, never errors
    Given a wiki page whose body links to a non-existent "/gone.md"
    When I run the OKF conformance lint
    Then the broken link is reported as a warning
    And no lint error is produced for the broken link
