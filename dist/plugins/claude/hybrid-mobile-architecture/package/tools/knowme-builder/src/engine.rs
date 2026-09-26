use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use include_dir::{Dir, DirEntry};
use sha2::{Digest, Sha256};
use tempfile::Builder as TempBuilder;

use crate::{
    bundle::{self, SKILLS, TEMPLATES},
    cli::{
        AddArgs, AdoptArgs, Cli, Command, CompletionShell, CompletionsArgs, DoctorArgs,
        ManifestArgs, ManifestCommand, NewArgs, SkillsArgs, SkillsCommand, UpgradeArgs,
    },
    model::{
        BUILDER_VERSION, CapabilityInstance, CommandResult, GeneratedFile, GeneratedLock,
        Ownership, PROMETHEUS_CONTRACT, PROMETHEUS_PACKAGE_VERSION, ProjectManifest,
    },
};

pub fn execute(cli: Cli) -> Result<()> {
    let json = cli.json;
    let result = match cli.command {
        Command::New(args) => create_project(args),
        Command::Adopt(args) => adopt_project(args),
        Command::Upgrade(args) => upgrade_project(args),
        Command::Add(args) => add_capability(args),
        Command::Skills(args) => manage_skills(args),
        Command::Audit(args) => audit_project(args.path, args.profile),
        Command::Doctor(args) => doctor(args),
        Command::Manifest(args) => manage_manifest(args),
        Command::Completions(args) => return completions(args),
    }?;
    print_result(&result, json)?;
    if !result.ok {
        bail!(
            "{} did not complete; inspect conflicts and diagnostics",
            result.operation
        );
    }
    Ok(())
}

fn add_capability(args: AddArgs) -> Result<CommandResult> {
    let destination = absolute_path(&args.path)?;
    crate::upgrade_journal::ensure_ready(&destination)?;
    let project_path = destination.join(".knowme-builder/project.toml");
    let lock_path = destination.join(".knowme-builder/generated.lock.json");
    let project_before = fs::read(&project_path)
        .context("project is not adopted; run `knowme-builder adopt --check` first")?;
    let lock_before = fs::read(&lock_path).context("project generated lock is missing")?;
    let mut project: ProjectManifest = toml::from_str(std::str::from_utf8(&project_before)?)?;
    let mut lock: GeneratedLock = serde_json::from_slice(&lock_before)?;
    validate_project_state(&project)?;
    if lock.schema_version != 1 || lock.builder_version != BUILDER_VERSION {
        bail!("upgrade the project before adding capabilities");
    }
    if !matches!(args.kind, crate::cli::AddKind::Feature) {
        bail!(
            "{} additions have no certified architecture adapter in this release; no project files were changed",
            args.kind.as_str()
        );
    }
    let kind = args.kind.as_str();
    let name = args.name.unwrap_or_else(|| kind.to_owned());
    let safe_name = slug(&name);
    if safe_name.is_empty() {
        bail!("capability name must contain an alphanumeric character");
    }
    if project.capabilities.iter().any(|item| item.id == safe_name) {
        bail!("capability already exists: {kind}/{safe_name}");
    }
    let mut result = CommandResult::new(format!("add-{kind}"));
    result.path = Some(destination.display().to_string());
    result.profile = Some(project.profile.as_str().to_owned());
    let descriptor_template = TEMPLATES
        .get_file("add/capability.json")
        .context("capability descriptor template is not packaged")?;
    let descriptor = capability_descriptor_bytes(&safe_name, &name, kind)?;
    let registries = capability_registry_paths(&project);
    if registries.is_empty() {
        bail!("no recognized capability registry exists for this project architecture");
    }
    project.capabilities.push(CapabilityInstance {
        id: safe_name.clone(),
        name: name.clone(),
        kind: kind.to_owned(),
        version: BUILDER_VERSION.to_owned(),
    });
    project
        .capabilities
        .sort_by(|left, right| left.id.cmp(&right.id));

    let mut changes = Vec::new();
    for registry in registries {
        ensure_no_symlinks(&destination, Path::new(registry))?;
        let registry_path = destination.join(registry);
        let before = read_optional(&registry_path)?;
        let Some(before_bytes) = before.as_ref() else {
            result.conflicts.push(registry.to_owned());
            continue;
        };
        if !lock.files.iter().any(|file| file.path == registry)
            && matches!(
                project.generation_mode,
                crate::cli::GenerationMode::Skeleton
            )
            && *before_bytes == capability_registry_bytes(&[])?
        {
            let template_id = capability_registry_template_id(&project, registry)?;
            let source = TEMPLATES
                .get_file(&template_id)
                .with_context(|| format!("missing capability registry template {template_id}"))?
                .contents();
            lock.files.push(GeneratedFile {
                path: registry.to_owned(),
                template_id,
                source_digest: digest(source),
                last_installed_digest: digest(before_bytes),
                ownership: Ownership::Builder,
                version: BUILDER_VERSION.to_owned(),
                render_name: None,
                render_kind: None,
            });
            result.actions.push(format!(
                "adopt recognized empty capability registry {registry}"
            ));
        }
        let Some(owned) = lock.files.iter_mut().find(|file| file.path == registry) else {
            result.conflicts.push(registry.to_owned());
            continue;
        };
        if !matches!(owned.ownership, Ownership::Builder)
            || digest(before_bytes) != owned.last_installed_digest
        {
            result.conflicts.push(registry.to_owned());
            continue;
        }
        let registry_after = capability_registry_bytes(&project.capabilities)?;
        owned.last_installed_digest = digest(&registry_after);
        owned.version = BUILDER_VERSION.to_owned();
        changes.push(crate::upgrade_journal::PlannedChange {
            path: registry_path,
            before,
            after: Some(registry_after),
        });

        let descriptor_relative = Path::new(registry)
            .parent()
            .context("capability registry has no parent")?
            .join(format!("{kind}-{safe_name}.json"));
        ensure_no_symlinks(&destination, &descriptor_relative)?;
        let descriptor_path = destination.join(&descriptor_relative);
        if descriptor_path.exists()
            || lock.files.iter().any(|file| {
                file.path
                    .eq_ignore_ascii_case(&descriptor_relative.to_string_lossy())
            })
        {
            result
                .conflicts
                .push(descriptor_relative.to_string_lossy().replace('\\', "/"));
            continue;
        }
        let descriptor_relative = descriptor_relative.to_string_lossy().replace('\\', "/");
        lock.files.push(GeneratedFile {
            path: descriptor_relative.clone(),
            template_id: "add/capability.json".to_owned(),
            source_digest: digest(descriptor_template.contents()),
            last_installed_digest: digest(&descriptor),
            ownership: Ownership::Builder,
            version: BUILDER_VERSION.to_owned(),
            render_name: Some(name.clone()),
            render_kind: Some(kind.to_owned()),
        });
        changes.push(crate::upgrade_journal::PlannedChange {
            path: descriptor_path,
            before: None,
            after: Some(descriptor.clone()),
        });
        result
            .actions
            .push(format!("register {descriptor_relative}"));
    }
    result.ok = result.conflicts.is_empty();
    if !result.ok {
        result
            .warnings
            .push("capability integration conflicts prevented all writes".to_owned());
        return Ok(result);
    }
    if args.check {
        return Ok(result);
    }
    let project_after = toml::to_string_pretty(&project)?.into_bytes();
    let lock_after = format!("{}\n", serde_json::to_string_pretty(&lock)?).into_bytes();
    changes.push(crate::upgrade_journal::PlannedChange {
        path: project_path.clone(),
        before: Some(project_before.clone()),
        after: Some(project_after),
    });
    changes.push(crate::upgrade_journal::PlannedChange {
        path: lock_path.clone(),
        before: Some(lock_before.clone()),
        after: Some(lock_after),
    });
    result.changed = crate::upgrade_journal::apply_changes(
        &destination,
        changes,
        &[
            (project_path.as_path(), project_before.as_slice()),
            (lock_path.as_path(), lock_before.as_slice()),
        ],
        &[],
    )?;
    Ok(result)
}

