pub mod metrics;
pub use metrics::{answer_exact_match, answer_passage_match, exact_match};

use crate::error::Result;
use crate::modules::Module;
use crate::primitives::{Example, Prediction};
use futures::stream::{self, StreamExt};
use std::sync::Arc;

pub type Metric = fn(&Example, &Prediction) -> f64;

#[derive(Clone)]
pub struct Dataset {
    pub examples: Vec<Example>,
}

impl Dataset {
    pub fn new(examples: Vec<Example>) -> Self {
        Self { examples }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Example> {
        self.examples.iter()
    }
}

pub struct Evaluate {
    pub metric: Metric,
    pub num_threads: usize,
    pub display_progress: bool,
    pub display_table: bool,
}

impl Evaluate {
    pub fn new(metric: Metric, num_threads: usize) -> Self {
        Self {
            metric,
            num_threads,
            display_progress: true,
            display_table: false,
        }
    }

    pub async fn run(&self, module: &(impl Module + Sync), dataset: &Dataset) -> Result<f64> {
        let total = dataset.examples.len();
        if total == 0 {
            return Ok(0.0);
        }

        let module = Arc::new(module); // Ensure module can be shared

        let results = stream::iter(dataset.examples.iter().cloned())
            .map(|example| {
                let module = module.clone(); // Clone Arc for each task (if needed, though reference might suffice if static lifetime issues didn't exist)
                // Actually, since module is &impl Module, we can't easily clone it unless it's Arc.
                // The caller passes &module.
                // To support concurrency, we probably need the module to be Clone or Arc internally handled.
                // For now, let's assume the caller handles Arc or the module is cheap to clone?
                // Wait, forward takes &self.
                // We'll restrict concurrency based on num_threads using buffer_unordered.
                async move {
                    // We need to access module safely.
                    // Since forward is async &self, we can share reference if valid.
                    // But async move block requires ownership or static reference.
                    // We wrap module in a helper or assume it's safe.
                    // Actually, simpler to just use the reference if we scope it right, but stream combinators usually require 'static or owned.
                    // Let's unsafe wrap? No.
                    // Let's require the module implementation to be robust.
                    // Ideally `module` here should be `Arc<dyn Module>` or similar.
                    // But `run` takes `&impl Module`.
                    // Let's change `run` to take `&Arc<impl Module>` or force caller to handle it?
                    // No, let's look at how Majority handled it. `Arc<dyn Module>`.
                    // For now, let's just do sequential if we can't solve lifetime easily, OR better:
                    // Actually, `stream::iter` doesn't require static if we await in current scope...
                    // BUT `buffer_unordered` does if it spawns? No, it just polls.
                    // However, the *closure* must be valid.
                    let pred = module.forward(&example).await;
                    (example, pred)
                }
            })
            // We need to deal with the lifetime issue: `module` reference must outlive the stream.
            // Since we await the stream in this function, it should be fine IF the compiler understands it.
            // But usually async blocks capture references and that makes the Future non-static.
            // `buffer_unordered` works with non-static futures as long as we await the stream itself here.
            .buffer_unordered(self.num_threads);

        // We can't really use Arc wrapping inside unless `module` input is changed.
        // Let's try attempting it; if lifetime errors, we'll ask user to change signature or clone.
        // Actually, Python DSPy often copies modules.

        // Let's collect results
        let mut score_sum = 0.0;
        let mut count = 0;

        // Pinning results stream is handled by iterating it
        let mut stream = results;
        while let Some((example, pred_res)) = stream.next().await {
            match pred_res {
                Ok(prediction) => {
                    let score = (self.metric)(&example, &prediction);
                    score_sum += score;
                    count += 1;
                    if self.display_progress {
                        // Simple print
                        // In real app, use indicatif or log
                    }
                }
                Err(e) => {
                    eprintln!("Error evaluating example: {}", e);
                }
            }
        }

        Ok(if count > 0 {
            score_sum / count as f64
        } else {
            0.0
        })
    }
}

// NOTE: To fix the lifetime issue cleanly without unsafe, we might need `module` to be Clone or Arc.
// For this iteration, let's assume `run` might block if we don't fix lifetimes, but `buffer_unordered` allows concurrency.
// A common pattern:
// fn run<M: Module + Sync>(..., module: &M, ...)
// The future returned by map contains &M.
// buffer_unordered requires Future: Send. &M is Send if M is Sync.
// So this should compile.
