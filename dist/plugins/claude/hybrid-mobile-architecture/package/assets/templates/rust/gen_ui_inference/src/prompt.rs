// TJ-ARCH-MOB-001 compliant
//! Prompt assembly shared by local engines whose model formats have no native
//! tool-call API: the tool schema is injected into the system message, keeping
//! the tool contract entirely in Rust and identical across engines.

use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::inference::{InferenceMessage, InferenceRequest, InferenceRole};

/// The request's effective messages with the tool schema folded into the
/// system message (appended to an existing one, or inserted as a new first
/// message). Errors when every message is blank — a prompt with no content is
/// a caller bug, not something to hand a model.
pub(crate) fn messages_with_tool_context(
    request: &InferenceRequest,
) -> CoreResult<Vec<InferenceMessage>> {
    let mut messages = request.effective_messages();
    if messages
        .iter()
        .all(|message| message.content.trim().is_empty())
    {
        return Err(CoreError::Terminal(
            "mobile inference: request contains no message content".into(),
        ));
    }
    if !request.tools.is_empty() {
        let tool_json = serde_json::to_string(&request.tools)
            .map_err(|error| CoreError::Terminal(format!("tool schema serialize: {error}")))?;
        let instruction = format!(
            "Available tools are described by this JSON. Call a tool only when needed and emit arguments that satisfy its parameters schema:\n{tool_json}"
        );
        if let Some(system) = messages
            .iter_mut()
            .find(|message| message.role == InferenceRole::System)
        {
            system.content.push_str("\n\n");
            system.content.push_str(&instruction);
        } else {
            messages.insert(
                0,
                InferenceMessage {
                    role: InferenceRole::System,
                    content: instruction,
                    name: None,
                    tool_call_id: None,
                },
            );
        }
    }
    Ok(messages)
}
