#[cfg(unix)]
use anyhow::anyhow;
use anyhow::{Context, Result};
use bytes::Bytes;
use hyper::{header::CONTENT_TYPE, Method, StatusCode};
use serde::Serialize;
use std::error::Error;
use std::io::ErrorKind;
use std::time::Duration;

#[cfg(unix)]
use http_body_util::{BodyExt, Full};
#[cfg(unix)]
use hyper::Request;
#[cfg(unix)]
use hyper_util::rt::TokioIo;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use tokio::net::UnixStream;

#[derive(Debug)]
pub(super) struct ControlResponse {
    pub(super) status: StatusCode,
    pub(super) content_type: Option<String>,
    pub(super) body: String,
}

#[derive(Debug)]
pub(super) enum TransportFailure {
    Unreachable(anyhow::Error),
    Disconnected(anyhow::Error),
    Ambiguous(anyhow::Error),
    Invalid(anyhow::Error),
}

impl TransportFailure {
    pub(super) fn into_error(self) -> anyhow::Error {
        match self {
            Self::Unreachable(error)
            | Self::Disconnected(error)
            | Self::Ambiguous(error)
            | Self::Invalid(error) => error,
        }
    }
}

#[derive(Debug)]
enum Target {
    Http(String),
    #[cfg(unix)]
    Unix(PathBuf),
}

/// Small control-plane HTTP client that prefers the managed same-user Unix
/// socket and retains the explicit TCP endpoint override for tests/operators.
pub(super) struct ControlTransport {
    http: reqwest::Client,
    target: Target,
    timeout: Duration,
}

impl ControlTransport {
    pub(super) fn new(timeout: Duration) -> Result<Self> {
        let target = match std::env::var("PROMETHEUS_CONTROL_ENDPOINT") {
            Ok(endpoint) => Target::Http(endpoint.trim_end_matches('/').to_string()),
            Err(_) => default_target(),
        };
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            target,
            timeout,
        })
    }

    #[cfg(all(test, target_family = "unix"))]
    fn for_unix_socket(socket: PathBuf, timeout: Duration) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().timeout(timeout).build()?,
            target: Target::Unix(socket),
            timeout,
        })
    }

    pub(super) async fn get(&self, path: &str) -> Result<ControlResponse> {
        self.get_classified(path)
            .await
            .map_err(TransportFailure::into_error)
    }

    pub(super) async fn get_classified(
        &self,
        path: &str,
    ) -> std::result::Result<ControlResponse, TransportFailure> {
        self.request(Method::GET, path, Bytes::new()).await
    }

    pub(super) async fn post_json<T: Serialize + ?Sized>(
        &self,
        path: &str,
        value: &T,
    ) -> std::result::Result<ControlResponse, TransportFailure> {
        let body = serde_json::to_vec(value)
            .context("serialize control-plane request")
            .map(Bytes::from)
            // A local serialization failure says nothing about daemon
            // reachability. Treat it as ambiguous so mutation callers never
            // fall back to a local commit on malformed request construction.
            .map_err(TransportFailure::Ambiguous)?;
        self.request(Method::POST, path, body).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Bytes,
    ) -> std::result::Result<ControlResponse, TransportFailure> {
        match &self.target {
            Target::Http(endpoint) => self.request_http(endpoint, method, path, body).await,
            #[cfg(unix)]
            Target::Unix(socket) => {
                match tokio::time::timeout(
                    self.timeout,
                    request_unix(socket.clone(), method, path.to_string(), body),
                )
                .await
                {
                    Ok(result) => result,
                    Err(error) => Err(TransportFailure::Ambiguous(
                        anyhow!(error).context("control-plane Unix request timed out"),
                    )),
                }
            }
        }
    }

    async fn request_http(
        &self,
        endpoint: &str,
        method: Method,
        path: &str,
        body: Bytes,
    ) -> std::result::Result<ControlResponse, TransportFailure> {
        let mut request = self.http.request(method, format!("{endpoint}{path}"));
        if !body.is_empty() {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(body);
        }
        let response = request.send().await.map_err(|error| {
            if error.is_builder() {
                TransportFailure::Invalid(error.into())
            } else if error.is_connect()
                && with_error_source(&error, |cause| {
                    cause
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| is_endpoint_unreachable(error.kind()))
                })
            {
                TransportFailure::Unreachable(error.into())
            } else {
                classify_started_request_error(error)
            }
        })?;
        let status = response.status();
        let content_type = response.headers().get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response
            .bytes()
            .await
            .map_err(|error| incomplete_response(status, error))?;
        control_response(status, content_type, body)
    }
}

