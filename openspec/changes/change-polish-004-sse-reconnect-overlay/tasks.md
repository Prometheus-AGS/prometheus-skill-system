# Tasks — change-polish-004-sse-reconnect-overlay

- [x] Add sseReconnecting: false to job model initialisation in deepResearchApp()
- [x] Set job.sseReconnecting = true in openSseStream() onerror handler
- [x] Set job.sseReconnecting = false in openSseStream() onopen handler
- [x] Clear sseReconnecting on terminal job states (done, error, cancelled)
- [x] Add .reconnect-overlay and .reconnect-label CSS to the style block
- [x] Add reconnect overlay HTML inside the progress ring container with x-show binding
- [x] Verify prefers-reduced-motion gate on animation
- [x] Commit the change
