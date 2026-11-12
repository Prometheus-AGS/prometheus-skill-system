use super::Adapter;
use crate::error::{DspyError, Result};
use crate::models::{Message, ModelRequest, ModelResponse};
use crate::primitives::{Example, Prediction, Record, Value};
use crate::signatures::Signature;
use regex;

pub struct ChatAdapter;

impl ChatAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Adapter for ChatAdapter {
    fn format(
        &self,
        signature: &Signature,
        demos: &[Example],
        input: &Example,
    ) -> Result<ModelRequest> {
        // Simple chat formatting:
        // System: Instruction
        // User: Input fields formatted
        // "Field Name: Value"

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: signature.instruction.clone(),
        }];

        // Add demos as Few-Shot examples
        // We simulate them as User -> Assistant turns
        for demo in demos {
            let mut demo_input = String::new();
            for field in &signature.inputs {
                if let Some(val) = demo.record.get(&field.name) {
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        v => serde_json::to_string(v).unwrap_or_default(),
                    };
                    demo_input.push_str(&format!(
                        "{}: {}\n",
                        field.prefix.as_deref().unwrap_or(&field.name),
                        val_str
                    ));
                }
            }
            messages.push(Message {
                role: "user".to_string(),
                content: demo_input,
            });

            let mut demo_output = String::new();
            for field in &signature.outputs {
                if let Some(val) = demo.record.get(&field.name) {
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        v => serde_json::to_string(v).unwrap_or_default(),
                    };
                    demo_output.push_str(&format!(
                        "{}: {}\n",
                        field.prefix.as_deref().unwrap_or(&field.name),
                        val_str
                    ));
                }
            }
            messages.push(Message {
                role: "assistant".to_string(),
                content: demo_output,
            });
        }

        let mut content = String::new();

        // Add inputs
        for field in &signature.inputs {
            let val = input.record.get(&field.name).ok_or_else(|| {
                DspyError::AdapterFormat(format!("Missing input field: {}", field.name))
            })?;

            // Basic stringify
            let val_str = match val {
                Value::String(s) => s.clone(),
                v => serde_json::to_string(v).unwrap_or_default(),
            };

            content.push_str(&format!(
                "{}: {}\n",
                field.prefix.as_deref().unwrap_or(&field.name),
                val_str
            ));
        }

        // Add output prefixes to prompt the model
        content.push_str("\nRespond with:\n");
        for field in &signature.outputs {
            content.push_str(&format!(
                "{}: ...\n",
                field.prefix.as_deref().unwrap_or(&field.name)
            ));
        }

        messages.push(Message {
            role: "user".to_string(),
            content,
        });

        Ok(ModelRequest {
            messages,
            temperature: 0.0,
            max_tokens: None,
        })
    }

    fn parse(&self, signature: &Signature, response: &ModelResponse) -> Result<Prediction> {
        let mut record = Record::new();
        let content = &response.content;

        // Strategy 1: Regex for "Field: Value"
        // We iterate over fields and try to find them in the content.
        // Regex: (?m)^Prefix:\s*(.*)$
        // where Prefix is field.prefix or field.name

        let mut all_found = true;
        for field in &signature.outputs {
            let prefix = field.prefix.as_deref().unwrap_or(&field.name);
            // Escape prefix for regex? usually safe if simple.
            // Using logic "find line starting with prefix:"
            // We search for key line.

            let pattern = format!(r"(?m)^{}:\s*(.*)$", regex::escape(prefix));
            // We need `regex` crate. Add to Cargo.toml if missing, or use basic string find.
            // Basic string parsing is safer to avoid dependency loop if regex not in dev-dep?
            // dspy might have regex dep.
            // Let's assume basic parsing first but more robust than single-line split.

            // Search for "Prefix:" in content
            let marker = format!("{}:", prefix);
            if let Some(idx) = content.find(&marker) {
                let rest = &content[idx + marker.len()..];
                // Take until next field marker or end of string?
                // This is hard if we don't know order.
                // Fallback: Take until newline if naive.
                // Or: Take until next known field marker.

                // For this iter, let's just take the line.
                // (Ideally we support multi-line values).

                let val_str = if let Some(newline_idx) = rest.find('\n') {
                    &rest[..newline_idx]
                } else {
                    rest
                };
                record.insert(&field.name, val_str.trim());
            } else {
                all_found = false;
            }
        }

        // Fallback: If strict parsing failed to find *any* field, maybe the whole content is the first output?
        if record.fields.is_empty() && !signature.outputs.is_empty() && !content.trim().is_empty() {
            // Assign whole content to the first output field
            let first_field = &signature.outputs[0];
            record.insert(&first_field.name, content.trim());
            all_found = true;
        }

        Ok(Prediction::new(record))
    }
}
