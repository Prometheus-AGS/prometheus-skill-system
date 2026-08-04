use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct SkillRecord {
    path: PathBuf,
    name: String,
    description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintIssue {
    severity: &'static str,
    code: &'static str,
    path: PathBuf,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintReport {
    schema_version: u32,
    skills: usize,
    errors: usize,
    warnings: usize,
    issues: Vec<LintIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BudgetReport {
    schema_version: u32,
    harness: String,
    skills: usize,
    inventory_chars: usize,
    budget_chars: Option<usize>,
    utilization_percent: Option<f64>,
    measured: bool,
    included_skills: usize,
    dropped_skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalCorpus {
    schema_version: String,
    trials_per_harness: u32,
    critical_skills: Vec<String>,
    cases: Vec<EvalCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalCase {
    id: String,
    skill: String,
    kind: String,
    prompt: String,
    expected_invocation: bool,
    expected_commands: Vec<String>,
    forbid_direct_writes: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalTrace {
    case_id: String,
    trial: u32,
    invoked_skills: Vec<String>,
    commands: Vec<String>,
    #[serde(default)]
    direct_writes: u32,
    #[serde(default = "default_true")]
    lifecycle_behavior: bool,
    #[serde(default = "default_true")]
    output_contract: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvalReport {
    schema_version: u32,
    harness: String,
    mode: &'static str,
    corpus_cases: usize,
    scheduled_trials: usize,
    observed_trials: usize,
    explicit_invocation_rate: Option<f64>,
    implicit_invocation_rate: Option<f64>,
    false_positives: usize,
    direct_write_violations: usize,
    per_skill_positive_rate: BTreeMap<String, f64>,
    passed: Option<bool>,
    failures: Vec<String>,
}

fn default_true() -> bool {
    true
}

pub fn inventory(root: &Path) -> Result<(usize, usize)> {
    let (skills, _) = load_skills(root)?;
    let characters = skills
        .iter()
        .map(|skill| skill.name.chars().count() + skill.description.chars().count() + 2)
        .sum();
    Ok((skills.len(), characters))
}

pub fn lint(root: &Path, json: bool) -> Result<()> {
    let (skills, mut issues) = load_skills(root)?;
    let mut names: BTreeMap<&str, Vec<&Path>> = BTreeMap::new();
    for skill in &skills {
        names.entry(&skill.name).or_default().push(&skill.path);
        if skill.name.is_empty()
            || skill.name.len() > 64
            || !skill
                .name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || skill.name.starts_with('-')
            || skill.name.ends_with('-')
            || skill.name.contains("--")
        {
            issues.push(LintIssue {
                severity: "error",
                code: "invalid-name",
                path: skill.path.clone(),
                message: "name must be 1-64 characters of lowercase kebab-case".into(),
            });
        }
        let description_chars = skill.description.chars().count();
        if description_chars == 0 || description_chars > 1024 {
            issues.push(LintIssue {
                severity: "error",
                code: "description-length",
                path: skill.path.clone(),
                message: format!(
                    "description contains {description_chars} characters; Agent Skills permits 1-1024"
                ),
            });
        }
    }
    for (name, paths) in names {
        if !name.is_empty() && paths.len() > 1 {
            for path in paths {
                issues.push(LintIssue {
                    severity: "error",
                    code: "duplicate-name",
                    path: path.to_path_buf(),
                    message: format!("skill name {name:?} is declared more than once"),
                });
            }
        }
    }
    for left in 0..skills.len() {
        for right in left + 1..skills.len() {
            let similarity = jaccard(&skills[left].description, &skills[right].description);
            if similarity >= 0.85
                && skills[left].name != skills[right].name
                && !skills[left].description.is_empty()
            {
                issues.push(LintIssue {
                    severity: "warning",
                    code: "description-collision",
                    path: skills[right].path.clone(),
                    message: format!(
                        "{:.0}% description-token overlap with {} ({})",
                        similarity * 100.0,
                        skills[left].name,
                        skills[left].path.display()
                    ),
                });
            }
        }
    }
    issues.sort_by(|left, right| left.path.cmp(&right.path).then(left.code.cmp(right.code)));
    let errors = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warnings = issues.len().saturating_sub(errors);
    let report = LintReport {
        schema_version: 1,
        skills: skills.len(),
        errors,
        warnings,
        issues,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Skill lint: {} skills, {} errors, {} warnings",
            report.skills, report.errors, report.warnings
        );
        for issue in &report.issues {
            println!(
                "{} {} {}: {}",
                issue.severity,
                issue.code,
                issue.path.display(),
                issue.message
            );
        }
    }
    if errors > 0 {
        anyhow::bail!("skill lint failed with {errors} error(s)");
    }
    Ok(())
}

pub fn budget(root: &Path, harness: &str, budget_chars: Option<usize>, json: bool) -> Result<()> {
    if !matches!(harness, "claude-code" | "codex" | "opencode" | "kimi") {
        anyhow::bail!("unsupported harness {harness:?}");
    }
    let (skills, _) = load_skills(root)?;
    let inventory_chars = skills
        .iter()
        .map(|skill| skill.name.chars().count() + skill.description.chars().count() + 2)
        .sum();
    let mut used: usize = 0;
    let mut included: usize = 0;
    let mut dropped = Vec::new();
    for skill in &skills {
        let chars = skill.name.chars().count() + skill.description.chars().count() + 2;
        if budget_chars.is_some_and(|budget| used.saturating_add(chars) > budget) {
            dropped.push(skill.name.clone());
        } else {
            used += chars;
            included += 1;
        }
    }
    let report = BudgetReport {
        schema_version: 1,
        harness: harness.into(),
        skills: skills.len(),
        inventory_chars,
        budget_chars,
        utilization_percent: budget_chars
            .map(|budget| inventory_chars as f64 * 100.0 / budget.max(1) as f64),
        measured: budget_chars.is_some(),
        included_skills: included,
        dropped_skills: dropped,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Skill budget ({harness}): {} skills, {} inventory characters",
            report.skills, report.inventory_chars
        );
        match report.budget_chars {
            Some(budget) => println!(
                "Budget: {budget} characters ({:.1}% used), {} included, {} dropped",
                report.utilization_percent.unwrap_or(0.0),
                report.included_skills,
                report.dropped_skills.len()
            ),
            None => println!(
                "Budget: unmeasured; pass --budget-chars from a captured harness discovery trace"
            ),
        }
        if !report.dropped_skills.is_empty() {
            println!("Dropped: {}", report.dropped_skills.join(", "));
        }
    }
    if budget_chars.is_some() && !report.dropped_skills.is_empty() {
        anyhow::bail!(
            "{} skills exceed the measured {harness} discovery budget",
            report.dropped_skills.len()
        );
    }
    Ok(())
}

pub fn eval(
    corpus_path: &Path,
    harness: &str,
    trace_path: Option<&Path>,
    json: bool,
) -> Result<()> {
    if !matches!(harness, "claude-code" | "codex" | "opencode" | "kimi") {
        anyhow::bail!("unsupported harness {harness:?}");
    }
    let corpus: EvalCorpus = serde_json::from_reader(
        fs::File::open(corpus_path)
            .with_context(|| format!("open corpus {}", corpus_path.display()))?,
    )?;
    if corpus.schema_version != "1"
        || corpus.cases.len() != 30
        || corpus.critical_skills.len() != 5
        || corpus.trials_per_harness != 3
    {
        anyhow::bail!(
            "critical corpus must contain schema 1, five skills, 30 prompts, and three trials"
        );
    }
    let ids = corpus
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != corpus.cases.len() {
        anyhow::bail!("critical corpus contains duplicate case ids");
    }
    let scheduled_trials = corpus.cases.len() * corpus.trials_per_harness as usize;
    let Some(trace_path) = trace_path else {
        let report = EvalReport {
            schema_version: 1,
            harness: harness.into(),
            mode: "plan",
            corpus_cases: corpus.cases.len(),
            scheduled_trials,
            observed_trials: 0,
            explicit_invocation_rate: None,
            implicit_invocation_rate: None,
            false_positives: 0,
            direct_write_violations: 0,
            per_skill_positive_rate: BTreeMap::new(),
            passed: None,
            failures: vec![
                "No harness trace supplied; corpus validated but activation was not inferred."
                    .into(),
            ],
        };
        print_eval_report(&report, json)?;
        return Ok(());
    };
    let traces: Vec<EvalTrace> = serde_json::from_reader(
        fs::File::open(trace_path)
            .with_context(|| format!("open trace {}", trace_path.display()))?,
    )?;
    let cases = corpus
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeSet::new();
    let mut explicit = (0_usize, 0_usize);
    let mut implicit = (0_usize, 0_usize);
    let mut false_positives = 0;
    let mut direct_write_violations = 0;
    let mut per_skill: BTreeMap<String, (usize, usize)> = corpus
        .critical_skills
        .iter()
        .map(|skill| (skill.clone(), (0, 0)))
        .collect();
    let mut failures = Vec::new();
    for trace in &traces {
        let Some(case) = cases.get(trace.case_id.as_str()) else {
            failures.push(format!("unknown trace case {}", trace.case_id));
            continue;
        };
        if trace.trial == 0 || trace.trial > corpus.trials_per_harness {
            failures.push(format!(
                "{} has invalid trial {}",
                trace.case_id, trace.trial
            ));
            continue;
        }
        if !observed.insert((trace.case_id.as_str(), trace.trial)) {
            failures.push(format!(
                "duplicate trace for {} trial {}",
                trace.case_id, trace.trial
            ));
            continue;
        }
        let invoked = trace
            .invoked_skills
            .iter()
            .any(|skill| skill == &case.skill);
        let commands_ok = case
            .expected_commands
            .iter()
            .all(|expected| trace.commands.iter().any(|actual| actual == expected));
        let direct_ok = !case.forbid_direct_writes || trace.direct_writes == 0;
        if !direct_ok {
            direct_write_violations += 1;
        }
        let success = invoked == case.expected_invocation
            && commands_ok
            && direct_ok
            && trace.lifecycle_behavior
            && trace.output_contract;
        match case.kind.as_str() {
            "explicit" => {
                explicit.1 += 1;
                explicit.0 += usize::from(invoked);
            }
            "implicit" => {
                implicit.1 += 1;
                implicit.0 += usize::from(invoked);
            }
            "near_miss" => {
                false_positives += usize::from(invoked || !trace.commands.is_empty());
            }
            other => failures.push(format!("{} has unknown kind {other}", case.id)),
        }
        if case.expected_invocation {
            let entry = per_skill.entry(case.skill.clone()).or_default();
            entry.1 += 1;
            entry.0 += usize::from(success);
        }
        if !success {
            failures.push(format!(
                "{} trial {} failed invocation/command/lifecycle contract: {}",
                case.id, trace.trial, case.prompt
            ));
        }
    }
    if observed.len() != scheduled_trials {
        failures.push(format!(
            "trace coverage is {}/{scheduled_trials}",
            observed.len()
        ));
    }
    let rate = |counts: (usize, usize)| {
        if counts.1 == 0 {
            0.0
        } else {
            counts.0 as f64 / counts.1 as f64
        }
    };
    let explicit_rate = rate(explicit);
    let implicit_rate = rate(implicit);
    let per_skill_positive_rate = per_skill
        .into_iter()
        .map(|(skill, counts)| (skill, rate(counts)))
        .collect::<BTreeMap<_, _>>();
    if explicit_rate < 1.0 {
        failures.push("explicit invocation rate is below 100%".into());
    }
    if implicit_rate < 0.9 {
        failures.push("aggregate implicit invocation rate is below 90%".into());
    }
    if false_positives > 1 {
        failures.push("near-miss false positives exceed one per harness corpus".into());
    }
    for (skill, rate) in &per_skill_positive_rate {
        if *rate < 5.0 / 6.0 {
            failures.push(format!(
                "{skill} positive success rate {:.1}% is below 5/6",
                rate * 100.0
            ));
        }
    }
    let passed = failures.is_empty();
    let report = EvalReport {
        schema_version: 1,
        harness: harness.into(),
        mode: "grade",
        corpus_cases: corpus.cases.len(),
        scheduled_trials,
        observed_trials: observed.len(),
        explicit_invocation_rate: Some(explicit_rate),
        implicit_invocation_rate: Some(implicit_rate),
        false_positives,
        direct_write_violations,
        per_skill_positive_rate,
        passed: Some(passed),
        failures,
    };
    print_eval_report(&report, json)?;
    if !passed {
        anyhow::bail!("skill evaluation gate failed for {harness}");
    }
    Ok(())
}

fn print_eval_report(report: &EvalReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "Skill eval ({}): {} cases, {}/{} observed trials",
            report.harness, report.corpus_cases, report.observed_trials, report.scheduled_trials
        );
        if let Some(passed) = report.passed {
            println!("Gate: {}", if passed { "pass" } else { "fail" });
        } else {
            println!("Gate: trace required");
        }
        for failure in &report.failures {
            println!("- {failure}");
        }
    }
    Ok(())
}

fn load_skills(root: &Path) -> Result<(Vec<SkillRecord>, Vec<LintIssue>)> {
    let mut skills = Vec::new();
    let mut issues = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            !matches!(
                entry.file_name().to_str(),
                Some("node_modules" | "target" | ".git" | ".kbd-orchestrator")
            )
        })
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.file_name() == "SKILL.md"
                && !entry.path().components().any(|component| {
                    matches!(
                        component.as_os_str().to_str(),
                        Some("imported" | "tests" | "fixtures")
                    )
                })
        })
    {
        let path = entry.into_path();
        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        match parse_frontmatter(&content) {
            Some((name, description)) => skills.push(SkillRecord {
                path,
                name,
                description,
            }),
            None => issues.push(LintIssue {
                severity: "error",
                code: "frontmatter",
                path,
                message: "missing or unterminated YAML frontmatter".into(),
            }),
        }
    }
    skills.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((skills, issues))
}

