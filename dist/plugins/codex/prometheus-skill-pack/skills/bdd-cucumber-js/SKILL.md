---
name: bdd-cucumber-js
version: '1.0.0'
license: MIT
description: >
  Author, run, and maintain BDD integration tests using cucumber-js 13 with
  Gherkin, playwright-bdd for browser-driven scenarios, and tsx for
  TypeScript. Use when writing behavior tests, E2E tests, integration
  tests, feature files, or step definitions in any Node/TypeScript project.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, bdd, cucumber, cucumber-js, playwright, gherkin, e2e]
---

# BDD Cucumber-js Skill

Author and run cucumber-js BDD tests in **any TypeScript project**. This
skill is project-agnostic — nothing here assumes Next.js, Nuxt, Astro, or
any specific application framework. Pair it with `bdd-lifecycle-loop` for
the create → run → triage → maintain workflow, and with `bdd-video-proof`
for certification bundles.

## When to invoke

- "Write a BDD test for feature X"
- "Add a cucumber-js scenario for the checkout flow"
- "Create integration tests using Gherkin"
- "Set up cucumber-js + Playwright in this repo"

## Stack (2026-07)

- `@cucumber/cucumber` **13.0.0** (weekly ~2.2M npm)
- `playwright-bdd` **9.2.0** — canonical cucumber-js ↔ Playwright bridge
- `tsx` — recommended TypeScript loader (per cucumber-js docs)

Direct `@cucumber/cucumber` + `playwright-core` in hooks is legacy in 2026.
`playwright-bdd` converts `.feature` files into native Playwright tests, so
you get the Playwright runner (workers, retries, trace viewer, video, HTML
report) for free.

## Directory layout

```
tests/
├── features/            ← Gherkin .feature files
│   ├── api/            ← @api scenarios (HTTP only, no browser)
│   ├── ui/             ← @ui scenarios (Playwright browser)
│   └── system/         ← Full system integration
├── steps/               ← TypeScript step definitions
│   ├── api.steps.ts
│   ├── ui.steps.ts
│   └── common.steps.ts
├── support/
│   ├── world.ts        ← Extended BddWorld or CustomWorld
│   └── hooks.ts        ← Before/After lifecycle
└── reports/
    ├── videos/         ← Playwright videos (WebM)
    ├── screenshots/    ← On-failure snapshots
    └── cucumber.json   ← Machine-readable report
```

Project root config:
- `cucumber.yml` (or `cucumber.js`) — profiles: `default`, `api`, `ui`
- `playwright.config.ts` (when using `playwright-bdd`)
- `tsconfig.cucumber.json` — TypeScript compile settings for tests

## Choose your scenario style

| Style | When to pick | Runner |
|-------|--------------|--------|
| **HTTP-only** (@api) | Testing REST/GraphQL/CLI. No browser needed. | `cucumber-js` directly, using Playwright's `request` API or `fetch` |
| **Playwright-driven** (@ui) | Any browser rendering, forms, navigation, visual proof | `playwright-bdd` — scenarios compile to Playwright tests |

Do NOT drive Playwright manually from cucumber-js `Before`/`After` hooks if
`playwright-bdd` fits — you'll lose Playwright's runner benefits (parallel
workers, trace viewer, video, HTML report).

## How to author a scenario

### 1. Write the feature file

`tests/features/{layer}/{name}.feature`:

```gherkin
@ui
Feature: User signs in with valid credentials
  As a registered user
  I need to sign in
  So that I can access my dashboard

  Background:
    Given the auth server is reachable

  Scenario: Happy path
    Given a registered user "alice@example.com" with password "hunter2"
    When they sign in with those credentials
    Then they land on the dashboard
    And the header greets them by name
```

**Rules:**
- One feature per file, one behavior per scenario
- Declarative steps ("sign in with those credentials", not "click the button with data-testid X and type Y")
- Use `Background` for shared preconditions
- Use `Scenario Outline` for data variations
- Tag every scenario: `@api`, `@ui`, `@system`, plus optional `@smoke`, `@slow`, `@flaky`
- Prefer `data-testid` selectors when using `@ui`

### 2. Write the step definitions

```typescript
import { Given, When, Then } from '@cucumber/cucumber';
import { expect } from '@playwright/test';
import type { CustomWorld } from '../support/world';

Given(
  'a registered user {string} with password {string}',
  async function (this: CustomWorld, email: string, password: string) {
    this.testUser = await this.factories.user({ email, password });
  }
);

When(
  'they sign in with those credentials',
  async function (this: CustomWorld) {
    await this.page.goto(`${this.baseUrl}/sign-in`);
    await this.page.getByTestId('email-input').fill(this.testUser.email);
    await this.page.getByTestId('password-input').fill(this.testUser.password);
    await this.page.getByTestId('submit-button').click();
  }
);

Then('they land on the dashboard', async function (this: CustomWorld) {
  await expect(this.page).toHaveURL(/\/dashboard/);
});
```

