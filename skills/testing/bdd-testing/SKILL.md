---
name: bdd-testing
description: >
  Generate BDD integration tests using Cucumber.js with Gherkin syntax, Playwright browser
  validation, and video recording. Use when asked to create behavior tests, E2E tests,
  integration tests, Gherkin scenarios, or feature files. Supports API, UI, and agent
  workflow testing layers with automatic video capture of test sessions.
---

# BDD Integration Testing Skill

Generate Cucumber.js BDD tests with Playwright for this Next.js SSR application.

## Project Structure

```
tests/
├── features/          ← Gherkin .feature files
│   ├── api/           ← @api tagged scenarios
│   ├── ui/            ← @ui tagged scenarios (Playwright browser)
│   ├── agents/        ← @agent tagged scenarios
│   └── system/        ← Full system integration
├── steps/             ← TypeScript step definitions
│   ├── api.steps.ts
│   ├── ui.steps.ts
│   ├── agent.steps.ts
│   └── common.steps.ts
├── support/
│   ├── world.ts       ← CustomWorld (shared state per scenario)
│   └── hooks.ts       ← Before/After lifecycle hooks
└── reports/
    ├── videos/        ← Playwright video recordings
    └── *.html         ← Cucumber HTML reports
```

Config files at project root:

- `cucumber.js` — Profiles: `default`, `api`, `ui`, `agents`, `video`
- `tsconfig.cucumber.json` — TypeScript compilation for tests

## How to Generate Tests

### Step 1: Identify the Testing Layer

| Tag      | Layer     | What it tests                        | Tools used               |
| -------- | --------- | ------------------------------------ | ------------------------ |
| `@api`   | API       | REST endpoints, auth, CRUD           | Playwright `request` API |
| `@ui`    | Frontend  | Page rendering, forms, navigation    | Playwright `page` API    |
| `@agent` | Agent     | AI agent orchestration, tool calling | Playwright `request` API |
| `@video` | Recording | Adds video recording to `@ui` tests  | Playwright `recordVideo` |

### Step 2: Write the Feature File

Create `tests/features/{layer}/{name}.feature`:

```gherkin
@ui @video
Feature: [Feature Name]
  As a [role]
  I need to [action]
  So that [business value]

  Background:
    Given [shared precondition]

  Scenario: [Happy path]
    Given [state setup]
    When [user action]
    Then [expected outcome]

  Scenario: [Error case]
    Given [error state]
    When [user action]
    Then [error feedback visible]

  Scenario Outline: [Data-driven variation]
    Given [parameterized setup]
    When [action with <param>]
    Then [outcome with <expected>]

    Examples:
      | param | expected |
      | val1  | result1  |
      | val2  | result2  |
```

**Rules:**

- One feature per file, one behavior per scenario
- Use **declarative** steps (say _what_, not _how_)
- Use `Background` for shared Given steps
- Use `Scenario Outline` for data variations
- Always add appropriate tags: `@api`, `@ui`, `@agent`, `@video`, `@smoke`, `@slow`
- Use `data-testid` selectors for UI elements

### Step 3: Write Step Definitions

Create `tests/steps/{layer}.steps.ts`. Always use the `CustomWorld` type:

```typescript
import { Given, When, Then } from '@cucumber/cucumber';
import { CustomWorld } from '../support/world';
import { expect } from '@playwright/test';

Given(
  'I am on the {string} page',
  async function (this: CustomWorld, pageName: string) {
    await this.page.goto(`${this.baseUrl}/${pageName}`);
    await this.page.waitForLoadState('networkidle');
  }
);

When(
  'I click the {string} button',
  async function (this: CustomWorld, buttonText: string) {
    await this.page.click(
      `[data-testid="${buttonText.toLowerCase().replace(/\s+/g, '-')}-button"]`
    );
  }
);

Then('I should see {string}', async function (this: CustomWorld, text: string) {
  await expect(this.page.locator(`text=${text}`)).toBeVisible();
});
```

