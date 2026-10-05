//! `plugins.source-topology` and `plugins.native-cache-skew`.
//!
//! A plugin whose marketplace `directory` source disappears fails every hook of every
//! turn in every session that loaded it, and nothing used to warn beforehand (the
//! incident: a topic-branch git worktree registered as the source, deleted when its
//! branch merged). The classification lives in one place,
//! `scripts/lib/plugin-source-topology.js`, which the installer scripts also call; this
//! module only runs it, maps the JSON, and reports. Doctor stays read-only.
//!
//! A missing registered source fails the run (the plugin's hooks are broken right now).
//! A topic-branch or dirty source, disagreeing clients, and a native cache that is behind
//! the active generation are advisory warnings. Live sessions on a superseded cache are
//! informational: they keep working while their bundle stays registered.

use super::{CheckResult, CheckStatus, RepairAction, Severity};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const SCRIPT: &str = "scripts/check-plugin-source.js";

pub struct Probe(Result<Value, String>);

/// Candidate locations for the checker, most specific first, so the doctor works from
/// any working directory: an explicit source root, the current checkout, then the
/// installed generation.
fn locate_script(home: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(root) = std::env::var_os("PROMETHEUS_SOURCE_ROOT") {
        candidates.push(PathBuf::from(root).join(SCRIPT));
    }
    candidates.push(PathBuf::from(SCRIPT));
    candidates.push(home.join(".prometheus/plugins/prometheus-skill-pack/current").join(SCRIPT));
    candidates.into_iter().find(|candidate| candidate.is_file())
}

pub fn probe() -> Probe {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let Some(script) = locate_script(&home) else {
        return Probe(Err(format!("{SCRIPT} was not found (checkout or installed generation)")));
    };
    let output = Command::new("node").arg(&script).arg("--json").arg("--home").arg(&home).output();
    Probe(match output {
        Err(error) => Err(format!("cannot run node: {error}")),
        // Exit 1 means "a registered source is missing" and still carries the JSON report.
        Ok(output) => serde_json::from_slice::<Value>(&output.stdout)
            .map_err(|error| format!("{SCRIPT} returned unreadable output: {error}")),
    })
}

