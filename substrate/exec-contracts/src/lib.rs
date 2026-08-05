//! Portable execution requests, receipts, signatures, and archive verification.
//!
//! This crate intentionally has no dependency on KBD, Sovereign Sync, a
//! transport, an async runtime, or an execution backend. A receipt remains
//! independently verifiable when exported from the Prometheus estate.

mod canonical;
mod crypto;
mod error;
mod evidence;
mod log;
mod model;
mod schema;
mod verify;

pub use canonical::{canonical_bytes, canonical_bytes_without, hash_bytes, hash_serializable};
pub use crypto::{
    key_id, sign_receipt_ed25519, sign_receipt_p256, sign_request_ed25519, sign_request_p256,
    verify_receipt_signature, verify_request_signature, VerificationKey,
};
pub use error::{ContractError, Result};
pub use evidence::{
    verify_evidence_bundle, ArtifactEvidence, EvidenceFile, EvidenceIdentity,
    EvidenceVerificationCheck, EvidenceVerificationFailure, EvidenceVerificationResult,
    ExecutionEvidenceIndex,
};
pub use log::{ReceiptLogEntry, ReceiptLogSegment, ReceiptLogSegmentHeader, MAX_SEGMENT_ENTRIES};
pub use model::*;
pub use schema::{contract_schemas, openapi_components};
pub use verify::{verify_receipt, VerificationCheck, VerificationFailure, VerificationResult};

pub const SCHEMA_VERSION: &str = "1";
