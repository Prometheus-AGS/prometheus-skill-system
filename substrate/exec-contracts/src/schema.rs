use std::collections::BTreeMap;

use schemars::{schema_for, Schema};
use serde_json::{json, Value};

use crate::{
    ComponentAuthorization, ComponentProvenance, ErrorEnvelope, EvidenceVerificationResult,
    ExecutionCertificationReport, ExecutionEvent, ExecutionEvidenceIndex, ExecutionFailure,
    ExecutionReceipt, ReceiptLogSegment, SignedExecRequest, TierWReplayRequest, TierWReplayResult,
    VerificationResult,
};

pub fn contract_schemas() -> BTreeMap<String, Schema> {
    BTreeMap::from([
        (
            "ComponentAuthorization".into(),
            schema_for!(ComponentAuthorization),
        ),
        (
            "ComponentProvenance".into(),
            schema_for!(ComponentProvenance),
        ),
        ("ErrorEnvelope".into(), schema_for!(ErrorEnvelope)),
        ("ExecutionEvent".into(), schema_for!(ExecutionEvent)),
        ("ExecutionFailure".into(), schema_for!(ExecutionFailure)),
        ("ExecutionReceipt".into(), schema_for!(ExecutionReceipt)),
        (
            "ExecutionEvidenceIndex".into(),
            schema_for!(ExecutionEvidenceIndex),
        ),
        (
            "EvidenceVerificationResult".into(),
            schema_for!(EvidenceVerificationResult),
        ),
        (
            "ExecutionCertificationReport".into(),
            schema_for!(ExecutionCertificationReport),
        ),
        ("ReceiptLogSegment".into(), schema_for!(ReceiptLogSegment)),
        ("SignedExecRequest".into(), schema_for!(SignedExecRequest)),
        ("TierWReplayRequest".into(), schema_for!(TierWReplayRequest)),
        ("TierWReplayResult".into(), schema_for!(TierWReplayResult)),
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
