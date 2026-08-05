use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{verify_receipt, ExecutionReceipt, VerificationKey};
use uuid::Uuid;

use crate::{
    sign_peer_response_ed25519, verify_peer_response, DispatchQueue, DispatchRecord,
    EnrollmentSnapshot, PeerDispatchRecord, PeerDispatchState, RemoteDispatchAggregate,
    RemoteError, Result, SignedPeerDispatchResponse, SignedRemoteDispatch, REMOTE_SCHEMA_VERSION,
};

/// Result returned by the local execution service port.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalExecutionOutcome {
    pub state: PeerDispatchState,
    pub run_id: Option<Uuid>,
    pub receipt: Option<ExecutionReceipt>,
    pub failure: Option<String>,
}

/// Durable identity returned after the local request ledger has accepted the
/// canonical request. Repeating `submit` for the same request must return the
/// same identity without starting a second execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalExecutionSubmission {
    pub run_id: Uuid,
}

#[async_trait]
pub trait LocalExecutionHandoff: Send + Sync {
    async fn submit(&self, dispatch: &SignedRemoteDispatch) -> Result<LocalExecutionSubmission>;

    async fn status(
        &self,
        dispatch: &SignedRemoteDispatch,
        run_id: Uuid,
    ) -> Result<LocalExecutionOutcome>;
}

/// Injected transport boundary. The remote crate does not depend on a concrete
/// network implementation or on Sovereign Sync.
#[async_trait]
pub trait RemoteTransport: Send + Sync {
    async fn deliver(&self, dispatch: SignedRemoteDispatch) -> Result<SignedPeerDispatchResponse>;
}

pub struct RemoteTarget<E> {
    endpoint_id: String,
    queue: DispatchQueue,
    enrollment: EnrollmentSnapshot,
    signing_key: SigningKey,
    executor: Arc<E>,
}

