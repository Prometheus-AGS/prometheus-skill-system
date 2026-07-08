use crate::agui::AguiEvent;

#[derive(serde::Serialize)]
struct UiIntent {
    intent_type: String,
    title: String,
    body: String,
    options: Option<Vec<String>>,
    multiselect: bool,
    request_id: String,
}

fn agui_to_ui_intent(event: &AguiEvent) -> UiIntent {
    match event {
        AguiEvent::AgentStatus {
            job_id,
            stage,
            stage_name,
            progress,
            status,
            ..
        } => UiIntent {
            intent_type: "progress".to_string(),
            title: format!("Stage {stage}: {stage_name}"),
            body: format!("{status} ({progress}%)"),
            options: None,
            multiselect: false,
            request_id: job_id.clone(),
        },
        AguiEvent::AgentMessage {
            job_id,
            message,
            level,
            ..
        } => UiIntent {
            intent_type: "feedback".to_string(),
            title: level.clone(),
            body: message.clone(),
            options: None,
            multiselect: false,
            request_id: job_id.clone(),
        },
        AguiEvent::AgentError {
            job_id,
            message,
            stage,
            ..
        } => UiIntent {
            intent_type: "feedback".to_string(),
            title: format!("Error at stage {stage}"),
            body: message.clone(),
            options: None,
            multiselect: false,
            request_id: job_id.clone(),
        },
        AguiEvent::A2uiComponent {
            job_id,
            component,
            props,
            ..
        } => UiIntent {
            intent_type: "prompt".to_string(),
            title: component.clone(),
            body: props.to_string(),
            options: None,
            multiselect: false,
            request_id: job_id.clone(),
        },
    }
}

pub async fn emit_to_surface_bridge(event: &AguiEvent, bridge_url: &str) {
    let intent = agui_to_ui_intent(event);
    let url = format!("{bridge_url}/mcp/render-ui-intent");
    let client = reqwest::Client::new();
    match client.post(&url).json(&intent).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!("surface-bridge: event emitted to {url}");
        }
        Ok(resp) => {
            tracing::warn!(
                "surface-bridge: unexpected status {} from {url}",
                resp.status()
            );
        }
        Err(e) => {
            tracing::warn!("surface-bridge: emit failed (non-fatal): {e}");
        }
    }
}
