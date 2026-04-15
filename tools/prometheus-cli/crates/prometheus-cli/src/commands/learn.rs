use anyhow::Result;
use colored::Colorize;
use prometheus_agents::trace_protocol::{build_trace_captures, TraceCapture};
use prometheus_learn::trace::{ExecutionTrace, TraceClassification, TraceStore};
use prometheus_learn::evaluate::evaluate_trace;
use std::path::Path;

pub async fn run(capture_session: bool, seed: bool, compile: bool, lint: bool, dry_run: bool) -> Result<()> {
    println!("{}", "🧠 Self-Learning Pipeline".bold());

    if dry_run {
        println!("  {}", "(dry run — no changes will be written)".dimmed());
    }

    let project_root = Path::new(".");
    let store = TraceStore::default_for_project(project_root);

    // ─── Capture session traces from all platforms ───────────────────
    if capture_session || seed {
        println!("\n  {} Capturing traces from detected platforms...", "→".cyan());

        let captures = build_trace_captures();
        let mut total_captured = 0usize;

        for capture in &captures {
            if !capture.is_available() {
                continue;
            }

            let platform_name = format!("{:?}", capture.platform());
            print!("  {} {}... ", "▸".dimmed(), platform_name);

            match capture.capture_latest() {
                Ok(Some(platform_trace)) => {
                    let invocations = platform_trace.skill_invocations.len();
                    let tool_calls = platform_trace.tool_calls_total;

                    if invocations == 0 && !seed {
                        println!("{} (no skill invocations)", "—".dimmed());
                        continue;
                    }

                    // Convert platform trace → execution traces
                    for invocation in &platform_trace.skill_invocations {
                        let mut exec_trace = ExecutionTrace {
                            id: format!("{}-{}", platform_trace.session_id, invocation.skill_name),
                            skill_name: invocation.skill_name.clone(),
                            timestamp: chrono::Utc::now(),
                            duration_ms: 0,
                            input_summary: format!("/{} {}", invocation.skill_name, invocation.args),
                            output_summary: String::new(), // would need tool_result correlation
                            tool_calls: invocation.tool_calls.iter().map(|tc| {
                                prometheus_learn::trace::ToolCall {
                                    tool_name: tc.tool_name.clone(),
                                    input_summary: String::new(),
                                    output_summary: String::new(),
                                    success: tc.success,
                                    duration_ms: 0,
                                }
                            }).collect(),
                            errors: vec![],
                            classification: TraceClassification::Success,
                            score: None,
                            reflection: None,
                            project_path: platform_trace.project_path.clone(),
                        };

                        // Evaluate the trace
                        evaluate_trace(&mut exec_trace);

                        if !dry_run {
                            store.store(&exec_trace)?;
                        }
                        total_captured += 1;
                    }

                    // If seeding, also capture the session-level summary as a trace
                    if seed && invocations == 0 && tool_calls > 0 {
                        let mut session_trace = ExecutionTrace {
                            id: platform_trace.session_id.clone(),
                            skill_name: "session".to_string(),
                            timestamp: chrono::Utc::now(),
                            duration_ms: platform_trace.duration_ms,
                            input_summary: "Full session".to_string(),
                            output_summary: format!("{} tool calls across session", tool_calls),
                            tool_calls: vec![],
                            errors: vec![],
                            classification: TraceClassification::Success,
                            score: Some(0.8),
                            reflection: None,
                            project_path: platform_trace.project_path.clone(),
                        };
                        evaluate_trace(&mut session_trace);
                        if !dry_run {
                            store.store(&session_trace)?;
                        }
                        total_captured += 1;
                    }

                    println!("{} ({} invocations, {} tool calls)",
                        "✅".green(), invocations, tool_calls);
                }
                Ok(None) => {
                    println!("{}", "no data".dimmed());
                }
                Err(e) => {
                    println!("{} {}", "❌".red(), e);
                }
            }
        }

        println!("\n  Total traces captured: {}", total_captured.to_string().cyan());
    }

    // ─── Compile traces into knowledge ──────────────────────────────
    if compile || (!capture_session && !seed && !lint) {
        println!("\n  {} Compiling traces into knowledge...", "→".cyan());
        let trace_count = store.count_all().unwrap_or(0);
        println!("  Total traces available: {}", trace_count.to_string().cyan());

        if trace_count == 0 {
            println!("  {} No traces found. Run with --capture-session or --seed first.", "⚠️".yellow());
            return Ok(());
        }

        #[cfg(feature = "knowledge")]
        {
            let wiki_dir = project_root.join(".prometheus").join("wiki");
            let compiler = prometheus_learn::KnowledgeCompiler::new(&wiki_dir).await?;
            // Load all traces and compile
            // ... (full implementation when feature enabled)
        }

        #[cfg(not(feature = "knowledge"))]
        {
            println!("  {} Knowledge compilation requires the `knowledge` feature", "ℹ️".dimmed());
            println!("  {} Rebuild with: cargo build -p prometheus-cli --features prometheus-learn/knowledge", "ℹ️".dimmed());
        }
    }

    // ─── Lint compiled knowledge ────────────────────────────────────
    if lint {
        println!("\n  {} Running knowledge lint...", "→".cyan());
        let wiki_dir = project_root.join(".prometheus").join("wiki");
        if wiki_dir.exists() {
            let count = std::fs::read_dir(&wiki_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .count();
            println!("  Wiki entries: {}", count.to_string().cyan());
        } else {
            println!("  {} No wiki entries found. Run --compile first.", "⚠️".yellow());
        }
    }

    println!("\n{}", "✨ Pipeline complete".green().bold());
    Ok(())
}
