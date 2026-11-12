use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A value in a DSPy record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Value>),
    // Multi-modal additions
    Image {
        data: String, // base64 or url
        mime_type: Option<String>,
    },
    Document {
        content: String,
        metadata: BTreeMap<String, Value>,
    },

    Object(BTreeMap<String, Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}
