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
    Runtime, RuntimeError, EVENT_SCHEMA_VERSION,
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
        let evidence = inspect_replica(&canonical, &manifest, None)?;

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
            let evidence = inspect_replica(&source_path, &adopted_manifest, None)?;
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
        let evidence = inspect_replica(source_path, source_manifest, None)?;
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

#[derive(Debug)]
struct ReplicaEvidence {
    kind: ReplicaKind,
    parent: Option<ParentLinkage>,
    head: Option<String>,
    origin: Option<String>,
    read_only: bool,
}

fn inspect_replica(
    path: &Path,
    _manifest: &ProjectManifest,
    forced_kind: Option<ReplicaKind>,
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
    let read_only = matches!(kind, ReplicaKind::Bare | ReplicaKind::Ci);
    Ok(ReplicaEvidence {
        kind,
        parent,
        head: git_output(path, &["rev-parse", "HEAD"]),
        origin: git_output(path, &["config", "--get", "remote.origin.url"]),
        read_only,
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
    for (index, source) in source_events.iter().enumerate() {
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: into_project_id.into(),
            run_id: source.run_id.clone(),
            event_id: Uuid::new_v4().to_string(),
            command_id: source
                .command_id
                .as_ref()
                .map(|command_id| format!("adopt:{new_replica_id}:{command_id}")),
            revision: index as u64 + 1,
            expected_revision: index as u64,
            causal_parent: previous_event_id.clone(),
            actor: source.actor.clone(),
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
