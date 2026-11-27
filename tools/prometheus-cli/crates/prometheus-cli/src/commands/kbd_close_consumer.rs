//! Optional, operator-installed consumer of durable terminal phase closures.

use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use walkdir::WalkDir;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Registration {
    schema: String,
    project_root: PathBuf,
    project_id: String,
    node_path: PathBuf,
    node_sha256: String,
    package_root: PathBuf,
    package_sha256: String,
    entry_path: PathBuf,
    entry_sha256: String,
}

pub(super) async fn notify(project: &Path, project_id: &str, phase: &str, key: &str) {
    let report = match deliver(project, project_id, phase, key).await {
        Ok(false) => return,
        Ok(true) => json!({"status": "acknowledged"}),
        Err(error) => json!({
            "status": "reconcile_required",
            "diagnostic": format!("{error:#}"),
            "recovery": "Inspect the consumer's records, then reconcile this closure key or replay the identical close."
        }),
    };
    eprintln!("{}", json!({
        "schema": "kbd.close-delivery/v1", "projectId": project_id,
        "phaseId": phase, "closureKey": key, "delivery": report
    }));
}

async fn deliver(project: &Path, project_id: &str, phase: &str, key: &str) -> Result<bool> {
    let home = dirs::home_dir().context("cannot locate operator home")?.canonicalize()?;
    let project = project.canonicalize()?;
    let root_text = project.to_str().context("project root must be UTF-8")?;
    let scope = format!("{:x}", Sha256::digest(root_text.as_bytes()));
    let path = home.join(".prometheus/kbd/close-consumers").join(format!("{scope}.json"));
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };
    ensure!(!path.starts_with(&project) && meta.is_file() && meta.len() <= 16384,
            "registration must be a small operator file outside the project");
    ensure!(path.canonicalize()? == path, "registration path must not traverse symlinks");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let owner = fs::metadata(&home)?.uid();
        for ancestor in path.ancestors().take_while(|p| p.starts_with(&home)) {
            let meta = fs::metadata(ancestor)?;
            ensure!(meta.uid() == owner && meta.mode() & 0o022 == 0,
                    "operator registration and directories must be owner-controlled");
        }
    }
    let reg: Registration = serde_json::from_reader(File::open(&path)?)
        .with_context(|| format!("invalid registration {}", path.display()))?;
    ensure!(reg.schema == "kbd.close-consumer/v1" && reg.project_root == project
        && reg.project_id == project_id, "registration scope or version mismatch");
    let install = InstallTrust::from_home(&home)?;
    for target in [&reg.package_root, &reg.node_path, &reg.entry_path] {
        ensure!(target.is_absolute() && !target.starts_with(&project)
            && target.canonicalize()? == *target, "expected canonical operator-installed path");
        for ancestor in target.ancestors() {
            install.check(ancestor, !ancestor.starts_with(&reg.package_root) && ancestor != target)?;
        }
    }
    ensure!(!project.starts_with(&reg.package_root) && reg.package_root.is_dir()
        && reg.entry_path.starts_with(&reg.package_root)
        && reg.entry_path.extension().is_some_and(|ext| ext == "mjs"),
        "entry must be an installed .mjs inside a separate consumer package");
    ensure!(file_hash(&reg.node_path)? == reg.node_sha256
        && file_hash(&reg.entry_path)? == reg.entry_sha256
        && package_hash(&reg.package_root, &install)? == reg.package_sha256,
        "registered installation digest changed; inspect installation before renewing trust");

    let mut cmd = Command::new(&reg.node_path);
    cmd.arg(&reg.entry_path).arg("--project-root").arg(&project)
        .arg("--phase-id").arg(phase).arg("--closure-key").arg(key)
        .current_dir(&reg.package_root).env_clear()
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(unix)]
    cmd.env("HOME", &home).env("PATH", "/usr/bin:/bin");
    #[cfg(windows)]
    {
        let system = std::env::var_os("SystemRoot").context("missing Windows system root")?;
        let bin = PathBuf::from(&system).join("System32");
        cmd.env("SystemRoot", &system).env("USERPROFILE", &home)
            .env("PATH", std::env::join_paths([bin.clone(), bin.join("OpenSSH")])?);
    }
    let mut child = cmd.spawn().context("start registered closure consumer")?;
    let status = match tokio::time::timeout(Duration::from_secs(30), child.wait()).await {
        Ok(result) => result?,
        Err(_) => {
            child.kill().await.context("stop timed-out closure consumer")?;
            bail!("consumer exceeded 30 seconds; its effects are uncertain");
        }
    };
    ensure!(status.success(), "registered closure consumer returned {status}");
    Ok(true)
}

fn file_hash(path: &Path) -> Result<String> {
    ensure!(fs::symlink_metadata(path)?.is_file(), "expected regular file");
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buf = [0; 32768];
    loop {
        let count = file.read(&mut buf)?;
        if count == 0 { break; }
        hash.update(&buf[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

// Digest checks detect changed contents. On Unix also reject installations which
// another local account could replace between checking and process startup.
struct InstallTrust {
    #[cfg(unix)]
    owner: u32,
}

impl InstallTrust {
    fn from_home(_home: &Path) -> Result<Self> {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Ok(Self {
            #[cfg(unix)]
            owner: fs::metadata(_home)?.uid(),
        })
    }

    fn check(&self, _path: &Path, _ancestor: bool) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let meta = fs::symlink_metadata(_path)?;
            let protected_parent = _ancestor && meta.is_dir() && meta.uid() == 0 && meta.mode() & 0o1000 != 0;
            ensure!((meta.uid() == self.owner || meta.uid() == 0)
                && (meta.mode() & 0o022 == 0 || protected_parent),
                "installation must be controlled by the operator or system: {}", _path.display());
        }
        Ok(())
    }
}

fn package_hash(root: &Path, install: &InstallTrust) -> Result<String> {
    let mut files = Vec::new();
    for item in WalkDir::new(root).follow_links(false) {
        let entry = item?;
        install.check(entry.path(), false)?;
        if entry.file_type().is_dir() { continue; }
        ensure!(entry.file_type().is_file(), "consumer package contains a link or special file");
        let relative = entry.path().strip_prefix(root)?;
        let parts: Option<Vec<&str>> = relative.iter().map(|part| part.to_str()).collect();
        let name = parts.context("consumer package paths must be UTF-8")?.join("/");
        files.push((name, entry.into_path()));
    }
    files.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut hash = Sha256::new();
    for (name, path) in files {
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update(file_hash(&path)?.as_bytes());
        hash.update(b"\n");
    }
    Ok(format!("{:x}", hash.finalize()))
}
