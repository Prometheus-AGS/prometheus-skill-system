# docusaurus-docs-site Specification

## Purpose

Define the branding, package reproducibility, and build validation requirements for the Docusaurus documentation site.

## Requirements

### Requirement: KnowMe-branded Docusaurus theme

The Docusaurus documentation site SHALL use KnowMe brand styling instead of the default Docusaurus purple theme.

#### Scenario: Theme uses KnowMe tokens

- **WHEN** the site theme CSS is inspected
- **THEN** the primary light theme color uses Ember `#E04E28`
- **AND** the primary dark theme color uses bright Ember `#FF6A3D`
- **AND** the theme defines the KnowMe font stack for base, heading, and monospace text.

#### Scenario: Navbar uses the Conviction mark

- **WHEN** the Docusaurus navbar is rendered
- **THEN** it references the KnowMe Conviction mark asset from `site/static/img`
- **AND** it provides a dark-theme variant for the mark.

### Requirement: Reproducible Docusaurus package install

The Docusaurus documentation site SHALL have a reproducible Node package install.

#### Scenario: Package versions are pinned

- **WHEN** `site/package.json` is inspected
- **THEN** production and development package versions are exact versions without floating range prefixes.

#### Scenario: Package lock is committed

- **WHEN** repository ignore rules are evaluated
- **THEN** `site/package-lock.json` is not ignored
- **AND** the lockfile records the pinned root package dependencies.

### Requirement: Docusaurus build validation

The Docusaurus documentation site SHALL pass its production build after branding and lockfile changes.

#### Scenario: Production build succeeds

- **WHEN** `npm run build` is executed from `site`
- **THEN** Docusaurus generates static files successfully.
