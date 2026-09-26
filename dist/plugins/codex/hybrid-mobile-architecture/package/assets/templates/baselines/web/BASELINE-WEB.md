# Web note baseline

From the project root: run `npm ci --prefix web`, `npm run build --prefix web`, then `cargo run --manifest-path server/Cargo.toml`. Open http://127.0.0.1:3000. Development: keep the server running and run `npm run dev --prefix web`.

Rust owns note validation and SQLite persistence. React components use hooks; the hooks read Prometheus Entity Management and UI-only Zustand state; the store registers the repository transport. HTTP is a projection boundary to the local Axum service. Data is stored at `APP_DATA_DIR/notes.sqlite3` (default .data). Stop/restart the server and reload the page to recover notes.

This is a local single-user baseline, not a hosted authenticated tenant service. The /api/runtime endpoint explicitly returns unavailable until a real UAR service adapter is configured. There is no generated agent loop.

Verification: install the Playwright Chromium browser with `node web/node_modules/playwright/cli.js install chromium`, then run `npm run test:e2e --prefix web`. The test builds the real application, writes a note through Chromium, stops the Axum process, restarts it against the same database, and verifies recovery. It also checks invalid input and the explicit unavailable UAR endpoint.

Capability registries: `web/public/capabilities/index.json`, `server/capabilities/index.json`, and `rust/capabilities/index.json`. Registry schema1 is read by the runtime; Rust registries are embedded during build, so rebuild after capability changes. The UI displays registered names; registration does not invent business implementations.
