//! Explicit compatibility edges and their checked-in, per-path operation plans.
//!
//! A registry edge is only routable when its concrete plan validates against
//! both the persisted historical ownership inventory and the newly rendered
//! target inventory. This keeps semantic upgrades closed to unlisted file
//! additions, removals, moves, and source changes.
use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::{
    bundle,
    model::{GeneratedFile, Ownership},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Migration {
    pub id: &'static str,
    pub from_version: &'static str,
    pub to_version: &'static str,
    plan_json: &'static str,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OperationKind {
    Add,
    Remove,
    Rename,
    Dependency,
    Codegen,
    Preserve,
    Update,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MigrationOperation {
    pub kind: OperationKind,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub before_source_digest: Option<String>,
    #[serde(default)]
    pub after_source_digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MigrationPlan {
    schema_version: u32,
    id: String,
    from_version: String,
    to_version: String,
    profile: String,
    generation_mode: String,
    pub operations: Vec<MigrationOperation>,
}

pub(crate) const ALPHA3_TO_ALPHA4: Migration = Migration {
    id: "builder-2.0.0-alpha.3-to-alpha.4",
    from_version: "2.0.0-alpha.3",
    to_version: "2.0.0-alpha.4",
    plan_json: include_str!("migration_plans/alpha3-alpha4-sovereign-hybrid.json"),
};

pub(crate) const REGISTRY: &[Migration] = &[ALPHA3_TO_ALPHA4];

/// Resolve exact, supported release transitions. No implicit semver range or
/// downgrade is accepted, and inconsistent applied-ID state is never repaired.
pub(crate) fn resolve(
    from: &str,
    to: &str,
    applied_ids: &[String],
) -> Result<Vec<&'static Migration>> {
    let mut applied = BTreeMap::new();
    for id in applied_ids {
        if applied.insert(id.as_str(), ()).is_some() {
            bail!("duplicate applied migration ID: {id}");
        }
        if !REGISTRY.iter().any(|migration| migration.id == id) {
            bail!("unknown applied migration ID: {id}");
        }
    }
    let mut version = from;
    let mut visited = BTreeMap::new();
    let mut result = Vec::new();
    while version != to {
        if visited.insert(version, ()).is_some() {
            bail!("cyclic migration registry at {version}");
        }
        let mut candidates = REGISTRY
            .iter()
            .filter(|migration| migration.from_version == version);
        let Some(migration) = candidates.next() else {
            bail!("no migration from {version} to {to} is registered");
        };
        if candidates.next().is_some() {
            bail!("ambiguous migration registry at {version}");
        }
        if applied.contains_key(migration.id) {
            bail!(
                "migration {} is recorded but source version is still {version}",
                migration.id
            );
        }
        result.push(migration);
        version = migration.to_version;
    }
    Ok(result)
}

/// Parse and prove that a migration's checked-in plan accounts for every
/// Builder-owned source and target record exactly once. The returned plan is
/// safe for the engine to execute directly; no implicit reconciliation remains.
pub(crate) fn validated_plan(
    migration: &Migration,
    profile: &str,
    generation_mode: &str,
    historical: &[GeneratedFile],
    desired: &[GeneratedFile],
) -> Result<MigrationPlan> {
    let plan: MigrationPlan = serde_json::from_str(migration.plan_json)
        .with_context(|| format!("invalid checked-in migration plan for {}", migration.id))?;
    validate_plan(
        migration,
        profile,
        generation_mode,
        historical,
        desired,
        &plan,
    )?;
    Ok(plan)
}

fn validate_plan(
    migration: &Migration,
    profile: &str,
    generation_mode: &str,
    historical: &[GeneratedFile],
    desired: &[GeneratedFile],
    plan: &MigrationPlan,
) -> Result<()> {
    if plan.schema_version != 1 {
        bail!(
            "unsupported migration plan schema {} for {}",
            plan.schema_version,
            migration.id
        );
    }
    if plan.id != migration.id
        || plan.from_version != migration.from_version
        || plan.to_version != migration.to_version
    {
        bail!(
            "migration plan identity does not match registry edge {}",
            migration.id
        );
    }
    if plan.profile != profile || plan.generation_mode != generation_mode {
        bail!(
            "migration {} supports only profile {}/{}; persisted project is {profile}/{generation_mode}",
            migration.id,
            plan.profile,
            plan.generation_mode
        );
    }
    if plan.operations.is_empty() {
        bail!("migration plan {} has no operations", migration.id);
    }

    let historical = builder_inventory(historical, migration.from_version, "historical")?;
    let desired = builder_inventory(desired, migration.to_version, "desired")?;
    let mut planned_from = BTreeMap::new();
    let mut planned_to = BTreeMap::new();

    for (index, operation) in plan.operations.iter().enumerate() {
        validate_operation_shape(operation, index)?;
        if let Some(path) = operation.from.as_deref() {
            let key = canonical_plan_path(path)?;
            if planned_from.insert(key.clone(), index).is_some() {
                bail!("migration plan has duplicate source path: {path}");
            }
            let file = historical.get(&key).with_context(|| {
                format!("migration plan source is not in historical ownership: {path}")
            })?;
            let expected = operation
                .before_source_digest
                .as_deref()
                .expect("operation shape validated");
            if file.source_digest != expected {
                bail!(
                    "migration source digest mismatch for {path}: plan {expected}, lock {}",
                    file.source_digest
                );
            }
        }
        if let Some(path) = operation.to.as_deref() {
            let key = canonical_plan_path(path)?;
            if planned_to.insert(key.clone(), index).is_some() {
                bail!("migration plan has duplicate target path: {path}");
            }
            let file = desired.get(&key).with_context(|| {
                format!("migration plan target is not in desired ownership: {path}")
            })?;
            let expected = operation
                .after_source_digest
                .as_deref()
                .expect("operation shape validated");
            if file.source_digest != expected {
                bail!(
                    "migration target digest mismatch for {path}: plan {expected}, rendered {}",
                    file.source_digest
                );
            }
        }
    }

    reject_unlisted("historical source", &historical, &planned_from)?;
    reject_unlisted("desired target", &desired, &planned_to)?;
    Ok(())
}

fn builder_inventory<'a>(
    files: &'a [GeneratedFile],
    expected_version: &str,
    label: &str,
) -> Result<BTreeMap<String, &'a GeneratedFile>> {
    let mut inventory = BTreeMap::new();
    for file in files {
        if !matches!(file.ownership, Ownership::Builder) {
            continue;
        }
        if file.version != expected_version {
            bail!(
                "{label} ownership record {} has version {}; expected {expected_version}",
                file.path,
                file.version
            );
        }
        validate_digest(
            &file.source_digest,
            &format!("{label} source digest for {}", file.path),
        )?;
        let key = canonical_plan_path(&file.path)?;
        if inventory.insert(key, file).is_some() {
            bail!(
                "{label} ownership inventory has duplicate path: {}",
                file.path
            );
        }
    }
    Ok(inventory)
}

