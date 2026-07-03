use serde::Serialize;
use std::{fmt, net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonHealthKind {
    Healthy,
    Missing,
    Occupied,
}

impl DaemonHealthKind {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Healthy => 0,
            Self::Missing => 1,
            Self::Occupied => 2,
        }
    }
}

impl fmt::Display for DaemonHealthKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => f.write_str("healthy"),
            Self::Missing => f.write_str("missing"),
            Self::Occupied => f.write_str("occupied"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DaemonHealthReport {
    pub status: DaemonHealthKind,
    pub port: u16,
    pub endpoint: String,
    pub message: String,
}

impl DaemonHealthReport {
    fn new(status: DaemonHealthKind, port: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            port,
            endpoint: format!("http://127.0.0.1:{port}/health"),
            message: message.into(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.status.exit_code()
    }
}

pub async fn detect_daemon_health(port: u16) -> DaemonHealthReport {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let connect = timeout(Duration::from_millis(750), TcpStream::connect(addr)).await;

    let mut stream = match connect {
        Ok(Ok(stream)) => stream,
        Ok(Err(err)) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Missing,
                port,
                "no process is listening on sovereign-sync port",
            );
        }
        Ok(Err(err)) => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Missing,
                port,
                format!("could not connect to sovereign-sync port: {err}"),
            );
        }
        Err(_) => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Missing,
                port,
                "timed out connecting to sovereign-sync port",
            );
        }
    };

    let request = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    if let Err(err) = stream.write_all(request.as_bytes()).await {
        return DaemonHealthReport::new(
            DaemonHealthKind::Occupied,
            port,
            format!("port is occupied but did not accept sovereign-sync health request: {err}"),
        );
    }

    let mut response = Vec::new();
    match timeout(
        Duration::from_millis(750),
        stream.read_to_end(&mut response),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Occupied,
                port,
                format!("port is occupied but health response failed: {err}"),
            );
        }
        Err(_) => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Occupied,
                port,
                "port is occupied but health response timed out",
            );
        }
    }

    classify_health_response(port, &String::from_utf8_lossy(&response))
}

fn classify_health_response(port: u16, response: &str) -> DaemonHealthReport {
    let mut parts = response.splitn(2, "\r\n\r\n");
    let headers = parts.next().unwrap_or_default();
    let body = parts.next().unwrap_or_default();
    let status_line = headers.lines().next().unwrap_or_default();

    if !status_line.contains(" 200 ") {
        return DaemonHealthReport::new(
            DaemonHealthKind::Occupied,
            port,
            format!("port is occupied but /health returned {status_line}"),
        );
    }

    let value = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(value) => value,
        Err(err) => {
            return DaemonHealthReport::new(
                DaemonHealthKind::Occupied,
                port,
                format!("port is occupied but /health returned invalid JSON: {err}"),
            );
        }
    };

    let is_sovereign_sync = value.get("service").and_then(|v| v.as_str()) == Some("sovereign-sync");
    let is_ok = value.get("status").and_then(|v| v.as_str()) == Some("ok");

    if is_sovereign_sync && is_ok {
        DaemonHealthReport::new(
            DaemonHealthKind::Healthy,
            port,
            "sovereign-sync daemon is healthy",
        )
    } else {
        DaemonHealthReport::new(
            DaemonHealthKind::Occupied,
            port,
            "port is occupied by a non-sovereign-sync service",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_daemon_health, DaemonHealthKind};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    async fn spawn_fixture(response: &'static str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0_u8; 1024];
            let _ = socket.read(&mut buffer).await.unwrap();
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        port
    }

    #[tokio::test]
    async fn detects_healthy_sovereign_sync_health_response() {
        let port = spawn_fixture(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ok\",\"service\":\"sovereign-sync\",\"version\":\"0.1.0\"}",
        )
        .await;

        let report = detect_daemon_health(port).await;

        assert_eq!(report.status, DaemonHealthKind::Healthy);
        assert_eq!(report.exit_code(), 0);
    }

    #[tokio::test]
    async fn detects_missing_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let report = detect_daemon_health(port).await;

        assert_eq!(report.status, DaemonHealthKind::Missing);
        assert_eq!(report.exit_code(), 1);
    }

    #[tokio::test]
    async fn detects_occupied_port_with_non_sovereign_service() {
        let port = spawn_fixture(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ok\",\"service\":\"other\"}",
        )
        .await;

        let report = detect_daemon_health(port).await;

        assert_eq!(report.status, DaemonHealthKind::Occupied);
        assert_eq!(report.exit_code(), 2);
    }
}
