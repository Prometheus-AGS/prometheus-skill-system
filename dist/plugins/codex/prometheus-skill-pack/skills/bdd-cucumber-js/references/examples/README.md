# cucumber-js Examples

Two examples demonstrating the same business behavior — **a user signs in
successfully** — tested two different ways. Read both, then decide which
style to reach for.

## Choose your style

| Question | Answer means |
|----------|--------------|
| Am I testing a REST/GraphQL/CLI surface, no browser needed? | **HTTP-only** — see [api-http-only](./api-http-only/) |
| Am I testing rendering, forms, navigation, or need visual proof? | **Playwright-driven** — see [ui-playwright](./ui-playwright/) |
| Am I testing both, but the browser flow is the primary experience? | Playwright-driven, and add an `@api` sub-scenario for the auth call |
| Do I care about video proof of the run? | Playwright-driven (WebM native → MP4 via ffmpeg) |
| Do I just want to prove the auth endpoint works? | HTTP-only. Faster, cheaper, less flaky. |

### Rule of thumb

**Prefer HTTP-only when it fits.** Browser tests are 10-100× slower and
flakier. Reach for Playwright when the behavior *requires* a rendered UI to
be meaningful.

## What each example shows

### `api-http-only/`

- Plain `cucumber-js` runner
- Uses `fetch()` from a step definition
- No Playwright dependency
- Runs in ~200 ms per scenario
- Video: N/A (no browser to record)

### `ui-playwright/`

- `playwright-bdd` runner (features compile to Playwright tests)
- Uses Playwright's `page` fixture
- Inherits Playwright's video, trace, screenshot, and retry mechanics
- Runs in ~3-5 s per scenario
- Video: WebM native, remux to MP4 via ffmpeg for certification bundles

## Running the examples

Each example is self-contained under its subdirectory. From the target
project root:

```bash
# HTTP-only
npx cucumber-js --config skills/testing/bdd-cucumber-js/references/examples/api-http-only/cucumber.yml

# Playwright-driven
npx playwright test --config skills/testing/bdd-cucumber-js/references/examples/ui-playwright/playwright.config.ts
```

Both examples assume a running auth service at `http://localhost:3000`.
See each subdirectory's README for the exact fixture setup.