fn validate_operation_shape(operation: &MigrationOperation, index: usize) -> Result<()> {
    let has_from = operation.from.is_some();
    let has_to = operation.to.is_some();
    let has_before = operation.before_source_digest.is_some();
    let has_after = operation.after_source_digest.is_some();
    let expected = match operation.kind {
        OperationKind::Add | OperationKind::Codegen => (false, true, false, true),
        OperationKind::Remove => (true, false, true, false),
        OperationKind::Rename
        | OperationKind::Dependency
        | OperationKind::Preserve
        | OperationKind::Update => (true, true, true, true),
    };
    if (has_from, has_to, has_before, has_after) != expected {
        bail!(
            "migration operation {index} has invalid fields for {:?}",
            operation.kind
        );
    }
    if let Some(digest) = operation.before_source_digest.as_deref() {
        validate_digest(digest, &format!("operation {index} beforeSourceDigest"))?;
    }
    if let Some(digest) = operation.after_source_digest.as_deref() {
        validate_digest(digest, &format!("operation {index} afterSourceDigest"))?;
    }
    match operation.kind {
        OperationKind::Dependency | OperationKind::Preserve | OperationKind::Update => {
            if operation.from != operation.to {
                bail!(
                    "migration operation {index} {:?} must keep the same path",
                    operation.kind
                );
            }
        }
        OperationKind::Rename => {
            if operation.from == operation.to {
                bail!("migration operation {index} rename must change the path");
            }
        }
        OperationKind::Add | OperationKind::Remove | OperationKind::Codegen => {}
    }
    if matches!(operation.kind, OperationKind::Preserve)
        && operation.before_source_digest != operation.after_source_digest
    {
        bail!("migration operation {index} preserve must keep the source digest");
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not a canonical SHA-256 digest");
    }
    Ok(())
}

