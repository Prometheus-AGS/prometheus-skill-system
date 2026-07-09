# Tasks — change-polish-004-sse-reconnect-overlay

- [ ] Add sseReconnecting: false to job model initialisation in deepResearchApp()
- [ ] Set job.sseReconnecting = true in openSseStream() onerror handler
- [ ] Set job.sseReconnecting = false in openSseStream() onopen handler
- [ ] Clear sseReconnecting on terminal job states (done, error, cancelled)
- [ ] Add .reconnect-overlay and .reconnect-label CSS to the style block
- [ ] Add reconnect overlay HTML inside the progress ring container with x-show binding
- [ ] Verify prefers-reduced-motion gate on animation
- [ ] Commit the change
