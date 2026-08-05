//! Signed, durable R-class dispatch contracts for Prometheus Exec.
//!
//! This crate is estate-only but intentionally has no dependency on KBD,
//! Sovereign Sync, a concrete transport, or an execution backend. Production
//! adapters inject enrollment snapshots and transport behavior.

#![forbid(unsafe_code)]

mod crypto;
mod error;
mod model;
mod queue;
#[cfg(feature = "transport")]
mod transport;

pub use crypto::{
    sign_dispatch_ed25519, sign_peer_response_ed25519, verify_dispatch, verify_peer_response,
};
pub use error::{RemoteError, Result};
pub use model::{
    EnrollmentBinding, EnrollmentSnapshot, PeerDispatchRecord, PeerDispatchState,
    RemoteDispatchAggregate, SignedPeerDispatchResponse, SignedRemoteDispatch,
    REMOTE_SCHEMA_VERSION,
};
pub use queue::{AcceptDispatchResult, DispatchQueue, DispatchRecord};
#[cfg(feature = "transport")]
pub use transport::{
    aggregate_records, LocalExecutionHandoff, LocalExecutionOutcome, RemoteOrigin, RemoteTarget,
    RemoteTransport,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use chrono::Utc;
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        key_id, sign_request_ed25519, CapabilityManifest, CodeIdentity, CodeKind, Digest,
        ExecutionLimits, ExecutionProvenance, RequestedTier, RuntimeKind, SignatureAlgorithm,
        SignedExecRequest, SCHEMA_VERSION,
    };
    use uuid::Uuid;

    use crate::{
        sign_dispatch_ed25519, verify_dispatch, EnrollmentBinding, EnrollmentSnapshot,
        PeerDispatchRecord, PeerDispatchState, RemoteDispatchAggregate, SignedRemoteDispatch,
        REMOTE_SCHEMA_VERSION,
    };

    pub(crate) fn fixture() -> (SignedRemoteDispatch, EnrollmentSnapshot, SigningKey) {
        let signing_key = SigningKey::from_bytes(&[11; 32]);
        let target_key = SigningKey::from_bytes(&[12; 32]);
        let now = Utc::now();
        let target = "endpoint-target".to_string();
        let mut request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id: Uuid::new_v4(),
            issued_at: now,
            queued_at: Some(now),
            validity_window_secs: 60,
            tier: RequestedTier::P,
            code: CodeIdentity {
                kind: CodeKind::File,
                hash: Digest::from_bytes(b"print(42)"),
                runtime: RuntimeKind::Python3,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities: CapabilityManifest::default(),
            limits: ExecutionLimits::default(),
            targets: vec![target.clone()],
            provenance: ExecutionProvenance::default(),
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        sign_request_ed25519(&mut request, &signing_key).expect("request signs");
        let bindings = [("endpoint-origin", &signing_key), (&target, &target_key)]
            .into_iter()
            .map(|(endpoint, key)| {
                let endpoint = endpoint.to_string();
                let public = key.verifying_key().to_bytes();
                (
                    endpoint.clone(),
                    EnrollmentBinding {
                        endpoint_id: endpoint,
                        sig_alg: SignatureAlgorithm::Ed25519,
                        key_id: key_id(SignatureAlgorithm::Ed25519, &public),
                        public_key: URL_SAFE_NO_PAD.encode(public),
                    },
                )
            })
            .collect();
        let enrollment = EnrollmentSnapshot {
            schema_version: REMOTE_SCHEMA_VERSION.into(),
            captured_at: now,
            bindings,
        };
        let mut dispatch = SignedRemoteDispatch {
            schema_version: REMOTE_SCHEMA_VERSION.into(),
            dispatch_id: Uuid::new_v4(),
            request_hash: request.request_hash().expect("request hash"),
            request,
            origin_endpoint_id: "endpoint-origin".into(),
            target_endpoint_id: target,
            enrollment_snapshot_hash: enrollment.snapshot_hash().expect("snapshot hash"),
            issued_at: now,
            validity_window_secs: 60,
            signer_key_id: String::new(),
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        sign_dispatch_ed25519(&mut dispatch, &signing_key).expect("dispatch signs");
        (dispatch, enrollment, signing_key)
    }

    #[test]
    fn signed_dispatch_binds_enrollment_request_and_target() {
        let (dispatch, enrollment, _) = fixture();
        verify_dispatch(&dispatch, &enrollment).expect("dispatch verifies");
        assert!(dispatch.dispatch_hash().is_ok());
    }

    #[test]
    fn signer_or_target_mutation_is_rejected() {
        let (mut dispatch, enrollment, _) = fixture();
        dispatch.target_endpoint_id = "endpoint-other".into();
        assert!(verify_dispatch(&dispatch, &enrollment).is_err());
    }

    #[test]
    fn aggregate_rejects_an_applied_peer_without_a_receipt() {
        let (dispatch, _, _) = fixture();
        let peers = BTreeMap::from([
            (
                "endpoint-target".into(),
                PeerDispatchRecord {
                    endpoint_id: "endpoint-target".into(),
                    state: PeerDispatchState::Applied,
                    run_id: Some(Uuid::new_v4()),
                    receipt: None,
                    failure: None,
                    updated_at: Utc::now(),
                },
            ),
            (
                "endpoint-other".into(),
                PeerDispatchRecord {
                    endpoint_id: "endpoint-other".into(),
                    state: PeerDispatchState::Rejected,
                    run_id: None,
                    receipt: None,
                    failure: Some("not enrolled".into()),
                    updated_at: Utc::now(),
                },
            ),
        ]);
        assert!(RemoteDispatchAggregate::derive(
            dispatch.dispatch_id,
            dispatch.request_hash,
            peers
        )
        .is_err());
    }
}
