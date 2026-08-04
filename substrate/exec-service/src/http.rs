use std::{collections::BTreeMap, convert::Infallible, sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::{
        rejection::{JsonRejection, PathRejection, QueryRejection},
        Path as AxumPath, Query, State,
    },
    http::{header, HeaderValue, StatusCode},
    response::{
        sse::{Event as SseEvent, KeepAlive},
        IntoResponse, Response, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{Digest, ExecutionReceipt, RunState, SignedExecRequest};
use prometheus_exec_core::{ArtifactStore, CasError};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{ExecutionService, ExecutionServiceError, RunEvent, RunLedgerError, RunRecord};

const STATE_ACCESS_BUDGET: Duration = Duration::from_millis(100);
const SSE_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadinessStatus {
    Initializing,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemReadiness {
    pub status: ReadinessStatus,
    pub detail: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSnapshot {
    pub ready: bool,
    pub subsystems: BTreeMap<String, SubsystemReadiness>,
}

#[derive(Clone, Debug)]
pub struct SidecarState {
    service: Arc<RwLock<Option<Arc<ExecutionService>>>>,
    artifacts: Arc<RwLock<Option<Arc<ArtifactStore>>>>,
    readiness: Arc<RwLock<BTreeMap<String, SubsystemReadiness>>>,
}

impl Default for SidecarState {
    fn default() -> Self {
        Self::new()
    }
}

impl SidecarState {
    pub fn new() -> Self {
        let now = Utc::now();
        let subsystems = BTreeMap::from([
            (
                "artifact-store".into(),
                SubsystemReadiness {
                    status: ReadinessStatus::Initializing,
                    detail: "artifact store has not been installed".into(),
                    updated_at: now,
                },
            ),
            (
                "durable-service".into(),
                SubsystemReadiness {
                    status: ReadinessStatus::Initializing,
                    detail: "durable execution service has not been installed".into(),
                    updated_at: now,
                },
            ),
        ]);
        Self {
            service: Arc::new(RwLock::new(None)),
            artifacts: Arc::new(RwLock::new(None)),
            readiness: Arc::new(RwLock::new(subsystems)),
        }
    }

    pub async fn install(&self, service: Arc<ExecutionService>, artifacts: Arc<ArtifactStore>) {
        *self.service.write().await = Some(service);
        *self.artifacts.write().await = Some(artifacts);
        self.set_readiness(
            "durable-service",
            ReadinessStatus::Ready,
            "run ledger and event log are ready",
        )
        .await;
        self.set_readiness(
            "artifact-store",
            ReadinessStatus::Ready,
            "content-addressed artifact store is ready",
        )
        .await;
    }

    pub async fn set_readiness(
        &self,
        subsystem: impl Into<String>,
        status: ReadinessStatus,
        detail: impl Into<String>,
    ) {
        self.readiness.write().await.insert(
            subsystem.into(),
            SubsystemReadiness {
                status,
                detail: detail.into(),
                updated_at: Utc::now(),
            },
        );
    }

    pub async fn readiness(&self) -> Result<ReadinessSnapshot, ApiErrorEnvelope> {
        let subsystems = tokio::time::timeout(STATE_ACCESS_BUDGET, self.readiness.read())
            .await
            .map_err(|_| {
                ApiErrorEnvelope::unavailable(
                    "readiness_timeout",
                    "readiness state exceeded its 100 ms access budget",
                )
            })?
            .clone();
        let ready = !subsystems.is_empty()
            && subsystems
                .values()
                .all(|check| check.status == ReadinessStatus::Ready);
        Ok(ReadinessSnapshot { ready, subsystems })
    }

    async fn service(&self) -> Result<Arc<ExecutionService>, ApiErrorEnvelope> {
        tokio::time::timeout(STATE_ACCESS_BUDGET, self.service.read())
            .await
            .map_err(|_| {
                ApiErrorEnvelope::unavailable(
                    "service_timeout",
                    "durable service state exceeded its 100 ms access budget",
                )
            })?
            .clone()
            .ok_or_else(|| {
                ApiErrorEnvelope::unavailable(
                    "service_initializing",
                    "durable execution service is still initializing",
                )
            })
    }

    async fn artifacts(&self) -> Result<Arc<ArtifactStore>, ApiErrorEnvelope> {
        tokio::time::timeout(STATE_ACCESS_BUDGET, self.artifacts.read())
            .await
            .map_err(|_| {
                ApiErrorEnvelope::unavailable(
                    "artifact_store_timeout",
                    "artifact store state exceeded its 100 ms access budget",
                )
            })?
            .clone()
            .ok_or_else(|| {
                ApiErrorEnvelope::unavailable(
                    "artifact_store_initializing",
                    "artifact store is still initializing",
                )
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorDetail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

impl ApiErrorEnvelope {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ApiErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }

    fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, message)
    }

    fn response(self, status: StatusCode) -> Response {
        (status, Json(self)).into_response()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResponse {
    pub run_id: Uuid,
    pub request_id: Uuid,
    pub request_hash: Digest,
    pub state: RunState,
    pub replayed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ExecutionReceipt>,
}

impl RunResponse {
    fn from_record(record: RunRecord, replayed: bool) -> Self {
        Self {
            run_id: record.run_id,
            request_id: record.request_id,
            request_hash: record.request_hash,
            state: record.state,
            replayed,
            receipt: record.terminal.map(|terminal| terminal.receipt),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct EventQuery {
    #[serde(default)]
    after: u64,
}

pub fn build_api_router(state: SidecarState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v2/exec/runs", post(create_run))
        .route("/api/v2/exec/runs/{run_id}", get(get_run))
        .route("/api/v2/exec/runs/{run_id}/events", get(get_run_events))
        .route("/api/v2/exec/receipts/{run_id}", get(get_receipt))
        .route("/api/v2/exec/artifacts/{digest}", get(get_artifact))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "prometheus-exec",
        "version": crate::VERSION,
    }))
}

async fn ready(State(state): State<SidecarState>) -> Response {
    match state.readiness().await {
        Ok(snapshot) if snapshot.ready => (StatusCode::OK, Json(snapshot)).into_response(),
        Ok(snapshot) => (StatusCode::SERVICE_UNAVAILABLE, Json(snapshot)).into_response(),
        Err(error) => error.response(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn create_run(
    State(state): State<SidecarState>,
    payload: Result<Json<SignedExecRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(payload) => payload,
        Err(error) => {
            return ApiErrorEnvelope::new("invalid_json", error.body_text())
                .response(StatusCode::UNPROCESSABLE_ENTITY)
        }
    };
    let service = match state.service().await {
        Ok(service) => service,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    let artifacts = match state.artifacts().await {
        Ok(artifacts) => artifacts,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    if let Err(error) = artifacts.transfer_upload_to_request(&request) {
        return ApiErrorEnvelope::new("artifact_unavailable", error.to_string())
            .response(StatusCode::SERVICE_UNAVAILABLE);
    }
    match service.submit(request.clone()) {
        Ok(result) => {
            if result.record.state.is_terminal() {
                if let Err(error) = artifacts.release_request(&request) {
                    eprintln!(
                        "prometheus-exec: terminal replay request-pin cleanup failed for {}: {error}",
                        request.request_id
                    );
                }
            }
            let status = if result.replayed {
                StatusCode::OK
            } else {
                StatusCode::ACCEPTED
            };
            (
                status,
                Json(RunResponse::from_record(result.record, result.replayed)),
            )
                .into_response()
        }
        Err(error) => {
            if let Err(release_error) = artifacts.release_request(&request) {
                eprintln!(
                    "prometheus-exec: request-pin rollback failed for {}: {release_error}",
                    request.request_id
                );
            }
            map_service_error(error)
        }
    }
}

async fn get_run(
    State(state): State<SidecarState>,
    path: Result<AxumPath<Uuid>, PathRejection>,
) -> Response {
    let run_id = match parse_uuid_path(path) {
        Ok(run_id) => run_id,
        Err(error) => return error.response(StatusCode::BAD_REQUEST),
    };
    let service = match state.service().await {
        Ok(service) => service,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    match service.run(run_id) {
        Ok(Some(record)) => Json(RunResponse::from_record(record, false)).into_response(),
        Ok(None) => ApiErrorEnvelope::new("run_not_found", format!("run {run_id} was not found"))
            .response(StatusCode::NOT_FOUND),
        Err(error) => map_service_error(error),
    }
}

async fn get_receipt(
    State(state): State<SidecarState>,
    path: Result<AxumPath<Uuid>, PathRejection>,
) -> Response {
    let run_id = match parse_uuid_path(path) {
        Ok(run_id) => run_id,
        Err(error) => return error.response(StatusCode::BAD_REQUEST),
    };
    let service = match state.service().await {
        Ok(service) => service,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    match service.receipt(run_id) {
        Ok(Some(receipt)) => Json(receipt).into_response(),
        Ok(None) => ApiErrorEnvelope::new(
            "receipt_not_found",
            format!("run {run_id} has no terminal receipt"),
        )
        .response(StatusCode::NOT_FOUND),
        Err(error) => map_service_error(error),
    }
}

async fn get_run_events(
    State(state): State<SidecarState>,
    path: Result<AxumPath<Uuid>, PathRejection>,
    query: Result<Query<EventQuery>, QueryRejection>,
) -> Response {
    let run_id = match parse_uuid_path(path) {
        Ok(run_id) => run_id,
        Err(error) => return error.response(StatusCode::BAD_REQUEST),
    };
    let after = match query {
        Ok(Query(query)) => query.after,
        Err(error) => {
            return ApiErrorEnvelope::new("invalid_query", error.body_text())
                .response(StatusCode::BAD_REQUEST)
        }
    };
    let service = match state.service().await {
        Ok(service) => service,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    if let Err(error) = service.events_after(run_id, after) {
        return map_service_error(error);
    }
    let events = async_stream::stream! {
        let mut cursor = after;
        'poll: loop {
            let reader = service.clone();
            let loaded = tokio::task::spawn_blocking(move || reader.events_after(run_id, cursor)).await;
            let loaded = match loaded {
                Ok(Ok(events)) => events,
                Ok(Err(error)) => {
                    yield Ok::<_, Infallible>(sse_error("event_read_failed", &error.to_string()));
                    break;
                }
                Err(error) => {
                    yield Ok::<_, Infallible>(sse_error("event_reader_failed", &error.to_string()));
                    break;
                }
            };
            if loaded.is_empty() {
                let reader = service.clone();
                let terminal = match tokio::task::spawn_blocking(move || reader.run(run_id)).await {
                    Ok(Ok(Some(record))) => record.state.is_terminal(),
                    Ok(Ok(None)) => {
                        yield Ok::<_, Infallible>(sse_error("run_not_found", "run disappeared while streaming"));
                        break;
                    }
                    Ok(Err(error)) => {
                        yield Ok::<_, Infallible>(sse_error("run_read_failed", &error.to_string()));
                        break;
                    }
                    Err(error) => {
                        yield Ok::<_, Infallible>(sse_error("run_reader_failed", &error.to_string()));
                        break;
                    }
                };
                if terminal {
                    break;
                }
                tokio::time::sleep(SSE_POLL_INTERVAL).await;
                continue;
            }
            for event in loaded {
                cursor = event.sequence;
                let terminal = matches!(&event.data, crate::RunEventData::Completed { .. });
                match encode_sse_event(&event) {
                    Ok(encoded) => yield Ok::<_, Infallible>(encoded),
                    Err(error) => {
                        yield Ok::<_, Infallible>(sse_error("event_serialization", &error.to_string()));
                        break 'poll;
                    }
                }
                if terminal {
                    break 'poll;
                }
            }
        }
    };
    Sse::new(events)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("prometheus-exec"),
        )
        .into_response()
}

async fn get_artifact(
    State(state): State<SidecarState>,
    path: Result<AxumPath<String>, PathRejection>,
) -> Response {
    let encoded = match path {
        Ok(AxumPath(encoded)) => encoded,
        Err(error) => {
            return ApiErrorEnvelope::new("invalid_path", error.body_text())
                .response(StatusCode::BAD_REQUEST)
        }
    };
    let digest = match Digest::parse(encoded) {
        Ok(digest) => digest,
        Err(error) => {
            return ApiErrorEnvelope::new("invalid_digest", error.to_string())
                .response(StatusCode::BAD_REQUEST)
        }
    };
    let artifacts = match state.artifacts().await {
        Ok(artifacts) => artifacts,
        Err(error) => return error.response(StatusCode::SERVICE_UNAVAILABLE),
    };
    match artifacts.get(&digest) {
        Ok(bytes) => {
            let mut response = Body::from(bytes).into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            if let Ok(value) = HeaderValue::from_str(&format!("\"{digest}\"")) {
                response.headers_mut().insert(header::ETAG, value);
            }
            response
        }
        Err(CasError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            ApiErrorEnvelope::new(
                "artifact_not_found",
                format!("artifact {digest} was not found"),
            )
            .response(StatusCode::NOT_FOUND)
        }
        Err(error) => ApiErrorEnvelope::new("artifact_unavailable", error.to_string())
            .response(StatusCode::SERVICE_UNAVAILABLE),
    }
}

fn parse_uuid_path(path: Result<AxumPath<Uuid>, PathRejection>) -> Result<Uuid, ApiErrorEnvelope> {
    path.map(|AxumPath(run_id)| run_id)
        .map_err(|error| ApiErrorEnvelope::new("invalid_run_id", error.body_text()))
}

fn map_service_error(error: ExecutionServiceError) -> Response {
    match error {
        ExecutionServiceError::Ledger(RunLedgerError::RequestHashConflict { .. }) => {
            ApiErrorEnvelope::new("request_hash_conflict", error.to_string())
                .response(StatusCode::CONFLICT)
        }
        ExecutionServiceError::Ledger(RunLedgerError::Contract(_)) => {
            ApiErrorEnvelope::new("invalid_request", error.to_string())
                .response(StatusCode::BAD_REQUEST)
        }
        ExecutionServiceError::RunNotFound(_) => {
            ApiErrorEnvelope::new("run_not_found", error.to_string())
                .response(StatusCode::NOT_FOUND)
        }
        _ => ApiErrorEnvelope::new("service_unavailable", error.to_string())
            .response(StatusCode::SERVICE_UNAVAILABLE),
    }
}

fn event_name(event: &RunEvent) -> &'static str {
    match &event.data {
        crate::RunEventData::Accepted { .. } => "accepted",
        crate::RunEventData::GrantPending { .. } => "grant-pending",
        crate::RunEventData::Started => "started",
        crate::RunEventData::Stdout { .. } => "stdout",
        crate::RunEventData::Stderr { .. } => "stderr",
        crate::RunEventData::Progress { .. } => "progress",
        crate::RunEventData::Completed { .. } => "completed",
    }
}

fn encode_sse_event(event: &RunEvent) -> Result<SseEvent, serde_json::Error> {
    Ok(SseEvent::default()
        .id(event.sequence.to_string())
        .event(event_name(event))
        .data(serde_json::to_string(event)?))
}

fn sse_error(code: &str, message: &str) -> SseEvent {
    let envelope = ApiErrorEnvelope::new(code, message);
    SseEvent::default()
        .event("error")
        .data(serde_json::to_string(&envelope).unwrap_or_else(|_| {
            "{\"error\":{\"code\":\"event_error\",\"message\":\"event stream failed\"}}".into()
        }))
}

#[cfg(unix)]
mod uds {
    use std::{
        fs::{self, File},
        io,
        os::unix::fs::{FileTypeExt as _, PermissionsExt as _},
        path::{Path, PathBuf},
    };

    use axum::{
        extract::{ConnectInfo, Request},
        http::StatusCode,
        middleware::{self, Next},
        response::Response,
        serve::IncomingStream,
    };
    use thiserror::Error;
    use tokio::{net::UnixListener, sync::oneshot, task::JoinHandle};
    use uuid::Uuid;

    use super::{build_api_router, ApiErrorEnvelope, SidecarState};

    #[derive(Clone, Debug)]
    struct UdsPeer {
        uid: Option<u32>,
    }

    impl axum::extract::connect_info::Connected<IncomingStream<'_, UnixListener>> for UdsPeer {
        fn connect_info(stream: IncomingStream<'_, UnixListener>) -> Self {
            Self {
                uid: stream
                    .io()
                    .peer_cred()
                    .ok()
                    .map(|credentials| credentials.uid()),
            }
        }
    }

    pub fn peer_is_same_user(expected_uid: u32, peer_uid: Option<u32>) -> bool {
        peer_uid == Some(expected_uid)
    }

    async fn require_same_uid(
        ConnectInfo(peer): ConnectInfo<UdsPeer>,
        request: Request,
        next: Next,
    ) -> Response {
        let expected = nix::unistd::geteuid().as_raw();
        if !peer_is_same_user(expected, peer.uid) {
            return ApiErrorEnvelope::new(
                "peer_uid_forbidden",
                "Unix socket peer credentials do not match the daemon UID",
            )
            .response(StatusCode::FORBIDDEN);
        }
        next.run(request).await
    }

    #[derive(Debug, Error)]
    pub enum UdsSidecarError {
        #[error("Unix sidecar I/O failed at {path}: {source}")]
        Io {
            path: PathBuf,
            #[source]
            source: io::Error,
        },
        #[error("Unix socket path is unsafe: {0}")]
        UnsafeSocket(PathBuf),
        #[error("Unix socket is already active: {0}")]
        SocketInUse(PathBuf),
        #[error("Unix sidecar task failed: {0}")]
        Join(#[from] tokio::task::JoinError),
    }

    pub struct UdsSidecar {
        socket_path: PathBuf,
        state: SidecarState,
        shutdown: Option<oneshot::Sender<()>>,
        task: JoinHandle<Result<(), UdsSidecarError>>,
    }

    impl UdsSidecar {
        pub async fn start(path: impl Into<PathBuf>) -> Result<Self, UdsSidecarError> {
            let socket_path = path.into();
            let listener = bind_unix_atomic(&socket_path).await?;
            let state = SidecarState::new();
            let router =
                build_api_router(state.clone()).layer(middleware::from_fn(require_same_uid));
            let (shutdown, shutdown_rx) = oneshot::channel();
            let task = tokio::spawn(async move {
                axum::serve(
                    listener,
                    router.into_make_service_with_connect_info::<UdsPeer>(),
                )
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .map_err(|source| UdsSidecarError::Io {
                    path: PathBuf::from("<unix-listener>"),
                    source,
                })
            });
            Ok(Self {
                socket_path,
                state,
                shutdown: Some(shutdown),
                task,
            })
        }

        pub fn socket_path(&self) -> &Path {
            &self.socket_path
        }

        pub fn state(&self) -> &SidecarState {
            &self.state
        }

        pub async fn shutdown(mut self) -> Result<(), UdsSidecarError> {
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(());
            }
            self.task.await??;
            remove_socket_if_present(&self.socket_path)?;
            Ok(())
        }
    }

    async fn bind_unix_atomic(path: &Path) -> Result<UnixListener, UdsSidecarError> {
        let parent = path
            .parent()
            .ok_or_else(|| UdsSidecarError::UnsafeSocket(path.to_path_buf()))?;
        fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(parent, source))?;
        if path.exists() {
            let metadata = fs::symlink_metadata(path).map_err(|source| io_error(path, source))?;
            if metadata.file_type().is_symlink() || !metadata.file_type().is_socket() {
                return Err(UdsSidecarError::UnsafeSocket(path.to_path_buf()));
            }
            if tokio::net::UnixStream::connect(path).await.is_ok() {
                return Err(UdsSidecarError::SocketInUse(path.to_path_buf()));
            }
            fs::remove_file(path).map_err(|source| io_error(path, source))?;
        }
        let nonce = Uuid::new_v4().simple().to_string();
        let temporary = parent.join(format!(".e-{}.sock", &nonce[..8]));
        let listener =
            UnixListener::bind(&temporary).map_err(|source| io_error(&temporary, source))?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .map_err(|source| io_error(&temporary, source))?;
        if let Err(source) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(io_error(path, source));
        }
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|source| io_error(parent, source))?;
        Ok(listener)
    }

    fn remove_socket_if_present(path: &Path) -> Result<(), UdsSidecarError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_socket() => {
                fs::remove_file(path).map_err(|source| io_error(path, source))
            }
            Ok(_) => Err(UdsSidecarError::UnsafeSocket(path.to_path_buf())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(io_error(path, source)),
        }
    }

    fn io_error(path: impl AsRef<Path>, source: io::Error) -> UdsSidecarError {
        UdsSidecarError::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

#[cfg(unix)]
pub use uds::{peer_is_same_user, UdsSidecar, UdsSidecarError};
