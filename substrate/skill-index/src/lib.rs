use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const SCHEMA_VERSION: &str = "prometheus-skill-index-v1";

fn canonicalize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(canonicalize).collect())
        }
        serde_json::Value::Object(object) => {
            let mut entries: Vec<_> = object.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = serde_json::Map::new();
            for (key, value) in entries {
                sorted.insert(key, canonicalize(value));
            }
            serde_json::Value::Object(sorted)
        }
        scalar => scalar,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillIndexEntry {
    pub id: String,
    pub description: String,
    pub relative_path: String,
    pub search_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillIndex {
    pub schema_version: String,
    pub entries: Vec<SkillIndexEntry>,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SkillIndexError {
    #[error("read skill index: {0}")]
    Read(#[from] std::io::Error),
    #[error("parse skill index: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unsupported skill index schema: {0}")]
    Schema(String),
    #[error("skill index entries are not strictly ordered and unique")]
    Ordering,
    #[error("skill index content hash mismatch")]
    Hash,
}

impl SkillIndex {
    pub fn from_json(json: &str) -> Result<Self, SkillIndexError> {
        let index: Self = serde_json::from_str(json)?;
        index.validate()?;
        Ok(index)
    }

    pub fn from_path(path: &Path) -> Result<Self, SkillIndexError> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }

    pub fn validate(&self) -> Result<(), SkillIndexError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(SkillIndexError::Schema(self.schema_version.clone()));
        }
        if self.entries.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(SkillIndexError::Ordering);
        }
        let body = canonicalize(serde_json::json!({
            "schemaVersion": self.schema_version,
            "entries": self.entries,
        }));
        let canonical = format!("{}\n", serde_json::to_string_pretty(&body)?);
        let actual = format!("{:x}", Sha256::digest(canonical.as_bytes()));
        if self.sha256 != actual {
            return Err(SkillIndexError::Hash);
        }
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SkillIndexEntry> {
        let tokens: Vec<String> = query
            .split_whitespace()
            .map(|token| token.to_lowercase())
            .filter(|token| !token.is_empty())
            .collect();
        let mut ranked: Vec<(usize, &SkillIndexEntry)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let score = tokens
                    .iter()
                    .filter(|token| entry.search_text.contains(token.as_str()))
                    .count();
                (score > 0).then_some((score, entry))
            })
            .collect();
        ranked.sort_by(|(left_score, left), (right_score, right)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.id.cmp(&right.id))
        });
        ranked
            .into_iter()
            .take(limit)
            .map(|(_, entry)| entry.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "schemaVersion":"prometheus-skill-index-v1",
      "entries":[
        {"id":"api-design","description":"REST contracts","relativePath":"backend/api-design","searchText":"api-design rest contracts"},
        {"id":"rust-review","description":"Rust correctness","relativePath":"rust/rust-review","searchText":"rust-review rust correctness"}
      ],
      "sha256":"08cd6d1575faa7966b5aef855f3091a0e4e1a53d8f5fba2806519934a16a9888"
    }"#;

    #[test]
    fn selection_is_ranked_and_deterministic() {
        let index = SkillIndex::from_json(FIXTURE).unwrap();
        assert_eq!(index.search("rust correctness", 5)[0].id, "rust-review");
        assert_eq!(index.search("rest", 1)[0].id, "api-design");
    }

    #[test]
    fn rejects_duplicate_or_unsorted_ids() {
        let mut index = SkillIndex::from_json(FIXTURE).unwrap();
        index.entries.reverse();
        assert!(matches!(index.validate(), Err(SkillIndexError::Ordering)));
    }
}