fn capability_registry_paths(project: &ProjectManifest) -> Vec<&'static str> {
    // Every maintained profile has a shared Rust application core even when
    // `rust-core` is not advertised as a user-facing surface.
    let mut paths = vec!["rust/capabilities/index.json"];
    for surface in &project.enabled_surfaces {
        let path = match surface.as_str() {
            "flutter-mobile" => "mobile/assets/capabilities/index.json",
            "tauri-desktop" => "desktop/public/capabilities/index.json",
            "rust-core" => "rust/capabilities/index.json",
            "react-web" => "web/public/capabilities/index.json",
            "axum-bff" => "server/capabilities/index.json",
            _ => continue,
        };
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

fn capability_registry_template_id(project: &ProjectManifest, registry: &str) -> Result<String> {
    let baseline = match registry {
        "mobile/assets/capabilities/index.json" => "flutter",
        "desktop/public/capabilities/index.json" => "tauri",
        "web/public/capabilities/index.json" | "server/capabilities/index.json" => "web",
        "rust/capabilities/index.json" => match project.profile {
            crate::cli::Profile::SovereignHybrid | crate::cli::Profile::FlutterMobile => "flutter",
            crate::cli::Profile::TauriDesktop => "tauri",
            crate::cli::Profile::GovernedWebShell | crate::cli::Profile::AxumWeb => "web",
        },
        _ => bail!("unsupported capability registry mapping: {registry}"),
    };
    Ok(format!("baselines/{baseline}/{registry}"))
}

fn capability_registry_bytes(capabilities: &[CapabilityInstance]) -> Result<Vec<u8>> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Registry<'a> {
        schema_version: u32,
        capabilities: Vec<Entry<'a>>,
    }
    #[derive(serde::Serialize)]
    struct Entry<'a> {
        id: &'a str,
        name: &'a str,
        kind: &'a str,
    }
    let value = Registry {
        schema_version: 1,
        capabilities: capabilities
            .iter()
            .map(|item| Entry {
                id: &item.id,
                name: &item.name,
                kind: &item.kind,
            })
            .collect(),
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?).into_bytes())
}

fn capability_descriptor_bytes(id: &str, name: &str, kind: &str) -> Result<Vec<u8>> {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Descriptor<'a> {
        schema_version: u32,
        id: &'a str,
        name: &'a str,
        kind: &'a str,
        builder_version: &'static str,
    }
    let value = Descriptor {
        schema_version: 1,
        id,
        name,
        kind,
        builder_version: BUILDER_VERSION,
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?).into_bytes())
}

fn manage_skills(args: SkillsArgs) -> Result<CommandResult> {
    let (path, check, force) = match args.command {
        SkillsCommand::Install { path, check, force } => (path, check, force),
        SkillsCommand::Check { path } => (path, true, false),
    };
    let destination = absolute_path(&path)?;
    let manifest = bundle::manifest()?;
    let harness_roots = [
        ".claude/skills",
        ".codex/skills",
        ".opencode/skills",
        ".kimi-code/skills",
        ".kimi/skills",
        ".agents/skills",
    ];
    let mut result = CommandResult::new(if check {
        "skills-check"
    } else {
        "skills-install"
    });
    result.path = Some(destination.display().to_string());
    for harness in harness_roots {
        for skill in &manifest.skills {
            let source = SKILLS
                .get_dir(skill)
                .with_context(|| format!("packaged skill is missing: {skill}"))?;
            let target = destination.join(harness).join(skill);
            let drift = dir_differs(source, source, &target)?;
            if !drift {
                continue;
            }
            result.actions.push(format!("sync {harness}/{skill}"));
            if target.exists() {
                if force && !check {
                    let backup = destination
                        .join(".knowme-builder/backups/skills")
                        .join(harness)
                        .join(skill);
                    if !backup.exists() {
                        if let Some(parent) = backup.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::rename(&target, &backup)?;
                        result.warnings.push(format!(
                            "previous payload preserved at {}",
                            backup.display()
                        ));
                    } else {
                        fs::remove_dir_all(&target)?;
                        result.warnings.push(format!(
                            "initial pre-Builder payload remains preserved at {}",
                            backup.display()
                        ));
                    }
                    write_embedded_dir(source, source, &target)?;
                    result.changed = true;
                } else {
                    result.conflicts.push(format!("{harness}/{skill}"));
                }
                if !check && !force {
                    let proposed = destination
                        .join(".knowme-builder/conflicts/skills")
                        .join(harness)
                        .join(skill);
                    write_embedded_dir(source, source, &proposed)?;
                    result.changed = true;
                }
            } else if !check {
                write_embedded_dir(source, source, &target)?;
                result.changed = true;
            }
        }
    }
    if !check {
        let lock_path = destination.join("skills-lock.json");
        let lock = skills_lock_bytes_preserving_third_party(&manifest.skills, &lock_path)?;
        if !lock_path.is_file() || fs::read(&lock_path)? != lock {
            atomic_write(&lock_path, &lock)?;
            result.actions.push("refresh skills-lock.json".to_owned());
            result.changed = true;
        }
    }
    result.ok = result.conflicts.is_empty();
    Ok(result)
}

fn audit_project(
    path: PathBuf,
    requested_profile: Option<crate::cli::Profile>,
) -> Result<CommandResult> {
    let destination = absolute_path(&path)?;
    let mut result = CommandResult::new("audit");
    result.path = Some(destination.display().to_string());
    let project_path = destination.join(".knowme-builder/project.toml");
    let project: Option<ProjectManifest> = if project_path.exists() {
        Some(toml::from_str(&fs::read_to_string(&project_path)?)?)
    } else {
        None
    };
    let profile = requested_profile
        .or_else(|| project.as_ref().map(|value| value.profile))
        .context("audit requires --profile or adopted project state")?;
    result.profile = Some(profile.as_str().to_owned());
    let required = match profile {
        crate::cli::Profile::SovereignHybrid => {
            vec!["rust", "desktop", "mobile", ".knowme-builder/project.toml"]
        }
        crate::cli::Profile::GovernedWebShell => {
            vec!["server", "web", ".knowme-builder/project.toml"]
        }
        crate::cli::Profile::FlutterMobile => {
            vec!["mobile", "rust", ".knowme-builder/project.toml"]
        }
        crate::cli::Profile::TauriDesktop => {
            vec!["desktop", "rust", ".knowme-builder/project.toml"]
        }
        crate::cli::Profile::AxumWeb => {
            vec!["server", "web", ".knowme-builder/project.toml"]
        }
    };
    for required_path in required {
        if !destination.join(required_path).exists() {
            result.ok = false;
            result
                .warnings
                .push(format!("missing profile surface: {required_path}"));
        }
    }
    for prohibited in [
        ".kbd-orchestrator/current-waypoint.json",
        ".kbd-orchestrator/progress.json",
        ".kbd-orchestrator/position.json",
    ] {
        if destination.join(prohibited).is_file() {
            result.warnings.push(format!(
                "compatibility projection must remain read-only: {prohibited}"
            ));
        }
    }
    Ok(result)
}

fn doctor(args: DoctorArgs) -> Result<CommandResult> {
    let destination = absolute_path(&args.path)?;
    let mut result = CommandResult::new("doctor");
    result.path = Some(destination.display().to_string());
    crate::native::inspect(&destination, &args.target, &mut result);
    if args.native_only {
        return Ok(result);
    }
    let manifest = bundle::manifest()?;
    let prometheus_contract = bundle::prometheus_contract()?;
    let _uar = bundle::uar_contract()?;
    result.actions.push(format!(
        "Builder {} manifest {} is internally consistent across {} harnesses",
        manifest.package.id,
        manifest.package.version,
        manifest.supported_harnesses.len()
    ));

    let required_package = prometheus_contract
        .get("minimumPackageVersion")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(PROMETHEUS_PACKAGE_VERSION);
    let required_contract = prometheus_contract
        .get("minimumContractVersion")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(PROMETHEUS_CONTRACT);

    match std::process::Command::new("prometheus")
        .arg("--version")
        .current_dir(&destination)
        .output()
    {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let package_version = prometheus_package_version(&text).unwrap_or_default();
            if version_at_least(package_version, required_package) {
                result.actions.push(format!(
                    "Prometheus package {package_version} is compatible (required {required_package})"
                ));
            } else {
                result.ok = false;
                result.warnings.push(format!(
                    "Prometheus package {package_version} is older than required {required_package}"
                ));
            }
        }
        Ok(output) => {
            result.ok = false;
            result.warnings.push(format!(
                "Prometheus package version probe failed with status {}",
                output.status
            ));
        }
        Err(error) => {
            result.ok = false;
            result
                .warnings
                .push(format!("Prometheus CLI is unavailable: {error}"));
        }
    }

    let prometheus = std::process::Command::new("prometheus")
        .args(["doctor", "--json"])
        .current_dir(&destination)
        .output();
    match prometheus {
        Ok(output) => {
            let assessment = assess_prometheus_doctor(
                &output.stdout,
                output.status.success(),
                required_contract,
            );
            result.ok &= assessment.ok;
            result.actions.extend(assessment.actions);
            result.warnings.extend(assessment.warnings);
        }
        Err(error) => {
            result.ok = false;
            result.warnings.push(format!(
                "Prometheus operational doctor is unavailable: {error}"
            ));
        }
    }

    let kbd_help = command_stdout(&destination, &["kbd", "--help"]);
    let learning_help = command_stdout(&destination, &["learning", "--help"]);
    let missing = missing_prometheus_capabilities(&kbd_help, &learning_help);
    if missing.is_empty() {
        result
            .actions
            .push("Prometheus typed KBD and durable learning commands are available".to_owned());
    } else {
        result.ok = false;
        result.warnings.push(format!(
            "Prometheus CLI lacks required capabilities: {}",
            missing.join(", ")
        ));
    }
    Ok(result)
}

