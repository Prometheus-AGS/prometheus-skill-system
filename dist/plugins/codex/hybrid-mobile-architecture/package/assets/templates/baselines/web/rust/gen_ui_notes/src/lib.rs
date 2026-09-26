// TJ-ARCH-MOB-001 compliant
use gen_ui_db::SqliteNotes;
pub use gen_ui_types::{Note, NoteError, NoteRepository, NoteTitle};
use std::path::Path;
pub struct Notes<R: NoteRepository> {
    repository: R,
}
impl<R: NoteRepository> Notes<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn list(&self) -> Result<Vec<Note>, NoteError> {
        self.repository.list()
    }
    pub fn create(&self, title: &str) -> Result<Note, NoteError> {
        self.repository.create(&NoteTitle::parse(title)?)
    }
}
pub fn open(path: &Path) -> Result<Notes<SqliteNotes>, NoteError> {
    SqliteNotes::open(path).map(Notes::new)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub kind: String,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityRegistry {
    schema_version: u32,
    capabilities: Vec<Capability>,
}
pub fn parse_capabilities(json: &str) -> Result<Vec<Capability>, NoteError> {
    let registry: CapabilityRegistry =
        serde_json::from_str(json).map_err(|e| NoteError::Storage(e.to_string()))?;
    if registry.schema_version != 1 {
        return Err(NoteError::Storage(
            "unsupported capability registry schema".to_owned(),
        ));
    }
    Ok(registry.capabilities)
}
pub fn capabilities() -> Result<Vec<Capability>, NoteError> {
    parse_capabilities(include_str!("../../capabilities/index.json"))
}
