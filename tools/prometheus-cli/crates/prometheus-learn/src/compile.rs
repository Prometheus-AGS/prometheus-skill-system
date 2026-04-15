//! Knowledge compilation via prometheus-knowledge (Karpathy method).
//!
//! Converts execution traces into durable, searchable WikiEntries through
//! the compile → lint → focus → fix pipeline. Requires the `knowledge` feature.
//!
//! When the feature is disabled, provides stub implementations that return
//! informative errors pointing to the feature flag.

use crate::memory::SurrealMemoryClient;
use crate::trace::ExecutionTrace;
use std::path::{Path, PathBuf};

/// Result of a knowledge compilation run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompilationReport {
    pub traces_compiled: usize,
    pub entries_created: usize,
    pub entries_updated: usize,
    pub lint_issues: usize,
    pub wiki_dir: PathBuf,
}

// ─── Feature-gated implementation ───────────────────────────────────────────

#[cfg(feature = "knowledge")]
mod inner {
    use super::*;
    use pk_core::types::{RawDoc, RawDocMediaType, WikiEntry};
    use pk_librarian::Librarian;
    use pk_store::MarkdownStore;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    pub struct KnowledgeCompiler {
        librarian: Arc<Librarian>,
        memory_client: Option<SurrealMemoryClient>,
        wiki_dir: PathBuf,
    }

    impl KnowledgeCompiler {
        pub async fn new(wiki_dir: &Path) -> anyhow::Result<Self> {
            let store = Arc::new(MarkdownStore::open(wiki_dir).await?);
            let router = pk_librarian::ModelRouter::from_env();
            let (tx, _) = broadcast::channel(64);
            let librarian = Arc::new(Librarian::new(store, router, tx));

            Ok(Self {
                librarian,
                memory_client: SurrealMemoryClient::from_env(),
                wiki_dir: wiki_dir.to_path_buf(),
            })
        }

        pub async fn compile_trace(&self, trace: &ExecutionTrace) -> anyhow::Result<WikiEntry> {
            let raw = RawDoc {
                id: trace.id.clone(),
                source_path: format!("trace/{}/{}", trace.skill_name, trace.id),
                content: trace.to_markdown(),
                media_type: RawDocMediaType::Markdown,
                ingested_at: chrono::Utc::now(),
                session_id: None,
            };

            let entry = self.librarian.compile(raw).await?;

            // Sync to surreal-memory if available
            if let Some(mem) = &self.memory_client {
                let observations = vec![
                    entry.title.clone(),
                    format!("tags: {}", entry.tags.join(", ")),
                    format!("skill: {}", trace.skill_name),
                    format!("score: {:.2}", trace.score.unwrap_or(0.0)),
                ];
                let _ = mem.create_entity("lesson", entry.id.as_str(), &observations).await;
                for tag in &entry.tags {
                    let _ = mem.create_relation(entry.id.as_str(), "tagged_with", tag).await;
                }
            }

            tracing::info!(
                id = %entry.id,
                title = %entry.title,
                "compiled trace into wiki entry"
            );
            Ok(entry)
        }

        pub async fn compile_batch(&self, traces: &[ExecutionTrace]) -> anyhow::Result<CompilationReport> {
            let mut created = 0usize;
            let mut updated = 0usize;

            for trace in traces {
                match self.compile_trace(trace).await {
                    Ok(entry) => {
                        if entry.revision <= 1 { created += 1; } else { updated += 1; }
                    }
                    Err(e) => {
                        tracing::warn!(trace_id = %trace.id, error = %e, "failed to compile trace");
                    }
                }
            }

            let lint_issues = self.librarian.lint().await
                .map(|reports| reports.len())
                .unwrap_or(0);

            Ok(CompilationReport {
                traces_compiled: traces.len(),
                entries_created: created,
                entries_updated: updated,
                lint_issues,
                wiki_dir: self.wiki_dir.clone(),
            })
        }

        pub async fn focus(&self, query: &str) -> anyhow::Result<String> {
            self.librarian.focus(query, 5).await.map_err(Into::into)
        }
    }
}

// ─── Stub implementation when feature is disabled ───────────────────────────

#[cfg(not(feature = "knowledge"))]
mod inner {
    use super::*;

    pub struct KnowledgeCompiler;

    impl KnowledgeCompiler {
        pub async fn new(_wiki_dir: &Path) -> anyhow::Result<Self> {
            anyhow::bail!(
                "Knowledge compilation requires the `knowledge` feature.\n\
                 Rebuild with: cargo build --features knowledge\n\
                 Requires: prometheus-knowledge-rs git dependency"
            )
        }
    }
}

pub use inner::KnowledgeCompiler;
