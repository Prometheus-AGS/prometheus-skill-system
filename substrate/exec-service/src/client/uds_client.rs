use std::path::Path;

use bytes::Bytes;
use http_body_util::{BodyExt as _, Full};
use hyper::{Method, Request};

const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[cfg(unix)]
pub async fn request(
    socket: &Path,
    method: Method,
    target: &str,
    body: Vec<u8>,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    request_inner(socket,method,target,body,false).await
}

#[cfg(unix)]
pub async fn request_same_uid(socket:&Path,method:Method,target:&str,body:Vec<u8>)->Result<HttpResponse,Box<dyn std::error::Error+Send+Sync>>{
    request_inner(socket,method,target,body,true).await
}

#[cfg(unix)]
async fn request_inner(socket:&Path,method:Method,target:&str,body:Vec<u8>,require_peer:bool)->Result<HttpResponse,Box<dyn std::error::Error+Send+Sync>>{
    let stream = tokio::net::UnixStream::connect(socket).await?;
    if require_peer && stream.peer_cred()?.uid() != nix::unistd::Uid::effective().as_raw() { return Err("Exec socket peer has unexpected UID".into()); }
    let io = hyper_util::rt::TokioIo::new(stream);
    let (mut sender, connection) = hyper::client::conn::http1::handshake(io).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let mut builder = Request::builder()
        .method(method)
        .uri(target)
        .header("host", "localhost");
    if !body.is_empty() {
        builder = builder.header("content-type", "application/json");
    }
    let response = sender
        .send_request(builder.body(Full::new(Bytes::from(body)))?)
        .await?;
    let status = response.status().as_u16();
    let mut body_stream = response.into_body();
    let mut bytes = Vec::new();
    while let Some(frame) = body_stream.frame().await {
        let frame = frame?;
        if let Ok(data) = frame.into_data() {
            if bytes.len().saturating_add(data.len()) > MAX_RESPONSE_BYTES {
                return Err("HTTP response exceeded 32 MiB".into());
            }
            bytes.extend_from_slice(&data);
        }
    }
    Ok(HttpResponse {
        status,
        body: bytes,
    })
}

#[cfg(not(unix))]
pub async fn request(
    _socket: &Path,
    _method: Method,
    _target: &str,
    _body: Vec<u8>,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    Err("Unix sidecar transport is unavailable on this platform".into())
}

#[cfg(not(unix))]
pub async fn request_same_uid(socket:&Path,method:Method,target:&str,body:Vec<u8>)->Result<HttpResponse,Box<dyn std::error::Error+Send+Sync>>{request(socket,method,target,body).await}
