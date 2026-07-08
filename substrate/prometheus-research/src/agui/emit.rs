use crate::agui::AguiEvent;

#[derive(serde::Serialize)]
struct UiIntent {
    intent: String,
    payload: serde_json::Value,
}

pub async fn emit_to_surface_bridge(event: &AguiEvent, bridge_url: &str) {
    let intent = UiIntent {
        intent: "research.event".to_string(),
        payload: match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("surface-bridge: failed to serialize event: {e}");
                return;
            }
        },
    };

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
