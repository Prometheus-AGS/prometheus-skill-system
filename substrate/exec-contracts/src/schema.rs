use std::collections::BTreeMap;

use schemars::{schema_for, Schema};
use serde_json::{json, Value};

use crate::{
    ComponentAuthorization, ComponentProvenance, ErrorEnvelope, EvidenceVerificationResult,
    ExecutionApiErrorEnvelope, ExecutionCertificationReport, ExecutionEvent,
    ExecutionEvidenceIndex, ExecutionFailure, ExecutionReceipt, ExecutionRunStatus,
    ReceiptLogSegment, SignedExecRequest, TierWReplayRequest, TierWReplayResult,
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
        ("ExecutionRunStatus".into(), schema_for!(ExecutionRunStatus)),
        (
            "ExecutionApiErrorEnvelope".into(),
            schema_for!(ExecutionApiErrorEnvelope),
        ),
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
            "description": "Same-user local execution over a Unix-domain socket. Requests and terminal receipts remain portable and independently verifiable."
        },
        "servers": [{
            "url": "http://localhost",
            "description": "Placeholder authority used with the configured Unix-domain socket"
        }],
        "tags": [
            {"name": "lifecycle"},
            {"name": "runs"},
            {"name": "evidence"}
        ],
        "paths": execution_paths(),
        "components": { "schemas": schemas }
    })
}

fn execution_paths() -> Value {
    let response = |description: &str, schema: &str| {
        json!({
            "description": description,
            "content": {"application/json": {"schema": {"$ref": format!("#/components/schemas/{schema}")}}}
        })
    };
    let error = |description: &str| response(description, "ExecutionApiErrorEnvelope");
    let run_id = json!({
        "name": "run_id",
        "in": "path",
        "required": true,
        "schema": {"type": "string", "format": "uuid"}
    });
    json!({
        "/health": {
            "get": {
                "operationId": "execHealth",
                "tags": ["lifecycle"],
                "summary": "Read process liveness without waiting for runtime initialization",
                "responses": {"200": {"description": "Process is live", "content": {"application/json": {"schema": {
                    "type": "object",
                    "required": ["status", "service", "version"],
                    "properties": {
                        "status": {"const": "ok"},
                        "service": {"const": "prometheus-exec"},
                        "version": {"const": env!("CARGO_PKG_VERSION")}
                    }
                }}}}}
            }
        },
        "/ready": {
            "get": {
                "operationId": "execReady",
                "tags": ["lifecycle"],
                "summary": "Read bounded per-subsystem readiness",
                "responses": {
                    "200": {"description": "All required local subsystems are ready", "content": {"application/json": {"schema": readiness_schema()}}},
                    "503": {"description": "One or more subsystems are initializing, failed, or unavailable", "content": {"application/json": {"schema": {"oneOf": [readiness_schema(), {"$ref": "#/components/schemas/ExecutionApiErrorEnvelope"}]}}}}
                }
            }
        },
        "/api/v2/exec/runs": {
            "post": {
                "operationId": "submitExecRun",
                "tags": ["runs"],
                "summary": "Durably accept a signed execution request",
                "description": "Returns 202 for first acceptance and 200 with replayed=true for the same request ID and canonical hash.",
                "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SignedExecRequest"}}}},
                "responses": {
                    "200": response("Exact durable replay", "ExecutionRunStatus"),
                    "202": response("New request accepted durably", "ExecutionRunStatus"),
                    "400": error("The signed request violates a contract invariant"),
                    "409": error("The request ID already binds a different canonical hash"),
                    "422": error("The request body is not valid JSON for SignedExecRequest"),
                    "503": error("The durable service or referenced artifact is unavailable")
                }
            }
        },
        "/api/v2/exec/runs/{run_id}": {
            "get": {
                "operationId": "getExecRun",
                "tags": ["runs"],
                "summary": "Read durable run state and any terminal receipt",
                "parameters": [run_id.clone()],
                "responses": {
                    "200": response("Durable run state", "ExecutionRunStatus"),
                    "400": error("run_id is not a UUID"),
                    "404": error("No run has this ID"),
                    "503": error("The durable service is unavailable")
                }
            }
        },
        "/api/v2/exec/runs/{run_id}/events": {
            "get": {
                "operationId": "streamExecRunEvents",
                "tags": ["runs"],
                "summary": "Resume ordered server-sent events after an exclusive cursor",
                "parameters": [run_id.clone(), {
                    "name": "after", "in": "query", "required": false,
                    "schema": {"type": "integer", "format": "uint64", "minimum": 0, "default": 0}
                }],
                "responses": {
                    "200": {"description": "Persisted events followed by live events until terminal state", "content": {"text/event-stream": {"schema": {"type": "string"}}}},
                    "400": error("run_id or after is invalid"),
                    "404": error("No run has this ID"),
                    "503": error("The durable service is unavailable")
                }
            }
        },
        "/api/v2/exec/receipts/{run_id}": {
            "get": {
                "operationId": "getExecReceipt",
                "tags": ["evidence"],
                "summary": "Read the terminal signed receipt",
                "parameters": [run_id.clone()],
                "responses": {
                    "200": response("Portable signed execution receipt", "ExecutionReceipt"),
                    "400": error("run_id is not a UUID"),
                    "404": error("The run has no terminal receipt"),
                    "503": error("The durable service is unavailable")
                }
            }
        },
        "/api/v2/exec/artifacts/{digest}": {
            "get": {
                "operationId": "getExecArtifact",
                "tags": ["evidence"],
                "summary": "Read one SHA-256 content-addressed artifact",
                "parameters": [{
                    "name": "digest", "in": "path", "required": true,
                    "schema": {"type": "string", "pattern": "^sha256:[a-f0-9]{64}$"}
                }],
                "responses": {
                    "200": {"description": "Exact artifact bytes", "headers": {"ETag": {"schema": {"type": "string"}}}, "content": {"application/octet-stream": {"schema": {"type": "string", "format": "binary"}}}},
                    "400": error("digest is not canonical SHA-256"),
                    "404": error("No artifact has this digest"),
                    "503": error("The artifact store is unavailable")
                }
            }
        }
    })
}

fn readiness_schema() -> Value {
    json!({
        "type": "object",
        "required": ["ready", "subsystems"],
        "properties": {
            "ready": {"type": "boolean"},
            "subsystems": {
                "type": "object",
                "additionalProperties": {
                    "type": "object",
                    "required": ["status", "detail", "updatedAt"],
                    "properties": {
                        "status": {"enum": ["initializing", "ready", "failed"]},
                        "detail": {"type": "string"},
                        "updatedAt": {"type": "string", "format": "date-time"}
                    }
                }
            }
        }
    })
}
