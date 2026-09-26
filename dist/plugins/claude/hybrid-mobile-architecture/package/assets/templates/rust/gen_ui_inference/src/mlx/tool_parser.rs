// TJ-ARCH-MOB-001 compliant
//! Streaming segmenter that splits a model's raw text stream into canonical
//! `StreamEvent`s: assistant content (`TextDelta`), extended reasoning
//! (`ThinkingDelta`), and tool calls (`ToolCallStarted/Delta/Complete`). The
//! host tool gate and the UAR driver bridge then consume the same event shape
//! they get from every other engine, and the shared `A2uiAdapter` turns
//! `ThinkingDelta` into a collapsible `thinking` ContentBlock.
//!
//! The parser is fed token deltas as they stream. Open/close tags can split
//! across deltas, so a bounded holdback buffer keeps just enough tail to detect
//! a tag that has only partially arrived; everything definitely-not-a-tag is
//! emitted immediately. Reasoning markup is per-model (`ReasoningFormat`): Qwen3
//! uses `<think>…</think>`; a model with `ReasoningFormat::None` skips the
//! reasoning tags entirely and only tool calls are extracted.

use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::ReasoningFormat;

const TOOL_OPEN: &str = "<tool_call>";
const TOOL_CLOSE: &str = "</tool_call>";
const THINK_OPEN: &str = "<think>";
const THINK_CLOSE: &str = "</think>";

#[derive(Debug, PartialEq)]
enum State {
    /// Passing assistant content through, watching for an open tag.
    Text,
    /// Inside a `<think>` block, emitting its body as `ThinkingDelta` until
    /// `</think>`.
    InThinking,
    /// Inside a tool call, accumulating its JSON body until `</tool_call>`.
    InToolCall,
}

/// The longest close tag governs how much tail we must hold back while a partial
/// tag might still complete. `</tool_call>` is the longest of the four.
const MAX_HOLDBACK: usize = TOOL_CLOSE.len() - 1;

/// Streaming segmenter. Retains the historical `ToolCallParser` name so existing
/// call sites and tests keep working; it now also splits reasoning when the
/// model's `ReasoningFormat` declares a thinking markup.
pub(crate) struct ToolCallParser {
    state: State,
    /// Unmatched tail: a possible partial open tag (Text), the accumulating tool
    /// JSON / partial close (InToolCall), or streamed reasoning (InThinking).
    buffer: String,
    next_call_index: usize,
    /// Whether `<think>` blocks are recognized. `false` for
    /// `ReasoningFormat::None`, so a literal `<think>` in content is passed
    /// through untouched.
    think_enabled: bool,
}

impl ToolCallParser {
    /// Tool-call-only parser (no reasoning split). Retained for existing tests
    /// and callers that predate per-model reasoning formats.
    pub(crate) fn new() -> Self {
        Self::with_reasoning(ReasoningFormat::None)
    }

    /// Parser configured for a model's reasoning markup. `Qwen3Think` (and, for
    /// now, `Harmony`, which shares the `<think>`-style recognition until a
    /// harmony model is wired) enable reasoning extraction; `None` disables it.
    pub(crate) fn with_reasoning(format: ReasoningFormat) -> Self {
        let think_enabled = matches!(format, ReasoningFormat::Qwen3Think);
        Self {
            state: State::Text,
            buffer: String::new(),
            next_call_index: 0,
            think_enabled,
        }
    }

    /// Feed a delta; returns the stream events it completes.
    pub(crate) fn push(&mut self, delta: &str) -> Vec<StreamEvent> {
        self.buffer.push_str(delta);
        let mut out = Vec::new();
        loop {
            match self.state {
                State::Text => {
                    // Find the earliest of the open tags we recognize.
                    let tool_at = self.buffer.find(TOOL_OPEN);
                    let think_at = if self.think_enabled {
                        self.buffer.find(THINK_OPEN)
                    } else {
                        None
                    };
                    let next = earliest(tool_at, think_at);
                    if let Some((pos, tag)) = next {
                        let text = self.buffer[..pos].to_string();
                        if !text.is_empty() {
                            out.push(StreamEvent::TextDelta {
                                index: 0,
                                delta: text,
                            });
                        }
                        self.buffer.drain(..pos + tag.len());
                        self.state = if tag == THINK_OPEN {
                            State::InThinking
                        } else {
                            State::InToolCall
                        };
                        continue;
                    }
                    // No complete open tag. Emit everything except a tail that
                    // could still become one of the open tags; keep that tail.
                    let safe = self.safe_text_emit_len();
                    if safe > 0 {
                        let text = self.buffer[..safe].to_string();
                        out.push(StreamEvent::TextDelta {
                            index: 0,
                            delta: text,
                        });
                        self.buffer.drain(..safe);
                    }
                    break;
                }
                State::InThinking => {
                    if let Some(pos) = self.buffer.find(THINK_CLOSE) {
                        let body = self.buffer[..pos].to_string();
                        if !body.is_empty() {
                            out.push(StreamEvent::ThinkingDelta {
                                index: 0,
                                delta: body,
                            });
                        }
                        self.buffer.drain(..pos + THINK_CLOSE.len());
                        self.state = State::Text;
                        continue;
                    }
                    // No close yet: stream the safe prefix as reasoning, holding
                    // back only a possible partial `</think>`.
                    let safe = safe_emit_len(&self.buffer, THINK_CLOSE);
                    if safe > 0 {
                        let body = self.buffer[..safe].to_string();
                        out.push(StreamEvent::ThinkingDelta {
                            index: 0,
                            delta: body,
                        });
                        self.buffer.drain(..safe);
                    }
                    break;
                }
                State::InToolCall => {
                    if let Some(pos) = self.buffer.find(TOOL_CLOSE) {
                        let body = self.buffer[..pos].to_string();
                        self.buffer.drain(..pos + TOOL_CLOSE.len());
                        self.state = State::Text;
                        out.extend(self.emit_tool_call(&body));
                        continue;
                    }
                    // Wait for more — the whole body stays buffered until the
                    // close tag arrives.
                    break;
                }
            }
        }
        out
    }

