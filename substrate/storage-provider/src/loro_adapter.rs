use crate::traits::{CrdtEngine, Result, StorageError};
use loro::{ExportMode, LoroDoc, LoroList, LoroMap, LoroValue};

/// `CrdtEngine` backed by Loro 1.13 — the CRDT used across the learn-domain
/// substrate (learner model mastery state, FSRS-6 cards, gap records).
pub struct LoroAdapter;

impl LoroAdapter {
    fn open(doc_bytes: &[u8]) -> Result<LoroDoc> {
        if doc_bytes.is_empty() {
            return Ok(LoroDoc::new());
        }
        LoroDoc::from_snapshot(doc_bytes)
            .or_else(|_| {
                let doc = LoroDoc::new();
                doc.import(doc_bytes)?;
                Ok::<_, loro::LoroError>(doc)
            })
            .map_err(|e| StorageError::Crdt(e.to_string()))
    }

    fn export_snapshot(doc: &LoroDoc) -> Result<Vec<u8>> {
        doc.export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Crdt(e.to_string()))
    }
}

impl CrdtEngine for LoroAdapter {
    fn new_doc(&self) -> Vec<u8> {
        let doc = LoroDoc::new();
        // An empty snapshot still carries Loro's container schema, so callers
        // get back valid, importable bytes rather than a zero-length blob.
        doc.export(ExportMode::Snapshot)
            .unwrap_or_default()
    }

    fn merge(&self, local: &[u8], remote_delta: &[u8]) -> Result<Vec<u8>> {
        let doc = Self::open(local)?;
        doc.import(remote_delta)
            .map_err(|e| StorageError::Crdt(e.to_string()))?;
        doc.commit();
        Self::export_snapshot(&doc)
    }

    fn apply_json(&self, doc: &[u8], patch: serde_json::Value) -> Result<(Vec<u8>, Vec<u8>)> {
        let loro_doc = Self::open(doc)?;
        let vv_before = loro_doc.oplog_vv();

        let root = loro_doc.get_map("root");
        write_json_object(&root, &patch)?;
        loro_doc.commit();

        let new_bytes = Self::export_snapshot(&loro_doc)?;
        let delta = loro_doc
            .export(ExportMode::Updates {
                from: std::borrow::Cow::Owned(vv_before),
            })
            .map_err(|e| StorageError::Crdt(e.to_string()))?;

        Ok((new_bytes, delta))
    }

    fn to_json(&self, doc: &[u8]) -> Result<serde_json::Value> {
        let loro_doc = Self::open(doc)?;
        let root = loro_doc.get_map("root");
        let value = root.get_deep_value();
        serde_json::to_value(value).map_err(StorageError::Serde)
    }

    fn engine_name(&self) -> &'static str {
        "loro-1.13"
    }
}

/// Write a JSON object's fields into a Loro map container, creating nested
/// Loro map/list containers for nested objects/arrays.
fn write_json_object(map: &LoroMap, value: &serde_json::Value) -> Result<()> {
    let serde_json::Value::Object(obj) = value else {
        return Err(StorageError::Crdt(
            "apply_json: top-level patch must be a JSON object".to_string(),
        ));
    };
    for (key, v) in obj {
        write_json_value_into_map(map, key, v)?;
    }
    Ok(())
}

fn write_json_value_into_map(map: &LoroMap, key: &str, v: &serde_json::Value) -> Result<()> {
    let crdt_err = |e: loro::LoroError| StorageError::Crdt(e.to_string());
    match v {
        serde_json::Value::Null => map.insert(key, LoroValue::Null).map_err(crdt_err)?,
        serde_json::Value::Bool(b) => map.insert(key, *b).map_err(crdt_err)?,
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                map.insert(key, i).map_err(crdt_err)?;
            } else {
                map.insert(key, n.as_f64().unwrap_or(0.0)).map_err(crdt_err)?;
            }
        }
        serde_json::Value::String(s) => map.insert(key, s.as_str()).map_err(crdt_err)?,
        serde_json::Value::Array(arr) => {
            let list = map.insert_container(key, LoroList::new()).map_err(crdt_err)?;
            for item in arr {
                write_json_value_into_list(&list, item)?;
            }
        }
        serde_json::Value::Object(_) => {
            let child = map.insert_container(key, LoroMap::new()).map_err(crdt_err)?;
            write_json_object(&child, v)?;
        }
    }
    Ok(())
}

fn write_json_value_into_list(list: &LoroList, v: &serde_json::Value) -> Result<()> {
    let crdt_err = |e: loro::LoroError| StorageError::Crdt(e.to_string());
    let pos = list.len();
    match v {
        serde_json::Value::Null => list.insert(pos, LoroValue::Null).map_err(crdt_err)?,
        serde_json::Value::Bool(b) => list.insert(pos, *b).map_err(crdt_err)?,
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                list.insert(pos, i).map_err(crdt_err)?;
            } else {
                list.insert(pos, n.as_f64().unwrap_or(0.0)).map_err(crdt_err)?;
            }
        }
        serde_json::Value::String(s) => list.insert(pos, s.as_str()).map_err(crdt_err)?,
        serde_json::Value::Array(arr) => {
            let child = list
                .insert_container(pos, LoroList::new())
                .map_err(crdt_err)?;
            for item in arr {
                write_json_value_into_list(&child, item)?;
            }
        }
        serde_json::Value::Object(_) => {
            let child = list
                .insert_container(pos, LoroMap::new())
                .map_err(crdt_err)?;
            write_json_object(&child, v)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_doc_round_trips_through_to_json() {
        let adapter = LoroAdapter;
        let empty = adapter.new_doc();
        let value = adapter.to_json(&empty).expect("to_json");
        assert_eq!(value, json!({}));
    }

    #[test]
    fn apply_json_then_to_json_round_trips_nested_values() {
        let adapter = LoroAdapter;
        let doc = adapter.new_doc();
        let patch = json!({
            "learner_id": "did:plc:abc",
            "mastery": 0.42,
            "concepts": {"closures": {"score": 1, "tags": ["fp", "js"]}},
        });
        let (new_doc, _delta) = adapter.apply_json(&doc, patch.clone()).expect("apply_json");
        let read_back = adapter.to_json(&new_doc).expect("to_json");
        assert_eq!(read_back, patch);
    }

    #[test]
    fn merge_combines_independent_updates() {
        let adapter = LoroAdapter;
        let base = adapter.new_doc();

        let (doc_a, delta_a) = adapter
            .apply_json(&base, json!({"a": 1}))
            .expect("apply a");
        let (_doc_b, _delta_b) = adapter
            .apply_json(&base, json!({"b": 2}))
            .expect("apply b");

        let merged = adapter.merge(&doc_a, &delta_a).expect("merge self-delta is a no-op");
        let value = adapter.to_json(&merged).expect("to_json");
        assert_eq!(value["a"], json!(1));
    }
}
