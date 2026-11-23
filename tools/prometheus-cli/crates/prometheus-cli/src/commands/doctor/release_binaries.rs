//! Verify release-installed executables against the manifest retained by the installer.
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const PACK: &str = include_str!("../../../../../../../package.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(super) fn provenance(
    home: &Path,
    name: &str,
    installed: &Path,
    installed_hash: &str,
) -> Result<Option<String>> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        _ => return Ok(None),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Ok(None),
    };
    let pack: Value = serde_json::from_str(PACK)?;
    let version = pack["version"].as_str().context("missing pack version")?;
    let tag = format!("v{version}");
    let platform = format!("{os}-{arch}");
    let cache = home
        .join(".prometheus/install/releases")
        .join(&tag)
        .join(&platform);
    let destination = std::path::absolute(installed)?;
    let destination = destination.to_str().context("non-UTF-8 install path")?;
    let key = sha256(destination.as_bytes());
    let receipt_path = cache
        .join("installed")
        .join(name)
        .join(format!("{key}.json"));
    if !receipt_path.try_exists()? {
        return Ok(None);
    }

    let receipt = json(&receipt_path)?;
    ensure!(
        receipt["schemaVersion"] == 1
            && receipt["name"] == name
            && receipt["destination"] == destination
            && receipt["installedSha256"] == installed_hash
            && receipt["tag"] == tag
            && receipt["platform"] == platform,
        "installed bytes or release receipt mismatch"
    );
    let bytes = fs::read(cache.join("manifest.json"))?;
    let digest = sha256(&bytes);
    let sidecar = format!("prometheus-binaries-{tag}-{platform}.manifest.json");
    let sums = fs::read_to_string(cache.join("SHA256SUMS"))?;
    let matches: Vec<_> = sums
        .lines()
        .filter_map(|line| {
            let (hash, file) = line.split_once(' ')?;
            (file.trim_start_matches([' ', '*']) == sidecar).then_some(hash)
        })
        .collect();
    ensure!(
        matches.as_slice() == [digest.as_str()] && receipt["manifestSha256"] == digest,
        "release manifest checksum mismatch"
    );
    let manifest: Value = serde_json::from_slice(&bytes)?;
    ensure!(
        manifest["schemaVersion"] == 1
            && manifest["version"] == version
            && manifest["tag"] == tag
            && manifest["platform"] == platform,
        "release manifest identity mismatch"
    );
    let files = manifest["files"]
        .as_array()
        .context("missing release file list")?;
    let mut entries = files.iter().filter(|record| record["name"] == name);
    let file = entries
        .next()
        .context("binary absent from release manifest")?;
    ensure!(
        entries.next().is_none()
            && file["path"] == format!("bin/{name}{}", std::env::consts::EXE_SUFFIX)
            && file["sha256"].is_string()
            && file["sha256"] == receipt["buildSha256"],
        "release binary digest mismatch"
    );
    if !(cfg!(target_os = "macos") && name == "prometheus-exec") {
        ensure!(
            file["sha256"] == installed_hash,
            "installed binary differs from release"
        );
    }
    Ok(Some(format!("{tag} {platform}")))
}
