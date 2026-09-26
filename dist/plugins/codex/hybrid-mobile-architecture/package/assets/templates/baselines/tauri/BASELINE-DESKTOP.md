# Desktop note baseline

Install the host Tauri prerequisites (macOS Xcode tools; Windows MSVC and WebView2; Linux WebKitGTK4.1 development libraries). From the project root run `npm ci --prefix desktop`, then `npm run tauri --prefix desktop -- dev`. Build with `npm run tauri --prefix desktop -- build --no-bundle`.

React → feature hook → PEM/entity store → typed Tauri commands → Rust use case → repository port → SQLite. Only local window capabilities expose list_notes/create_note. Rust stores notes in the OS application-data directory; APP_DATA_DIR overrides it for isolated verification. Quit/relaunch and verify the saved note. Windows x64 and ARM64 each require native build AND GUI execution evidence; cross-checking Rust alone does not certify a desktop application.

UAR is not configured in this baseline. No agent responses or tool success are simulated.

Capability registries: `desktop/public/capabilities/index.json` and `rust/capabilities/index.json`. The local webview reads its asset registry and invokes the Rust registry command, with no external networking. Rebuild after changing the Rust registry.
