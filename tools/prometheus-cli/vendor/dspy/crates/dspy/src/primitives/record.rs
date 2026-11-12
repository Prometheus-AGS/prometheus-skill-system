use super::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A generic record mapping field names to values.
/// Used for Examples, Predictions, and intermediate state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub fields: BTreeMap<String, Value>,
}

impl Record {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.fields.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.fields.get_mut(key)
    }
}