fn prometheus_package_version(output: &str) -> Option<&str> {
    output
        .split_whitespace()
        .map(|candidate| {
            candidate.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && character != '.' && character != '-'
            })
        })
        .map(|candidate| candidate.strip_prefix('v').unwrap_or(candidate))
        .find(|candidate| version_at_least(candidate, "0.0.0"))
}

#[derive(Debug, PartialEq, Eq)]
struct PrometheusDoctorAssessment {
    ok: bool,
    actions: Vec<String>,
    warnings: Vec<String>,
}

fn assess_prometheus_doctor(
    output: &[u8],
    status_success: bool,
    required_contract: &str,
) -> PrometheusDoctorAssessment {
    let mut assessment = PrometheusDoctorAssessment {
        ok: true,
        actions: Vec::new(),
        warnings: Vec::new(),
    };
    let parsed: serde_json::Value = match serde_json::from_slice(output) {
        Ok(parsed) => parsed,
        Err(error) => {
            assessment.ok = false;
            assessment
                .warnings
                .push(format!("Prometheus doctor returned invalid JSON: {error}"));
            return assessment;
        }
    };
    let contract = parsed
        .get("contractVersion")
        .or_else(|| parsed.pointer("/controlPlane/contractVersion"))
        .and_then(serde_json::Value::as_str);
    match contract {
        None => {
            assessment.ok = false;
            assessment
                .warnings
                .push("Prometheus doctor lacks a machine-readable contractVersion".to_owned());
        }
        Some(contract) if !version_at_least(contract, required_contract) => {
            assessment.ok = false;
            assessment.warnings.push(format!(
                "Prometheus contract {contract} is older than required {required_contract}"
            ));
        }
        Some(contract) => assessment.actions.push(format!(
            "Prometheus control-plane contract {contract} is compatible"
        )),
    }
    if status_success {
        assessment
            .actions
            .push("Prometheus doctor passed".to_owned());
    } else {
        assessment.ok = false;
        let failed = parsed
            .pointer("/summary/failed")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        assessment.warnings.push(format!(
            "Prometheus doctor reported {failed} failing health check(s)"
        ));
    }
    assessment
}

fn command_stdout(destination: &Path, arguments: &[&str]) -> String {
    std::process::Command::new("prometheus")
        .args(arguments)
        .current_dir(destination)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}

fn missing_prometheus_capabilities(kbd_help: &str, learning_help: &str) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for command in [
        "status",
        "projects",
        "register",
        "replicas",
        "adopt",
        "conflicts",
        "resolve",
        "claim",
        "pause",
        "revise",
        "resume",
        "cancel",
        "audit",
        "phase",
        "stage",
        "change",
        "task",
        "completion",
        "decision",
        "blocker",
    ] {
        if !kbd_help
            .lines()
            .any(|line| line.split_whitespace().next() == Some(command))
        {
            missing.push(command);
        }
    }
    if !learning_help
        .lines()
        .any(|line| line.split_whitespace().next() == Some("status"))
    {
        missing.push("learning-status");
    }
    missing
}

fn version_at_least(actual: &str, required: &str) -> bool {
    fn parts(value: &str) -> Option<(u64, u64, u64)> {
        let stable = value.split_once('-').map_or(value, |(prefix, _)| prefix);
        let mut values = stable.split('.').map(str::parse::<u64>);
        Some((
            values.next()?.ok()?,
            values.next()?.ok()?,
            values.next()?.ok()?,
        ))
    }
    matches!((parts(actual), parts(required)), (Some(actual), Some(required)) if actual >= required)
}

fn manage_manifest(args: ManifestArgs) -> Result<CommandResult> {
    let mut result = CommandResult::new("manifest");
    let manifest = bundle::manifest()?;
    let _prometheus = bundle::prometheus_contract()?;
    let _uar = bundle::uar_contract()?;
    result.actions.push(format!(
        "validated embedded manifest {} {}",
        manifest.package.id, manifest.package.version
    ));
    let root = std::env::current_dir()?;
    let script = root.join("scripts/generate-builder-manifests.mjs");
    match args.command {
        ManifestCommand::Check if script.is_file() => {
            let status = std::process::Command::new("node")
                .arg(&script)
                .arg("--check")
                .current_dir(&root)
                .status()?;
            if !status.success() {
                result.ok = false;
                result
                    .warnings
                    .push("generated manifests drifted".to_owned());
            }
        }
        ManifestCommand::Generate if script.is_file() => {
            let status = std::process::Command::new("node")
                .arg(&script)
                .current_dir(&root)
                .status()?;
            if !status.success() {
                bail!("manifest generator failed");
            }
            result.changed = true;
        }
        ManifestCommand::Generate => {
            bail!("manifest generation requires the Builder source checkout");
        }
        ManifestCommand::Check => {}
    }
    Ok(result)
}

fn completions(args: CompletionsArgs) -> Result<()> {
    let shell = match args.shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Elvish => Shell::Elvish,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::PowerShell => Shell::PowerShell,
        CompletionShell::Zsh => Shell::Zsh,
    };
    let mut command = crate::cli::Cli::command();
    generate(
        shell,
        &mut command,
        "knowme-builder",
        &mut std::io::stdout(),
    );
    Ok(())
}

fn dir_differs(root: &Dir<'_>, dir: &Dir<'_>, target: &Path) -> Result<bool> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => {
                if dir_differs(root, child, target)? {
                    return Ok(true);
                }
            }
            DirEntry::File(file) => {
                let relative = file.path().strip_prefix(root.path())?;
                let installed = target.join(relative);
                if !installed.is_file() || fs::read(&installed)? != file.contents() {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn write_embedded_dir(root: &Dir<'_>, dir: &Dir<'_>, destination: &Path) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => write_embedded_dir(root, child, destination)?,
            DirEntry::File(file) => {
                let relative = file.path().strip_prefix(root.path())?;
                bundle::validate_relative_path(relative)?;
                atomic_write(&destination.join(relative), file.contents())?;
            }
        }
    }
    Ok(())
}

