pub mod emit;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AguiEvent {
    #[serde(rename = "agent.status")]
    AgentStatus {
        job_id: String,
        stage: u32,
        stage_name: String,
        progress: u32,
        status: String,
        tokens: u64,
        timestamp: String,
    },
    #[serde(rename = "agent.message")]
    AgentMessage {
        job_id: String,
        message: String,
        level: String,
        timestamp: String,
    },
    #[serde(rename = "agent.error")]
    AgentError {
        job_id: String,
        message: String,
        stage: u32,
        timestamp: String,
    },
    #[serde(rename = "a2ui.component")]
    A2uiComponent {
        job_id: String,
        component: String,
        props: serde_json::Value,
        timestamp: String,
    },
}