**Patterns:**
- Always type `this: CustomWorld` on step functions
- Use `async function` — NOT arrow functions (cucumber binds `this`)
- Prefer Cucumber Expressions (`{string}`, `{int}`) over regex
- Import `expect` from `@playwright/test` for browser assertions
- Keep steps thin — push complex logic into helpers
- Put generic steps in `common.steps.ts`

### 3. Set up the World

`tests/support/world.ts`:

```typescript
import { setWorldConstructor, World, IWorldOptions } from '@cucumber/cucumber';
import type { APIRequestContext, Browser, BrowserContext, Page } from '@playwright/test';

export interface CustomWorldFields {
  baseUrl: string;
  apiContext?: APIRequestContext;
  browser?: Browser;
  context?: BrowserContext;
  page?: Page;
  testData: Record<string, unknown>;
}

export class CustomWorld extends World implements CustomWorldFields {
  baseUrl = process.env.BASE_URL ?? 'http://localhost:3000';
  testData: Record<string, unknown> = {};
  apiContext?: APIRequestContext;
  browser?: Browser;
  context?: BrowserContext;
  page?: Page;

  constructor(options: IWorldOptions) {
    super(options);
  }
}

setWorldConstructor(CustomWorld);
```

### 4. Configure the runner

`cucumber.yml`:

```yaml
default:
  requireModule:
    - tsx/esm
  import:
    - tests/support/*.ts
    - tests/steps/*.ts
  format:
    - progress-bar
    - html:tests/reports/cucumber.html
    - json:tests/reports/cucumber.json
  paths:
    - tests/features/**/*.feature

api:
  inherits: default
  tags: '@api and not @slow'

ui:
  inherits: default
  tags: '@ui and not @flaky'
```

`package.json` scripts:

```json
{
  "scripts": {
    "test:bdd": "cucumber-js",
    "test:bdd:api": "cucumber-js -p api",
    "test:bdd:ui": "cucumber-js -p ui",
    "test:bdd:tag": "cucumber-js --tags"
  }
}
```

## Running

```bash
# All scenarios
npx cucumber-js

# By profile
npx cucumber-js -p api
npx cucumber-js -p ui

# By tag
npx cucumber-js --tags "@smoke"
npx cucumber-js --tags "not @flaky"

# Dry-run (verify step wiring, do not execute)
npx cucumber-js --dry-run

# Retry only flaky scenarios
npx cucumber-js --retry 2 --retry-tag-filter "@flaky"
```

## Playwright integration (playwright-bdd)

For browser-driven scenarios, switch to `playwright-bdd` — feature files
compile to Playwright tests and inherit the Playwright runner:

```typescript
// playwright.config.ts
import { defineConfig } from '@playwright/test';
import { defineBddConfig, cucumberReporter } from 'playwright-bdd';

const testDir = defineBddConfig({
  features: 'tests/features/ui/**/*.feature',
  steps: 'tests/steps/**/*.ts',
});

export default defineConfig({
  testDir,
  reporter: [
    ['html'],
    cucumberReporter('json', { outputFile: 'tests/reports/cucumber.json' }),
  ],
  use: {
    baseURL: 'http://localhost:3000',
    video: 'retain-on-failure',
    trace: 'on-first-retry',
  },
});
```

Run: `npx playwright test`

## Video capture (WebM native, MP4 via ffmpeg)

Playwright records **WebM (VP8)** natively via `video: 'retain-on-failure'`
or `video: 'on'`. To convert to MP4 for certification bundles (see
[bdd-video-proof](../bdd-video-proof/SKILL.md)):

```bash
ffmpeg -i input.webm -c copy output.mp4    # lossless stream copy
```

## See also

- [bdd-cucumber-rs](../bdd-cucumber-rs/SKILL.md) — Rust equivalent using `cucumber` 0.23
- [bdd-lifecycle-loop](../bdd-lifecycle-loop/SKILL.md) — create → run → triage → maintain workflow
- [bdd-video-proof](../bdd-video-proof/SKILL.md) — certification bundle format
- `docs/future-work/02-bdd-testing-evolution/BDD-005-testid-drift-detection.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-007-candidate-test-drafts.md`