impl<E> RemoteTarget<E>
where
    E: LocalExecutionHandoff,
{
    pub fn new(
        endpoint_id: impl Into<String>,
        queue: DispatchQueue,
        enrollment: EnrollmentSnapshot,
        signing_key: SigningKey,
        executor: Arc<E>,
    ) -> Result<Self> {
        let endpoint_id = endpoint_id.into();
        let binding = enrollment.binding(&endpoint_id)?;
        if binding.key_id
            != prometheus_exec_contracts::key_id(
                prometheus_exec_contracts::SignatureAlgorithm::Ed25519,
                &signing_key.verifying_key().to_bytes(),
            )
        {
            return Err(RemoteError::SignerMismatch(endpoint_id));
        }
        Ok(Self {
            endpoint_id,
            queue,
            enrollment,
            signing_key,
            executor,
        })
    }

    pub async fn receive(
        &self,
        dispatch: SignedRemoteDispatch,
        now: DateTime<Utc>,
    ) -> Result<SignedPeerDispatchResponse> {
        if dispatch.target_endpoint_id != self.endpoint_id {
            return Err(RemoteError::UnknownEndpoint(
                dispatch.target_endpoint_id.clone(),
            ));
        }
        let accepted = self
            .queue
            .accept_at_target(dispatch.clone(), &self.enrollment, now)?;
        if accepted.record.state.is_terminal() {
            return self.response_from_record(&accepted.record);
        }
        let received = match accepted.record.state {
            PeerDispatchState::Queued | PeerDispatchState::Unavailable => self.queue.transition(
                dispatch.dispatch_id,
                PeerDispatchState::Received,
                None,
                None,
                None,
                now,
            )?,
            PeerDispatchState::Received | PeerDispatchState::Running => accepted.record,
            _ => {
                return Err(RemoteError::InvalidTransition(format!(
                    "cannot resume target from {:?}",
                    accepted.record.state
                )))
            }
        };
        let execution_now = Utc::now();
        if received.state != PeerDispatchState::Running && execution_now > dispatch.expires_at()? {
            let expired = self.queue.transition(
                dispatch.dispatch_id,
                PeerDispatchState::Expired,
                received.run_id,
                None,
                Some("remote dispatch validity window expired before local execution".into()),
                execution_now,
            )?;
            return self.response_from_record(&expired);
        }
        let running = match received.state {
            PeerDispatchState::Received => {
                let submission = self.executor.submit(&dispatch).await.map_err(|error| {
                    RemoteError::Execution(format!("{}: {error}", dispatch.dispatch_id))
                })?;
                self.queue.transition(
                    dispatch.dispatch_id,
                    PeerDispatchState::Running,
                    Some(submission.run_id),
                    None,
                    None,
                    Utc::now(),
                )?
            }
            PeerDispatchState::Running => received,
            _ => unreachable!("target resume normalizes to received or running"),
        };
        let run_id = running.run_id.ok_or_else(|| {
            RemoteError::InvalidTransition("running target record has no local run ID".into())
        })?;
        let outcome = self
            .executor
            .status(&dispatch, run_id)
            .await
            .map_err(|error| {
                RemoteError::Execution(format!("{}: {error}", dispatch.dispatch_id))
            })?;
        if outcome
            .run_id
            .is_some_and(|outcome_id| outcome_id != run_id)
        {
            return Err(RemoteError::InvalidPeerResponse(format!(
                "local status replaced durable run ID {run_id}"
            )));
        }
        let run_id = Some(run_id);
        if let Some(receipt) = outcome.receipt.as_ref() {
            self.verify_target_receipt(receipt, &dispatch)?;
        }
        let terminal = self.queue.transition(
            dispatch.dispatch_id,
            outcome.state,
            run_id,
            outcome.receipt,
            outcome.failure,
            Utc::now(),
        )?;
        self.response_from_record(&terminal)
    }

    fn verify_target_receipt(
        &self,
        receipt: &ExecutionReceipt,
        dispatch: &SignedRemoteDispatch,
    ) -> Result<()> {
        let binding = self.enrollment.binding(&self.endpoint_id)?;
        let key = VerificationKey::from_base64url(binding.sig_alg, &binding.public_key)?;
        let verification = verify_receipt(receipt, &key, Some(&dispatch.request), None);
        if verification.valid {
            Ok(())
        } else {
            Err(RemoteError::InvalidPeerResponse(format!(
                "target receipt verification failed: {:?}",
                verification.failures
            )))
        }
    }

    fn response_from_record(&self, record: &DispatchRecord) -> Result<SignedPeerDispatchResponse> {
        let mut response = SignedPeerDispatchResponse {
            schema_version: REMOTE_SCHEMA_VERSION.into(),
            dispatch_id: record.dispatch.dispatch_id,
            dispatch_hash: record.dispatch_hash.clone(),
            request_hash: record.dispatch.request_hash.clone(),
            endpoint_id: self.endpoint_id.clone(),
            state: record.state,
            run_id: record.run_id,
            receipt: record.receipt.clone(),
            failure: record.failure.clone(),
            completed_at: record.updated_at,
            signer_key_id: String::new(),
            sig_alg: prometheus_exec_contracts::SignatureAlgorithm::Ed25519,
            signature: None,
        };
        sign_peer_response_ed25519(&mut response, &self.signing_key)?;
        Ok(response)
    }
}

pub struct RemoteOrigin<T> {
    queue: DispatchQueue,
    enrollment: EnrollmentSnapshot,
    transport: Arc<T>,
}

