#![cfg(target_os = "macos")]

use std::{collections::BTreeMap, time::Instant};

use chrono::Utc;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, EnvironmentCapabilities,
    ExecutionLimits, ExecutionProvenance, FilesystemCapabilities, NetworkCapabilities,
    RequestedTier, RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest,
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
            capabilities: CapabilityManifest {
                fs: FilesystemCapabilities {
                    read_only: vec![".".into()],
                    read_write: vec!["outputs/".into()],
                },
                net: NetworkCapabilities::default(),
                env: EnvironmentCapabilities::default(),
                clock: true,
                random: true,
            },
            limits: ExecutionLimits {
                memory_mb: 256,
                fuel: 1,
                wall_clock_ms,
                output_mb,
                stack_kb: 512,
            },
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
