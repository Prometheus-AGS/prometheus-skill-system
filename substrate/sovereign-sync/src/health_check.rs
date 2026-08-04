use serde::Serialize;
use std::{fmt, net::SocketAddr, path::Path, time::Duration};
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
    #[serde(skip)]
    timed_out: bool,
}

impl DaemonHealthReport {
    fn new(status: DaemonHealthKind, port: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            port,
            endpoint: format!("http://127.0.0.1:{port}/health"),
            message: message.into(),
            timed_out: false,
        }
    }

    fn timeout(status: DaemonHealthKind, port: u16, message: impl Into<String>) -> Self {
        let mut report = Self::new(status, port, message);
        report.timed_out = true;
        report
    }

    #[cfg(unix)]
    fn unix(
        status: DaemonHealthKind,
        path: &Path,
        message: impl Into<String>,
        timed_out: bool,
    ) -> Self {
        Self {
            status,
            port: 0,
            endpoint: format!("unix://{}/health", path.display()),
            message: message.into(),
            timed_out,
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.status.exit_code()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencySummary {
    pub samples: usize,
    pub warmup: usize,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
    pub maximum_ms: Option<f64>,
    pub failures: usize,
    pub timeouts: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyBudgets {
    pub p99_budget_ms: Option<f64>,
    pub max_budget_ms: Option<f64>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SampledDaemonHealthReport {
    #[serde(flatten)]
    pub health: DaemonHealthReport,
    pub latency: LatencySummary,
    pub budgets: LatencyBudgets,
}

impl SampledDaemonHealthReport {
    pub fn exit_code(&self) -> i32 {
        if self.health.exit_code() != 0 {
            self.health.exit_code()
        } else if !self.budgets.passed {
            3
        } else {
            0
        }
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
            return DaemonHealthReport::timeout(
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
            return DaemonHealthReport::timeout(
                DaemonHealthKind::Occupied,
                port,
                "port is occupied but health response timed out",
            );
        }
    }

    classify_health_response(port, &String::from_utf8_lossy(&response))
}

pub async fn sample_daemon_health(
    port: u16,
    samples: usize,
    warmup: usize,
    p99_budget_ms: Option<f64>,
    max_budget_ms: Option<f64>,
) -> SampledDaemonHealthReport {
    sample_daemon_health_with_token(port, samples, warmup, p99_budget_ms, max_budget_ms, None).await
}

pub async fn sample_daemon_health_with_token(
    port: u16,
    samples: usize,
    warmup: usize,
    p99_budget_ms: Option<f64>,
    max_budget_ms: Option<f64>,
    token: Option<&str>,
) -> SampledDaemonHealthReport {
    let samples = samples.max(1);
    let mut stream = None;
    for _ in 0..warmup {
        let report = probe_keep_alive(port, &mut stream, token).await;
        if report.status != DaemonHealthKind::Healthy {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let mut latencies = Vec::with_capacity(samples);
    let mut failures = 0;
    let mut timeouts = 0;
    let mut first_failure = None;
    let mut last_report = None;
    for _ in 0..samples {
        let started = std::time::Instant::now();
        let report = probe_keep_alive(port, &mut stream, token).await;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;
        if report.status == DaemonHealthKind::Healthy {
            latencies.push(elapsed_ms);
        } else {
            failures += 1;
            timeouts += usize::from(report.timed_out);
            first_failure.get_or_insert_with(|| report.clone());
        }
        last_report = Some(report);
    }
    latencies.sort_by(f64::total_cmp);
    let summary = LatencySummary {
        samples,
        warmup,
        p50_ms: percentile(&latencies, 0.50),
        p95_ms: percentile(&latencies, 0.95),
        p99_ms: percentile(&latencies, 0.99),
        maximum_ms: latencies.last().copied(),
        failures,
        timeouts,
    };
    let budgets = LatencyBudgets {
        p99_budget_ms,
        max_budget_ms,
        passed: p99_budget_ms
            .is_none_or(|budget| summary.p99_ms.is_some_and(|value| value <= budget))
            && max_budget_ms
                .is_none_or(|budget| summary.maximum_ms.is_some_and(|value| value <= budget)),
    };
    SampledDaemonHealthReport {
        health: first_failure.or(last_report).unwrap_or_else(|| {
            DaemonHealthReport::new(DaemonHealthKind::Missing, port, "no samples completed")
        }),
        latency: summary,
        budgets,
    }
}

#[cfg(unix)]
pub async fn sample_unix_daemon_health(
    path: &Path,
    samples: usize,
    warmup: usize,
    p99_budget_ms: Option<f64>,
    max_budget_ms: Option<f64>,
) -> SampledDaemonHealthReport {
    for _ in 0..warmup {
        let _ = probe_unix(path).await;
    }
    let samples = samples.max(1);
    let mut latencies = Vec::with_capacity(samples);
    let mut failures = 0;
    let mut timeouts = 0;
    let mut first_failure = None;
    let mut last_report = None;
    for _ in 0..samples {
        let started = std::time::Instant::now();
        let report = probe_unix(path).await;
        if report.status == DaemonHealthKind::Healthy {
            latencies.push(started.elapsed().as_secs_f64() * 1_000.0);
        } else {
            failures += 1;
            timeouts += usize::from(report.timed_out);
            first_failure.get_or_insert_with(|| report.clone());
        }
        last_report = Some(report);
    }
    latencies.sort_by(f64::total_cmp);
    let summary = LatencySummary {
        samples,
        warmup,
        p50_ms: percentile(&latencies, 0.50),
        p95_ms: percentile(&latencies, 0.95),
        p99_ms: percentile(&latencies, 0.99),
        maximum_ms: latencies.last().copied(),
        failures,
        timeouts,
    };
    let budgets = LatencyBudgets {
        p99_budget_ms,
        max_budget_ms,
        passed: p99_budget_ms
            .is_none_or(|budget| summary.p99_ms.is_some_and(|value| value <= budget))
            && max_budget_ms
                .is_none_or(|budget| summary.maximum_ms.is_some_and(|value| value <= budget)),
    };
    SampledDaemonHealthReport {
        health: first_failure.or(last_report).unwrap_or_else(|| {
            DaemonHealthReport::unix(
                DaemonHealthKind::Missing,
                path,
                "no samples completed",
                false,
            )
        }),
        latency: summary,
        budgets,
    }
}

#[cfg(unix)]
async fn probe_unix(path: &Path) -> DaemonHealthReport {
    let connect = timeout(
        Duration::from_millis(750),
        tokio::net::UnixStream::connect(path),
    )
    .await;
    let mut stream = match connect {
        Ok(Ok(stream)) => stream,
        Ok(Err(error)) => {
            return DaemonHealthReport::unix(
                DaemonHealthKind::Missing,
                path,
                format!("could not connect to sovereign-sync socket: {error}"),
                false,
            );
        }
        Err(_) => {
            return DaemonHealthReport::unix(
                DaemonHealthKind::Missing,
                path,
                "timed out connecting to sovereign-sync socket",
                true,
            );
        }
    };
    if let Err(error) = stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
    {
        return DaemonHealthReport::unix(
            DaemonHealthKind::Occupied,
            path,
            format!("socket did not accept sovereign-sync health request: {error}"),
            false,
        );
    }
    let mut response = Vec::new();
    match timeout(
        Duration::from_millis(750),
        stream.read_to_end(&mut response),
    )
    .await
    {
        Ok(Ok(_)) if response.starts_with(b"HTTP/1.1 200") => DaemonHealthReport::unix(
            DaemonHealthKind::Healthy,
            path,
            "sovereign-sync health endpoint responded successfully",
            false,
        ),
        Ok(Ok(_)) => DaemonHealthReport::unix(
            DaemonHealthKind::Occupied,
            path,
            "socket response was not sovereign-sync health",
            false,
        ),
        Ok(Err(error)) => DaemonHealthReport::unix(
            DaemonHealthKind::Occupied,
            path,
            format!("socket health response failed: {error}"),
            false,
        ),
        Err(_) => DaemonHealthReport::unix(
            DaemonHealthKind::Occupied,
            path,
            "socket health response timed out",
            true,
        ),
    }
}

async fn probe_keep_alive(
    port: u16,
    stream: &mut Option<TcpStream>,
    token: Option<&str>,
) -> DaemonHealthReport {
    if stream.is_none() {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match timeout(Duration::from_millis(750), TcpStream::connect(addr)).await {
            Ok(Ok(connected)) => *stream = Some(connected),
            Ok(Err(error)) => {
                return DaemonHealthReport::new(
                    DaemonHealthKind::Missing,
                    port,
                    format!("could not connect to sovereign-sync port: {error}"),
                );
            }
            Err(_) => {
                return DaemonHealthReport::timeout(
                    DaemonHealthKind::Missing,
                    port,
                    "timed out connecting to sovereign-sync port",
                );
            }
        }
    }

    let authorization = token.map_or_else(String::new, |token| {
        format!("Authorization: Bearer {token}\r\n")
    });
    let request = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{authorization}Connection: keep-alive\r\nAccept: application/json\r\n\r\n"
    );
    let exchange = async {
        let stream = stream.as_mut().expect("stream was connected above");
        stream.write_all(request.as_bytes()).await?;
        let mut response = Vec::with_capacity(512);
        loop {
            let mut chunk = [0_u8; 1024];
            let read = stream.read(&mut chunk).await?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "health connection closed before a complete response",
                ));
            }
            response.extend_from_slice(&chunk[..read]);
            if response_complete_len(&response).is_some_and(|length| response.len() >= length) {
                return Ok::<_, std::io::Error>(response);
            }
            if response.len() > 64 * 1024 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "health response exceeded 64 KiB",
                ));
            }
        }
    };
    match timeout(Duration::from_millis(750), exchange).await {
        Ok(Ok(response)) => classify_health_response(port, &String::from_utf8_lossy(&response)),
        Ok(Err(error)) => {
            *stream = None;
            DaemonHealthReport::new(
                DaemonHealthKind::Occupied,
                port,
                format!("health exchange failed: {error}"),
            )
        }
        Err(_) => {
            *stream = None;
            DaemonHealthReport::timeout(
                DaemonHealthKind::Occupied,
                port,
                "health exchange timed out",
            )
        }
    }
}

