//! `learning.queue`: age-based backlog check for the asynchronous learning queue.
//!
//! A record sitting in `memory/accepted` is normal for a while. Only records that
//! have not changed for longer than the worker's stale threshold are a finding,
//! and that finding is advisory: it never fails `prometheus doctor`.

use super::{CheckResult, CheckStatus, RepairAction, Severity};
use chrono::{DateTime, Utc};
use prometheus_learn::memory::SurrealMemoryClient;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const DEFAULT_STALE_AFTER: &str = "6h";
const OLDEST_LISTED: usize = 5;
const SERVER_TIMEOUT: Duration = Duration::from_secs(3);
const JOB_STATES: [&str; 4] = ["pending", "processing", "completed", "rejected"];
const MEMORY_STATES: [&str; 6] = [
    "pending",
    "submitting",
    "accepted",
    "stalled",
    "completed",
    "rejected",
];
/// Directories whose records are still waiting on something.
const IN_FLIGHT_DIRS: [&str; 5] = [
    "pending",
    "processing",
    "memory/pending",
    "memory/submitting",
    "memory/accepted",
];

#[derive(Debug, Clone)]
struct Pending {
    id: String,
    state: String,
    age: Duration,
}

pub(super) fn queue_root(home: &Path) -> PathBuf {
    std::env::var_os("PROMETHEUS_LEARNING_QUEUE")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".prometheus/learning-queue"))
}

pub(super) fn count_json(root: &Path, relative: &str) -> usize {
    json_files(&root.join(relative)).len()
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect()
}

/// Parse `90s`, `30m`, `6h`, `2d` or a bare number of seconds.
fn parse_duration(text: &str) -> Option<Duration> {
    let text = text.trim();
    let (digits, unit) = match text.char_indices().find(|(_, c)| !c.is_ascii_digit()) {
        Some((index, _)) => text.split_at(index),
        None => (text, "s"),
    };
    let value: u64 = digits.parse().ok()?;
    let factor = match unit.trim() {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        _ => return None,
    };
    Some(Duration::from_secs(value.checked_mul(factor)?))
}

fn stale_after() -> (String, Duration) {
    let raw = std::env::var("PROMETHEUS_LEARNING_STALE_AFTER")
        .ok()
        .filter(|value| parse_duration(value).is_some())
        .unwrap_or_else(|| DEFAULT_STALE_AFTER.to_string());
    let duration = parse_duration(&raw).unwrap_or(Duration::from_secs(6 * 3600));
    (raw, duration)
}

fn format_age(age: Duration) -> String {
    let secs = age.as_secs();
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        _ => format!("{}h{:02}m", secs / 3600, (secs % 3600) / 60),
    }
}

fn since(timestamp: &str, now: DateTime<Utc>) -> Option<Duration> {
    let parsed = DateTime::parse_from_rfc3339(timestamp).ok()?;
    Some(
        now.signed_duration_since(parsed.with_timezone(&Utc))
            .to_std()
            .unwrap_or_default(),
    )
}

/// Mirrors the worker's `stale_since`: `lastReceiptChangeAt`, then
/// `firstAcceptedAt`, then `queuedAt`, then the file's mtime.
fn record_age(path: &Path, value: &serde_json::Value, now: DateTime<Utc>) -> Duration {
    ["lastReceiptChangeAt", "firstAcceptedAt", "queuedAt"]
        .iter()
        .filter_map(|key| value.get(*key).and_then(|v| v.as_str()))
        .find_map(|timestamp| since(timestamp, now))
        .unwrap_or_else(|| file_age(path))
}

fn file_age(path: &Path) -> Duration {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .unwrap_or_default()
}

fn scan_in_flight(root: &Path, now: DateTime<Utc>) -> Vec<Pending> {
    let mut found = Vec::new();
    for relative in IN_FLIGHT_DIRS {
        let is_memory = relative.starts_with("memory/");
        for path in json_files(&root.join(relative)) {
            let value = fs::read_to_string(&path)
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                .unwrap_or(serde_json::Value::Null);
            let stem = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            let id = value
                .get("operationId")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .unwrap_or(stem);
            let age = if is_memory {
                record_age(&path, &value, now)
            } else {
                file_age(&path)
            };
            found.push(Pending {
                id,
                state: relative.to_string(),
                age,
            });
        }
    }
    found.sort_by(|a, b| b.age.cmp(&a.age));
    found
}

fn count_line(root: &Path, label: &str, states: &[&str], prefix: &str) -> String {
    let counts: Vec<String> = states
        .iter()
        .map(|state| count_json(root, &format!("{prefix}{state}")).to_string())
        .collect();
    format!("{label} {} ({})", counts.join("/"), states.join("/"))
}

fn http_client() -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(SERVER_TIMEOUT)
        .no_proxy()
        .build()
        .ok()
}

async fn get_json(client: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

fn describe_stats(stats: &serde_json::Value) -> String {
    let paused = stats
        .get("paused")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "?".into());
    match stats.get("oldest_nonterminal").filter(|v| !v.is_null()) {
        Some(oldest) => format!(
            "server: paused {paused}, oldest non-terminal {} state {} age {}s",
            oldest
                .get("operation_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?"),
            oldest.get("state").and_then(|v| v.as_str()).unwrap_or("?"),
            oldest
                .get("age_seconds")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".into()),
        ),
        None => format!("server: paused {paused}, no non-terminal operations"),
    }
}

