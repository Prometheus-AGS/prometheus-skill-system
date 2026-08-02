use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

use crate::{
    read_project_manifest, DeviceSigner, Event, MigrationProvenance, ProjectManifest, Result,
    Runtime, RuntimeError, SubmodulePin, EVENT_SCHEMA_VERSION,
};

pub const REGISTRY_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaKind {
    Standalone,
    Worktree,
    Submodule,
    Bare,
    Ci,
    Mobile,
}

impl ReplicaKind {
    fn authority_rank(&self) -> u8 {
        match self {
            Self::Standalone => 0,
            Self::Worktree => 1,
            Self::Submodule => 2,
            Self::Ci => 3,
            Self::Bare => 4,
            Self::Mobile => 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParentLinkage {
    pub path: String,
    pub project_id: Option<String>,
    pub replica_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaRegistration {
    pub project_id: String,
    pub replica_id: String,
    pub machine_id: String,
    pub kind: ReplicaKind,
    pub parent: Option<ParentLinkage>,
    pub head: Option<String>,
    pub origin: Option<String>,
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_reason: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionRedirect {
    pub former_project_id: String,
    pub into_project_id: String,
    pub replica_id: String,
    pub path: String,
    pub backup_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryDocument {
    pub schema_version: String,
    pub machine_id: String,
    pub replicas: BTreeMap<String, ReplicaRegistration>,
    #[serde(default)]
    pub redirects: BTreeMap<String, AdoptionRedirect>,
}

impl RegistryDocument {
    fn new() -> Self {
        Self {
            schema_version: REGISTRY_SCHEMA_VERSION.into(),
            machine_id: Uuid::new_v4().to_string(),
            replicas: BTreeMap::new(),
            redirects: BTreeMap::new(),
        }
    }

    pub fn project_replicas(&self, project_id: &str) -> Vec<(&str, &ReplicaRegistration)> {
        self.replicas
            .iter()
            .filter(|(_, replica)| replica.project_id == project_id)
            .map(|(path, replica)| (path.as_str(), replica))
            .collect()
    }

    pub fn authoritative_replica(&self, project_id: &str) -> Option<(&str, &ReplicaRegistration)> {
        self.project_replicas(project_id)
            .into_iter()
            .min_by_key(|(path, replica)| (replica.kind.authority_rank(), *path))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCandidate {
    pub path: String,
    pub project_id: String,
    pub replica_id: String,
    pub origin_match: bool,
    pub head_match: bool,
    pub authoritative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationOutcome {
    pub path: String,
    pub registration: ReplicaRegistration,
    pub created: bool,
    pub duplicate_candidates: Vec<DuplicateCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubmoduleScanResult {
    pub parent_path: String,
    pub pins: Vec<SubmodulePin>,
    pub skipped: Vec<String>,
}

pub fn scan_submodule_pins(project_root: impl AsRef<Path>) -> Result<SubmoduleScanResult> {
    let parent = canonical_existing_path(project_root.as_ref())?;
    let output = Command::new("git")
        .arg("-C")
        .arg(&parent)
        .args(["ls-files", "--stage", "-z"])
        .output()?;
    if !output.status.success() {
        return Err(RuntimeError::InvalidState(format!(
            "{} is not a readable Git work tree",
            parent.display()
        )));
    }
    let mut pins = Vec::new();
    let mut skipped = Vec::new();
    for entry in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let Some(tab) = entry.iter().position(|byte| *byte == b'\t') else {
            continue;
        };
        let metadata = std::str::from_utf8(&entry[..tab]).map_err(|_| {
            RuntimeError::InvalidState("gitlink metadata must be valid UTF-8".into())
        })?;
        let mut fields = metadata.split_whitespace();
        if fields.next() != Some("160000") {
            continue;
        }
        let Some(gitlink_sha) = fields.next() else {
            continue;
        };
        let path = std::str::from_utf8(&entry[tab + 1..])
            .map_err(|_| RuntimeError::InvalidState("gitlink path must be valid UTF-8".into()))?;
        let child_root = parent.join(path);
        let Some(manifest) = read_project_manifest(&child_root)? else {
            skipped.push(format!("{path}: child project manifest is unavailable"));
            continue;
        };
        pins.push(SubmodulePin {
            path: path.to_owned(),
            child_project_id: manifest.project_id,
            gitlink_sha: gitlink_sha.to_owned(),
        });
    }
    pins.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort();
    Ok(SubmoduleScanResult {
        parent_path: path_key(&parent)?,
        pins,
        skipped,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionPlan {
    pub source_path: String,
    pub former_project_id: String,
    pub into_project_id: String,
    pub source_replica_id: Option<String>,
    pub target_authority_path: String,
    pub source_event_count: usize,
    pub source_event_hashes: Vec<String>,
    pub backup_root: String,
    pub source_kind: ReplicaKind,
    pub target_kind: ReplicaKind,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionBackupEntry {
    pub source: String,
    pub backup: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionBackupManifest {
    pub schema_version: String,
    pub former_project_id: String,
    pub into_project_id: String,
    pub created_at: DateTime<Utc>,
    pub files: Vec<AdoptionBackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionResult {
    pub plan: AdoptionPlan,
    pub registration: ReplicaRegistration,
    pub redirect: AdoptionRedirect,
    pub migrated_journal: String,
    pub backup_manifest: String,
}

#[derive(Debug, Clone)]
pub struct ProjectRegistry {
    data_root: PathBuf,
    root: PathBuf,
}

impl ProjectRegistry {
    pub fn open() -> Self {
        let data_root = std::env::var_os("PROMETHEUS_DATA_DIR")
            .map(PathBuf::from)
            .or_else(dirs_next::data_local_dir)
            .unwrap_or_else(|| std::env::temp_dir().join("prometheus-data"));
        Self::open_at(data_root)
    }

    pub fn open_at(data_root: impl AsRef<Path>) -> Self {
        let data_root = data_root.as_ref().to_path_buf();
        Self {
            root: data_root.join("prometheus").join("kbd"),
            data_root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn registry_path(&self) -> PathBuf {
        self.root.join("registry.json")
    }

    pub fn lock_path(&self) -> PathBuf {
        self.root.join("registry.lock")
    }

    pub fn load(&self) -> Result<RegistryDocument> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = self.load_or_create_locked();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn register_existing(&self, project_root: impl AsRef<Path>) -> Result<RegistrationOutcome> {
        let canonical = canonical_existing_path(project_root.as_ref())?;
        let manifest = read_project_manifest(&canonical)?.ok_or_else(|| {
            RuntimeError::InvalidState(format!(
                "{} has no .prometheus/project.json; registration never infers or creates project identity",
                canonical.display()
            ))
        })?;
        let evidence = inspect_replica(&canonical, &manifest, None, &self.root)?;

        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = (|| {
            let mut document = self.load_or_create_locked()?;
            let path = path_key(&canonical)?;
            let now = Utc::now();
            let existing = document.replicas.get(&path).cloned();
            if let Some(existing) = &existing {
                if existing.project_id != manifest.project_id {
                    return Err(RuntimeError::InvalidState(format!(
                        "registered path {} changed project identity from {} to {}; use adoption or repair the manifest",
                        canonical.display(), existing.project_id, manifest.project_id
                    )));
                }
            }
            let replica_id = existing
                .as_ref()
                .map(|registration| registration.replica_id.clone())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let registered_at = existing
                .as_ref()
                .map(|registration| registration.registered_at)
                .unwrap_or(now);
            let parent = resolve_parent_replica(&evidence.parent, &document);
            let registration = ReplicaRegistration {
                project_id: manifest.project_id.clone(),
                replica_id,
                machine_id: document.machine_id.clone(),
                kind: evidence.kind,
                parent,
                head: evidence.head,
                origin: evidence.origin,
                read_only: evidence.read_only,
                read_only_reason: evidence.read_only_reason,
                registered_at,
                updated_at: now,
            };
            document.replicas.insert(path.clone(), registration.clone());
            let duplicate_candidates = duplicate_candidates(&document, &path, &registration);
            self.write_locked(&document)?;
            Ok(RegistrationOutcome {
                path,
                registration,
                created: existing.is_none(),
                duplicate_candidates,
            })
        })();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn lookup_project(&self, project_id: &str) -> Result<Vec<(PathBuf, ReplicaRegistration)>> {
        let document = self.load()?;
        Ok(document
            .replicas
            .into_iter()
            .filter(|(_, replica)| replica.project_id == project_id)
            .map(|(path, replica)| (PathBuf::from(path), replica))
            .collect())
    }

    pub fn lookup_path(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<Option<ReplicaRegistration>> {
        let canonical = canonical_existing_path(project_root.as_ref())?;
        Ok(self.load()?.replicas.get(&path_key(&canonical)?).cloned())
    }

    pub fn plan_adoption(
        &self,
        source_path: impl AsRef<Path>,
        into_project_id: &str,
    ) -> Result<AdoptionPlan> {
        let source_path = canonical_existing_path(source_path.as_ref())?;
        let source_manifest = read_project_manifest(&source_path)?.ok_or_else(|| {
            RuntimeError::InvalidState(format!(
                "{} has no .prometheus/project.json",
                source_path.display()
            ))
        })?;
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_shared()?;
        let result = (|| {
            let document = self.load_existing_locked()?;
            self.adoption_plan_locked(&document, &source_path, &source_manifest, into_project_id)
        })();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn apply_adoption(
        &self,
        source_path: impl AsRef<Path>,
        into_project_id: &str,
    ) -> Result<AdoptionResult> {
        self.apply_adoption_inner(source_path.as_ref(), into_project_id, None)
    }

    fn apply_adoption_inner(
        &self,
        source_path: &Path,
        into_project_id: &str,
        signer_override: Option<&DeviceSigner>,
    ) -> Result<AdoptionResult> {
        let source_path = canonical_existing_path(source_path)?;
        let source_manifest = read_project_manifest(&source_path)?.ok_or_else(|| {
            RuntimeError::InvalidState(format!(
                "{} has no .prometheus/project.json",
                source_path.display()
            ))
        })?;
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = (|| {
            let mut document = self.load_or_create_locked()?;
            let plan = self.adoption_plan_locked(
                &document,
                &source_path,
                &source_manifest,
                into_project_id,
            )?;
            if plan.source_kind == ReplicaKind::Standalone
                && plan.target_kind == ReplicaKind::Standalone
            {
                return Err(RuntimeError::InvalidState(
                    "standalone-to-standalone identity is ambiguous; adoption remains a dry-run suggestion until an explicit adjudication surface is available".into(),
                ));
            }

            let owned_signer;
            let signer = if let Some(signer) = signer_override {
                signer
            } else {
                let target_runtime = Runtime::open_registered_at(
                    Path::new(&plan.target_authority_path),
                    &self.data_root,
                    into_project_id,
                )?;
                owned_signer = target_runtime.device_signer()?;
                &owned_signer
            };
            let source_events = read_events(&self.project_events_path(&plan.former_project_id))?;
            let replica_id = Uuid::new_v4().to_string();
            let migrated_events = resign_adopted_events(
                &source_events,
                into_project_id,
                plan.source_replica_id.as_deref(),
                &replica_id,
                signer,
            )?;

            let backup_root = PathBuf::from(&plan.backup_root);
            fs::create_dir_all(&backup_root)?;
            let mut backup_entries = Vec::new();
            backup_entries.push(copy_with_checksum(
                &source_path.join(".prometheus/project.json"),
                &backup_root.join("project.json"),
            )?);
            backup_entries.push(copy_with_checksum(
                &self.registry_path(),
                &backup_root.join("registry.json"),
            )?);
            let source_journal = self.project_events_path(&plan.former_project_id);
            if source_journal.exists() {
                backup_entries.push(copy_with_checksum(
                    &source_journal,
                    &backup_root.join("events.jsonl"),
                )?);
            }
            let backup_manifest = AdoptionBackupManifest {
                schema_version: "1".into(),
                former_project_id: plan.former_project_id.clone(),
                into_project_id: into_project_id.into(),
                created_at: Utc::now(),
                files: backup_entries,
            };
            let backup_manifest_path = backup_root.join("backup-manifest.json");
            write_json_file(&backup_manifest_path, &backup_manifest)?;
            write_rollback_instructions(
                &backup_root,
                &source_path,
                &self.registry_path(),
                &replica_id,
                into_project_id,
            )?;

            let migrated_journal = self
                .root
                .join("projects")
                .join(into_project_id)
                .join("replicas")
                .join(&replica_id)
                .join("events.jsonl");
            write_event_journal(&migrated_journal, &migrated_events)?;

            let adopted_manifest = ProjectManifest {
                schema_version: source_manifest.schema_version.clone(),
                project_id: into_project_id.into(),
                repository_fingerprint: source_manifest.repository_fingerprint.clone(),
            };
            crate::atomic_json(
                &source_path.join(".prometheus/project.json"),
                &serde_json::to_value(&adopted_manifest)?,
            )?;

            let source_key = path_key(&source_path)?;
            let now = Utc::now();
            let evidence = inspect_replica(&source_path, &adopted_manifest, None, &self.root)?;
            let parent = resolve_parent_replica(&evidence.parent, &document);
            let registration = ReplicaRegistration {
                project_id: into_project_id.into(),
                replica_id: replica_id.clone(),
                machine_id: document.machine_id.clone(),
                kind: evidence.kind,
                parent,
                head: evidence.head,
                origin: evidence.origin,
                read_only: evidence.read_only,
                read_only_reason: evidence.read_only_reason,
                registered_at: now,
                updated_at: now,
            };
            document
                .replicas
                .insert(source_key.clone(), registration.clone());
            let redirect = AdoptionRedirect {
                former_project_id: plan.former_project_id.clone(),
                into_project_id: into_project_id.into(),
                replica_id: replica_id.clone(),
                path: source_key,
                backup_path: path_key(&backup_root)?,
                created_at: now,
            };
            document
                .redirects
                .insert(plan.former_project_id.clone(), redirect.clone());
            self.write_locked(&document)?;

            Ok(AdoptionResult {
                plan,
                registration,
                redirect,
                migrated_journal: path_key(&migrated_journal)?,
                backup_manifest: path_key(&backup_manifest_path)?,
            })
        })();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn record_redirect(&self, redirect: AdoptionRedirect) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = (|| {
            let mut document = self.load_or_create_locked()?;
            document
                .redirects
                .insert(redirect.former_project_id.clone(), redirect);
            self.write_locked(&document)
        })();
        FileExt::unlock(&lock)?;
        result
    }

    fn open_lock(&self) -> Result<File> {
        Ok(OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?)
    }

    fn load_existing_locked(&self) -> Result<RegistryDocument> {
        let path = self.registry_path();
        if !path.exists() {
            return Err(RuntimeError::InvalidState(
                "KBD registry is empty; register the target project before adoption".into(),
            ));
        }
        let document: RegistryDocument = serde_json::from_reader(File::open(&path)?)?;
        if document.schema_version != REGISTRY_SCHEMA_VERSION
            || Uuid::parse_str(&document.machine_id).is_err()
        {
            return Err(RuntimeError::InvalidState(format!(
                "{} is not a valid KBD registry",
                path.display()
            )));
        }
        Ok(document)
    }

    fn adoption_plan_locked(
        &self,
        document: &RegistryDocument,
        source_path: &Path,
        source_manifest: &ProjectManifest,
        into_project_id: &str,
    ) -> Result<AdoptionPlan> {
        if source_manifest.project_id == into_project_id {
            return Err(RuntimeError::InvalidState(format!(
                "{} is already a replica of project {}",
                source_path.display(),
                into_project_id
            )));
        }
        let (target_path, target) =
            document
                .authoritative_replica(into_project_id)
                .ok_or_else(|| {
                    RuntimeError::InvalidState(format!(
                        "target project {into_project_id} is not registered"
                    ))
                })?;
        let source_key = path_key(source_path)?;
        let source_registration = document.replicas.get(&source_key);
        let evidence = inspect_replica(source_path, source_manifest, None, &self.root)?;
        let source_events = read_events(&self.project_events_path(&source_manifest.project_id))?;
        let mut warnings = Vec::new();
        if evidence.kind == ReplicaKind::Standalone && target.kind == ReplicaKind::Standalone {
            warnings.push(
                "both source and target are standalone checkouts; no authority winner is inferred"
                    .into(),
            );
        } else if target.kind != ReplicaKind::Standalone {
            warnings.push(format!(
                "target authority is {:?}; a canonical standalone checkout is preferred",
                target.kind
            ));
        }
        let stamp = Utc::now().format("%Y%m%dT%H%M%S%.fZ");
        let backup_root = self.root.join("adoption-backups").join(format!(
            "{stamp}-{}-into-{}",
            source_manifest.project_id, into_project_id
        ));
        Ok(AdoptionPlan {
            source_path: source_key,
            former_project_id: source_manifest.project_id.clone(),
            into_project_id: into_project_id.into(),
            source_replica_id: source_registration.map(|replica| replica.replica_id.clone()),
            target_authority_path: target_path.into(),
            source_event_count: source_events.len(),
            source_event_hashes: source_events
                .iter()
                .map(|event| event.integrity_hash.clone())
                .collect(),
            backup_root: path_key(&backup_root)?,
            source_kind: evidence.kind,
            target_kind: target.kind.clone(),
            warnings,
        })
    }

    fn project_events_path(&self, project_id: &str) -> PathBuf {
        self.root
            .join("projects")
            .join(project_id)
            .join("events.jsonl")
    }

    fn load_or_create_locked(&self) -> Result<RegistryDocument> {
        let path = self.registry_path();
        if !path.exists() {
            let document = RegistryDocument::new();
            self.write_locked(&document)?;
            return Ok(document);
        }
        let document: RegistryDocument = serde_json::from_reader(File::open(&path)?)?;
        if document.schema_version != REGISTRY_SCHEMA_VERSION
            || Uuid::parse_str(&document.machine_id).is_err()
        {
            return Err(RuntimeError::InvalidState(format!(
                "{} is not a valid KBD registry",
                path.display()
            )));
        }
        Ok(document)
    }

    fn write_locked(&self, document: &RegistryDocument) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        let path = self.registry_path();
        let temporary = self
            .root
            .join(format!(".registry.json.{}.tmp", Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        serde_json::to_writer_pretty(&mut file, document)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, &path)?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FlockProbe {
    Proven {
        child_pid: u32,
    },
    Unavailable {
        child_pid: Option<u32>,
        reason: String,
    },
}

/// Verify lock exclusion with a separately opened descriptor in a real child
/// process. A volume is writable only when the child observes `WouldBlock`.
#[cfg(unix)]
pub fn probe_flock_exclusion(probe_root: &Path) -> Result<FlockProbe> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;

    if let Err(error) = fs::create_dir_all(probe_root) {
        return Ok(FlockProbe::Unavailable {
            child_pid: None,
            reason: format!("cannot create lock probe directory: {error}"),
        });
    }
    let path = probe_root.join(format!(".flock-probe-{}", Uuid::new_v4()));
    let parent = match OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) => {
            return Ok(FlockProbe::Unavailable {
                child_pid: None,
                reason: format!("cannot create lock probe file: {error}"),
            })
        }
    };
    if let Err(error) = parent.lock_exclusive() {
        drop(parent);
        let _ = fs::remove_file(&path);
        return Ok(FlockProbe::Unavailable {
            child_pid: None,
            reason: format!("cannot acquire parent lock: {error}"),
        });
    }
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        RuntimeError::InvalidState("flock probe path contains an interior NUL".into())
    })?;
    let parent_fd = parent.as_raw_fd();
    // SAFETY: the child performs only async-signal-safe libc calls and exits
    // with `_exit`; all Rust-owned cleanup remains in the parent.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        FileExt::unlock(&parent)?;
        fs::remove_file(&path)?;
        return Ok(FlockProbe::Unavailable {
            child_pid: None,
            reason: std::io::Error::last_os_error().to_string(),
        });
    }
    if pid == 0 {
        unsafe {
            libc::close(parent_fd);
            let child_fd = libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC);
            if child_fd < 0 {
                libc::_exit(2);
            }
            let result = libc::flock(child_fd, libc::LOCK_EX | libc::LOCK_NB);
            if result == 0 {
                libc::_exit(1);
            }
            #[cfg(target_vendor = "apple")]
            let errno = *libc::__error();
            #[cfg(target_os = "android")]
            let errno = *libc::__errno();
            #[cfg(all(not(target_vendor = "apple"), not(target_os = "android")))]
            let errno = *libc::__errno_location();
            if errno == libc::EWOULDBLOCK || errno == libc::EAGAIN {
                libc::_exit(0);
            }
            libc::_exit(2);
        }
    }

    let mut status = 0;
    let waited = loop {
        let result = unsafe { libc::waitpid(pid, &mut status, 0) };
        if result >= 0 {
            break result;
        }
        if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
            break result;
        }
    };
    FileExt::unlock(&parent)?;
    drop(parent);
    fs::remove_file(&path)?;
    File::open(probe_root)?.sync_all()?;
    let child_pid = pid as u32;
    if waited != pid || !libc::WIFEXITED(status) {
        return Ok(FlockProbe::Unavailable {
            child_pid: Some(child_pid),
            reason: "flock probe child did not exit normally".into(),
        });
    }
    match libc::WEXITSTATUS(status) {
        0 => Ok(FlockProbe::Proven { child_pid }),
        1 => Ok(FlockProbe::Unavailable {
            child_pid: Some(child_pid),
            reason: "a second process acquired the exclusive lock".into(),
        }),
        _ => Ok(FlockProbe::Unavailable {
            child_pid: Some(child_pid),
            reason: "the child could not verify an expected WouldBlock result".into(),
        }),
    }
}

#[cfg(not(unix))]
pub fn probe_flock_exclusion(_probe_root: &Path) -> Result<FlockProbe> {
    Ok(FlockProbe::Unavailable {
        child_pid: None,
        reason: "cross-process flock probing is unsupported on this platform".into(),
    })
}

#[derive(Debug)]
struct ReplicaEvidence {
    kind: ReplicaKind,
    parent: Option<ParentLinkage>,
    head: Option<String>,
    origin: Option<String>,
    read_only: bool,
    read_only_reason: Option<String>,
}

fn inspect_replica(
    path: &Path,
    _manifest: &ProjectManifest,
    forced_kind: Option<ReplicaKind>,
    probe_root: &Path,
) -> Result<ReplicaEvidence> {
    let bare = git_output(path, &["rev-parse", "--is-bare-repository"]).as_deref() == Some("true");
    let superproject = git_output(path, &["rev-parse", "--show-superproject-working-tree"])
        .filter(|value| !value.is_empty());
    let git_dir = git_output(path, &["rev-parse", "--git-dir"]);
    let common_dir = git_output(path, &["rev-parse", "--git-common-dir"]);
    let ci = ["CI", "GITHUB_ACTIONS", "BUILDKITE", "GITLAB_CI"]
        .iter()
        .any(|name| std::env::var_os(name).is_some());
    let kind = forced_kind.unwrap_or_else(|| {
        if ci {
            ReplicaKind::Ci
        } else if bare {
            ReplicaKind::Bare
        } else if superproject.is_some() {
            ReplicaKind::Submodule
        } else if git_dir.is_some() && common_dir.is_some() && git_dir != common_dir {
            ReplicaKind::Worktree
        } else {
            ReplicaKind::Standalone
        }
    });
    let parent = superproject.and_then(|parent| {
        let parent = canonical_existing_path(Path::new(&parent)).ok()?;
        let project_id = read_project_manifest(&parent)
            .ok()
            .flatten()
            .map(|manifest| manifest.project_id);
        Some(ParentLinkage {
            path: path_key(&parent).ok()?,
            project_id,
            replica_id: None,
        })
    });
    let (read_only, read_only_reason) = match kind {
        ReplicaKind::Bare => (
            true,
            Some("bare replicas do not have a writable worktree".into()),
        ),
        ReplicaKind::Ci => (
            true,
            Some("CI replicas are observation-only and cannot commit KBD state".into()),
        ),
        _ => {
            let probes = [
                ("KBD data volume", probe_root.to_path_buf()),
                ("project volume", path.join(".prometheus")),
            ];
            let unavailable =
                probes
                    .into_iter()
                    .find_map(|(label, root)| match probe_flock_exclusion(&root) {
                        Ok(FlockProbe::Proven { .. }) => None,
                        Ok(FlockProbe::Unavailable { reason, .. }) => {
                            Some(format!("{label}: {reason}"))
                        }
                        Err(error) => Some(format!("{label}: {error}")),
                    });
            match unavailable {
                None => (false, None),
                Some(reason) => (
                    true,
                    Some(format!(
                        "cross-process flock exclusion could not be proven: {reason}"
                    )),
                ),
            }
        }
    };
    Ok(ReplicaEvidence {
        kind,
        parent,
        head: git_output(path, &["rev-parse", "HEAD"]),
        origin: git_output(path, &["config", "--get", "remote.origin.url"]),
        read_only,
        read_only_reason,
    })
}

fn resolve_parent_replica(
    parent: &Option<ParentLinkage>,
    document: &RegistryDocument,
) -> Option<ParentLinkage> {
    parent.as_ref().map(|parent| {
        let mut parent = parent.clone();
        if let Some(registration) = document.replicas.get(&parent.path) {
            parent.project_id = Some(registration.project_id.clone());
            parent.replica_id = Some(registration.replica_id.clone());
        }
        parent
    })
}

fn duplicate_candidates(
    document: &RegistryDocument,
    registered_path: &str,
    registered: &ReplicaRegistration,
) -> Vec<DuplicateCandidate> {
    let mut candidates = Vec::new();
    for (path, candidate) in &document.replicas {
        if path == registered_path || candidate.project_id == registered.project_id {
            continue;
        }
        let origin_match = registered.origin.is_some() && registered.origin == candidate.origin;
        let head_match = registered.head.is_some() && registered.head == candidate.head;
        if !origin_match && !head_match {
            continue;
        }
        let authoritative = document
            .authoritative_replica(&candidate.project_id)
            .is_some_and(|(authority_path, _)| authority_path == path);
        candidates.push(DuplicateCandidate {
            path: path.clone(),
            project_id: candidate.project_id.clone(),
            replica_id: candidate.replica_id.clone(),
            origin_match,
            head_match,
            authoritative,
        });
    }
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    candidates
}

fn canonical_existing_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|error| {
        RuntimeError::InvalidState(format!(
            "cannot canonicalize registry path {}: {error}",
            path.display()
        ))
    })
}

fn path_key(path: &Path) -> Result<String> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        RuntimeError::InvalidState(format!(
            "registry paths must be valid UTF-8: {}",
            path.display()
        ))
    })
}

fn git_output(path: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn read_events(path: &Path) -> Result<Vec<Event>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if !line.trim().is_empty() {
            events.push(serde_json::from_str(&line)?);
        }
    }
    crate::replay_events(&events)?;
    Ok(events)
}

fn resign_adopted_events(
    source_events: &[Event],
    into_project_id: &str,
    source_replica_id: Option<&str>,
    new_replica_id: &str,
    signer: &DeviceSigner,
) -> Result<Vec<Event>> {
    let mut migrated = Vec::with_capacity(source_events.len());
    let mut previous_event_id = None;
    let mut previous_hash = None;
    let mut frontier = crate::CausalFrontier::empty();
    for (index, source) in source_events.iter().enumerate() {
        let lamport = index as u64 + 1;
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: into_project_id.into(),
            replica_id: new_replica_id.into(),
            run_id: source.run_id.clone(),
            event_id: Uuid::new_v4().to_string(),
            command_id: source
                .command_id
                .as_ref()
                .map(|command_id| format!("adopt:{new_replica_id}:{command_id}")),
            revision: lamport,
            expected_revision: index as u64,
            lamport,
            frontier: frontier.clone(),
            causal_parent: previous_event_id.clone(),
            actor: source.actor.clone(),
            actor_id: source.actor.id.clone(),
            timestamp: source.timestamp,
            kind: source.kind.clone(),
            previous_hash: previous_hash.clone(),
            migration_provenance: Some(MigrationProvenance {
                source_project_id: source.project_id.clone(),
                source_replica_id: source_replica_id.map(str::to_owned),
                source_event_id: source.event_id.clone(),
                source_integrity_hash: source.integrity_hash.clone(),
                source_previous_hash: source.previous_hash.clone(),
            }),
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        event.seal(signer)?;
        previous_event_id = Some(event.event_id.clone());
        previous_hash = Some(event.integrity_hash.clone());
        frontier.advance(new_replica_id, lamport);
        migrated.push(event);
    }
    crate::replay_events(&migrated)?;
    Ok(migrated)
}

fn copy_with_checksum(source: &Path, backup: &Path) -> Result<AdoptionBackupEntry> {
    let bytes = fs::read(source)?;
    if let Some(parent) = backup.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(backup)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(AdoptionBackupEntry {
        source: path_key(source)?,
        backup: path_key(backup)?,
        bytes: bytes.len() as u64,
        sha256: format!("{:x}", Sha256::digest(&bytes)),
    })
}

fn write_json_file(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn write_event_journal(path: &Path, events: &[Event]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    for event in events {
        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn write_rollback_instructions(
    backup_root: &Path,
    source_path: &Path,
    registry_path: &Path,
    replica_id: &str,
    into_project_id: &str,
) -> Result<()> {
    let migrated_replica = registry_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("projects")
        .join(into_project_id)
        .join("replicas")
        .join(replica_id);
    let instructions = format!(
        "# KBD adoption rollback\n\n\
         1. Stop Sovereign Sync.\n\
         2. Verify `backup-manifest.json` SHA-256 values.\n\
         3. Restore `{backup}/project.json` to `{source}/.prometheus/project.json`.\n\
         4. Restore `{backup}/registry.json` to `{registry}` while holding the registry lock.\n\
         5. Move `{replica}` to an archive name; do not delete it.\n\
         6. Restart the service and verify registry plus journal readiness.\n",
        backup = backup_root.display(),
        source = source_path.display(),
        registry = registry_path.display(),
        replica = migrated_replica.display(),
    );
    let path = backup_root.join("ROLLBACK.md");
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(instructions.as_bytes())?;
    file.sync_all()?;
    File::open(backup_root)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn write_manifest(path: &Path, project_id: &str) {
        fs::create_dir_all(path.join(".prometheus")).unwrap();
        fs::write(
            path.join(".prometheus/project.json"),
            serde_json::to_vec_pretty(&ProjectManifest {
                schema_version: "1".into(),
                project_id: project_id.into(),
                repository_fingerprint: "sha256:test".into(),
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn init_git(path: &Path, origin: &str) {
        assert!(Command::new("git")
            .args(["init", "-q"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["config", "user.email", "kbd@example.invalid"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["config", "user.name", "KBD Test"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["remote", "add", "origin", origin])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        fs::write(path.join("tracked.txt"), "same\n").unwrap();
        assert!(Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["commit", "-qm", "initial"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }

    fn commit_all(path: &Path, message: &str) {
        assert!(Command::new("git")
            .args(["add", "-A"])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["commit", "-qm", message])
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }

    #[test]
    fn registration_requires_an_existing_identity_manifest() {
        let fixture = tempdir().unwrap();
        let project = fixture.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let registry = ProjectRegistry::open_at(fixture.path().join("data"));
        let error = registry.register_existing(&project).unwrap_err();
        assert!(error.to_string().contains("never infers or creates"));
        assert!(!project.join(".prometheus/project.json").exists());
    }

    #[test]
    fn two_paths_with_one_uuid_are_distinct_replicas() {
        let fixture = tempdir().unwrap();
        let project_id = Uuid::new_v4().to_string();
        let first = fixture.path().join("first");
        let second = fixture.path().join("second");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        write_manifest(&first, &project_id);
        write_manifest(&second, &project_id);
        let registry = ProjectRegistry::open_at(fixture.path().join("data"));
        let one = registry.register_existing(&first).unwrap();
        let two = registry.register_existing(&second).unwrap();
        assert_eq!(one.registration.project_id, two.registration.project_id);
        assert_ne!(one.registration.replica_id, two.registration.replica_id);
        assert_eq!(registry.lookup_project(&project_id).unwrap().len(), 2);
    }

    #[test]
    fn flock_probe_uses_a_real_child_and_leaves_no_probe_file() {
        let fixture = tempdir().unwrap();
        let probe_root = fixture.path().join("probe");
        let result = probe_flock_exclusion(&probe_root).unwrap();
        match result {
            FlockProbe::Proven { child_pid } => {
                assert_ne!(child_pid, std::process::id());
            }
            FlockProbe::Unavailable { reason, .. } => {
                panic!("temporary local filesystem must support flock exclusion: {reason}")
            }
        }
        assert_eq!(fs::read_dir(probe_root).unwrap().count(), 0);
    }

    #[test]
    fn bare_replica_is_registered_read_only_with_an_actionable_reason() {
        let fixture = tempdir().unwrap();
        let project = fixture.path().join("bare.git");
        assert!(Command::new("git")
            .args(["init", "--bare", "--quiet"])
            .arg(&project)
            .status()
            .unwrap()
            .success());
        let project_id = Uuid::new_v4().to_string();
        write_manifest(&project, &project_id);
        let data_root = fixture.path().join("data");
        let registry = ProjectRegistry::open_at(&data_root);
        let outcome = registry.register_existing(&project).unwrap();
        assert_eq!(outcome.registration.kind, ReplicaKind::Bare);
        assert!(outcome.registration.read_only);
        assert!(outcome
            .registration
            .read_only_reason
            .as_deref()
            .unwrap()
            .contains("writable worktree"));

        let runtime = Runtime::open_registered_at(&project, &data_root, &project_id).unwrap();
        assert!(matches!(
            runtime.initialize(
                &project_id,
                "run",
                crate::Actor::operator("operator", "bare-test")
            ),
            Err(RuntimeError::ReplicaReadOnly { .. })
        ));
    }

    #[test]
    fn matching_origin_and_head_only_suggest_a_duplicate() {
        let fixture = tempdir().unwrap();
        let first = fixture.path().join("first");
        let second = fixture.path().join("second");
        fs::create_dir_all(&first).unwrap();
        init_git(&first, "https://example.invalid/repo.git");
        assert!(Command::new("git")
            .args(["clone", "-q"])
            .arg(&first)
            .arg(&second)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args([
                "remote",
                "set-url",
                "origin",
                "https://example.invalid/repo.git",
            ])
            .current_dir(&second)
            .status()
            .unwrap()
            .success());
        write_manifest(&first, &Uuid::new_v4().to_string());
        write_manifest(&second, &Uuid::new_v4().to_string());
        let registry = ProjectRegistry::open_at(fixture.path().join("data"));
        registry.register_existing(&first).unwrap();
        let outcome = registry.register_existing(&second).unwrap();
        assert_eq!(outcome.duplicate_candidates.len(), 1);
        assert!(outcome.duplicate_candidates[0].origin_match);
        assert!(outcome.duplicate_candidates[0].head_match);
        assert_ne!(
            outcome.registration.project_id,
            outcome.duplicate_candidates[0].project_id
        );
    }

    #[test]
    fn gitlink_scan_records_child_uuid_sha_and_path_without_mutating_child_state() {
        let fixture = tempdir().unwrap();
        let parent = fixture.path().join("parent");
        let child = parent.join("vendor/child");
        fs::create_dir_all(&child).unwrap();
        init_git(&parent, "https://example.invalid/parent.git");
        init_git(&child, "https://example.invalid/child.git");
        let child_project_id = Uuid::new_v4().to_string();
        write_manifest(&child, &child_project_id);
        commit_all(&child, "child identity");
        let child_head = git_output(&child, &["rev-parse", "HEAD"]).unwrap();
        assert!(Command::new("git")
            .args(["update-index", "--add", "--cacheinfo", "160000"])
            .arg(&child_head)
            .arg("vendor/child")
            .current_dir(&parent)
            .status()
            .unwrap()
            .success());

        let scanned = scan_submodule_pins(&parent).unwrap();
        assert!(scanned.skipped.is_empty());
        assert_eq!(
            scanned.pins,
            vec![SubmodulePin {
                path: "vendor/child".into(),
                child_project_id,
                gitlink_sha: child_head,
            }]
        );
        assert!(child.join(".prometheus/project.json").exists());
    }

    #[test]
    fn standalone_replica_wins_the_authority_order() {
        let now = Utc::now();
        let project_id = Uuid::new_v4().to_string();
        let machine_id = Uuid::new_v4().to_string();
        let registration = |kind| ReplicaRegistration {
            project_id: project_id.clone(),
            replica_id: Uuid::new_v4().to_string(),
            machine_id: machine_id.clone(),
            kind,
            parent: None,
            head: None,
            origin: None,
            read_only: false,
            read_only_reason: None,
            registered_at: now,
            updated_at: now,
        };
        let mut document = RegistryDocument::new();
        document
            .replicas
            .insert("/embedded".into(), registration(ReplicaKind::Submodule));
        document
            .replicas
            .insert("/canonical".into(), registration(ReplicaKind::Standalone));
        assert_eq!(
            document.authoritative_replica(&project_id).unwrap().0,
            "/canonical"
        );
    }

    #[test]
    fn adoption_of_submodule_is_backed_up_provenanced_and_reversible() {
        let fixture = tempdir().unwrap();
        let data_root = fixture.path().join("data");
        let registry = ProjectRegistry::open_at(&data_root);
        let target_id = Uuid::new_v4().to_string();
        let source_id = Uuid::new_v4().to_string();

        let target = fixture.path().join("target");
        fs::create_dir_all(&target).unwrap();
        init_git(&target, "https://example.invalid/target.git");
        write_manifest(&target, &target_id);
        registry.register_existing(&target).unwrap();

        let source_repository = fixture.path().join("source-repository");
        fs::create_dir_all(&source_repository).unwrap();
        init_git(&source_repository, "https://example.invalid/source.git");
        write_manifest(&source_repository, &source_id);
        commit_all(&source_repository, "declare source identity");

        let parent = fixture.path().join("parent");
        fs::create_dir_all(&parent).unwrap();
        init_git(&parent, "https://example.invalid/parent.git");
        assert!(Command::new("git")
            .args(["-c", "protocol.file.allow=always", "submodule", "add", "-q"])
            .arg(&source_repository)
            .arg("embedded")
            .current_dir(&parent)
            .status()
            .unwrap()
            .success());
        commit_all(&parent, "embed source");
        let embedded = parent.join("embedded").canonicalize().unwrap();
        let source_registration = registry.register_existing(&embedded).unwrap();
        assert_eq!(
            source_registration.registration.kind,
            ReplicaKind::Submodule
        );

        let seed = fixture.path().join("seed");
        fs::create_dir_all(&seed).unwrap();
        let source_runtime = Runtime::open(&seed);
        source_runtime
            .initialize(
                &source_id,
                "source-run",
                crate::Actor {
                    kind: crate::ActorKind::Operator,
                    id: "adoption-test".into(),
                    device: "test-device".into(),
                    harness: "test-harness".into(),
                    session: "test-session".into(),
                },
            )
            .unwrap();
        let source_events = source_runtime.events().unwrap();
        let source_journal = registry.project_events_path(&source_id);
        write_event_journal(&source_journal, &source_events).unwrap();
        let original_journal = fs::read(&source_journal).unwrap();

        let plan = registry.plan_adoption(&embedded, &target_id).unwrap();
        assert_eq!(plan.source_kind, ReplicaKind::Submodule);
        assert_eq!(plan.target_kind, ReplicaKind::Standalone);
        assert_eq!(plan.source_event_count, source_events.len());
        assert_eq!(
            plan.source_event_hashes,
            source_events
                .iter()
                .map(|event| event.integrity_hash.clone())
                .collect::<Vec<_>>()
        );

        let signer = DeviceSigner::generate();
        let result = registry
            .apply_adoption_inner(&embedded, &target_id, Some(&signer))
            .unwrap();

        assert_eq!(fs::read(&source_journal).unwrap(), original_journal);
        assert_eq!(
            read_project_manifest(&embedded)
                .unwrap()
                .unwrap()
                .project_id,
            target_id
        );
        assert_eq!(
            read_project_manifest(&source_repository)
                .unwrap()
                .unwrap()
                .project_id,
            source_id
        );

        let migrated = read_events(Path::new(&result.migrated_journal)).unwrap();
        assert_eq!(migrated.len(), source_events.len());
        for (migrated, source) in migrated.iter().zip(&source_events) {
            assert_eq!(migrated.project_id, target_id);
            assert_eq!(migrated.signer_key_id.as_deref(), Some(signer.key_id()));
            let provenance = migrated.migration_provenance.as_ref().unwrap();
            assert_eq!(provenance.source_project_id, source_id);
            assert_eq!(
                provenance.source_replica_id.as_deref(),
                Some(source_registration.registration.replica_id.as_str())
            );
            assert_eq!(provenance.source_event_id, source.event_id);
            assert_eq!(provenance.source_integrity_hash, source.integrity_hash);
            assert_eq!(provenance.source_previous_hash, source.previous_hash);
        }

        let backup_manifest: AdoptionBackupManifest =
            serde_json::from_reader(File::open(&result.backup_manifest).unwrap()).unwrap();
        assert_eq!(backup_manifest.files.len(), 3);
        for entry in &backup_manifest.files {
            let backup_bytes = fs::read(&entry.backup).unwrap();
            assert_eq!(backup_bytes.len() as u64, entry.bytes);
            assert_eq!(format!("{:x}", Sha256::digest(&backup_bytes)), entry.sha256);
        }
        let backup_root = Path::new(&result.backup_manifest).parent().unwrap();
        assert!(backup_root.join("ROLLBACK.md").is_file());

        let document = registry.load().unwrap();
        let registered = document.replicas.get(embedded.to_str().unwrap()).unwrap();
        assert_eq!(registered.project_id, target_id);
        assert_eq!(registered.replica_id, result.registration.replica_id);
        assert_eq!(document.redirects.get(&source_id), Some(&result.redirect));
    }
}