**Patterns:**

- Use `this: CustomWorld` type annotation on every step function
- Use `async function` (not arrow functions — Cucumber binds `this`)
- Use Cucumber Expressions (`{string}`, `{int}`) over regex
- Use Playwright's `expect` from `@playwright/test` for assertions
- Keep steps thin — delegate complex logic to helpers
- Put reusable steps in `common.steps.ts`

### Step 4: Video Recording

For `@ui` tests, video recording is automatic via `hooks.ts`. The `@video` tag enables it explicitly.

**Playwright `recordVideo` config** (applied in hooks.ts):

```typescript
this.context = await this.browser.newContext({
  recordVideo: {
    dir: 'tests/reports/videos/',
    size: { width: 1280, height: 720 },
  },
});
```

Videos are saved as WebM files in `tests/reports/videos/` after `context.close()`.

**Video lifecycle:**

1. `Before(@ui)` → Creates browser context with `recordVideo`
2. Test runs → All page interactions are recorded
3. `After(@ui)` → Saves video path, attaches to report, closes context

### Step 5: Running Tests

```bash
# All BDD tests
pnpm test:bdd

# By layer
pnpm test:bdd:api
pnpm test:bdd:ui
pnpm test:bdd:agents

# With video recording
pnpm test:bdd:video

# Specific tag
pnpm test:bdd:tag "@smoke"

# Dry run (verify wiring without execution)
npx cucumber-js --dry-run
```

## Antigravity-Specific: Browser Recording

When running tests inside Antigravity IDE, use the `browser_subagent` tool to produce
WebP video recordings that appear in the artifacts panel:

```
browser_subagent(
  TaskName: "BDD Test: Login Flow",
  Task: "Navigate to http://localhost:3000/login, fill email and password, click login, verify dashboard loads",
  RecordingName: "login_flow_test"
)
```

The `RecordingName` produces a `.webp` video saved to the artifacts directory, viewable
directly in the Antigravity conversation. Use this for:

- Visual verification of UI test scenarios
- Debugging failing UI tests
- Creating demo recordings of user flows

## CustomWorld Reference

The `CustomWorld` object provides these fields per scenario:

| Field               | Type                 | Layer        | Description                  |
| ------------------- | -------------------- | ------------ | ---------------------------- |
| `apiContext`        | `APIRequestContext`  | @api, @agent | Playwright HTTP client       |
| `response`          | `any`                | @api         | Last HTTP response           |
| `responseBody`      | `any`                | @api         | Parsed response body         |
| `authToken`         | `string`             | @api         | JWT auth token               |
| `browser`           | `Browser`            | @ui          | Chromium browser instance    |
| `context`           | `BrowserContext`     | @ui          | Browser context (with video) |
| `page`              | `Page`               | @ui          | Active page                  |
| `agentResponse`     | `any`                | @agent       | Agent chat response          |
| `agentToolCalls`    | `any[]`              | @agent       | Tools invoked by agent       |
| `agentStreamChunks` | `string[]`           | @agent       | SSE stream chunks            |
| `baseUrl`           | `string`             | all          | `http://localhost:3000`      |
| `testData`          | `Record<string,any>` | all          | Arbitrary test data          |
| `videoPath`         | `string or null`     | @ui          | Path to recorded video       |
| `tracePath`         | `string or null`     | @ui          | Path to trace file           |

## Best Practices

1. **Declarative over imperative** — `When I log in as "admin"` not `When I click username field and type...`
2. **One assertion per Then** — Keep Then steps focused
3. **Reuse steps** — Put generic steps in `common.steps.ts`
4. **Use `data-testid`** — Never rely on CSS classes or element structure
5. **Clean up** — After hooks handle teardown; use test fixtures for data
6. **Tag everything** — At minimum use `@api`, `@ui`, or `@agent`
7. **Video for UI only** — Video recording is for `@ui` tests; API tests don't need it
