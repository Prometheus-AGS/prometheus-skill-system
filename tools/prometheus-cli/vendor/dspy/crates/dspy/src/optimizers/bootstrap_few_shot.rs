use crate::evaluations::{Dataset, Metric};
use crate::modules::{Module, Predict};
use crate::primitives::Example;
use std::sync::Arc;

pub struct BootstrapFewShot<M: Module> {
    pub teacher: Arc<M>,
    pub metric: Metric,
    pub max_demos: usize,
}

impl<M: Module> BootstrapFewShot<M> {
    pub fn new(teacher: Arc<M>, metric: Metric) -> Self {
        Self {
            teacher,
            metric,
            max_demos: 5,
        }
    }

    pub async fn compile(&self, mut student: Predict, dataset: &Dataset) -> Predict {
        // Simple bootstrapping loop
        let mut demos = Vec::new();

        println!("Compiling demos...");
        for example in &dataset.examples {
            if demos.len() >= self.max_demos {
                break;
            }

            // Teacher prediction
            if let Ok(prediction) = self.teacher.forward(example).await {
                // Check metric
                let score = (self.metric)(example, &prediction);
                if score >= 1.0 {
                    // Assume metric returns 1.0 for success, or > 0.0
                    println!("Bootstrapping: Success!");
                    let mut _demo = example.clone();
                    // Add traces ...
                    // Add to demos
                    // A valid demo is Input + Output (Prediction)
                    // Implementation detail: we merge input fields and prediction fields into one Example
                    let mut demo_record = example.record.clone();
                    for (k, v) in &prediction.record.fields {
                        demo_record.insert(k, v.clone());
                    }

                    println!("  Found valid demo!");
                    demos.push(Example::new(demo_record));
                }
            }
        }

        println!("Compiled {} demos.", demos.len());
        student.demos = demos;
        student
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::primitives::{Prediction, Record};
    use crate::signatures::Signature;
    use async_trait::async_trait;

    // Mock Module for testing
    struct MockTeacher {
        should_succeed: bool,
    }

    #[async_trait]
    impl Module for MockTeacher {
        async fn forward(&self, _example: &Example) -> Result<Prediction> {
            if self.should_succeed {
                let mut record = Record::new();
                record.insert("answer", "42"); // Always predict 42
                Ok(Prediction::new(record))
            } else {
                Err(crate::error::DspyError::ModelCall("Fail".into()))
            }
        }
    }

    #[tokio::test]
    async fn test_bootstrap_collects_valid_demos() {
        let teacher = Arc::new(MockTeacher {
            should_succeed: true,
        });

        // Metric: Accept if answer is "42"
        let metric: Metric = |_example, pred| {
            if let Some(val) = pred.record.get("answer") {
                if val.as_str().unwrap_or("") == "42" {
                    1.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        let optimizer = BootstrapFewShot {
            teacher: teacher,
            metric,
            max_demos: 2,
        };

        // Create dummy signature and student
        let signature = Signature::new("TestSig");
        // We need a dummy model for the student, generic is fine since we won't run the student here, just compile it
        // But Predict requires a model. We can use the same MockTeacher if it implements helper or just OpenAIModel if available.
        // Actually Predict uses Arc<dyn LanguageModel>. Let's assume we can pass a dummy.
        // For unit test simplicity, we might strictly need a real Predict instance.
        // Let's rely on standard creation. Explicitly mocking LanguageModel might be verbose.
        // We will just use the real Predict with a dummy provider if possible, or just build it manually.
        // Since Predict fields are public we can construct it? No, fields are private likely.
        // We use Predict::new. It needs a model.
        // Let's implement a MockModel to satisfy the student creation.

        struct MockModel;

        #[async_trait]
        impl crate::models::LanguageModel for MockModel {
            async fn complete(
                &self,
                _request: crate::models::ModelRequest,
            ) -> Result<crate::models::ModelResponse> {
                Ok(crate::models::ModelResponse {
                    content: "dummy".into(),
                    usage: None,
                })
            }

            async fn stream(
                &self,
                _request: crate::models::ModelRequest,
            ) -> Result<futures::stream::BoxStream<'static, Result<crate::primitives::Chunk>>>
            {
                todo!("MockModel stream")
            }
        }

        let model = Arc::new(MockModel);
        let student = Predict::new(signature.clone(), model);

        // Dataset
        let mut examples = Vec::new();
        for _ in 0..5 {
            let mut r = Record::new();
            r.insert("question", "val");
            examples.push(Example::new(r));
        }
        let dataset = Dataset::new(examples);

        // Compile
        let compiled_student = optimizer.compile(student, &dataset).await;

        // Check demos
        assert_eq!(compiled_student.demos.len(), 2); // Max demos reached

        // Verify demo content
        let demo = &compiled_student.demos[0];
        assert_eq!(demo.record.get("answer").unwrap().as_str().unwrap(), "42");
    }
}
