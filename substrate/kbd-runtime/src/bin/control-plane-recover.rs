use chrono::Utc;
use kbd_runtime::registry::{ProjectRegistry, ReplicaKind, ReplicaRegistration};
use kbd_runtime::{JournalMigrationSummary, Runtime};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Options {
    data_root: PathBuf,
    registrations: Vec<(PathBuf, Option<ReplicaKind>)>,
    recovered: Vec<String>,
    expect_projects: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArchivedRaft {
    project_id: String,
    archive_path: PathBuf,
    sha256: String,
    bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryReport {
    schema_version: String,
    started_at: String,
    completed_at: String,
    data_root: PathBuf,
    registry_backup: Option<PathBuf>,
    inventory_path: PathBuf,
    registrations: BTreeMap<String, ReplicaRegistration>,
    migrations: Vec<JournalMigrationSummary>,
    projects_without_journals: Vec<String>,
    archived_raft: Vec<ArchivedRaft>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let registry = ProjectRegistry::open_at(&options.data_root);
    let runtime_projects = registry.root().join("projects");
    let mut project_ids = directory_names(&runtime_projects)?;
    project_ids.sort();
    if project_ids.len() != options.expect_projects {
        return Err(format!(
            "expected {} runtime projects, found {}",
            options.expect_projects,
            project_ids.len()
        )
        .into());
    }
    for project_id in &project_ids {
        uuid::Uuid::parse_str(project_id)?;
    }

    let started_at = Utc::now();
    let report_root = registry.root().join("migration-reports").join(format!(
        "{}-control-plane-recovery",
        started_at.format("%Y%m%dT%H%M%S%.fZ")
    ));
    fs::create_dir_all(&report_root)?;
    atomic_write(
        &report_root.join("STARTED.md"),
        format!(
            "# KBD control-plane live recovery\n\nStarted: `{}`\n\nThe sovereign-sync launch agent must remain stopped until this report contains `report.json`.\n",
            started_at.to_rfc3339()
        )
        .as_bytes(),
    )?;
    let inventory_path = report_root.join("pre-migration-sha256.json");
    atomic_json(&inventory_path, &hash_inventory(registry.root())?)?;
    let registry_backup = if registry.registry_path().is_file() {
        let path = report_root.join("registry.pre-migration.json");
        fs::copy(registry.registry_path(), &path)?;
        Some(path)
    } else {
        None
    };

    let mut registrations = BTreeMap::new();
    for (path, kind) in &options.registrations {
        let outcome = match kind {
            Some(kind) => registry.register_existing_as(path, kind.clone())?,
            None => registry.register_existing(path)?,
        };
        registrations.insert(outcome.path, outcome.registration);
    }
    for project_id in &options.recovered {
        let outcome = registry.register_recovered(project_id)?;
        registrations.insert(outcome.path, outcome.registration);
    }

    let document = registry.load()?;
    let registered_projects = document
        .replicas
        .values()
        .map(|replica| replica.project_id.clone())
        .collect::<BTreeSet<_>>();
    let expected_projects = project_ids.iter().cloned().collect::<BTreeSet<_>>();
    if registered_projects != expected_projects {
        let missing = expected_projects
            .difference(&registered_projects)
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = registered_projects
            .difference(&expected_projects)
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "registry/runtime identity mismatch; missing={missing:?}, unexpected={unexpected:?}"
        )
        .into());
    }

    let mut migrations = Vec::new();
    let mut projects_without_journals = Vec::new();
    for project_id in &project_ids {
        let (path, _) = document
            .authoritative_replica(project_id)
            .ok_or_else(|| format!("project {project_id} has no authoritative replica"))?;
        let runtime = Runtime::open_registered_at(Path::new(path), &options.data_root, project_id)?;
        match runtime.migrate_v1_journal()? {
            Some(summary) => migrations.push(summary),
            None => projects_without_journals.push(project_id.clone()),
        }
    }
    if migrations.len() != 14 || projects_without_journals.len() != 4 {
        return Err(format!(
            "unexpected migration totals: {} journals, {} projects without journals",
            migrations.len(),
            projects_without_journals.len()
        )
        .into());
    }

    let mut archived_raft = Vec::new();
    for project_id in &project_ids {
        let root = runtime_projects.join(project_id);
        let source = root.join("raft.redb");
        let archive = root.join("raft.redb.archive");
        if source.exists() && archive.exists() {
            return Err(format!(
                "both {} and {} exist; preserve and adjudicate before continuing",
                source.display(),
                archive.display()
            )
            .into());
        }
        if source.is_file() {
            let sha256 = hash_file(&source)?;
            let bytes = fs::metadata(&source)?.len();
            fs::rename(&source, &archive)?;
            File::open(&root)?.sync_all()?;
            atomic_write(
                &root.join("raft.redb.archive.sha256"),
                format!("{sha256}  raft.redb.archive\n").as_bytes(),
            )?;
            atomic_write(
                &root.join("RAFT-ARCHIVE-ROLLBACK.md"),
                format!(
                    "# Raft archive rollback\n\n1. Stop Sovereign Sync.\n2. Verify `raft.redb.archive.sha256`.\n3. Confirm the journal/Loro migration is being rolled back.\n4. Rename `raft.redb.archive` to `raft.redb`; never delete either file.\n\nOriginal SHA-256: `{sha256}`\n"
                )
                .as_bytes(),
            )?;
            archived_raft.push(ArchivedRaft {
                project_id: project_id.clone(),
                archive_path: archive,
                sha256,
                bytes,
            });
        }
    }
    if archived_raft.len() != 5 {
        return Err(format!(
            "expected to archive 5 raft.redb files, archived {}",
            archived_raft.len()
        )
        .into());
    }

    let report = RecoveryReport {
        schema_version: "1".into(),
        started_at: started_at.to_rfc3339(),
        completed_at: Utc::now().to_rfc3339(),
        data_root: options.data_root,
        registry_backup,
        inventory_path,
        registrations: document.replicas,
        migrations,
        projects_without_journals,
        archived_raft,
    };
    let report_path = report_root.join("report.json");
    atomic_json(&report_path, &report)?;
    atomic_write(
        &report_root.join("ROLLBACK.md"),
        b"# Live recovery rollback\n\n1. Keep Sovereign Sync stopped.\n2. Verify `pre-migration-sha256.json` and each journal/Raft checksum.\n3. Restore `registry.pre-migration.json` while holding the registry lock.\n4. Follow every project `JOURNAL-MIGRATION-ROLLBACK.md`.\n5. Follow every `RAFT-ARCHIVE-ROLLBACK.md`; never delete an archive.\n6. Move managed recovered replica directories to timestamped archive names if reverting; do not delete them.\n",
    )?;
    println!("{}", report_path.display());
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let mut data_root = None;
    let mut registrations = Vec::new();
    let mut recovered = Vec::new();
    let mut expect_projects = 18usize;
    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--data-root") => {
                data_root = Some(PathBuf::from(next(&mut arguments, "--data-root")?))
            }
            Some("--register") => {
                registrations.push((PathBuf::from(next(&mut arguments, "--register")?), None))
            }
            Some("--register-ci") => registrations.push((
                PathBuf::from(next(&mut arguments, "--register-ci")?),
                Some(ReplicaKind::Ci),
            )),
            Some("--register-recovered") => registrations.push((
                PathBuf::from(next(&mut arguments, "--register-recovered")?),
                Some(ReplicaKind::Recovered),
            )),
            Some("--recover") => recovered.push(
                next(&mut arguments, "--recover")?
                    .to_string_lossy()
                    .into_owned(),
            ),
            Some("--expect-projects") => {
                expect_projects = next(&mut arguments, "--expect-projects")?
                    .to_string_lossy()
                    .parse()?
            }
            _ => return Err(format!("unknown argument {}", argument.to_string_lossy()).into()),
        }
    }
    Ok(Options {
        data_root: data_root.ok_or("--data-root is required")?,
        registrations,
        recovered,
        expect_projects,
    })
}

fn next(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    flag: &str,
) -> Result<std::ffi::OsString, Box<dyn Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("{flag} requires a value").into())
}

fn directory_names(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(root)?
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect())
}

fn hash_inventory(root: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    fn visit(
        root: &Path,
        path: &Path,
        hashes: &mut BTreeMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if path.file_name().and_then(|name| name.to_str()) == Some("migration-reports") {
                    continue;
                }
                visit(root, &path, hashes)?;
            } else if file_type.is_file() {
                hashes.insert(
                    path.strip_prefix(root)?.to_string_lossy().into_owned(),
                    hash_file(&path)?,
                );
            } else {
                return Err(format!("inventory refuses symlink {}", path.display()).into());
            }
        }
        Ok(())
    }
    let mut hashes = BTreeMap::new();
    visit(root, root, &mut hashes)?;
    Ok(hashes)
}

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(path.parent().ok_or("output path has no parent")?)?;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}
