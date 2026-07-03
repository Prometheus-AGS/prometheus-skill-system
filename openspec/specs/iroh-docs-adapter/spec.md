# iroh-docs-adapter Specification

## Purpose

Defines the public `IrohDocsAdapter` behaviors required for native iroh-docs ticket sharing, ticket import, and cross-author reads after live peer sync.

## Requirements

### Requirement: Iroh docs share ticket export

`IrohDocsAdapter` SHALL expose public methods for exporting read-only and writable native iroh-docs share tickets for the adapter's active document namespace.

#### Scenario: Writable ticket export includes peer addressing

- **GIVEN** an initialized `IrohDocsAdapter`
- **WHEN** a caller requests a writable share ticket
- **THEN** the adapter returns a native iroh-docs `DocTicket`
- **AND** the ticket contains addressing information that another adapter can use to import and join live sync.

### Requirement: Iroh docs ticket import

`IrohDocsAdapter` SHALL expose constructors for importing a native iroh-docs share ticket into memory-backed and persistent adapters.

#### Scenario: Importing a ticket joins the shared namespace

- **GIVEN** a source adapter with a writable share ticket
- **WHEN** a second adapter is constructed from that ticket
- **THEN** the second adapter imports the same document namespace
- **AND** the second adapter starts live sync with the peers contained in the ticket during lazy initialization.

### Requirement: Cross-author reads after sync

`IrohDocsAdapter` SHALL read the latest synced value for a key across authors, not only entries written by its local default author.

#### Scenario: Imported adapter reads source write

- **GIVEN** adapter A writes a key before exporting a writable ticket
- **AND** adapter B imports that ticket
- **WHEN** live sync completes
- **THEN** adapter B can read the value written by adapter A through the `StorageProvider::read` interface.

#### Scenario: Source adapter reads imported peer write

- **GIVEN** adapter B imports a writable ticket from adapter A
- **WHEN** adapter B writes a key and live sync completes
- **THEN** adapter A can read the value written by adapter B through the `StorageProvider::read` interface.
