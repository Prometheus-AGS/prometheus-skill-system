#[cfg(target_os = "macos")]
mod macos {
    use kbd_runtime::{EventKind, Runtime};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        config: PathBuf,
        project: PathBuf,
        data: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().expect("create authority fixture");
            let config = temp.path().join("config");
            let project = temp.path().join("project");
            let data = temp.path().join("data");
            fs::create_dir_all(project.join(".kbd-orchestrator")).unwrap();
            fs::create_dir_all(&config).unwrap();
            Self {
                _temp: temp,
                config,
                project,
                data,
            }
        }

        fn command(&self, args: &[&str], explicit_key: Option<&Path>) -> Output {
            let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
                .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
            let mut command = Command::new(binary);
            command
                .current_dir(&self.project)
                .env("XDG_CONFIG_HOME", &self.config)
                .env("PROMETHEUS_DATA_DIR", &self.data)
                .env("PROMETHEUS_DEVICE_ID", "kbd-authority-integration")
                .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:1")
                .env("PROMETHEUS_HARNESS", "kbd-device-authority-integration")
                .args(["kbd", "--path", self.project.to_str().unwrap()])
                .args(args);
            if let Some(path) = explicit_key {
                command.env("PROMETHEUS_DEVICE_KEY_FILE", path);
            }
            command.output().expect("run prometheus CLI")
        }

        fn runtime(&self) -> (Runtime, String) {
            let manifest: serde_json::Value = serde_json::from_slice(
                &fs::read(self.project.join(".prometheus/project.json")).unwrap(),
            )
            .unwrap();
            let project_id = manifest["projectId"].as_str().unwrap().to_string();
            (
                Runtime::open_registered_at(&self.project, &self.data, &project_id).unwrap(),
                project_id,
            )
        }
    }

    struct KeychainCleanup(String);

    impl Drop for KeychainCleanup {
        fn drop(&mut self) {
            let _ = Command::new("security")
                .args([
                    "delete-generic-password",
                    "-s",
                    "prometheus-kbd-device",
                    "-a",
                    &self.0,
                ])
                .output();
        }
    }

    fn success(output: &Output) {
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn enrolled_keychain_operator_wins_over_later_implicit_managed_key() {
        let fixture = Fixture::new();
        success(&fixture.command(
            &[
                "phase",
                "create",
                "--command-id",
                "bootstrap-phase",
                "--id",
                "bootstrap",
                "--title",
                "Bootstrap",
            ],
            None,
        ));

        let (runtime, project_id) = fixture.runtime();
        let _cleanup = KeychainCleanup(format!("{project_id}:kbd-authority-integration"));
        let before = runtime.events().unwrap();
        assert_eq!(before.len(), 2);
        let enrolled_operator = before[0].signer_key_id.clone().unwrap();

        let key_runtime = Runtime::open(fixture._temp.path().join("managed-key-source"));
        let managed = key_runtime.device_signer().unwrap();
        assert_ne!(managed.key_id(), enrolled_operator);
        let managed_path = fixture.config.join("sovereign-sync/device-key.json");
        fs::create_dir_all(managed_path.parent().unwrap()).unwrap();
        fs::copy(
            key_runtime.runtime_root().join("device-key.json"),
            &managed_path,
        )
        .unwrap();

        success(&fixture.command(
            &[
                "phase",
                "create",
                "--command-id",
                "second-phase",
                "--id",
                "second",
                "--title",
                "Second",
            ],
            None,
        ));

        let after = runtime.events().unwrap();
        assert_eq!(after.len(), before.len() + 1);
        assert_eq!(
            after.last().unwrap().signer_key_id.as_deref(),
            Some(enrolled_operator.as_str())
        );
        assert_eq!(
            after
                .iter()
                .filter(|event| matches!(event.kind, EventKind::DeviceEnrolled { .. }))
                .count(),
            0
        );

        let rejected = fixture.command(
            &[
                "phase",
                "create",
                "--command-id",
                "explicit-wrong-key",
                "--id",
                "must-not-exist",
                "--title",
                "Must not exist",
            ],
            Some(&managed_path),
        );
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr)
            .contains("explicit PROMETHEUS_DEVICE_KEY_FILE signer"));
        assert_eq!(runtime.events().unwrap().len(), after.len());
    }
}

mod portable {
    use kbd_runtime::{Actor, EventKind, Phase, Runtime, RuntimeError, WorkStatus};
    use std::collections::BTreeMap;
    use std::fs;
    use std::process::Command;

    fn phase(title: &str) -> Phase {
        Phase {
            id: "phase-1".into(),
            slug: "phase-1".into(),
            title: title.into(),
            parent_phase_id: None,
            status: WorkStatus::Pending,
            stages: BTreeMap::new(),
            changes: BTreeMap::new(),
            legacy_completion_baseline: None,
            legacy_read_only: false,
        }
    }

    #[test]
    fn replay_ignores_tampered_checkpoint_and_compaction_reconstructs_without_caches() {
        let fixture = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(fixture.path().join("runtime"));
        runtime
            .initialize(
                "project-a",
                "run-a",
                Actor::operator("operator", "integration"),
            )
            .unwrap();
        runtime
            .append(
                Actor::operator("operator", "integration"),
                1,
                EventKind::PhaseDefined {
                    phase: phase("authoritative"),
                },
            )
            .unwrap();

        let pointer: serde_json::Value = serde_json::from_slice(
            &fs::read(runtime.runtime_root().join("checkpoints/current.json")).unwrap(),
        )
        .unwrap();
        let checkpoint = runtime
            .runtime_root()
            .join("checkpoints")
            .join(pointer["checkpoint"].as_str().unwrap());
        let mut body: serde_json::Value =
            serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
        body["state"]["phases"]["phase-1"]["title"] = "tampered".into();
        fs::write(&checkpoint, serde_json::to_vec_pretty(&body).unwrap()).unwrap();
        assert_eq!(
            runtime.replay_authority().unwrap().phases["phase-1"].title,
            "authoritative"
        );

        fs::remove_dir_all(runtime.runtime_root().join("checkpoints")).unwrap();
        runtime.compact_journal(1).unwrap().unwrap();
        let reopened = Runtime::open(fixture.path().join("runtime"));
        let replayed = reopened.replay_authority().unwrap();
        assert_eq!(replayed.revision, 2);
        assert_eq!(replayed.phases["phase-1"].title, "authoritative");
    }

    #[test]
    fn bare_option_three_replica_cannot_originate_or_create_credentials() {
        let fixture = tempfile::tempdir().unwrap();
        let project = fixture.path().join("bare.git");
        let data = fixture.path().join("data");
        let initialized = Command::new("git")
            .args(["init", "--bare", project.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(initialized.status.success());
        let runtime = Runtime::open_canonical_at(&project, &data).unwrap();
        let events_before = fs::read(runtime.events_path()).ok();
        let checkpoint_before = fs::read_dir(runtime.runtime_root().join("checkpoints")).ok();

        let error = runtime
            .initialize(
                "project-a",
                "run-a",
                Actor::operator("operator", "integration"),
            )
            .unwrap_err();
        assert!(matches!(error, RuntimeError::ReplicaReadOnly { .. }));
        assert_eq!(fs::read(runtime.events_path()).ok(), events_before);
        assert_eq!(
            fs::read_dir(runtime.runtime_root().join("checkpoints")).is_ok(),
            checkpoint_before.is_some()
        );
        assert!(!runtime.runtime_root().join("device-key.json").exists());
    }
}