fn messages(findings: &Value) -> Vec<String> {
    findings
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|finding| {
                    let severity = finding["severity"].as_str()?;
                    let message = finding["message"].as_str()?;
                    Some(format!("[{severity}] {message}"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn unavailable(id: &str, label: &str, why: &str) -> CheckResult {
    CheckResult {
        id: id.into(),
        group: "plugins".into(),
        label: label.into(),
        severity: Severity::Green,
        status: CheckStatus::Skip,
        summary: "plugin source checker unavailable".into(),
        details: vec![why.into()],
        optional: true,
        actions: vec![],
    }
}

fn review_action(id: &str, description: &str, hint: &str) -> RepairAction {
    RepairAction {
        id: id.into(),
        description: description.into(),
        safe: false,
        reversible: true,
        dry_run_only: false,
        command_hint: Some(hint.into()),
        reason_blocked: Some("Changing a plugin source needs an operator decision; doctor stays read-only.".into()),
    }
}

pub fn source_topology_check(probe: &Probe) -> CheckResult {
    const ID: &str = "plugins.source-topology";
    const LABEL: &str = "Plugin marketplace source";
    let report = match &probe.0 {
        Ok(report) => report,
        Err(why) => return unavailable(ID, LABEL, why),
    };
    let topology = &report["topology"];
    // A report that does not say ok/warn/fail/skip must never read as a pass.
    if !matches!(topology["status"].as_str(), Some("ok" | "warn" | "fail" | "skip")) {
        return unavailable(ID, LABEL, "the checker returned a report without a recognised topology status");
    }
    let details = messages(&topology["findings"]);
    let (status, severity, optional, summary) = match topology["status"].as_str() {
        Some("fail") => (
            CheckStatus::Fail,
            Severity::Red,
            false,
            "a registered plugin source does not exist; the plugin's hooks fail until it is restored",
        ),
        Some("warn") => (
            CheckStatus::Warn,
            Severity::Yellow,
            true,
            "the plugin source is registered but risky (topic branch, dirty tree, or clients disagree)",
        ),
        Some("skip") => (
            CheckStatus::Skip,
            Severity::Green,
            true,
            "no directory marketplace source is registered for this pack",
        ),
        _ => (CheckStatus::Pass, Severity::Green, true, "plugin marketplace sources are durable and agree"),
        // The status was validated above; `_` is exactly "ok".
    };
    let needs_action = matches!(status, CheckStatus::Fail | CheckStatus::Warn);
    CheckResult {
        id: ID.into(),
        group: "plugins".into(),
        label: LABEL.into(),
        severity,
        status,
        summary: summary_text(summary, &details),
        details,
        optional,
        actions: if needs_action {
            vec![review_action(
                "manual.repoint-plugin-source",
                "Point the marketplace at a durable release-line checkout, then restart running sessions.",
                "node scripts/check-plugin-source.js",
            )]
        } else {
            vec![]
        },
    }
}

fn summary_text(base: &str, details: &[String]) -> String {
    if details.is_empty() {
        base.into()
    } else {
        format!("{base} ({} finding(s))", details.len())
    }
}

pub fn native_cache_skew_check(probe: &Probe) -> CheckResult {
    const ID: &str = "plugins.native-cache-skew";
    const LABEL: &str = "Native plugin cache";
    let report = match &probe.0 {
        Ok(report) => report,
        Err(why) => return unavailable(ID, LABEL, why),
    };
    let skew = &report["skew"];
    let Some(findings) = skew["findings"].as_array().cloned() else {
        return unavailable(ID, LABEL, "the checker returned a report without native cache findings");
    };
    // Only claim a comparison that actually happened: both versions present, and no sign
    // that the cache could not be inspected.
    if findings.iter().any(|finding| finding["code"] == "SKEW_UNREADABLE") {
        return unavailable(ID, LABEL, "the native plugin cache could not be inspected");
    }
    let (Some(installed), Some(active)) = (
        skew["installedVersion"].as_str(),
        skew["activeGeneration"].as_str(),
    ) else {
        return unavailable(
            ID,
            LABEL,
            "the installed plugin version or the active generation version could not be determined",
        );
    };
    let behind = findings.iter().any(|finding| finding["code"] == "CACHE_BEHIND_GENERATION")
        || installed != active;
    let mut details = vec![format!(
        "installed Claude plugin: {installed}; active generation: {active}"
    )];
    details.extend(messages(&skew["findings"]));
    CheckResult {
        id: ID.into(),
        group: "plugins".into(),
        label: LABEL.into(),
        severity: if behind { Severity::Yellow } else { Severity::Green },
        status: if behind { CheckStatus::Warn } else { CheckStatus::Pass },
        summary: if behind {
            "the installed native plugin is behind the active generation".into()
        } else if findings.is_empty() {
            "the native plugin cache matches the active generation".into()
        } else {
            "the native plugin cache matches the active generation; some sessions run an older version".into()
        },
        details,
        optional: true,
        actions: if behind {
            vec![review_action(
                "manual.refresh-native-plugins",
                "Refresh the native plugin installs from the active generation.",
                "bash scripts/update-skill-pack.sh",
            )]
        } else {
            vec![]
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn probe(topology: &str, findings: Value, skew: Value) -> Probe {
        Probe(Ok(json!({ "topology": { "status": topology, "findings": findings }, "skew": skew })))
    }

    fn finding(code: &str, severity: &str, message: &str) -> Value {
        json!({ "code": code, "severity": severity, "message": message })
    }

    #[test]
    fn a_missing_source_fails_and_names_the_path() {
        let p = probe(
            "fail",
            json!([finding("SOURCE_MISSING", "fail", "claude marketplace source /gone does not exist")]),
            json!({ "findings": [] }),
        );
        let result = source_topology_check(&p);
        assert!(matches!(result.status, CheckStatus::Fail));
        assert!(!result.optional, "a missing source breaks hooks now, so it is not advisory");
        assert!(result.details.iter().any(|line| line.contains("/gone")));
        assert!(!result.actions.is_empty());
    }

    #[test]
    fn a_topic_branch_source_warns_without_failing_the_run() {
        let p = probe(
            "warn",
            json!([finding("TOPIC_BRANCH", "warn", "source is on topic branch 'fix/x'")]),
            json!({ "findings": [] }),
        );
        let result = source_topology_check(&p);
        assert!(matches!(result.status, CheckStatus::Warn));
        assert!(result.optional, "advisory: it must not fail install or doctor");
    }

    #[test]
    fn a_durable_source_passes_and_a_release_line_worktree_is_only_noted() {
        let p = probe(
            "ok",
            json!([finding("LINKED_WORKTREE", "info", "source is a linked git worktree")]),
            json!({ "findings": [] }),
        );
        let result = source_topology_check(&p);
        assert!(matches!(result.status, CheckStatus::Pass));
        assert!(result.details.iter().any(|line| line.contains("linked git worktree")));
        assert!(result.actions.is_empty());
    }

    #[test]
    fn nothing_registered_is_skipped() {
        let result = source_topology_check(&probe("skip", json!([]), json!({ "findings": [] })));
        assert!(matches!(result.status, CheckStatus::Skip));
    }

    #[test]
    fn live_sessions_on_an_old_version_are_informational_not_a_warning() {
        let skew = json!({
            "installedVersion": "1.11.1",
            "activeGeneration": "1.11.1",
            "findings": [finding("LIVE_SESSIONS_ON_SUPERSEDED", "info", "2 live process(es) (1, 2) still use 1.10.0")],
        });
        let result = native_cache_skew_check(&probe("ok", json!([]), skew));
        assert!(matches!(result.status, CheckStatus::Pass));
        assert!(result.optional);
        assert!(result.details.iter().any(|line| line.contains("1.10.0")));
    }

    #[test]
    fn a_cache_behind_the_generation_warns_with_a_refresh_action() {
        let skew = json!({
            "installedVersion": "1.11.0",
            "activeGeneration": "1.11.2",
            "findings": [finding("CACHE_BEHIND_GENERATION", "warn", "installed 1.11.0, active 1.11.2")],
        });
        let result = native_cache_skew_check(&probe("ok", json!([]), skew));
        assert!(matches!(result.status, CheckStatus::Warn));
        assert!(result.actions.iter().any(|a| a.id == "manual.refresh-native-plugins"));
    }

    #[test]
    fn an_uninspectable_or_incomplete_cache_is_skipped_not_reported_as_matching() {
        let unreadable = json!({
            "installedVersion": null,
            "activeGeneration": null,
            "findings": [finding("SKEW_UNREADABLE", "info", "the native plugin cache could not be inspected: EACCES")],
        });
        assert!(matches!(
            native_cache_skew_check(&probe("ok", json!([]), unreadable)).status,
            CheckStatus::Skip
        ));
        for skew in [
            json!({ "installedVersion": "1.11.1", "activeGeneration": null, "findings": [] }),
            json!({ "installedVersion": null, "activeGeneration": "1.11.1", "findings": [] }),
            json!({ "installedVersion": null, "activeGeneration": null, "findings": [] }),
        ] {
            let result = native_cache_skew_check(&probe("ok", json!([]), skew));
            assert!(matches!(result.status, CheckStatus::Skip), "nothing was compared: {result:?}");
            assert!(!result.summary.contains("matches"));
        }
        let equal = json!({ "installedVersion": "1.11.1", "activeGeneration": "1.11.1", "findings": [] });
        let result = native_cache_skew_check(&probe("ok", json!([]), equal));
        assert!(matches!(result.status, CheckStatus::Pass));
        assert!(result.summary.contains("matches"));
    }

    #[test]
    fn a_malformed_report_is_skipped_never_passed() {
        for body in [json!({}), json!({ "topology": {} }), json!({ "topology": { "status": "banana" } }), json!([])] {
            let p = Probe(Ok(body));
            assert!(
                matches!(source_topology_check(&p).status, CheckStatus::Skip),
                "an unrecognised report must not read as a pass"
            );
        }
        let p = Probe(Ok(json!({ "topology": { "status": "ok", "findings": [] } })));
        assert!(
            matches!(native_cache_skew_check(&p).status, CheckStatus::Skip),
            "a report with no native cache findings array is malformed"
        );
        assert!(matches!(
            source_topology_check(&probe("ok", json!([]), json!({ "findings": [] }))).status,
            CheckStatus::Pass
        ));
    }

    #[test]
    fn an_unavailable_checker_is_skipped_not_failed() {
        let p = Probe(Err("scripts/check-plugin-source.js was not found".into()));
        assert!(matches!(source_topology_check(&p).status, CheckStatus::Skip));
        assert!(matches!(native_cache_skew_check(&p).status, CheckStatus::Skip));
    }
}
