//! Optional application-owned runtime, discovered beside the installed executable.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf, process::Command};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Host {
    owner: String,
    configuration: PathBuf,
}

/// Re-exec with the host environment before Tokio starts; no process-global env mutation.
pub fn launch_if_needed() -> Result<Option<i32>> {
    if is_managed() { return Ok(None); }
    let exe = std::env::current_exe()?;
    let manifest = exe.with_file_name("prometheus-host.json");
    let bytes = match std::fs::read(&manifest) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Cannot read Prometheus host manifest"),
    };
    let host: Host = serde_json::from_slice(&bytes)?;
    ensure!(host.owner == "the-boss", "Unsupported Prometheus host");
    let config: BTreeMap<String, String> = serde_json::from_slice(
        &std::fs::read(host.configuration).context("Open The Boss to restore its CLI configuration")?
    )?;
    let allowed = ["PROMETHEUS_PACK_ROOT", "PROMETHEUS_COMMAND_DIRECTORY", "PROMETHEUS_SERVICE_DIRECTORY", "PROMETHEUS_SERVICE_MODE",
        "LITER_LLM_BASE_URL", "LITER_LLM_MASTER_KEY", "LITER_LLM_CONFIG",
        "PROMETHEUS_KBD_JUDGE_MODEL", "PROMETHEUS_KBD_CRITIC_MODEL", "SURREAL_MEMORY_URL", "SURREAL_MEMORY_TOKEN"];
    let mut command = Command::new(&exe);
    command.args(std::env::args_os().skip(1)).env("PROMETHEUS_HOST_ACTIVE", "the-boss");
    command.envs(config.iter().filter(|(key, _)| allowed.contains(&key.as_str())));
    if let Some(directory) = config.get("PROMETHEUS_COMMAND_DIRECTORY") {
        let mut paths = vec![PathBuf::from(directory)];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        command.env("PATH", std::env::join_paths(paths)?);
    }
    Ok(Some(command.status().context("Cannot start the bundled Prometheus CLI")?.code().unwrap_or(1)))
}

pub fn is_managed() -> bool {
    std::env::var("PROMETHEUS_HOST_ACTIVE").is_ok_and(|value| value == "the-boss")
}

pub fn run_mini(script: &str, args: &[&str]) -> Result<()> {
    let root = PathBuf::from(std::env::var_os("PROMETHEUS_PACK_ROOT").context("Mini payload is not configured")?);
    let node = std::env::current_exe()?.with_file_name(format!("node{}", std::env::consts::EXE_SUFFIX));
    let status = Command::new(node).arg(root.join("scripts").join(script)).args(args)
        .status().context("Cannot start the packaged mini command")?;
    ensure!(status.success(), "Mini command failed with exit code {:?}", status.code());
    Ok(())
}

pub fn doctor(options: &crate::commands::doctor::DoctorOptions) -> Result<()> {
    ensure!(!options.refresh, "The Boss owns bundled updates. Use its application updater and Settings > Prometheus.");
    ensure!(options.check.is_none() && options.exclude.is_empty(), "The mini doctor reports its full JSON-lines inventory; use Settings > Prometheus for individual operations.");
    if options.fix {
        if options.dry_run || !options.yes {
            println!("Available repair: copy-skills. Run prometheus doctor --fix --yes to apply it; user-owned skills and full-pack installations are preserved.");
            return Ok(());
        }
        return run_mini("doctor.mjs", &["--fix", "copy-skills"]);
    }
    run_mini("doctor.mjs", if options.json { &[] } else { &["--human"] })
}

pub fn setup(full: bool, check: bool, dry_run: bool, rebuild: bool) -> Result<()> {
    ensure!(!rebuild, "The Boss supplies prebuilt tools. Update the application instead of rebuilding installed binaries.");
    if !full || check { return run_mini("doctor.mjs", &["--human"]); }
    ensure!(std::env::var("PROMETHEUS_SERVICE_MODE").as_deref() == Ok("managed"),
        "External services are configured in The Boss; their lifecycle belongs to their operator.");
    let directory = std::env::var("PROMETHEUS_SERVICE_DIRECTORY").context("Configure services in The Boss first")?;
    ensure!(PathBuf::from(&directory).join(".env").is_file(), "Use The Boss Settings > Prometheus > Setup and Start to initialize credentials first.");
    if dry_run {
        println!("Would start the existing the-boss-prometheus Compose project; volumes are retained.");
        return Ok(());
    }
    run_mini("services.mjs", &["up", "--directory", &directory])
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let filename = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(&filename)).find(|candidate| candidate.is_file())
}
