# cucumber-rs Examples

Two examples of the same business behavior — **a user signs in
successfully** — tested two different ways. Same shape as the cucumber-js
examples so you can compare stack-to-stack.

## Choose your style

| Question | Answer means |
|----------|--------------|
| Am I testing a REST/gRPC/CLI surface, no browser needed? | **HTTP-only** — see [api-http-only](./api-http-only/) |
| Am I testing rendering, forms, navigation, or need visual proof? | **Browser** — see [ui-thirtyfour](./ui-thirtyfour/) |
| Am I testing both, but the browser flow is the primary experience? | Browser, and add an `@api` sub-scenario for the auth call |
| Do I care about video proof of the run? | Browser (WebM/MP4 via `ffmpeg` sidecar or `selenium/video`) |
| Do I just want to prove the auth endpoint works? | HTTP-only. Faster (~ms), cheaper, less flaky. |

### Rule of thumb

**Prefer HTTP-only when it fits.** Browser tests are 10-100× slower and
require a running WebDriver process. Reach for `thirtyfour` when the
behavior *requires* a rendered UI.

## What each example shows

### `api-http-only/`

- `cucumber` 0.23 + `tokio` + `reqwest`
- No browser, no WebDriver — pure HTTP client
- Runs in ~50 ms per scenario
- No feature flag needed
- Video: N/A

### `ui-thirtyfour/`

- `cucumber` 0.23 + `tokio` + `thirtyfour` 0.37
- Requires `chromedriver` on `:4444` before running tests
- Gated behind a `ui` Cargo feature so headless CI can skip
- Runs in ~2-5 s per scenario
- Video: use `ffmpeg` screen capture around `cargo test` for MP4 output

## Running the examples

Each example is a self-contained Cargo crate. To try them:

```bash
# HTTP-only
cd api-http-only
cargo test --test features -- --tags @api

# Browser — requires chromedriver on :4444
cd ui-thirtyfour
chromedriver --port=4444 &
cargo test --features ui --test features -- --tags @ui
```

Both examples assume a running auth service at `http://localhost:3000`.
Fixtures for that service are out of scope — plug your own.