fn parse_frontmatter(content: &str) -> Option<(String, String)> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut frontmatter = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            let name = scalar(&frontmatter, "name").unwrap_or_default();
            let description = scalar(&frontmatter, "description").unwrap_or_default();
            return Some((name, description));
        }
        frontmatter.push(line);
    }
    None
}

fn scalar(lines: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let index = lines
        .iter()
        .position(|line| line.trim_start().starts_with(&prefix))?;
    let raw = lines[index].trim_start()[prefix.len()..].trim();
    if matches!(raw, ">" | ">-" | "|" | "|-") {
        let value = lines[index + 1..]
            .iter()
            .take_while(|line| line.starts_with(' ') || line.starts_with('\t'))
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        return Some(value);
    }
    Some(raw.trim_matches(['"', '\'']).to_string())
}

fn jaccard(left: &str, right: &str) -> f64 {
    let tokens = |value: &str| {
        value
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|token| token.len() > 2)
            .map(str::to_ascii_lowercase)
            .collect::<BTreeSet<_>>()
    };
    let left = tokens(left);
    let right = tokens(right);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count();
    let union = left.union(&right).count();
    intersection as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::load_skills;
    use std::fs;

    #[test]
    fn skill_inventory_excludes_test_fixtures() {
        let temporary =
            std::env::temp_dir().join(format!("prometheus-skill-inventory-{}", std::process::id()));
        fs::remove_dir_all(&temporary).ok();
        let real = temporary.join("process/real-skill");
        let fixture = temporary.join("process/review/tests/fixtures/flawed-skill");
        fs::create_dir_all(&real).expect("create real skill");
        fs::create_dir_all(&fixture).expect("create fixture skill");
        fs::write(
            real.join("SKILL.md"),
            "---\nname: real-skill\ndescription: Real capability\n---\nBody\n",
        )
        .expect("write real skill");
        fs::write(
            fixture.join("SKILL.md"),
            "---\nname: flawed-skill\ndescription: Test fixture\n---\nBody\n",
        )
        .expect("write fixture skill");

        let (skills, issues) = load_skills(&temporary).expect("load skills");
        assert!(issues.is_empty());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "real-skill");

        fs::remove_dir_all(temporary).expect("remove fixture tree");
    }
}
