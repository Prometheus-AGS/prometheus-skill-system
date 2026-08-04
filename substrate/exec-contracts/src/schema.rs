use std::collections::BTreeMap;

use schemars::{schema_for, Schema};
use serde_json::{json, Value};

use crate::{
    ErrorEnvelope, ExecutionEvent, ExecutionReceipt, ReceiptLogSegment, SignedExecRequest,
    VerificationResult,
};

pub fn contract_schemas() -> BTreeMap<String, Schema> {
    BTreeMap::from([
        ("ErrorEnvelope".into(), schema_for!(ErrorEnvelope)),
        ("ExecutionEvent".into(), schema_for!(ExecutionEvent)),
        ("ExecutionReceipt".into(), schema_for!(ExecutionReceipt)),
        ("ReceiptLogSegment".into(), schema_for!(ReceiptLogSegment)),
        ("SignedExecRequest".into(), schema_for!(SignedExecRequest)),
        ("VerificationResult".into(), schema_for!(VerificationResult)),
    ])
}

pub fn openapi_components() -> Value {
    let schemas: BTreeMap<String, Value> = contract_schemas()
        .into_iter()
        .map(|(name, schema)| {
            (
                name,
                serde_json::to_value(schema).expect("schema serializes"),
            )
        })
        .collect();
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Prometheus Exec API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Portable execution evidence components. Runtime paths are added by exec-service."
        },
        "paths": {},
        "components": { "schemas": schemas }
    })
}
