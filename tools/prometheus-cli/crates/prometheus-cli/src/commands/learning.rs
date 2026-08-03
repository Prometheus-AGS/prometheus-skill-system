use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path, path::PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LearningStatus {
    queue_root: String,
    worker_installed: bool,
    pending: usize,
    processing: usize,
    retry: usize,
    completed: usize,
    dead_letter: usize,
    memory_pending: usize,
    memory_dead_letter: usize,
    last_run: Option<Value>,
}

pub fn status(json: bool) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let queue_root = std::env::var_os("PROMETHEUS_LEARNING_QUEUE")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".prometheus/learning-queue"));
    let worker_installed = std::env::var_os("PROMETHEUS_LEARNING_WORKER_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/bin/prometheus-learning-worker"))
        .is_file();
    let report = LearningStatus {
        queue_root: queue_root.display().to_string(),
        worker_installed,
        pending: json_count(&queue_root.join("pending")),
        processing: json_count(&queue_root.join("processing")),
        retry: json_count(&queue_root.join("retry")),
        completed: json_count(&queue_root.join("completed")),
        dead_letter: json_count(&queue_root.join("dead-letter")),
        memory_pending: json_count(&queue_root.join("memory/pending"))
            + json_count(&queue_root.join("memory/retry")),
        memory_dead_letter: json_count(&queue_root.join("memory/dead-letter")),
        last_run: fs::read(queue_root.join("status.json"))
            .ok()
            .and_then(|raw| serde_json::from_slice(&raw).ok()),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Learning worker installed: {}", report.worker_installed);
        println!("Queue: {}", report.queue_root);
        println!("Pending: {}", report.pending);
        println!("Processing: {}", report.processing);
        println!("Retry: {}", report.retry);
        println!("Completed: {}", report.completed);
        println!("Dead-letter: {}", report.dead_letter);
        println!("Memory pending: {}", report.memory_pending);
        println!("Memory dead-letter: {}", report.memory_dead_letter);
    }
    Ok(())
}

fn json_count(path: &Path) -> usize {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .count()
}