fn adopt_project(args: AdoptArgs) -> Result<CommandResult> {
    let destination = absolute_path(&args.path)?;
    if !destination.is_dir() {
        bail!(
            "adoption target must be an existing directory: {}",
            destination.display()
        );
    }
    let manifest = bundle::manifest()?;
    let profile = bundle::profile(&manifest, args.profile)?;
    let mut result = CommandResult::new("adopt");
    result.path = Some(destination.display().to_string());
    result.profile = Some(args.profile.as_str().to_owned());

    let existing_project = destination.join(".knowme-builder/project.toml");
    ensure_no_symlinks(&destination, Path::new(".knowme-builder/project.toml"))?;
    if existing_project.exists() {
        let existing: ProjectManifest = toml::from_str(&fs::read_to_string(&existing_project)?)?;
        validate_project_state(&existing)?;
        ensure_no_symlinks(
            &destination,
            Path::new(".knowme-builder/generated.lock.json"),
        )?;
        let existing_lock: GeneratedLock = serde_json::from_slice(
            &fs::read(destination.join(".knowme-builder/generated.lock.json"))
                .context("incomplete adoption state: generated lock is missing")?,
        )?;
        if existing_lock.schema_version != 1
            || existing_lock.builder_version != existing.builder_version
        {
            bail!("unsupported or inconsistent adoption lock; explicit migration required");
        }
        if existing.profile.as_str() != args.profile.as_str() {
            bail!("changing an adopted profile requires an explicit migration");
        }
        let proposed_skills = skills_lock_bytes(&manifest.skills)?;
        let trailing_controls = [
            (
                Path::new(&existing.policy_overlay_path),
                b"# Project-local policy overlay.\n# Consumer-specific roles, tenants, and release gates belong here.\n".as_slice(),
            ),
            (
                Path::new(".knowme-builder/activation-manifest.json"),
                bundle::activation_manifest(),
            ),
            (Path::new("skills-lock.json"), proposed_skills.as_slice()),
        ];
        let mut missing = Vec::new();
        // Validate the entire recovery plan before creating any missing file.
        // Existing policy and skill-lock contents remain owned by the application.
        for (relative, bytes) in trailing_controls {
            ensure_no_symlinks(&destination, relative)?;
            let path = destination.join(relative);
            if !path.exists() {
                result.actions.push(format!(
                    "restore missing control file {}",
                    relative.display()
                ));
                missing.push((path, bytes));
            } else if !path.is_file() {
                bail!(
                    "adoption control path is not a file: {}",
                    relative.display()
                );
            }
        }
        if !missing.is_empty() {
            if args.check {
                result.ok = false;
                result.warnings.push(
                    "incomplete adoption state; use adopt --apply to restore missing control files"
                        .to_owned(),
                );
            } else {
                for (path, bytes) in missing {
                    atomic_write(&path, bytes)?;
                    result.changed = true;
                }
            }
            return Ok(result);
        }
        result.actions.push(
            "already adopted; use audit to inspect surfaces and upgrade for managed changes"
                .to_owned(),
        );
        return Ok(result);
    }
    let (enabled_surfaces, unsupported_surfaces): (Vec<_>, Vec<_>) = profile
        .surfaces
        .iter()
        .cloned()
        .partition(|surface| surface_present(&destination, surface));
    result.warnings.push("adoption records detected manifests only; build, runtime and architecture remain unverified".to_owned());
    for surface in &unsupported_surfaces {
        result
            .warnings
            .push(format!("missing or unmapped surface: {surface}"));
    }
    let project = ProjectManifest {
        schema_version: 1,
        profile: args.profile,
        builder_version: BUILDER_VERSION.to_owned(),
        required_prometheus_contract: PROMETHEUS_CONTRACT.to_owned(),
        uar_mode: profile.uar_mode.clone(),
        enabled_surfaces,
        generation_mode: crate::cli::GenerationMode::Skeleton,
        policy_overlay_path: ".knowme-builder/policy-overlay.toml".to_owned(),
        unsupported_surfaces,
        app_name: None,
        capabilities: Vec::new(),
    };
    let state_dir = destination.join(".knowme-builder");
    let project_bytes = toml::to_string_pretty(&project)?;
    let lock = GeneratedLock {
        schema_version: 1,
        builder_version: BUILDER_VERSION.to_owned(),
        applied_migrations: Vec::new(),
        files: Vec::new(),
    };
    let lock_bytes = format!("{}\n", serde_json::to_string_pretty(&lock)?);
    let controls = [
        (state_dir.join("project.toml"), project_bytes.as_bytes()),
        (state_dir.join("generated.lock.json"), lock_bytes.as_bytes()),
        (state_dir.join("policy-overlay.toml"), b"# Project-local policy overlay.\n# Consumer-specific roles, tenants, and release gates belong here.\n".as_slice()),
        (state_dir.join("activation-manifest.json"), bundle::activation_manifest()),
    ];
    for (path, bytes) in &controls {
        ensure_no_symlinks(&destination, path.strip_prefix(&destination)?)?;
        plan_control_file(path, bytes, false, &mut result)?;
    }
    ensure_no_symlinks(&destination, Path::new("skills-lock.json"))?;
    let proposed_skills = skills_lock_bytes(&manifest.skills)?;
    let skills_lock_path = destination.join("skills-lock.json");
    if skills_lock_path.exists() && fs::read(&skills_lock_path)? != proposed_skills {
        result.conflicts.push("skills-lock.json".to_owned());
    }
    if !result.conflicts.is_empty() {
        result.ok = false;
        return Ok(result);
    }
    if args.apply {
        for (path, bytes) in controls {
            if !path.exists() {
                atomic_write(&path, bytes)?;
                result.changed = true;
            }
        }
    }
    plan_control_file(&skills_lock_path, &proposed_skills, args.apply, &mut result)?;
    Ok(result)
}

fn surface_present(root: &Path, surface: &str) -> bool {
    let manifest = match surface {
        "flutter-mobile" => "mobile/pubspec.yaml",
        "tauri-desktop" => "desktop/src-tauri/Cargo.toml",
        "rust-core" => "rust/Cargo.toml",
        "react-web" => "web/package.json",
        "axum-bff" => "server/Cargo.toml",
        _ => return false,
    };
    root.join(manifest).is_file()
}

fn validate_project_state(project: &ProjectManifest) -> Result<()> {
    if project.schema_version != 1 {
        bail!(
            "unsupported project schema {}; an explicit migration is required",
            project.schema_version
        );
    }
    if project.builder_version != BUILDER_VERSION {
        bail!(
            "unsupported Builder version {}; no migration to {} is registered",
            project.builder_version,
            BUILDER_VERSION
        );
    }
    Ok(())
}

fn rendering_identity(project: &ProjectManifest, requested: Option<String>) -> Result<String> {
    if project
        .app_name
        .as_deref()
        .is_some_and(|value| slug(value).is_empty())
    {
        bail!("persisted rendering identity must contain an alphanumeric character");
    }
    if requested
        .as_deref()
        .is_some_and(|value| slug(value).is_empty())
    {
        bail!("--app-name rendering identity must contain an alphanumeric character");
    }
    if let (Some(saved), Some(requested)) = (&project.app_name, &requested)
        && saved != requested
    {
        bail!("--app-name conflicts with persisted rendering identity");
    }
    project.app_name.clone().or(requested).context(
        "legacy state has no rendering identity; supply --app-name with the original generation name",
    )
}

