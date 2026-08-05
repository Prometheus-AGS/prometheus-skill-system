#![cfg(target_os = "macos")]

use std::{collections::BTreeMap, time::Instant};

use chrono::Utc;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorization,
    ComponentAuthorizationMode, ExecutionLimits, ExecutionProvenance, RequestedTier, RunState,
    RuntimeKind, SignatureAlgorithm, SignedExecRequest,
};
use prometheus_exec_core::{ExecutionJob, ExecutionPort};
use prometheus_exec_tier_p::{SeatbeltConfig, SeatbeltExecutor};
use uuid::Uuid;

fn job(
    code: &[u8],
    runtime: RuntimeKind,
    wall_clock_ms: u64,
    output_mb: u64,
) -> prometheus_exec_core::ValidatedExecutionJob {
    job_with_capabilities(
        code,
        runtime,
        wall_clock_ms,
        output_mb,
        CapabilityManifest::default(),
    )
}

fn job_with_capabilities(
    code: &[u8],
    runtime: RuntimeKind,
    wall_clock_ms: u64,
    output_mb: u64,
    capabilities: CapabilityManifest,
) -> prometheus_exec_core::ValidatedExecutionJob {
    job_with_limits(
        code,
        runtime,
        ExecutionLimits {
            memory_mb: 256,
            fuel: 1,
            wall_clock_ms,
            output_mb,
            stack_kb: 512,
        },
        capabilities,
    )
}

fn job_with_limits(
    code: &[u8],
    runtime: RuntimeKind,
    limits: ExecutionLimits,
    capabilities: CapabilityManifest,
) -> prometheus_exec_core::ValidatedExecutionJob {
    ExecutionJob {
        request: SignedExecRequest {
            schema_version: prometheus_exec_contracts::SCHEMA_VERSION.into(),
            request_id: Uuid::new_v4(),
            issued_at: Utc::now(),
            queued_at: None,
            validity_window_secs: 60,
            tier: RequestedTier::P,
            code: CodeIdentity {
                kind: CodeKind::Inline,
                hash: hash_bytes(code),
                runtime,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities,
            limits,
            targets: Vec::new(),
            provenance: ExecutionProvenance::default(),
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        },
        code: code.to_vec(),
        inputs: BTreeMap::new(),
        grants: Vec::new(),
    }
    .validate()
    .unwrap()
}

#[tokio::test]
async fn requested_stack_limit_is_applied_before_the_runtime_starts() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let execution = executor
        .execute(&job_with_limits(
            b"ulimit -s | tr -d ' \\n'\n",
            RuntimeKind::Bash,
            ExecutionLimits {
                stack_kb: 256,
                ..ExecutionLimits::default()
            },
            CapabilityManifest::default(),
        ))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Succeeded);
    assert_eq!(execution.stdout, b"256");
}

#[tokio::test]
async fn memory_limit_terminates_the_complete_process_group() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let code = br#"import time
payload = bytearray(96 * 1024 * 1024)
payload[0] = 1
time.sleep(30)
"#;
    let started = Instant::now();
    let execution = executor
        .execute(&job_with_limits(
            code,
            RuntimeKind::Python3,
            ExecutionLimits {
                memory_mb: 32,
                wall_clock_ms: 5_000,
                ..ExecutionLimits::default()
            },
            CapabilityManifest::default(),
        ))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Failed);
    assert!(execution
        .exit
        .signal_or_trap
        .as_deref()
        .is_some_and(|trap| trap.starts_with("memory_limit_exceeded:")));
    assert!(execution.usage.peak_mem_mb > 32);
    assert!(started.elapsed().as_secs() < 3);
}

#[tokio::test]
async fn receipt_usage_reports_observed_cpu_and_peak_memory() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let code = br#"import time
payload = bytearray(8 * 1024 * 1024)
deadline = time.monotonic() + 0.2
while time.monotonic() < deadline:
    sum(range(1000))
print(len(payload), end="")
"#;
    let execution = executor
        .execute(&job(code, RuntimeKind::Python3, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Succeeded);
    assert!(execution.usage.cpu_ms > 0);
    assert!(execution.usage.peak_mem_mb > 0);
}

