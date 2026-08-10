# Browser drivers for cucumber-rs

Three viable browser-automation crates for `@ui` scenarios. The primary
recommendation is `thirtyfour`. Reach for the alternatives only when their
specific strengths matter.

## Comparison (2026-07)

| Crate | Version | Released | Downloads (total / 90d) | GitHub stars | Protocol | API style |
|-------|---------|----------|-------------------------|--------------|----------|-----------|
| **thirtyfour** *(primary)* | 0.37.2 | 2026-07-05 | 1.51M / 282k | 1,429 | W3C WebDriver | Typed elements, ergonomic async, Selenium-like |
| fantoccini | 0.22.1 | 2026-02-28 | 3.45M / 455k | 2,011 | W3C WebDriver | Futures-first, minimalist |
| headless_chrome | 1.0.22 | 2026-06-11 | 2.60M / 934k | 2,923 | Chrome DevTools Protocol | Puppeteer-equivalent |

## When to pick which

### `thirtyfour` — pick by default

- Modern typed async API (element handles are strongly typed)
- Multi-browser (Chrome, Firefox, Safari, Edge via WebDriver)
- Ships often — most recent release of the three
- Selenium-familiar API means step definitions read like classic Selenium
  code, which is easy for reviewers who've written Selenium in any language

Requires a running WebDriver: `chromedriver` on `:4444` (default) or
`geckodriver` for Firefox.

### `fantoccini` — pick if…

- The project already uses `fantoccini` (compat / familiarity)
- You want a smaller dependency footprint
- You need the raw WebDriver protocol without wrappers
- Recent maintenance is fine; releases just come less often

### `headless_chrome` — pick if…

- **Chrome-only is acceptable** (CDP is Chrome/Chromium/Edge only)
- You need CDP-specific features: `Page.startScreencast` for per-frame
  captures, network interception via `Fetch.enable`, or `Runtime.evaluate`
  for direct JS execution
- No WebDriver process to manage — the crate speaks CDP directly to Chrome
- Highest download volume in 2026, suggesting community drift toward CDP
  for Chrome-only automation

## Migration notes

If you inherit a `fantoccini`-based test suite and want to move to
`thirtyfour`, the mapping is mostly mechanical: `Client` → `WebDriver`,
`.goto()` is present in both, `.find(Locator::Id(x))` →
`.find(By::Id(x))`. The bigger difference is that `thirtyfour` returns
typed element handles that expose `.click()`, `.send_keys()`, etc.
directly instead of routing through the client.

## Sample capability strings

```rust
use thirtyfour::DesiredCapabilities;

// Headless Chrome
let mut caps = DesiredCapabilities::chrome();
caps.add_arg("--headless=new").unwrap();
caps.add_arg("--window-size=1280,720").unwrap();

// Firefox
let mut caps = DesiredCapabilities::firefox();
caps.add_arg("--headless").unwrap();
```

## Choosing a WebDriver process

| Browser | Driver | Install |
|---------|--------|---------|
| Chrome/Chromium/Edge | `chromedriver` | `brew install chromedriver` or `apt install chromium-driver` |
| Firefox | `geckodriver` | `brew install geckodriver` or `cargo binstall geckodriver` |
| Safari | Built-in | `safaridriver --enable` (macOS only) |

Start on the standard WebDriver port before running `cargo test`:

```bash
chromedriver --port=4444
```
