// TJ-ARCH-MOB-001 compliant
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
}
#[derive(Debug, thiserror::Error)]
pub enum NoteError {
    #[error("enter a note between 1 and 200 characters")]
    InvalidTitle,
    #[error("note storage: {0}")]
    Storage(String),
}
#[derive(Debug, Clone)]
pub struct NoteTitle(String);
impl NoteTitle {
    pub fn parse(value: &str) -> Result<Self, NoteError> {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 200 {
            return Err(NoteError::InvalidTitle);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
pub trait NoteRepository {
    fn list(&self) -> Result<Vec<Note>, NoteError>;
    fn create(&self, title: &NoteTitle) -> Result<Note, NoteError>;
}
