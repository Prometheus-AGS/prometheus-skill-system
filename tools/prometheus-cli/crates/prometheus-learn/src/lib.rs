//! Self-learning pipeline for prometheus-skill-pack.
//!
//! Captures execution traces, compiles them into durable knowledge via the
//! Karpathy method (prometheus-knowledge), and optimizes skill prompts via
//! dspy-rs. Integrates with surreal-memory for graph-based reasoning.

pub mod evaluate;
pub mod memory;
pub mod optimize;
pub mod trace;

pub use trace::{ExecutionTrace, TraceStore};
