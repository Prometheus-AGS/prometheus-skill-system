//! Self-learning pipeline for prometheus-skill-pack.
//!
//! Captures execution traces, compiles them into durable knowledge via the
//! Karpathy method (prometheus-knowledge), and optimizes skill prompts via
//! dspy-rs. Integrates with surreal-memory for graph-based reasoning.
//!
//! ## Feature Flags
//!
//! - `knowledge` — Enables knowledge compilation via prometheus-knowledge
//! - `optimize` — Enables prompt optimization via dspy-rs
//! - `full` — Enables both
//!
//! Without feature flags, the crate provides trace capture, evaluation,
//! SKILL.md parsing, and surreal-memory client — no LLM API access needed.

pub mod compile;
pub mod evaluate;
pub mod memory;
pub mod optimize;
pub mod trace;

pub use compile::KnowledgeCompiler;
pub use optimize::{parse_skill_to_signature, SkillOptimizer, SkillSignature, OptimizationReport};
pub use trace::{ExecutionTrace, TraceStore};