fn with_error_source(
    mut error: &(dyn Error + 'static),
    predicate: impl Fn(&(dyn Error + 'static)) -> bool,
) -> bool {
    loop {
        if predicate(error) {
            return true;
        }
        let Some(source) = error.source() else {
            return false;
        };
        error = source;
    }
}

fn is_endpoint_unreachable(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::NotFound | ErrorKind::ConnectionRefused | ErrorKind::AddrNotAvailable
            | ErrorKind::NetworkUnreachable | ErrorKind::HostUnreachable
    )
}

fn classify_started_request_error(
    error: impl Error + Send + Sync + 'static,
) -> TransportFailure {
    if with_error_source(&error, |cause| {
        cause.downcast_ref::<hyper::Error>().is_some_and(hyper::Error::is_parse)
    }) {
        TransportFailure::Invalid(error.into())
    } else if with_error_source(&error, |cause| {
        cause.downcast_ref::<hyper::Error>().is_some_and(|error| {
            error.is_incomplete_message() || error.is_closed() || error.is_canceled()
        }) || cause.downcast_ref::<std::io::Error>().is_some_and(|error| {
            matches!(error.kind(), ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted
                | ErrorKind::BrokenPipe | ErrorKind::UnexpectedEof)
        })
    }) {
        TransportFailure::Disconnected(error.into())
    } else {
        TransportFailure::Ambiguous(error.into())
    }
}

fn control_response(
    status: StatusCode,
    content_type: Option<String>,
    body: Bytes,
) -> std::result::Result<ControlResponse, TransportFailure> {
    let body = String::from_utf8(body.to_vec())
        .context("control-plane response was not UTF-8")
        .map_err(TransportFailure::Invalid)?;
    Ok(ControlResponse { status, content_type, body })
}

fn incomplete_response(
    status: StatusCode,
    error: impl Error + Send + Sync + 'static,
) -> TransportFailure {
    let error = anyhow::Error::new(error)
        .context(format!("incomplete control-plane response with HTTP status {status}"));
    // A response with protocol headers already identifies a reachable endpoint.
    // Preserve a lost success response as an ambiguous submission, but never
    // treat a partial response as proof that the authority was unreachable.
    if status.is_success() || status.is_server_error() {
        TransportFailure::Ambiguous(error)
    } else {
        TransportFailure::Invalid(error)
    }
}

fn default_target() -> Target {
    #[cfg(unix)]
    {
        // An explicit socket is an operator contract, including while the
        // supervised daemon is between unlink/bind during a restart. Preserve
        // that target and let connect failure classify as unreachable instead
        // of silently switching to a TCP listener the managed service lacks.
        if let Some(socket) = std::env::var_os("SOVEREIGN_SYNC_SOCKET") {
            return Target::Unix(PathBuf::from(socket));
        }
        if let Some(socket) =
            dirs::data_local_dir().map(|root| root.join("prometheus/run/sovereign-sync.sock"))
        {
            return Target::Unix(socket);
        }
    }
    Target::Http("http://127.0.0.1:7892".into())
}

#[cfg(unix)]
struct ControlConnection(tokio::task::JoinHandle<std::result::Result<(), hyper::Error>>);

#[cfg(unix)]
impl Drop for ControlConnection {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(unix)]
async fn request_unix(
    socket: PathBuf,
    method: Method,
    path: String,
    body: Bytes,
) -> std::result::Result<ControlResponse, TransportFailure> {
    let stream = UnixStream::connect(&socket).await.map_err(|error| {
        let kind = error.kind();
        let error = anyhow!(error).context(format!("connect to control socket {}", socket.display()));
        if is_endpoint_unreachable(kind) {
            TransportFailure::Unreachable(error)
        } else {
            TransportFailure::Invalid(error)
        }
    })?;
    let (mut sender, connection) =
        hyper::client::conn::http1::handshake::<_, Full<Bytes>>(TokioIo::new(stream))
            .await
            .map_err(|error| TransportFailure::Invalid(error.into()))?;
    let _connection = ControlConnection(tokio::spawn(connection));

    let mut request = Request::builder()
        .method(method)
        .uri(&path)
        .header("host", "localhost");
    if !body.is_empty() {
        request = request.header(CONTENT_TYPE, "application/json");
    }
    let request = request
        .body(Full::new(body))
        .map_err(|error| TransportFailure::Invalid(error.into()))?;
    let response = sender
        .send_request(request)
        .await
        .map_err(classify_started_request_error)?;
    let status = response.status();
    let content_type = response.headers().get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|error| incomplete_response(status, error))?
        .to_bytes();
    control_response(status, content_type, body)
}

#[cfg(all(test, target_family = "unix"))]
mod tests {
    use super::*;
    use serde::Serializer;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn sends_get_and_post_over_the_configured_unix_socket() {
        let fixture = tempfile::tempdir().unwrap();
        let socket = fixture.path().join("control.sock");
        let listener = UnixListener::bind(&socket).unwrap();

        let server = tokio::spawn(async move {
            for expected in ["GET /status", "POST /commands"] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = vec![0; 4096];
                let read = stream.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                assert!(request.starts_with(expected), "{request}");
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 11\r\nconnection: close\r\n\r\n{\"ok\":true}",
                    )
                    .await
                    .unwrap();
            }
        });

        let client = ControlTransport::for_unix_socket(socket, Duration::from_secs(2)).unwrap();
        assert_eq!(client.get("/status").await.unwrap().status, StatusCode::OK);
        assert_eq!(
            client
                .post_json("/commands", &serde_json::json!({"command":"test"}))
                .await
                .unwrap()
                .status,
            StatusCode::OK
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn serialization_failure_is_not_classified_as_unreachable() {
        struct RejectSerialization;

        impl Serialize for RejectSerialization {
            fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                Err(serde::ser::Error::custom(
                    "intentional serialization failure",
                ))
            }
        }

        let fixture = tempfile::tempdir().unwrap();
        let client = ControlTransport::for_unix_socket(
            fixture.path().join("unused.sock"),
            Duration::from_secs(2),
        )
        .unwrap();
        let failure = client
            .post_json("/commands", &RejectSerialization)
            .await
            .unwrap_err();

        assert!(matches!(failure, TransportFailure::Ambiguous(_)));
    }
}