    /// Flush at end of stream: emit any buffered content, reasoning, or a
    /// best-effort tool call if a block was left open, so nothing is dropped.
    pub(crate) fn finish(&mut self) -> Vec<StreamEvent> {
        let mut out = Vec::new();
        match self.state {
            State::Text => {
                if !self.buffer.is_empty() {
                    out.push(StreamEvent::TextDelta {
                        index: 0,
                        delta: std::mem::take(&mut self.buffer),
                    });
                }
            }
            State::InThinking => {
                // Stream ended mid-`<think>`. Emit the remainder as reasoning
                // rather than losing it; the close tag simply never arrived.
                if !self.buffer.is_empty() {
                    out.push(StreamEvent::ThinkingDelta {
                        index: 0,
                        delta: std::mem::take(&mut self.buffer),
                    });
                }
                self.state = State::Text;
            }
            State::InToolCall => {
                // Stream ended mid-tool-call. Emit what we have as a best-effort
                // call rather than losing it; malformed JSON is handled by
                // emit_tool_call's fallback.
                let body = std::mem::take(&mut self.buffer);
                out.extend(self.emit_tool_call(&body));
                self.state = State::Text;
            }
        }
        out
    }

    /// Bytes of `buffer` emittable now as text without risking that a suffix is
    /// the start of any recognized open tag. Holds back the longest suffix that
    /// is a proper prefix of `<tool_call>` or (when enabled) `<think>`.
    fn safe_text_emit_len(&self) -> usize {
        let tool = safe_emit_len(&self.buffer, TOOL_OPEN);
        if self.think_enabled {
            tool.min(safe_emit_len(&self.buffer, THINK_OPEN))
        } else {
            tool
        }
    }

    /// Turn one `<tool_call>` body (expected `{"name":…,"arguments":…}`) into
    /// the canonical `ToolCallStarted`/`Delta`/`Complete` trio. Malformed JSON
    /// is surfaced as a `Custom` diagnostic plus the raw body as text, never a
    /// dropped turn.
    fn emit_tool_call(&mut self, body: &str) -> Vec<StreamEvent> {
        let index = self.next_call_index;
        self.next_call_index += 1;
        let id = format!("mlx-tool-{index}");

        let parsed = serde_json::from_str::<serde_json::Value>(body.trim());
        let (name, arguments) = match parsed.as_ref() {
            Ok(value) => {
                let name = value
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                let arguments = value
                    .get("arguments")
                    .map(|arguments| match arguments {
                        serde_json::Value::String(raw) => raw.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_else(|| "{}".to_owned());
                (name, arguments)
            }
            Err(_) => (None, String::new()),
        };

        match name {
            Some(name) => vec![
                StreamEvent::ToolCallStarted {
                    id: id.clone(),
                    name,
                },
                StreamEvent::ToolCallDelta {
                    id: id.clone(),
                    delta: arguments,
                },
                StreamEvent::ToolCallComplete { id },
            ],
            None => vec![
                StreamEvent::Custom {
                    source: "mlx".into(),
                    event_name: "malformed_tool_call".into(),
                    payload: serde_json::json!({
                        "message": "model emitted a <tool_call> block that did not parse as {name, arguments}",
                        "raw": body,
                    }),
                },
                StreamEvent::TextDelta {
                    index: 0,
                    delta: body.to_string(),
                },
            ],
        }
    }
}

/// Pick the earlier of two optional tag matches, returning its position and the
/// tag literal. `None` positions lose to `Some`; equal positions can't happen
/// (the tags differ), but tool wins the tie defensively.
fn earliest(
    tool_at: Option<usize>,
    think_at: Option<usize>,
) -> Option<(usize, &'static str)> {
    match (tool_at, think_at) {
        (Some(t), Some(k)) if k < t => Some((k, THINK_OPEN)),
        (Some(t), _) => Some((t, TOOL_OPEN)),
        (None, Some(k)) => Some((k, THINK_OPEN)),
        (None, None) => None,
    }
}

/// How many leading bytes of `buffer` can be emitted now without risking that
/// a suffix is the start of `tag`. Keeps back the longest suffix of `buffer`
/// that is a proper prefix of `tag` (bounded by `MAX_HOLDBACK`). All tag bytes
/// are ASCII, so a suffix can only match on a char boundary; non-boundary
/// offsets are skipped, which also avoids splitting a multi-byte char mid-way.
fn safe_emit_len(buffer: &str, tag: &str) -> usize {
    let max = buffer.len().min(MAX_HOLDBACK);
    for keep in (1..=max).rev() {
        let start = buffer.len() - keep;
        if !buffer.is_char_boundary(start) {
            continue;
        }
        if tag.starts_with(&buffer[start..]) {
            return start;
        }
    }
    buffer.len()
}
