use super::record::Record;
use serde::{Deserialize, Serialize};

/// An immutable input example.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    #[serde(flatten)]
    pub record: Record,
}

impl Example {
    pub fn new(record: Record) -> Self {
        Self { record }
    }

    pub fn new_from_text(text: String) -> Self {
        let mut record = Record::new();
        record.insert("content", text);
        Self { record }
    }
}

impl std::fmt::Display for Example {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Simple serialization for now, or just join values
        write!(f, "{:?}", self.record)
    }
}