async fn server_lines(base: &str, ids: &[&str]) -> Vec<String> {
    let Some(client) = http_client() else {
        return vec!["server: could not construct an HTTP client".into()];
    };
    let Some(stats) = get_json(&client, &format!("{base}/api/v2/operations/stats")).await else {
        return vec![format!("server: surreal-memory not reachable at {base}")];
    };
    let mut lines = vec![describe_stats(&stats)];
    for id in ids {
        let state = get_json(&client, &format!("{base}/api/v2/operations/{id}"))
            .await
            .and_then(|op| op.get("state").and_then(|v| v.as_str()).map(str::to_owned));
        lines.push(format!(
            "server: {id} state {}",
            state.as_deref().unwrap_or("unknown")
        ));
    }
    lines
}

fn action(id: &str, description: String, hint: String, blocked: Option<&str>) -> RepairAction {
    RepairAction {
        id: id.into(),
        description,
        safe: false,
        reversible: true,
        dry_run_only: false,
        command_hint: Some(hint),
        reason_blocked: blocked.map(str::to_owned),
    }
}

fn stale_actions(raw_threshold: &str, base: &str, oldest: &str) -> Vec<RepairAction> {
    let review =
        Some("Review the dry run first; this is an operator decision, not an automatic repair.");
    vec![
        action(
            "learning.queue-quarantine-preview",
            "Preview which stale records the worker would move to memory/stalled.".into(),
            format!("prometheus-learning-worker quarantine --older-than {raw_threshold} --dry-run"),
            None,
        ),
        action(
            "learning.queue-quarantine",
            "Quarantine stale records so they stop counting as backlog.".into(),
            format!("prometheus-learning-worker quarantine --older-than {raw_threshold}"),
            review,
        ),
        action(
            "learning.queue-release",
            "Release quarantined records back into the queue.".into(),
            "prometheus-learning-worker release --all".into(),
            review,
        ),
        action(
            "learning.queue-retry",
            format!("Re-drive operation {oldest} on the memory server."),
            format!("curl -X POST {base}/api/v2/operations/{oldest}/retry"),
            review,
        ),
        action(
            "learning.queue-reject",
            format!("Dead-letter operation {oldest} on the memory server."),
            format!(
                "curl -X POST -H 'content-type: application/json' -d '{{\"reason\":\"operator\"}}' {base}/api/v2/operations/{oldest}/reject"
            ),
            review,
        ),
        RepairAction {
            id: "learning.queue-restart-note".into(),
            description: "Restarting services will not clear this backlog.".into(),
            safe: false,
            reversible: true,
            dry_run_only: false,
            command_hint: None,
            reason_blocked: Some(
                "Stale records are waiting on a receipt, not on a dead process; restarting the worker does not change them.".into(),
            ),
        },
    ]
}

pub(super) async fn check_learning_queue() -> CheckResult {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let root = queue_root(&home);
    let (raw_threshold, threshold) = stale_after();
    let in_flight = scan_in_flight(&root, Utc::now());
    let stale: Vec<&Pending> = in_flight.iter().filter(|p| p.age >= threshold).collect();
    let fresh = in_flight.len() - stale.len();
    let legacy = count_json(&root, "retry")
        + count_json(&root, "memory/retry")
        + count_json(&root, "dead-letter")
        + count_json(&root, "memory/dead-letter");
    let base = SurrealMemoryClient::from_env()
        .map(|client| client.base_url().to_string())
        .unwrap_or_else(|| "http://127.0.0.1:23001".into());
    let oldest: Vec<&Pending> = in_flight.iter().take(OLDEST_LISTED).collect();
    let ids: Vec<&str> = oldest.iter().map(|p| p.id.as_str()).collect();

    let mut details = vec![
        format!("queue: {}", root.display()),
        count_line(&root, "jobs", &JOB_STATES, ""),
        count_line(&root, "memory", &MEMORY_STATES, "memory/"),
        format!("legacy retry/dead-letter: {legacy}"),
        format!("stale after {raw_threshold} (PROMETHEUS_LEARNING_STALE_AFTER); {fresh} in flight, {} stale", stale.len()),
    ];
    if let Some(first) = in_flight.first() {
        details.push(format!(
            "oldest: {} in {} for {}",
            first.id,
            first.state,
            format_age(first.age)
        ));
    }
    details.extend(
        oldest
            .iter()
            .map(|p| format!("  {} {} {}", p.id, p.state, format_age(p.age))),
    );
    details.extend(server_lines(&base, &ids).await);

    let needs_attention = !stale.is_empty() || legacy > 0;
    if needs_attention {
        details.push("Restarting services will not clear this; use quarantine/release or the per-operation retry/reject.".into());
    }
    let oldest_id = in_flight.first().map(|p| p.id.as_str()).unwrap_or("<id>");
    CheckResult {
        id: "learning.queue".into(),
        group: "learning".into(),
        label: "Learning queue backlog".into(),
        severity: if needs_attention {
            Severity::Yellow
        } else {
            Severity::Green
        },
        status: if needs_attention {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        },
        summary: summary(
            fresh,
            stale.len(),
            legacy,
            in_flight.first(),
            &raw_threshold,
        ),
        details,
        optional: true,
        actions: if needs_attention {
            stale_actions(&raw_threshold, &base, oldest_id)
        } else {
            vec![]
        },
    }
}

fn summary(
    fresh: usize,
    stale: usize,
    legacy: usize,
    oldest: Option<&Pending>,
    raw: &str,
) -> String {
    let age = oldest
        .map(|p| format!(", oldest {}", format_age(p.age)))
        .unwrap_or_default();
    if stale == 0 && legacy == 0 {
        return format!("{fresh} record(s) in flight, none older than {raw}{age}");
    }
    format!("{stale} stale record(s) older than {raw}, {fresh} in flight, legacy retry/dead {legacy}{age}")
}
