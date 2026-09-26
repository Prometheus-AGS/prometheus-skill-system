use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub run_id: String,
    pub message: String,
    pub idempotency_id: String,
    pub max_turns: u16,
    pub max_retries: u16,
    pub max_duration_ms: u64,
    pub max_output_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UarEvent {
    ModelDelta { text: String },
    ToolApprovalRequired { tool: String },
    ToolResult { tool: String, result: String },
    A2uiSurface { surface_id: String, json: String },
    Completed,
    Cancelled,
    Unknown { event_type: String, artifact: String },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PersistedProjection {
    pub run_id: String,
    pub events: Vec<UarEvent>,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    BudgetExceeded,
    Cancelled,
    GovernanceDenied,
    Persistence(String),
}

pub trait UarRuntimeFacade {
    fn run(&mut self, request: RunRequest) -> Result<Vec<UarEvent>, RuntimeError>;
    fn cancel(&mut self, run_id: &str) -> Result<(), RuntimeError>;
    fn recover(&self, run_id: &str) -> Result<Option<PersistedProjection>, RuntimeError>;
}

#[derive(Default)]
pub struct DeterministicUarRuntime {
    projections: BTreeMap<String, PersistedProjection>,
}

impl UarRuntimeFacade for DeterministicUarRuntime {
    fn run(&mut self, request: RunRequest) -> Result<Vec<UarEvent>, RuntimeError> {
        if request.max_turns == 0
            || request.max_duration_ms == 0
            || request.max_output_bytes == 0
        {
            return Err(RuntimeError::BudgetExceeded);
        }
        if let Some(existing) = self.projections.get(&request.run_id) {
            return Ok(existing.events.clone());
        }
        let events = vec![
            UarEvent::ModelDelta {
                text: format!("Received: {}", request.message),
            },
            UarEvent::ToolApprovalRequired {
                tool: "governed.echo".to_owned(),
            },
            UarEvent::ToolResult {
                tool: "governed.echo".to_owned(),
                result: request.message,
            },
            UarEvent::A2uiSurface {
                surface_id: "result".to_owned(),
                json: r#"{"component":"text","props":{"value":"complete"}}"#.to_owned(),
            },
            UarEvent::Completed,
        ];
        self.projections.insert(
            request.run_id.clone(),
            PersistedProjection {
                run_id: request.run_id,
                events: events.clone(),
                completed: true,
            },
        );
        Ok(events)
    }

    fn cancel(&mut self, run_id: &str) -> Result<(), RuntimeError> {
        let projection = self
            .projections
            .entry(run_id.to_owned())
            .or_insert_with(|| PersistedProjection {
                run_id: run_id.to_owned(),
                ..PersistedProjection::default()
            });
        projection.events.push(UarEvent::Cancelled);
        Ok(())
    }

    fn recover(&self, run_id: &str) -> Result<Option<PersistedProjection>, RuntimeError> {
        Ok(self.projections.get(run_id).cloned())
    }
}