impl<T> RemoteOrigin<T>
where
    T: RemoteTransport,
{
    pub fn new(queue: DispatchQueue, enrollment: EnrollmentSnapshot, transport: Arc<T>) -> Self {
        Self {
            queue,
            enrollment,
            transport,
        }
    }

    pub async fn dispatch(
        &self,
        dispatch: SignedRemoteDispatch,
        now: DateTime<Utc>,
    ) -> Result<PeerDispatchRecord> {
        let accepted = self.queue.accept(dispatch.clone(), &self.enrollment, now)?;
        if accepted.record.state.is_terminal() {
            return Ok(peer_record(&accepted.record));
        }
        let response = match self.transport.deliver(dispatch.clone()).await {
            Ok(response) => response,
            Err(error) => {
                let record = self.queue.transition(
                    dispatch.dispatch_id,
                    PeerDispatchState::Unavailable,
                    accepted.record.run_id,
                    None,
                    Some(error.to_string()),
                    Utc::now(),
                )?;
                return Ok(peer_record(&record));
            }
        };
        verify_peer_response(&response, &dispatch, &self.enrollment)?;
        if let Some(receipt) = response.receipt.as_ref() {
            let binding = self.enrollment.binding(&response.endpoint_id)?;
            let key = VerificationKey::from_base64url(binding.sig_alg, &binding.public_key)?;
            let verification = verify_receipt(receipt, &key, Some(&dispatch.request), None);
            if !verification.valid {
                return Err(RemoteError::InvalidPeerResponse(format!(
                    "peer receipt verification failed: {:?}",
                    verification.failures
                )));
            }
        }
        let mut current = self
            .queue
            .get(dispatch.dispatch_id)?
            .ok_or(RemoteError::DispatchNotFound(dispatch.dispatch_id))?;
        if matches!(
            current.state,
            PeerDispatchState::Queued | PeerDispatchState::Unavailable
        ) {
            current = self.queue.transition(
                dispatch.dispatch_id,
                PeerDispatchState::Received,
                response.run_id,
                None,
                None,
                Utc::now(),
            )?;
        }
        if response.state == PeerDispatchState::Applied
            && current.state == PeerDispatchState::Received
        {
            self.queue.transition(
                dispatch.dispatch_id,
                PeerDispatchState::Running,
                response.run_id,
                None,
                None,
                Utc::now(),
            )?;
        }
        let record = self.queue.transition(
            dispatch.dispatch_id,
            response.state,
            response.run_id,
            response.receipt,
            response.failure,
            response.completed_at,
        )?;
        Ok(peer_record(&record))
    }
}

pub fn aggregate_records(
    dispatch: &SignedRemoteDispatch,
    records: impl IntoIterator<Item = PeerDispatchRecord>,
) -> Result<RemoteDispatchAggregate> {
    let peers: BTreeMap<_, _> = records
        .into_iter()
        .map(|record| (record.endpoint_id.clone(), record))
        .collect();
    RemoteDispatchAggregate::derive(dispatch.dispatch_id, dispatch.request_hash.clone(), peers)
}