fn canonical_plan_path(path: &str) -> Result<String> {
    if path.contains(['\\', ':']) {
        bail!("migration path is not canonical: {path}");
    }
    let relative = Path::new(path);
    bundle::validate_relative_path(relative)?;
    let key = relative.to_string_lossy().to_lowercase();
    if key == ".knowme-builder" || key.starts_with(".knowme-builder/") {
        bail!("migration path is reserved: {path}");
    }
    Ok(key)
}

fn reject_unlisted<T>(
    label: &str,
    inventory: &BTreeMap<String, T>,
    planned: &BTreeMap<String, usize>,
) -> Result<()> {
    if let Some(path) = inventory.keys().find(|path| !planned.contains_key(*path)) {
        bail!("migration plan omits {label} path: {path}");
    }
    if inventory.len() != planned.len() {
        bail!("migration plan {label} inventory does not match ownership state");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, digest: &str, version: &str) -> GeneratedFile {
        GeneratedFile {
            path: path.into(),
            template_id: format!("templates/{path}"),
            source_digest: digest.into(),
            last_installed_digest: digest.into(),
            ownership: Ownership::Builder,
            version: version.into(),
            render_name: None,
            render_kind: None,
        }
    }

    fn edge(plan_json: &'static str) -> Migration {
        Migration {
            id: "test-edge",
            from_version: "old",
            to_version: "new",
            plan_json,
        }
    }

    #[test]
    fn exact_registered_transition_and_repeated_target() {
        let chain = resolve("2.0.0-alpha.3", "2.0.0-alpha.4", &[]).unwrap();
        assert_eq!(chain, vec![&ALPHA3_TO_ALPHA4]);
        assert!(
            resolve(
                "2.0.0-alpha.4",
                "2.0.0-alpha.4",
                &[ALPHA3_TO_ALPHA4.id.into()]
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn unknown_history_downgrade_and_inconsistent_ids_fail_closed() {
        assert!(resolve("2.0.0-alpha.2", "2.0.0-alpha.4", &[]).is_err());
        assert!(resolve("2.0.0-alpha.4", "2.0.0-alpha.3", &[]).is_err());
        assert!(
            resolve(
                "2.0.0-alpha.3",
                "2.0.0-alpha.4",
                &[ALPHA3_TO_ALPHA4.id.into()]
            )
            .is_err()
        );
        assert!(resolve("2.0.0-alpha.4", "2.0.0-alpha.4", &["invented".into()]).is_err());
        assert!(
            resolve(
                "2.0.0-alpha.4",
                "2.0.0-alpha.4",
                &[ALPHA3_TO_ALPHA4.id.into(), ALPHA3_TO_ALPHA4.id.into()]
            )
            .is_err()
        );
    }

    #[test]
    fn plan_rejects_unlisted_inventory_and_invalid_pairs() {
        const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        const VALID: &str = r#"{
          "schemaVersion": 1,
          "id": "test-edge",
          "fromVersion": "old",
          "toVersion": "new",
          "profile": "sovereign-hybrid",
          "generationMode": "runnable",
          "operations": [{
            "kind": "update",
            "from": "old.txt",
            "to": "old.txt",
            "beforeSourceDigest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "afterSourceDigest": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
          }]
        }"#;
        let migration = edge(VALID);
        let old = vec![file("old.txt", DIGEST_A, "old")];
        let desired = vec![file("old.txt", DIGEST_B, "new")];
        assert!(validated_plan(&migration, "sovereign-hybrid", "runnable", &old, &desired).is_ok());

        let mut unlisted = old.clone();
        unlisted.push(file("unlisted.txt", DIGEST_A, "old"));
        let error = validated_plan(
            &migration,
            "sovereign-hybrid",
            "runnable",
            &unlisted,
            &desired,
        )
        .unwrap_err();
        assert!(error.to_string().contains("omits historical source"));

        const INVALID_PAIR: &str = r#"{
          "schemaVersion": 1,
          "id": "test-edge",
          "fromVersion": "old",
          "toVersion": "new",
          "profile": "sovereign-hybrid",
          "generationMode": "runnable",
          "operations": [{
            "kind": "update",
            "from": "old.txt",
            "to": "other.txt",
            "beforeSourceDigest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "afterSourceDigest": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
          }]
        }"#;
        let invalid = edge(INVALID_PAIR);
        let error =
            validated_plan(&invalid, "sovereign-hybrid", "runnable", &old, &desired).unwrap_err();
        assert!(error.to_string().contains("must keep the same path"));
    }
}
