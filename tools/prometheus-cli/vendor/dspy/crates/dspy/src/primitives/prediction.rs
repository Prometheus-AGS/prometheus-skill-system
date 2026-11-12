use super::record::Record;
use serde::{Deserialize, Serialize};

/// An output prediction from a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    #[serde(flatten)]
    pub record: Record,
}

impl Prediction {
    pub fn new(record: Record) -> Self {
        Self { record }
    }
}
