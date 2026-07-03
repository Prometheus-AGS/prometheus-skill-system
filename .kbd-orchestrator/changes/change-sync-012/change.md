# change-sync-012: sovereign-client Rust SDK

**Phase:** phase-learn-sovereign-sync
**Tier:** 3 (after Tier 2)
**Status:** pending
**Gap:** G-NEW-1

## Summary

Create a Rust SDK crate (`substrate/sovereign-client`) that wraps the REST API.
Used by BossFang agents and other Rust applications that want to interact with
sovereign-sync programmatically.

## Files to change

- `substrate/sovereign-client/Cargo.toml` — new crate
- `substrate/sovereign-client/src/lib.rs` — SovereignClient struct

## API surface

```rust
pub struct SovereignClient { base_url: Url, client: reqwest::Client }

impl SovereignClient {
    pub async fn connect(url: &str) -> Result<Self>
    pub async fn list_skills(&self) -> Result<Vec<SkillEntry>>
    pub async fn search_skills(&self, query: &str) -> Result<Vec<SkillEntry>>
    pub async fn sync_push(&self, domains: &[&str]) -> Result<SyncResult>
    pub async fn sync_pull(&self, domains: &[&str]) -> Result<SyncResult>
    pub async fn sync_status(&self) -> Result<SyncStatus>
    pub async fn stream_task(
        &self,
        task_id: &str,
        input: serde_json::Value,
    ) -> Result<impl Stream<Item = AgUiEvent>>
}
```

## Tasks

- [ ] Initialize crate
- [ ] Implement SovereignClient with reqwest
- [ ] Implement stream_task using eventsource-client for SSE
- [ ] Add to workspace members
- [ ] Test: connect to a running daemon and call list_skills