fn peer_record(record: &DispatchRecord) -> PeerDispatchRecord {
    PeerDispatchRecord {
        endpoint_id: record.dispatch.target_endpoint_id.clone(),
        state: record.state,
        run_id: record.run_id,
        receipt: record.receipt.clone(),
        failure: record.failure.clone(),
        updated_at: record.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::Utc;
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        hash_bytes, sign_receipt_ed25519, EvidenceClass, ExecutingDevice, ExecutionBackend,
        ExecutionExit, ExecutionOutputs, ExecutionReceipt, ExecutionTier, ResourceUsage, RunState,
        SignatureAlgorithm, SCHEMA_VERSION,
    };
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{
        aggregate_records, LocalExecutionHandoff, LocalExecutionOutcome, LocalExecutionSubmission,
        RemoteOrigin, RemoteTarget, RemoteTransport,
    };
    use crate::{
        PeerDispatchRecord, PeerDispatchState, Result, SignedPeerDispatchResponse,
        SignedRemoteDispatch,
    };

    struct FixtureExecutor {
        key: SigningKey,
    }

    #[async_trait]
    impl LocalExecutionHandoff for FixtureExecutor {
        async fn submit(
            &self,
            _dispatch: &SignedRemoteDispatch,
        ) -> Result<LocalExecutionSubmission> {
            Ok(LocalExecutionSubmission {
                run_id: Uuid::new_v4(),
            })
        }

        async fn status(
            &self,
            dispatch: &SignedRemoteDispatch,
            run_id: Uuid,
        ) -> Result<LocalExecutionOutcome> {
            let now = Utc::now();
            let mut receipt = ExecutionReceipt {
                schema_version: SCHEMA_VERSION.into(),
                run_id,
                request_hash: dispatch.request_hash.clone(),
                state: RunState::Succeeded,
                evidence_class: EvidenceClass::Attested,
                tier: ExecutionTier::P,
                code_hash: dispatch.request.code.hash.clone(),
                input_set_hash: hash_bytes(b"remote-fixture-inputs"),
                env_hash: hash_bytes(b"remote-fixture-environment"),
                toolchain_hash: Some(hash_bytes(b"python3")),
                sandbox_profile_hash: hash_bytes(b"seatbelt"),
                backend: ExecutionBackend::Seatbelt,
                exit: ExecutionExit {
                    status: 0,
                    signal_or_trap: None,
                },
                outputs: ExecutionOutputs {
                    stdout: hash_bytes(b"42\n"),
                    stderr: hash_bytes(b""),
                    artifacts: Vec::new(),
                },
                usage: ResourceUsage::default(),
                started_at: now,
                finished_at: now,
                executing_device: ExecutingDevice {
                    key_id: String::new(),
                    sig_alg: SignatureAlgorithm::Ed25519,
                    platform: "fixture-target".into(),
                },
                grants: Vec::new(),
                component: None,
                failure: None,
                signature: None,
            };
            sign_receipt_ed25519(&mut receipt, &self.key)?;
            Ok(LocalExecutionOutcome {
                state: PeerDispatchState::Applied,
                run_id: Some(run_id),
                receipt: Some(receipt),
                failure: None,
            })
        }
    }

    struct FixtureTransport {
        target: Arc<RemoteTarget<FixtureExecutor>>,
    }

    #[async_trait]
    impl RemoteTransport for FixtureTransport {
        async fn deliver(
            &self,
            dispatch: SignedRemoteDispatch,
        ) -> Result<SignedPeerDispatchResponse> {
            self.target.receive(dispatch, Utc::now()).await
        }
    }

    #[test]
    fn origin_and_target_verify_the_complete_execution_handoff() {
        let (dispatch, enrollment, _) = crate::tests::fixture();
        let origin_dir = tempdir().unwrap();
        let target_dir = tempdir().unwrap();
        let target_key = SigningKey::from_bytes(&[12; 32]);
        let executor = Arc::new(FixtureExecutor {
            key: target_key.clone(),
        });
        let target = Arc::new(
            RemoteTarget::new(
                "endpoint-target",
                crate::DispatchQueue::open(target_dir.path()).unwrap(),
                enrollment.clone(),
                target_key,
                executor,
            )
            .unwrap(),
        );
        let origin = RemoteOrigin::new(
            crate::DispatchQueue::open(origin_dir.path()).unwrap(),
            enrollment,
            Arc::new(FixtureTransport { target }),
        );

        let peer = futures::executor::block_on(origin.dispatch(dispatch.clone(), Utc::now()))
            .expect("remote flow succeeds");
        assert_eq!(peer.state, PeerDispatchState::Applied);
        let receipt = peer
            .receipt
            .clone()
            .expect("applied response has a receipt");
        assert_eq!(receipt.request_hash, dispatch.request_hash);

        let mut applied = peer;
        applied.receipt = Some(receipt);
        let rejected = PeerDispatchRecord {
            endpoint_id: "endpoint-rejected".into(),
            state: PeerDispatchState::Rejected,
            run_id: None,
            receipt: None,
            failure: Some("policy denied execution".into()),
            updated_at: Utc::now(),
        };
        let aggregate = aggregate_records(&dispatch, [applied, rejected]).unwrap();
        assert!(aggregate.terminal);
        assert!(!aggregate.universally_applied);
    }
}
