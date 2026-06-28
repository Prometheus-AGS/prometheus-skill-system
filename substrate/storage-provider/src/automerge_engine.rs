use crate::traits::{CrdtEngine, Result, StorageError};
use automerge::{transaction::Transactable, AutoCommit, ObjType, ReadDoc};
use serde_json;

pub struct AutomergeEngine;

impl AutomergeEngine {
    pub fn new() -> Self {
        AutomergeEngine
    }
}

impl Default for AutomergeEngine {
    fn default() -> Self {
        AutomergeEngine::new()
    }
}

impl CrdtEngine for AutomergeEngine {
    fn new_doc(&self) -> Vec<u8> {
        let doc = AutoCommit::new();
        doc.save()
    }

    fn merge(&self, local: &[u8], remote_delta: &[u8]) -> Result<Vec<u8>> {
        let mut local_doc = AutoCommit::load(local)
            .map_err(|e| StorageError::Crdt(format!("Failed to load local doc: {e}")))?;
        let remote_doc = AutoCommit::load(remote_delta)
            .map_err(|e| StorageError::Crdt(format!("Failed to load remote doc: {e}")))?;
        local_doc
            .merge(&mut remote_doc.fork())
            .map_err(|e| StorageError::Crdt(format!("Merge failed: {e}")))?;
        Ok(local_doc.save())
    }

    fn apply_json(&self, doc: &[u8], patch: serde_json::Value) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut am_doc = AutoCommit::load(doc)
            .map_err(|e| StorageError::Crdt(format!("Failed to load doc: {e}")))?;

        if let serde_json::Value::Object(map) = patch {
            let root = automerge::ROOT;
            // Ensure root is a map object — create keys if needed
            for (key, value) in &map {
                // Check whether the key already exists as a map; if not, put scalar or nested map
                set_value(&mut am_doc, &root, key, value)?;
            }
        }

        let change_bytes = am_doc
            .get_last_local_change()
            .map(|c| c.raw_bytes().to_vec())
            .unwrap_or_default();

        Ok((am_doc.save(), change_bytes))
    }

    fn to_json(&self, doc: &[u8]) -> Result<serde_json::Value> {
        let am_doc = AutoCommit::load(doc)
            .map_err(|e| StorageError::Crdt(format!("Failed to load doc: {e}")))?;
        let json = automerge::AutoSerde::from(am_doc.fork());
        serde_json::to_value(&json).map_err(StorageError::Serde)
    }

    fn engine_name(&self) -> &'static str {
        "automerge"
    }
}

/// Recursively set a JSON value into an automerge document at the given object and key.
fn set_value(
    doc: &mut AutoCommit,
    obj: &automerge::ObjId,
    key: &str,
    value: &serde_json::Value,
) -> Result<()> {
    match value {
        serde_json::Value::Null => {
            doc.put(obj, key, automerge::ScalarValue::Null)
                .map_err(|e| StorageError::Crdt(format!("Put failed: {e}")))?;
        }
        serde_json::Value::Bool(b) => {
            doc.put(obj, key, *b)
                .map_err(|e| StorageError::Crdt(format!("Put failed: {e}")))?;
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                doc.put(obj, key, i)
                    .map_err(|e| StorageError::Crdt(format!("Put failed: {e}")))?;
            } else if let Some(f) = n.as_f64() {
                doc.put(obj, key, f)
                    .map_err(|e| StorageError::Crdt(format!("Put failed: {e}")))?;
            }
        }
        serde_json::Value::String(s) => {
            doc.put(obj, key, s.as_str())
                .map_err(|e| StorageError::Crdt(format!("Put failed: {e}")))?;
        }
        serde_json::Value::Array(arr) => {
            let list_id = doc
                .put_object(obj, key, ObjType::List)
                .map_err(|e| StorageError::Crdt(format!("put_object (list) failed: {e}")))?;
            for (i, item) in arr.iter().enumerate() {
                set_list_value(doc, &list_id, i, item)?;
            }
        }
        serde_json::Value::Object(map) => {
            let nested_id = doc
                .put_object(obj, key, ObjType::Map)
                .map_err(|e| StorageError::Crdt(format!("put_object (map) failed: {e}")))?;
            for (k, v) in map {
                set_value(doc, &nested_id, k, v)?;
            }
        }
    }
    Ok(())
}

fn set_list_value(
    doc: &mut AutoCommit,
    list: &automerge::ObjId,
    index: usize,
    value: &serde_json::Value,
) -> Result<()> {
    match value {
        serde_json::Value::Null => {
            doc.insert(list, index, automerge::ScalarValue::Null)
                .map_err(|e| StorageError::Crdt(format!("insert failed: {e}")))?;
        }
        serde_json::Value::Bool(b) => {
            doc.insert(list, index, *b)
                .map_err(|e| StorageError::Crdt(format!("insert failed: {e}")))?;
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                doc.insert(list, index, i)
                    .map_err(|e| StorageError::Crdt(format!("insert failed: {e}")))?;
            } else if let Some(f) = n.as_f64() {
                doc.insert(list, index, f)
                    .map_err(|e| StorageError::Crdt(format!("insert failed: {e}")))?;
            }
        }
        serde_json::Value::String(s) => {
            doc.insert(list, index, s.as_str())
                .map_err(|e| StorageError::Crdt(format!("insert failed: {e}")))?;
        }
        serde_json::Value::Array(arr) => {
            let nested = doc
                .insert_object(list, index, ObjType::List)
                .map_err(|e| StorageError::Crdt(format!("insert_object failed: {e}")))?;
            for (i, item) in arr.iter().enumerate() {
                set_list_value(doc, &nested, i, item)?;
            }
        }
        serde_json::Value::Object(map) => {
            let nested = doc
                .insert_object(list, index, ObjType::Map)
                .map_err(|e| StorageError::Crdt(format!("insert_object failed: {e}")))?;
            for (k, v) in map {
                set_value(doc, &nested, k, v)?;
            }
        }
    }
    Ok(())
}