#[tokio::test]
async fn real_seatbelt_run_denies_external_write_and_collects_output() {
    let forbidden = tempfile::tempdir().unwrap();
    let forbidden_path = forbidden.path().join("escape.txt");
    let code = format!(
        "printf artifact > \"$PROMETHEUS_OUTPUT_DIR/result.txt\"\n\
         if printf escaped > \"{}\" 2>/dev/null; then exit 90; fi\n\
         printf sandboxed\n",
        forbidden_path.display()
    );
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let execution = executor
        .execute(&job(code.as_bytes(), RuntimeKind::Bash, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(
        execution.state,
        RunState::Succeeded,
        "{:?}",
        execution.stderr
    );
    assert_eq!(execution.stdout, b"sandboxed");
    assert!(!forbidden_path.exists());
    assert_eq!(execution.artifacts.len(), 1);
    assert_eq!(execution.artifacts[0].path, "outputs/result.txt");
    assert_eq!(execution.artifacts[0].bytes, b"artifact");
    assert_eq!(
        execution.sandbox_profile_hash.as_str().len(),
        "sha256:".len() + 64
    );
}

#[tokio::test]
async fn timeout_terminates_the_process_group_promptly() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let started = Instant::now();

    let execution = executor
        .execute(&job(b"/bin/sleep 30\n", RuntimeKind::Bash, 150, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Failed);
    assert_eq!(
        execution.exit.signal_or_trap.as_deref(),
        Some("wall_clock_timeout")
    );
    assert!(started.elapsed().as_secs() < 3);
}

#[tokio::test]
async fn stream_output_limit_terminates_the_process_group() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let code = b"while :; do printf 0123456789abcdef; done\n";
    let started = Instant::now();

    let execution = executor
        .execute(&job(code, RuntimeKind::Bash, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Failed);
    assert_eq!(
        execution.exit.signal_or_trap.as_deref(),
        Some("output_limit_exceeded")
    );
    assert!(execution.stdout.len() <= 1024 * 1024);
    assert!(started.elapsed().as_secs() < 3);
}

#[tokio::test]
async fn real_seatbelt_executes_python_and_node() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let python = br#"import os
from pathlib import Path
Path(os.environ["PROMETHEUS_OUTPUT_DIR"], "python.txt").write_text("python")
print("python-ok", end="")
"#;
    let python_execution = executor
        .execute(&job(python, RuntimeKind::Python3, 5_000, 1))
        .await
        .unwrap();
    assert_eq!(
        python_execution.state,
        RunState::Succeeded,
        "{}",
        String::from_utf8_lossy(&python_execution.stderr)
    );
    assert_eq!(python_execution.stdout, b"python-ok");
    assert_eq!(python_execution.artifacts[0].bytes, b"python");

    let node = br#"const fs = require("fs");
fs.writeFileSync(process.env.PROMETHEUS_OUTPUT_DIR + "/node.txt", "node");
process.stdout.write("node-ok");
"#;
    let node_execution = executor
        .execute(&job(node, RuntimeKind::Node, 5_000, 1))
        .await
        .unwrap();
    assert_eq!(
        node_execution.state,
        RunState::Succeeded,
        "{}",
        String::from_utf8_lossy(&node_execution.stderr)
    );
    assert_eq!(node_execution.stdout, b"node-ok");
    assert_eq!(node_execution.artifacts[0].bytes, b"node");
}

#[tokio::test]
async fn real_seatbelt_denies_external_reads() {
    let forbidden = tempfile::tempdir().unwrap();
    let forbidden_path = forbidden.path().join("secret.txt");
    std::fs::write(&forbidden_path, b"secret").unwrap();
    let code = format!(
        "if IFS= read -r value < \"{}\" 2>/dev/null; then exit 91; fi\n\
         printf read-denied\n",
        forbidden_path.display()
    );
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let execution = executor
        .execute(&job(code.as_bytes(), RuntimeKind::Bash, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Succeeded);
    assert_eq!(execution.stdout, b"read-denied");
    assert_eq!(std::fs::read(forbidden_path).unwrap(), b"secret");
}

#[tokio::test]
async fn real_seatbelt_filters_environment_to_explicit_values() {
    let mut capabilities = CapabilityManifest::default();
    capabilities.env.read = vec!["PROMETHEUS_FIXTURE_VISIBLE".into()];
    let code = br#"import os
assert os.environ.get("PROMETHEUS_FIXTURE_VISIBLE") == "visible-value"
assert "PATH" not in os.environ
assert "VOLTA_HOME" not in os.environ
print("environment-filtered", end="")
"#;
    let config = SeatbeltConfig::detect()
        .unwrap()
        .allow_environment("PROMETHEUS_FIXTURE_VISIBLE", "visible-value")
        .unwrap();
    let executor = SeatbeltExecutor::new(config);

    let execution = executor
        .execute(&job_with_capabilities(
            code,
            RuntimeKind::Python3,
            5_000,
            1,
            capabilities,
        ))
        .await
        .unwrap();

    assert_eq!(
        execution.state,
        RunState::Succeeded,
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert_eq!(execution.stdout, b"environment-filtered");
    assert_eq!(
        execution
            .environment
            .get("PROMETHEUS_FIXTURE_VISIBLE")
            .map(String::as_str),
        Some("visible-value")
    );
    assert_eq!(execution.environment.len(), 1);
}

#[tokio::test]
async fn real_seatbelt_denies_loopback_network_connections() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let code = format!(
        r#"import socket
try:
    connection = socket.create_connection(("127.0.0.1", {port}), timeout=0.25)
except OSError:
    print("network-denied", end="")
else:
    connection.close()
    raise SystemExit(91)
"#
    );
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let execution = executor
        .execute(&job(code.as_bytes(), RuntimeKind::Python3, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(
        execution.state,
        RunState::Succeeded,
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert_eq!(execution.stdout, b"network-denied");
    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[tokio::test]
async fn artifact_output_limit_fails_without_persisting_oversized_bytes() {
    let code = br#"import os
from pathlib import Path
Path(os.environ["PROMETHEUS_OUTPUT_DIR"], "oversized.bin").write_bytes(b"x" * (2 * 1024 * 1024))
"#;
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let execution = executor
        .execute(&job(code, RuntimeKind::Python3, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Failed);
    assert_eq!(
        execution.exit.signal_or_trap.as_deref(),
        Some("artifact output limit exceeded")
    );
    assert!(execution.artifacts.is_empty());
}

#[tokio::test]
async fn symlink_artifacts_are_rejected_as_escape_attempts() {
    let code = b"/bin/ln -s /etc/passwd \"$PROMETHEUS_OUTPUT_DIR/leak\"\n";
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let execution = executor
        .execute(&job(code, RuntimeKind::Bash, 5_000, 1))
        .await
        .unwrap();

    assert_eq!(execution.state, RunState::Failed);
    assert!(execution
        .exit
        .signal_or_trap
        .as_deref()
        .is_some_and(|trap| trap.contains("unsafe file type")));
    assert!(execution.artifacts.is_empty());
}

#[tokio::test]
async fn wasm_requests_are_rejected_before_any_process_spawn() {
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());
    let code = b"not-wasm";
    let component_job = ExecutionJob {
        request: SignedExecRequest {
            schema_version: prometheus_exec_contracts::SCHEMA_VERSION.into(),
            request_id: Uuid::new_v4(),
            issued_at: Utc::now(),
            queued_at: None,
            validity_window_secs: 60,
            tier: RequestedTier::W,
            code: CodeIdentity {
                kind: CodeKind::Component,
                hash: hash_bytes(code),
                runtime: RuntimeKind::WasmComponent,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities: CapabilityManifest::default(),
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                component_authorization: Some(ComponentAuthorization {
                    mode: ComponentAuthorizationMode::HashPin,
                    world: "prometheus:component@0.1.0".into(),
                    manifest_hash: None,
                    generation_id: None,
                }),
                ..ExecutionProvenance::default()
            },
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        },
        code: code.to_vec(),
        inputs: BTreeMap::new(),
        grants: Vec::new(),
    }
    .validate()
    .unwrap();
    let error = executor.execute(&component_job).await.unwrap_err();

    assert!(error
        .to_string()
        .contains("requested tier does not permit Tier P execution"));
}

#[tokio::test]
async fn requested_network_egress_is_rejected_before_spawn() {
    let marker_root = tempfile::tempdir().unwrap();
    let marker = marker_root.path().join("must-not-exist");
    let code = format!("printf spawned > \"{}\"\n", marker.display());
    let mut capabilities = CapabilityManifest::default();
    capabilities.net.egress = vec!["https://example.invalid".into()];
    let executor = SeatbeltExecutor::new(SeatbeltConfig::detect().unwrap());

    let error = executor
        .execute(&job_with_capabilities(
            code.as_bytes(),
            RuntimeKind::Bash,
            5_000,
            1,
            capabilities,
        ))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("deny-all networking only"));
    assert!(!marker.exists());
}
