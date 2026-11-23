//! One verified native release stage per CLI installation.
use anyhow::{ensure, Context, Result};
use std::{path::Path, process::Command};

#[derive(Default)]
pub(super) struct ReleaseBinaries {
    checked: bool,
    stage: Option<tempfile::TempDir>,
}

impl ReleaseBinaries {
    pub fn install(&mut self, root: &Path, destination: &Path, names: &[&str]) -> Result<bool> {
        let script = root.join("scripts/binary-release.mjs");
        if !script.is_file() {
            return Ok(false); // Older source-only checkout.
        }
        if !self.checked {
            let stage = tempfile::tempdir()?;
            let status = Command::new("node")
                .arg(&script).arg("prepare").arg(stage.path())
                .status().context("Could not resolve the native release")?;
            match status.code() {
                Some(0) => self.stage = Some(stage),
                Some(3) => {}
                _ => anyhow::bail!("Native release verification failed"),
            }
            self.checked = true;
        }
        let Some(stage) = &self.stage else { return Ok(false) };
        for name in names {
            let filename = format!("{name}{}", std::env::consts::EXE_SUFFIX);
            let status = Command::new("node")
                .arg(&script).arg("copy")
                .arg(destination.join(&filename)).arg(stage.path().join("bin").join(&filename))
                .status()?;
            ensure!(status.success(), "Verified release install failed for {name}");
        }
        Ok(true)
    }
}

pub(super) fn clear_source_marker(root: &Path, destination: &Path) -> Result<()> {
    let script = root.join("scripts/binary-release.mjs");
    if script.is_file() {
        let status = Command::new("node").arg(&script).arg("mark").arg(destination).status()?;
        ensure!(status.success(), "Could not update binary installation provenance");
    }
    Ok(())
}