fn response_complete_len(response: &[u8]) -> Option<usize> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")?
        + 4;
    let headers = std::str::from_utf8(&response[..header_end]).ok()?;
    let content_length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })?;
    header_end.checked_add(content_length)
}

fn percentile(sorted: &[f64], quantile: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (quantile * sorted.len() as f64).ceil().max(1.0) as usize;
    sorted.get(rank.saturating_sub(1)).copied()
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
    use super::{detect_daemon_health, sample_daemon_health, DaemonHealthKind};
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

    async fn spawn_keep_alive_fixture() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let body = r#"{"status":"ok","service":"sovereign-sync","version":"0.1.0"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: keep-alive\r\n\r\n{body}",
                body.len()
            );
            loop {
                let mut request = Vec::new();
                loop {
                    let mut chunk = [0_u8; 1024];
                    let read = socket.read(&mut chunk).await.unwrap();
                    if read == 0 {
                        return;
                    }
                    request.extend_from_slice(&chunk[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                socket.write_all(response.as_bytes()).await.unwrap();
            }
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

    #[tokio::test]
    async fn samples_one_warm_connection_and_enforces_explicit_budgets() {
        let port = spawn_keep_alive_fixture().await;

        let report = sample_daemon_health(port, 5, 2, Some(1_000.0), Some(1_000.0)).await;

        assert_eq!(report.health.status, DaemonHealthKind::Healthy);
        assert_eq!(report.latency.samples, 5);
        assert_eq!(report.latency.warmup, 2);
        assert_eq!(report.latency.failures, 0);
        assert_eq!(report.latency.timeouts, 0);
        assert!(report.latency.p50_ms.is_some());
        assert!(report.latency.p95_ms.is_some());
        assert!(report.latency.p99_ms.is_some());
        assert!(report.latency.maximum_ms.is_some());
        assert!(report.budgets.passed);
        assert_eq!(report.exit_code(), 0);

        let strict_port = spawn_keep_alive_fixture().await;
        let failed = sample_daemon_health(strict_port, 1, 0, Some(0.0), Some(0.0)).await;
        assert!(!failed.budgets.passed);
        assert_eq!(failed.exit_code(), 3);
    }
}
