//! Machine-local measurements made at the native model-visible catalogue boundary.
use super::{command_stdout, hash_path, CheckResult, CheckStatus, Severity};
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Measurement {
    schema_version: u32,
    harness: String,
    harness_version: String,
    inventory_sha256: String,
    inventory_chars: usize,
    inventory_skills: usize,
    samples: Vec<Sample>,
    config_sha256: Option<String>,
    environment_budget: Option<String>,
    projection: Option<Projection>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sample {
    count: usize,
    input_chars: usize,
    names_seen: usize,
    complete_descriptions: usize,
    prompt_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Projection {
    policy_sha256: String,
    prompt_sha256: String,
    rows: Vec<ProjectedSkill>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectedSkill {
    name: String,
    seen: bool,
    valid_prefix: bool,
    complete: bool,
    rendered_chars: usize,
    reference_chars: usize,
}

fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn assess_projection(projection: Projection, root: &Path) -> Result<(bool, String)> {
    let policy = root.join("config/codex-catalog.txt");
    if hash_path(&policy).as_ref() != Some(&projection.policy_sha256)
        || !digest(&projection.prompt_sha256)
    {
        bail!("Codex projection policy or prompt identity has changed");
    }
    let total = projection.rows.len();
    let unique: std::collections::BTreeSet<_> = projection.rows.iter().map(|r| &r.name).collect();
    if total == 0 || unique.len() != total {
        bail!("invalid configured Codex projection");
    }
    let mut complete = 0;
    let mut seen = 0;
    let mut valid = 0;
    let mut characters = 0;
    let mut minimum = usize::MAX;
    let mut poor = Vec::new();
    for row in projection.rows {
        if row.reference_chars == 0
            || row.name.is_empty()
            || row.complete && (!row.valid_prefix || row.rendered_chars != row.reference_chars)
        {
            bail!("invalid projection reference for {}", row.name);
        }
        seen += usize::from(row.seen);
        valid += usize::from(row.seen && row.valid_prefix && row.rendered_chars > 0);
        complete += usize::from(row.complete);
        characters += row.rendered_chars;
        minimum = minimum.min(row.rendered_chars);
        if !row.valid_prefix || row.rendered_chars < 55 && !row.complete {
            poor.push(row.name);
        }
    }
    let mean = characters as f64 / total as f64;
    // Existing "healthy" mean in scripts/codex-catalog-stat.py, with exact path coverage.
    let healthy = valid == total && mean >= 120.0;
    let mut summary = format!("configured projection: {seen}/{total} exact installed paths, {valid}/{total} valid descriptions, {complete} complete, {} shortened, minimum {minimum}, mean {mean:.1} characters", seen.saturating_sub(complete));
    if !poor.is_empty() {
        summary.push_str(&format!(
            ", missing or poorly rendered: {}",
            poor.join(", ")
        ));
    }
    Ok((healthy, summary))
}

fn config_path(harness: &str, home: &Path) -> Result<PathBuf> {
    match harness {
        "claude-code" => Ok(home.join(".claude/settings.json")),
        "codex" => super::effective_doctor_codex_home(home)
            .map(|dir| dir.join("config.toml"))
            .map_err(|error| anyhow!(error)),
        "opencode" => Ok(home.join(".config/opencode/opencode.json")),
        "kimi" => Ok(home.join(".kimi-code/config.toml")),
        _ => bail!("unsupported harness"),
    }
}

fn describe(
    file: &Path,
    harness: &str,
    inventory: &(usize, usize, String),
    home: &Path,
    root: &Path,
) -> Result<(bool, String)> {
    let receipt: Measurement = serde_json::from_slice(
        &fs::read(file)
            .with_context(|| format!("no readable local measurement at {}", file.display()))?,
    )?;
    if receipt.schema_version != 2
        || receipt.harness != harness
        || receipt.harness_version.is_empty()
    {
        bail!("measurement identity is invalid");
    }
    let config = config_path(harness, home)?;
    if hash_path(&config) != receipt.config_sha256
        || harness == "claude-code"
            && std::env::var("SLASH_COMMAND_TOOL_CHAR_BUDGET").ok() != receipt.environment_budget
    {
        bail!("installed harness settings changed since measurement");
    }
    let program = if harness == "claude-code" {
        "claude"
    } else {
        harness
    };
    let installed_version = command_stdout(&[program, "--version"])
        .ok_or_else(|| anyhow!("cannot determine installed version of {program}"))?;
    if installed_version.trim() != receipt.harness_version {
        bail!(
            "installed version differs from the measured version {}",
            receipt.harness_version
        );
    }
    if (receipt.inventory_skills, receipt.inventory_chars) != (inventory.0, inventory.1)
        || receipt.inventory_sha256 != inventory.2
    {
        bail!("catalogue changed since measurement");
    }
    let mut last = 0;
    let mut floor = None;
    let mut cutoff = None;
    for sample in receipt.samples {
        if sample.input_chars <= last
            || sample.count == 0
            || sample.names_seen > sample.count
            || sample.complete_descriptions > sample.names_seen
            || !digest(&sample.prompt_sha256)
        {
            bail!("invalid native prompt sample");
        }
        last = sample.input_chars;
        if sample.complete_descriptions == sample.count {
            if cutoff.is_some() {
                bail!("non-monotone native catalogue observations");
            }
            floor = Some(last);
        } else {
            cutoff.get_or_insert(last);
        }
    }
    let floor = floor.ok_or_else(|| anyhow!("no fully rendered calibration catalogue"))?;
    let bounds = match cutoff {
        Some(upper) if upper > floor => {
            format!("complete at {floor} characters; truncation observed at {upper}")
        }
        None if floor >= inventory.1 => {
            format!("at least {floor} characters fully rendered; ceiling not reached")
        }
        _ => bail!("no cutoff or fully rendered sample covering the current inventory"),
    };
    let (covered, profile) = if harness == "codex" {
        assess_projection(
            receipt
                .projection
                .ok_or_else(|| anyhow!("missing configured Codex projection"))?,
            root,
        )?
    } else if floor < inventory.1 {
        (
            false,
            "full source inventory exceeds the demonstrated capacity".into(),
        )
    } else {
        (
            true,
            "calibration covers the full source inventory size".into(),
        )
    };
    Ok((
        covered,
        format!("{harness} {}: {bounds}; {profile}", receipt.harness_version),
    ))
}

fn inventory_root(home: &Path) -> PathBuf {
    if let Some(selections) = super::installed_selections(&super::doctor_plugin_store()) {
        if let Some(source) = selections.into_iter().map(|s| s.source_root)
            .find(|path| path.is_absolute() && path.join("evals/skill-activation/harness-budgets.json").is_file())
        {
            return source;
        }
    }
    let project = super::project_root::resolve_project_root().path;
    if project
        .join("evals/skill-activation/harness-budgets.json")
        .is_file()
    {
        return project;
    }
    let plugin = std::env::var_os("PROMETHEUS_PLUGIN_ROOT")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".prometheus/plugins/prometheus-skill-pack"));
    if let Ok(source) = fs::read_to_string(plugin.join(".source-repo")) {
        let source = PathBuf::from(source.trim());
        if source.is_absolute()
            && source
                .join("evals/skill-activation/harness-budgets.json")
                .is_file()
        {
            return source;
        }
    }
    plugin.join("current")
}

pub(super) fn check(home: &Path) -> CheckResult {
    let root = inventory_root(home);
    let inventory = super::super::skill::inventory(&root.join("skills"));
    let dir = home.join(".prometheus/skill-activation/harness-budgets");
    let all = ["claude-code", "codex", "opencode", "kimi"];
    let configured = super::installed_selections(&super::doctor_plugin_store()).map(|profiles| {
        let recorded: std::collections::BTreeSet<_> = profiles.into_iter().flat_map(|p| p.harnesses).collect();
        all.into_iter().filter(|name| {
            recorded.contains(*name) || config_path(name, home).map_or(true, |path| path.is_file())
        }).collect::<Vec<_>>()
    }).unwrap_or_else(|| all.to_vec());
    let expected = configured.len();
    let mut details = Vec::new();
    let mut measured = 0;
    let mut covered = 0;
    let (skills, characters) = match inventory {
        Ok(inventory) if inventory.0 > 0 => {
            for harness in configured {
                match describe(
                    &dir.join(format!("{harness}.json")),
                    harness,
                    &inventory,
                    home,
                    &root,
                ) {
                    Ok((covers_inventory, observation)) => {
                        measured += 1;
                        covered += usize::from(covers_inventory);
                        details.push(observation);
                    }
                    Err(error) => details.push(format!("{harness}: {error}")),
                }
            }
            (inventory.0, inventory.1)
        }
        Ok(_) => {
            details.push("source skills catalogue is empty".into());
            (0, 0)
        }
        Err(error) => {
            details.push(error.to_string());
            (0, 0)
        }
    };
    if expected == 0 {
        details.push("Install selection and current local configuration identify no supported discovery harness.".into());
    }
    if measured != expected {
        details.push("Capture current native prompt measurements with scripts/certification/measure-discovery.py.".into());
    }
    CheckResult {
        id: "skills.discovery-budget".into(),
        group: "skills".into(),
        label: "Harness instruction discovery budgets".into(),
        severity: if covered == expected && skills > 0 { Severity::Green } else { Severity::Yellow },
        status: if expected == 0 && skills > 0 { CheckStatus::Skip } else if covered == expected && skills > 0 { CheckStatus::Pass } else { CheckStatus::Warn },
        summary: format!("{skills} skills, {characters} discovery characters, {measured}/{expected} measured harnesses, {covered}/{expected} cover configured discovery profiles"),
        details,
        optional: expected == 0 && skills > 0,
        actions: vec![],
    }
}