pub(crate) fn ensure_no_symlinks(root: &Path, relative: &Path) -> Result<()> {
    bundle::validate_relative_path(relative)?;
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_symlink() => {
                bail!("managed path contains a symbolic link: {}", path.display())
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn upgrade_project(args: UpgradeArgs) -> Result<CommandResult> {
    let destination = absolute_path(&args.path)?;
    let state_dir = destination.join(".knowme-builder");
    if args.rollback {
        return crate::upgrade_journal::rollback(&destination);
    }
    crate::upgrade_journal::ensure_ready(&destination)?;
    let project_path = state_dir.join("project.toml");
    let lock_path = state_dir.join("generated.lock.json");
    let project_before = fs::read(&project_path)
        .with_context(|| format!("missing adoption state {}", project_path.display()))?;
    let lock_before = fs::read(&lock_path)
        .with_context(|| format!("missing generated lock {}", lock_path.display()))?;
    let mut project: ProjectManifest = toml::from_str(std::str::from_utf8(&project_before)?)
        .context("invalid .knowme-builder/project.toml")?;
    let mut lock: GeneratedLock = serde_json::from_slice(&lock_before)
        .context("invalid .knowme-builder/generated.lock.json")?;

    if lock.schema_version != 1 || lock.builder_version != project.builder_version {
        bail!(
            "unsupported or inconsistent generated lock schema/version; explicit migration required"
        );
    }
    if project.schema_version != 1 {
        bail!(
            "unsupported project schema {}; an explicit migration is required",
            project.schema_version
        );
    }
    if project.builder_version != BUILDER_VERSION {
        return migrate_project(
            &destination,
            &state_dir,
            project_path,
            lock_path,
            project_before,
            lock_before,
            project,
            lock,
            args,
        );
    }
    validate_project_state(&project)?;
    ensure_no_symlinks(&destination, Path::new(".knowme-builder"))?;
    let mut result = CommandResult::new("upgrade");
    result.path = Some(destination.display().to_string());
    result.profile = Some(project.profile.as_str().to_owned());
    let app_name = rendering_identity(&project, args.app_name)?;
    if lock.files.is_empty() {
        result.warnings.push("no generated application files are owned; brownfield integration requires an explicit migration".to_owned());
        return Ok(result);
    }
    let mut writes: Vec<crate::upgrade_journal::PlannedWrite> = Vec::new();
    let mut proposals: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut managed_paths = std::collections::BTreeSet::new();
    for file in &mut lock.files {
        if !matches!(file.ownership, Ownership::Builder) {
            continue;
        }
        let portable_path = file.path.replace('\\', "/");
        let relative = Path::new(&portable_path);
        ensure_no_symlinks(&destination, relative)?;
        let canonical = relative
            .components()
            .filter_map(|part| match part {
                std::path::Component::Normal(value) => Some(value.to_string_lossy()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/")
            .to_lowercase();
        if canonical == ".knowme-builder"
            || canonical.starts_with(".knowme-builder/")
            || !managed_paths.insert(canonical)
        {
            bail!(
                "duplicate or reserved managed application path: {}",
                file.path
            );
        }
        let target = destination.join(relative);
        let current = if target.exists() {
            Some(fs::read(&target)?)
        } else {
            None
        };
        let unmodified = current
            .as_ref()
            .is_none_or(|bytes| digest(bytes) == file.last_installed_digest);
        let template = TEMPLATES
            .get_file(file.template_id.replace('\\', "/"))
            .with_context(|| format!("missing template {}", file.template_id))?;
        let source = template.contents();
        let proposed = if file.path.ends_with("/capabilities/index.json") {
            capability_registry_bytes(&project.capabilities)?
        } else if file.template_id == "add/capability.json" {
            let name = file
                .render_name
                .as_deref()
                .context("capability ownership record has no rendering name")?;
            let kind = file
                .render_kind
                .as_deref()
                .context("capability ownership record has no kind")?;
            capability_descriptor_bytes(&slug(name), name, kind)?
        } else {
            let render_context = RenderContext {
                app_name: file.render_name.as_deref().unwrap_or(&app_name),
                profile: project.profile.as_str(),
                mode: project.generation_mode.as_str(),
                capability_kind: file.render_kind.as_deref(),
            };
            match std::str::from_utf8(source) {
                Ok(text) => render_text(text, &render_context).into_bytes(),
                Err(_) => source.to_vec(),
            }
        };
        let proposed_digest = digest(&proposed);
        if current
            .as_ref()
            .is_some_and(|bytes| digest(bytes) == proposed_digest)
        {
            file.source_digest = digest(source);
            file.last_installed_digest = proposed_digest;
            file.version = BUILDER_VERSION.to_owned();
            continue;
        }

        if unmodified {
            result.actions.push(format!("upgrade {}", file.path));
            writes.push((target, current, proposed));
            file.source_digest = digest(source);
            file.last_installed_digest = proposed_digest;
            file.version = BUILDER_VERSION.to_owned();
        } else {
            result.conflicts.push(file.path.clone());
            if args.apply {
                let sidecar = state_dir
                    .join("conflicts")
                    .join(relative)
                    .with_extension(format!(
                        "{}proposed",
                        relative
                            .extension()
                            .map(|value| format!("{}.", value.to_string_lossy()))
                            .unwrap_or_default()
                    ));
                ensure_no_symlinks(&destination, sidecar.strip_prefix(&destination)?)?;
                proposals.push((sidecar, proposed));
            }
        }
    }

    result.ok = result.conflicts.is_empty();
    if !result.ok {
        for (path, bytes) in proposals {
            atomic_write(&path, &bytes)?;
            result.changed = true;
        }
        result
            .warnings
            .push("conflicts prevented all managed application and version changes".to_owned());
        return Ok(result);
    }
    if args.apply {
        project.app_name = Some(app_name);
        lock.builder_version = BUILDER_VERSION.to_owned();
        let lock_bytes = format!("{}\n", serde_json::to_string_pretty(&lock)?);
        let project_bytes = toml::to_string_pretty(&project)?;
        for (path, before, bytes) in [
            (&lock_path, &lock_before, lock_bytes.as_bytes()),
            (&project_path, &project_before, project_bytes.as_bytes()),
        ] {
            if before != bytes {
                writes.push((path.clone(), Some(before.clone()), bytes.to_vec()));
            }
        }
        result.changed = crate::upgrade_journal::apply(
            &destination,
            writes,
            &[
                (lock_path.as_path(), lock_before.as_slice()),
                (project_path.as_path(), project_before.as_slice()),
            ],
        )?;
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn migrate_project(
    destination: &Path,
    state_dir: &Path,
    project_path: PathBuf,
    lock_path: PathBuf,
    project_before: Vec<u8>,
    lock_before: Vec<u8>,
    mut project: ProjectManifest,
    lock: GeneratedLock,
    args: UpgradeArgs,
) -> Result<CommandResult> {
    let chain = crate::migrations::resolve(
        &project.builder_version,
        BUILDER_VERSION,
        &lock.applied_migrations,
    )?;
    let [migration] = chain.as_slice() else {
        bail!("semantic migration requires exactly one concrete operation plan");
    };
    let migration_ids = chain
        .iter()
        .map(|migration| migration.id.to_owned())
        .collect::<Vec<_>>();
    let app_name = rendering_identity(&project, args.app_name)?;
    let (rendered, mut desired_lock) = render_current_profile(&project, &app_name)?;
    let plan = crate::migrations::validated_plan(
        migration,
        project.profile.as_str(),
        project.generation_mode.as_str(),
        &lock.files,
        &desired_lock.files,
    )?;
    let mut result = CommandResult::new("upgrade");
    result.path = Some(destination.display().to_string());
    result.profile = Some(project.profile.as_str().to_owned());

    let mut historical_by_path = BTreeMap::new();
    let mut retained_user_files = Vec::new();
    for file in &lock.files {
        let key = canonical_managed_key(&file.path)?;
        if !matches!(file.ownership, Ownership::Builder) {
            retained_user_files.push(file.clone());
            continue;
        }
        if historical_by_path.insert(key, file).is_some() {
            bail!(
                "duplicate or reserved managed application path: {}",
                file.path
            );
        }
    }
    let desired_by_path = desired_lock
        .files
        .iter()
        .filter(|file| matches!(file.ownership, Ownership::Builder))
        .map(|file| Ok((canonical_managed_key(&file.path)?, file)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    if desired_by_path.len()
        != desired_lock
            .files
            .iter()
            .filter(|file| matches!(file.ownership, Ownership::Builder))
            .count()
    {
        bail!("current profile templates contain duplicate paths");
    }
    for file in &retained_user_files {
        let key = canonical_managed_key(&file.path)?;
        if desired_by_path.contains_key(&key) {
            bail!(
                "user-owned historical path collides with migration target: {}",
                file.path
            );
        }
    }

    let mut changes = Vec::new();
    let mut migration_proposals = Vec::new();
    for operation in &plan.operations {
        use crate::migrations::OperationKind;

        match operation.kind {
            OperationKind::Add | OperationKind::Codegen => {
                let to = operation.to.as_deref().expect("validated migration target");
                let key = canonical_managed_key(to)?;
                let desired = desired_by_path
                    .get(&key)
                    .expect("validated desired migration record");
                let relative = Path::new(to);
                ensure_no_symlinks(destination, relative)?;
                let target = destination.join(relative);
                let current = read_optional(&target)?;
                let after = fs::read(rendered.path().join(&desired.path))?;
                if current.is_some() {
                    result.conflicts.push(to.to_owned());
                    migration_proposals.push((relative.to_path_buf(), after));
                    continue;
                }
                let verb = if matches!(operation.kind, OperationKind::Codegen) {
                    "generate"
                } else {
                    "add"
                };
                result.actions.push(format!("{verb} {to}"));
                changes.push(crate::upgrade_journal::PlannedChange {
                    path: target,
                    before: None,
                    after: Some(after),
                });
            }
            OperationKind::Remove => {
                let from = operation
                    .from
                    .as_deref()
                    .expect("validated migration source");
                let key = canonical_managed_key(from)?;
                let old = historical_by_path
                    .get(&key)
                    .expect("validated historical migration record");
                let relative = Path::new(from);
                ensure_no_symlinks(destination, relative)?;
                let target = destination.join(relative);
                let current = read_optional(&target)?;
                if !historical_file_is_pristine(old, current.as_deref()) {
                    result.conflicts.push(from.to_owned());
                    continue;
                }
                result.actions.push(format!("remove {from}"));
                changes.push(crate::upgrade_journal::PlannedChange {
                    path: target,
                    before: current,
                    after: None,
                });
            }
            OperationKind::Rename => {
                let from = operation
                    .from
                    .as_deref()
                    .expect("validated migration source");
                let to = operation.to.as_deref().expect("validated migration target");
                let old_key = canonical_managed_key(from)?;
                let desired_key = canonical_managed_key(to)?;
                let old = historical_by_path
                    .get(&old_key)
                    .expect("validated historical migration record");
                let desired = desired_by_path
                    .get(&desired_key)
                    .expect("validated desired migration record");
                let from_relative = Path::new(from);
                let to_relative = Path::new(to);
                ensure_no_symlinks(destination, from_relative)?;
                ensure_no_symlinks(destination, to_relative)?;
                let from_target = destination.join(from_relative);
                let to_target = destination.join(to_relative);
                let current = read_optional(&from_target)?;
                let to_current = read_optional(&to_target)?;
                let after = fs::read(rendered.path().join(&desired.path))?;
                let pristine = historical_file_is_pristine(old, current.as_deref());
                if !pristine || to_current.is_some() {
                    result.conflicts.push(if !pristine {
                        from.to_owned()
                    } else {
                        to.to_owned()
                    });
                    migration_proposals.push((to_relative.to_path_buf(), after));
                    continue;
                }
                result.actions.push(format!("rename {from} -> {to}"));
                changes.push(crate::upgrade_journal::PlannedChange {
                    path: from_target,
                    before: current,
                    after: None,
                });
                changes.push(crate::upgrade_journal::PlannedChange {
                    path: to_target,
                    before: None,
                    after: Some(after),
                });
            }
            OperationKind::Dependency | OperationKind::Preserve | OperationKind::Update => {
                let path = operation
                    .from
                    .as_deref()
                    .expect("validated migration source and target");
                let key = canonical_managed_key(path)?;
                let old = historical_by_path
                    .get(&key)
                    .expect("validated historical migration record");
                let desired = desired_by_path
                    .get(&key)
                    .expect("validated desired migration record");
                let relative = Path::new(path);
                ensure_no_symlinks(destination, relative)?;
                let target = destination.join(relative);
                let current = read_optional(&target)?;
                let after = fs::read(rendered.path().join(&desired.path))?;
                if current.as_deref() == Some(after.as_slice()) {
                    continue;
                }
                if !historical_file_is_pristine(old, current.as_deref()) {
                    result.conflicts.push(path.to_owned());
                    migration_proposals.push((relative.to_path_buf(), after));
                    continue;
                }
                let verb = match operation.kind {
                    OperationKind::Dependency => "update dependency manifest",
                    OperationKind::Preserve => "preserve",
                    OperationKind::Update => "update",
                    _ => unreachable!(),
                };
                result.actions.push(format!("{verb} {path}"));
                changes.push(crate::upgrade_journal::PlannedChange {
                    path: target,
                    before: current,
                    after: Some(after),
                });
            }
        }
    }
    result.ok = result.conflicts.is_empty();
    if !result.ok {
        if args.apply {
            for (relative, bytes) in migration_proposals {
                let sidecar = proposed_sidecar(state_dir, &relative);
                ensure_no_symlinks(destination, sidecar.strip_prefix(destination)?)?;
                atomic_write(&sidecar, &bytes)?;
                result.changed = true;
            }
        }
        result.warnings.push(
            "semantic migration conflicts prevented all application and version changes".to_owned(),
        );
        return Ok(result);
    }
    if !args.apply {
        return Ok(result);
    }

    desired_lock.files.extend(retained_user_files);
    desired_lock.builder_version = BUILDER_VERSION.to_owned();
    desired_lock.applied_migrations = lock.applied_migrations.clone();
    desired_lock
        .applied_migrations
        .extend(migration_ids.clone());
    project.builder_version = BUILDER_VERSION.to_owned();
    project.app_name = Some(app_name);
    let new_lock = format!("{}\n", serde_json::to_string_pretty(&desired_lock)?).into_bytes();
    let new_project = toml::to_string_pretty(&project)?.into_bytes();
    changes.push(crate::upgrade_journal::PlannedChange {
        path: lock_path.clone(),
        before: Some(lock_before.clone()),
        after: Some(new_lock),
    });
    changes.push(crate::upgrade_journal::PlannedChange {
        path: project_path.clone(),
        before: Some(project_before.clone()),
        after: Some(new_project),
    });
    result.changed = crate::upgrade_journal::apply_changes(
        destination,
        changes,
        &[
            (lock_path.as_path(), lock_before.as_slice()),
            (project_path.as_path(), project_before.as_slice()),
        ],
        &migration_ids,
    )?;
    if result.changed {
        result
            .actions
            .push(format!("record migrations {}", migration_ids.join(", ")));
    }
    let _ = state_dir;
    Ok(result)
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn historical_file_is_pristine(file: &GeneratedFile, current: Option<&[u8]>) -> bool {
    current.is_some_and(|bytes| digest(bytes) == file.last_installed_digest)
}

fn canonical_managed_key(path: &str) -> Result<String> {
    if path.contains(['\\', ':']) {
        bail!("managed path is not canonical: {path}");
    }
    let relative = Path::new(path);
    bundle::validate_relative_path(relative)?;
    let key = relative.to_string_lossy().to_lowercase();
    if key == ".knowme-builder" || key.starts_with(".knowme-builder/") {
        bail!("managed path is reserved: {path}");
    }
    Ok(key)
}

fn proposed_sidecar(state_dir: &Path, relative: &Path) -> PathBuf {
    state_dir
        .join("conflicts")
        .join(relative)
        .with_extension(format!(
            "{}proposed",
            relative
                .extension()
                .map(|value| format!("{}.", value.to_string_lossy()))
                .unwrap_or_default()
        ))
}

fn render_current_profile(
    project: &ProjectManifest,
    app_name: &str,
) -> Result<(tempfile::TempDir, GeneratedLock)> {
    let stage = TempBuilder::new().prefix("knowme-migration-").tempdir()?;
    let mut lock = GeneratedLock {
        schema_version: 1,
        builder_version: BUILDER_VERSION.to_owned(),
        applied_migrations: Vec::new(),
        files: Vec::new(),
    };
    let context = RenderContext {
        app_name,
        profile: project.profile.as_str(),
        mode: project.generation_mode.as_str(),
        capability_kind: None,
    };
    for template in profile_template_dirs(project.profile, project.generation_mode)? {
        render_dir(template, template, stage.path(), &context, &mut lock)?;
    }
    if project
        .enabled_surfaces
        .iter()
        .any(|surface| surface == "tauri-desktop")
    {
        install_command_manifest(stage.path(), &mut lock)?;
    }
    render_capability_instances(stage.path(), project, &mut lock)?;
    Ok((stage, lock))
}

fn render_capability_instances(
    root: &Path,
    project: &ProjectManifest,
    lock: &mut GeneratedLock,
) -> Result<()> {
    let descriptor_template = TEMPLATES
        .get_file("add/capability.json")
        .context("capability descriptor template is not packaged")?;
    let registry_bytes = capability_registry_bytes(&project.capabilities)?;
    let mut ids = BTreeSet::new();
    for capability in &project.capabilities {
        if capability.id.is_empty()
            || capability.id != slug(&capability.name)
            || !ids.insert(capability.id.to_lowercase())
        {
            bail!(
                "capability manifest contains an invalid or duplicate identity: {}",
                capability.id
            );
        }
    }
    for registry in capability_registry_paths(project) {
        let registry_path = root.join(registry);
        let owned = lock
            .files
            .iter_mut()
            .find(|file| file.path == registry)
            .with_context(|| {
                format!("profile surface has no managed capability registry: {registry}")
            })?;
        fs::write(&registry_path, &registry_bytes)?;
        owned.last_installed_digest = digest(&registry_bytes);
        for capability in &project.capabilities {
            let relative = Path::new(registry)
                .parent()
                .context("capability registry has no parent")?
                .join(format!("{}-{}.json", capability.kind, capability.id));
            let portable = relative.to_string_lossy().replace('\\', "/");
            if lock
                .files
                .iter()
                .any(|file| file.path.eq_ignore_ascii_case(&portable))
            {
                bail!("duplicate capability output path: {portable}");
            }
            let bytes =
                capability_descriptor_bytes(&capability.id, &capability.name, &capability.kind)?;
            if let Some(parent) = root.join(&relative).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(root.join(&relative), &bytes)?;
            lock.files.push(GeneratedFile {
                path: portable,
                template_id: "add/capability.json".to_owned(),
                source_digest: digest(descriptor_template.contents()),
                last_installed_digest: digest(&bytes),
                ownership: Ownership::Builder,
                version: BUILDER_VERSION.to_owned(),
                render_name: Some(capability.name.clone()),
                render_kind: Some(capability.kind.clone()),
            });
        }
    }
    Ok(())
}

fn plan_control_file(
    path: &Path,
    expected: &[u8],
    apply: bool,
    result: &mut CommandResult,
) -> Result<()> {
    if path.exists() {
        if fs::read(path)? != expected {
            result.conflicts.push(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        return Ok(());
    }
    result.actions.push(format!("create {}", path.display()));
    if apply {
        atomic_write(path, expected)?;
        result.changed = true;
    }
    Ok(())
}

fn create_project(args: NewArgs) -> Result<CommandResult> {
    let destination = absolute_path(&args.path)?;
    let manifest = bundle::manifest()?;
    let profile = bundle::profile(&manifest, args.profile)?;
    let mut result = CommandResult::new("new");
    result.path = Some(destination.display().to_string());
    result.profile = Some(args.profile.as_str().to_owned());

    if args.adopt {
        bail!("use `knowme-builder adopt` for non-destructive adoption");
    }

    let destination_state = destination_state(&destination)?;
    if destination_state == DestinationState::NonEmpty && !args.force {
        bail!(
            "destination is not empty; use `adopt` for an evolved application or explicit `--force`: {}",
            destination.display()
        );
    }

    let templates = profile_template_dirs(args.profile, args.mode)?;
    let mut planned_paths = Vec::new();
    for template in &templates {
        planned_paths.extend(collect_template_paths(template)?);
    }
    planned_paths.sort();
    planned_paths.dedup_by(|left, right| {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    });
    result.actions.extend(
        planned_paths
            .iter()
            .map(|path| format!("generate {}", path.display())),
    );
    result
        .actions
        .push("write .knowme-builder/project.toml".to_owned());
    result
        .actions
        .push("write .knowme-builder/generated.lock.json".to_owned());
    result.actions.push("write skills-lock.json".to_owned());

    if args.check {
        return Ok(result);
    }

    let parent = destination
        .parent()
        .context("destination must have a parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create destination parent {}", parent.display()))?;
    let staging = TempBuilder::new()
        .prefix(".knowme-builder-stage-")
        .tempdir_in(parent)
        .context("failed to create staging directory")?;
    let staging_path = staging.path().to_path_buf();
    let app_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .context("destination must have a UTF-8 file name")?;

    let mut lock = GeneratedLock {
        schema_version: 1,
        builder_version: BUILDER_VERSION.to_owned(),
        applied_migrations: Vec::new(),
        files: Vec::new(),
    };
    let render_context = RenderContext {
        app_name,
        profile: args.profile.as_str(),
        mode: args.mode.as_str(),
        capability_kind: None,
    };
    for template in templates {
        render_dir(
            template,
            template,
            &staging_path,
            &render_context,
            &mut lock,
        )?;
    }
    if profile
        .surfaces
        .iter()
        .any(|surface| surface == "tauri-desktop")
    {
        install_command_manifest(&staging_path, &mut lock)?;
    }

    let project = ProjectManifest {
        schema_version: 1,
        profile: args.profile,
        builder_version: BUILDER_VERSION.to_owned(),
        required_prometheus_contract: PROMETHEUS_CONTRACT.to_owned(),
        uar_mode: profile.uar_mode.clone(),
        enabled_surfaces: profile.surfaces.clone(),
        generation_mode: args.mode,
        policy_overlay_path: ".knowme-builder/policy-overlay.toml".to_owned(),
        app_name: Some(app_name.to_owned()),
        capabilities: Vec::new(),
        unsupported_surfaces: if matches!(args.mode, crate::cli::GenerationMode::Skeleton)
            || !profile.runnable_vertical_slice
        {
            profile.surfaces.clone()
        } else {
            Vec::new()
        },
    };
    write_project_state(&staging_path, &project, &lock, &manifest.skills)?;
    validate_generated_project(&staging_path)?;

    let retained_stage = staging.keep();
    if destination_state == DestinationState::Empty {
        fs::remove_dir(&destination).with_context(|| {
            format!(
                "failed to remove empty destination {}",
                destination.display()
            )
        })?;
    } else if destination_state == DestinationState::NonEmpty {
        let backup = backup_path(&destination)?;
        fs::rename(&destination, &backup).with_context(|| {
            format!(
                "failed to preserve existing destination as {}",
                backup.display()
            )
        })?;
        result.warnings.push(format!(
            "existing destination preserved at {}",
            backup.display()
        ));
    }
    fs::rename(&retained_stage, &destination)
        .with_context(|| format!("failed to atomically install {}", destination.display()))?;
    result.changed = true;
    Ok(result)
}

fn profile_template_dirs(
    profile: crate::cli::Profile,
    mode: crate::cli::GenerationMode,
) -> Result<Vec<&'static Dir<'static>>> {
    let profile_path = format!("profiles/{}/{}", profile.as_str(), mode.as_str());
    let mut paths = vec![profile_path];
    if matches!(mode, crate::cli::GenerationMode::Runnable) {
        match profile {
            crate::cli::Profile::SovereignHybrid => {
                paths.extend(["baselines/flutter".to_owned(), "baselines/tauri".to_owned()])
            }
            crate::cli::Profile::FlutterMobile => {
                paths.push("baselines/flutter".to_owned());
            }
            crate::cli::Profile::TauriDesktop => {
                paths.push("baselines/tauri".to_owned());
            }
            crate::cli::Profile::GovernedWebShell | crate::cli::Profile::AxumWeb => {
                paths.push("baselines/web".to_owned());
            }
        }
    }
    paths
        .into_iter()
        .map(|path| {
            TEMPLATES
                .get_dir(&path)
                .with_context(|| format!("profile template component is not packaged: {path}"))
        })
        .collect()
}

fn install_command_manifest(root: &Path, lock: &mut GeneratedLock) -> Result<()> {
    let source = TEMPLATES
        .get_file("command-contract/commands.json")
        .context("command contract is not packaged")?
        .contents();
    let relative = Path::new("desktop/src-tauri/command-manifest.json");
    atomic_write(&root.join(relative), source)?;
    lock.files.push(GeneratedFile {
        path: relative.to_string_lossy().into_owned(),
        template_id: "command-contract/commands.json".to_owned(),
        source_digest: digest(source),
        last_installed_digest: digest(source),
        ownership: Ownership::Builder,
        version: BUILDER_VERSION.to_owned(),
        render_name: None,
        render_kind: None,
    });
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DestinationState {
    Missing,
    Empty,
    NonEmpty,
}

fn destination_state(path: &Path) -> Result<DestinationState> {
    if !path.exists() {
        return Ok(DestinationState::Missing);
    }
    if !path.is_dir() {
        return Ok(DestinationState::NonEmpty);
    }
    let mut entries =
        fs::read_dir(path).with_context(|| format!("failed to inspect {}", path.display()))?;
    Ok(if entries.next().is_none() {
        DestinationState::Empty
    } else {
        DestinationState::NonEmpty
    })
}

struct RenderContext<'a> {
    app_name: &'a str,
    profile: &'a str,
    mode: &'a str,
    capability_kind: Option<&'a str>,
}

fn render_dir(
    root: &Dir<'_>,
    dir: &Dir<'_>,
    destination: &Path,
    context: &RenderContext<'_>,
    lock: &mut GeneratedLock,
) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => render_dir(root, child, destination, context, lock)?,
            DirEntry::File(file) => {
                let relative = file
                    .path()
                    .strip_prefix(root.path())
                    .context("template path escaped profile root")?;
                bundle::validate_relative_path(relative)?;
                let output_relative =
                    PathBuf::from(render_text(&relative.to_string_lossy(), context));
                bundle::validate_relative_path(&output_relative)?;
                let output = destination.join(&output_relative);
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                let source = file.contents();
                let rendered = match std::str::from_utf8(source) {
                    Ok(text) => render_text(text, context).into_bytes(),
                    Err(_) => source.to_vec(),
                };
                let portable_path = output_relative.to_string_lossy().replace('\\', "/");
                if lock
                    .files
                    .iter()
                    .any(|entry| entry.path.eq_ignore_ascii_case(&portable_path))
                {
                    if fs::read(&output)? == rendered {
                        continue;
                    }
                    bail!("profile templates produce conflicting path {portable_path}");
                }
                let mut output_file = fs::File::create(&output)?;
                output_file.write_all(&rendered)?;
                output_file.sync_all()?;
                lock.files.push(GeneratedFile {
                    path: portable_path,
                    template_id: file.path().to_string_lossy().replace('\\', "/"),
                    source_digest: digest(source),
                    last_installed_digest: digest(&rendered),
                    ownership: Ownership::Builder,
                    version: BUILDER_VERSION.to_owned(),
                    render_name: None,
                    render_kind: None,
                });
            }
        }
    }
    Ok(())
}

fn render_text(input: &str, context: &RenderContext<'_>) -> String {
    let slug = app_slug(context.app_name);
    let crate_name = language_identifier(&slug);
    input
        .replace("__APP_NAME__", context.app_name)
        .replace("__APP_SLUG__", &slug)
        .replace("__APP_CRATE__", &crate_name)
        .replace("__PROFILE__", context.profile)
        .replace("__MODE__", context.mode)
        .replace(
            "__CAPABILITY_KIND__",
            context.capability_kind.unwrap_or("feature"),
        )
        .replace("__BUILDER_VERSION__", BUILDER_VERSION)
}

fn language_identifier(slug: &str) -> String {
    let mut identifier = slug.replace('-', "_");
    if identifier.is_empty() {
        identifier.push_str("app");
    }
    if identifier.as_bytes()[0].is_ascii_digit() {
        identifier.insert_str(0, "app_");
    }
    identifier
}

fn collect_template_paths(root: &Dir<'_>) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    fn collect(root: &Dir<'_>, dir: &Dir<'_>, paths: &mut Vec<PathBuf>) -> Result<()> {
        for entry in dir.entries() {
            match entry {
                DirEntry::Dir(child) => collect(root, child, paths)?,
                DirEntry::File(file) => {
                    paths.push(file.path().strip_prefix(root.path())?.to_path_buf());
                }
            }
        }
        Ok(())
    }
    collect(root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn write_project_state(
    root: &Path,
    project: &ProjectManifest,
    lock: &GeneratedLock,
    skills: &[String],
) -> Result<()> {
    let state_dir = root.join(".knowme-builder");
    fs::create_dir_all(&state_dir)?;
    atomic_write(
        &state_dir.join("project.toml"),
        toml::to_string_pretty(project)?.as_bytes(),
    )?;
    atomic_write(
        &state_dir.join("generated.lock.json"),
        format!("{}\n", serde_json::to_string_pretty(lock)?).as_bytes(),
    )?;
    atomic_write(
        &state_dir.join("activation-manifest.json"),
        bundle::activation_manifest(),
    )?;
    atomic_write(&root.join("skills-lock.json"), &skills_lock_bytes(skills)?)
}

fn skills_lock_bytes(skills: &[String]) -> Result<Vec<u8>> {
    let pinned_skills = skills
        .iter()
        .map(|skill| {
            let source = SKILLS
                .get_dir(skill)
                .with_context(|| format!("packaged skill is missing: {skill}"))?;
            Ok(serde_json::json!({
                "id": skill,
                "source": format!("builder:skills/{skill}"),
                "version": BUILDER_VERSION,
                "digest": digest_embedded_dir(source)?
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    let skills_lock = serde_json::json!({
        "schemaVersion": 1,
        "builder": {
            "id": "hybrid-mobile-architecture",
            "version": BUILDER_VERSION,
            "digest": digest(include_bytes!("../../../builder.manifest.json"))
        },
        "prometheus": {
            "contractVersion": PROMETHEUS_CONTRACT,
            "source": "external:prometheus-skill-pack",
            "contractDigest": digest(include_bytes!("../../../compatibility/prometheus-control-plane.json"))
        },
        "openSpec": {
            "source": "external",
            "version": "1.6.0"
        },
        "skills": pinned_skills
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&skills_lock)?).into_bytes())
}

fn skills_lock_bytes_preserving_third_party(skills: &[String], path: &Path) -> Result<Vec<u8>> {
    let mut generated: serde_json::Value = serde_json::from_slice(&skills_lock_bytes(skills)?)?;
    if path.is_file() {
        let existing: serde_json::Value = serde_json::from_slice(&fs::read(path)?)
            .with_context(|| format!("invalid existing skill lock {}", path.display()))?;
        let third_party = existing.get("thirdParty").cloned().or_else(|| {
            let version = existing.get("version")?.as_u64()?;
            let skills = existing.get("skills")?;
            skills.is_object().then(|| {
                serde_json::json!({
                    "schemaVersion": version,
                    "skills": skills
                })
            })
        });
        if let Some(third_party) = third_party {
            generated
                .as_object_mut()
                .expect("generated skill lock is an object")
                .insert("thirdParty".to_owned(), third_party);
        }
    }
    Ok(format!("{}\n", serde_json::to_string_pretty(&generated)?).into_bytes())
}

fn digest_embedded_dir(root: &Dir<'_>) -> Result<String> {
    let mut entries = Vec::new();
    fn collect(root: &Dir<'_>, dir: &Dir<'_>, entries: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        for entry in dir.entries() {
            match entry {
                DirEntry::Dir(child) => collect(root, child, entries)?,
                DirEntry::File(file) => entries.push((
                    file.path()
                        .strip_prefix(root.path())?
                        .to_string_lossy()
                        .into_owned(),
                    file.contents().to_vec(),
                )),
            }
        }
        Ok(())
    }
    collect(root, root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (path, content) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(content);
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_generated_project(root: &Path) -> Result<()> {
    for required in [
        ".knowme-builder/project.toml",
        ".knowme-builder/generated.lock.json",
        "skills-lock.json",
    ] {
        if !root.join(required).is_file() {
            bail!("staged project is missing required file {required}");
        }
    }
    Ok(())
}

pub(crate) fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temp = TempBuilder::new().prefix(".write-").tempfile_in(parent)?;
    temp.write_all(content)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("failed to atomically write {}", path.display()))?;
    Ok(())
}

fn digest(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn app_slug(value: &str) -> String {
    let slug = slug(value);
    if slug.is_empty() {
        "app".to_owned()
    } else {
        slug
    }
}

fn backup_path(destination: &Path) -> Result<PathBuf> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .context("destination must have a UTF-8 file name")?;
    Ok(destination.with_file_name(format!("{name}.knowme-builder-backup-{seconds}")))
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn print_result(result: &CommandResult, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
        return Ok(());
    }
    println!(
        "{}: {}",
        if result.changed { "changed" } else { "checked" },
        result.operation
    );
    if let Some(path) = &result.path {
        println!("path: {path}");
    }
    for action in &result.actions {
        println!("  {action}");
    }
    for warning in &result.warnings {
        eprintln!("warning: {warning}");
    }
    for conflict in &result.conflicts {
        eprintln!("conflict: {conflict}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        assess_prometheus_doctor, missing_prometheus_capabilities, prometheus_package_version,
        version_at_least,
    };

    #[test]
    fn compares_control_plane_versions() {
        assert!(version_at_least("1.7.0", "1.7.0"));
        assert!(!version_at_least("1.6.2", "1.7.0"));
        assert!(version_at_least("2.0.0", "2.0.0"));
        assert!(version_at_least("2.1.0", "2.0.0"));
        assert!(!version_at_least("1.99.0", "2.0.0"));
        assert!(!version_at_least("not-a-version", "2.0.0"));
    }

    #[test]
    fn reports_missing_prometheus_capabilities() {
        let kbd = include_str!("../tests/fixtures/prometheus-doctor/kbd-missing.txt");
        let learning = include_str!("../tests/fixtures/prometheus-doctor/learning-current.txt");
        let missing = missing_prometheus_capabilities(kbd, learning);
        assert!(!missing.contains(&"status"));
        assert!(!missing.contains(&"pause"));
        assert!(!missing.contains(&"learning-status"));
        assert!(missing.contains(&"conflicts"));
        assert!(missing.contains(&"completion"));
    }

    #[test]
    fn distinguishes_package_release_compatibility() {
        let outdated = include_str!("../tests/fixtures/prometheus-doctor/package-outdated.txt");
        let current = include_str!("../tests/fixtures/prometheus-doctor/package-current.txt");
        assert!(!version_at_least(
            prometheus_package_version(outdated).unwrap_or_default(),
            "1.7.0"
        ));
        assert!(version_at_least(
            prometheus_package_version(current).unwrap_or_default(),
            "1.7.0"
        ));
    }

    #[test]
    fn distinguishes_contract_and_operational_health() {
        let outdated = assess_prometheus_doctor(
            include_bytes!("../tests/fixtures/prometheus-doctor/outdated-contract.json"),
            true,
            "2.0.0",
        );
        assert!(!outdated.ok);
        assert!(outdated.warnings[0].contains("older than required"));

        let current = assess_prometheus_doctor(
            include_bytes!("../tests/fixtures/prometheus-doctor/current.json"),
            true,
            "2.0.0",
        );
        assert!(current.ok);

        let unhealthy = assess_prometheus_doctor(
            include_bytes!("../tests/fixtures/prometheus-doctor/compatible-unhealthy.json"),
            false,
            "2.0.0",
        );
        assert!(!unhealthy.ok);
        assert!(unhealthy.actions[0].contains("compatible"));
        assert!(unhealthy.warnings[0].contains("3 failing"));

        let malformed = assess_prometheus_doctor(
            include_bytes!("../tests/fixtures/prometheus-doctor/malformed.json"),
            false,
            "2.0.0",
        );
        assert!(!malformed.ok);
        assert!(malformed.warnings[0].contains("invalid JSON"));
    }
}
