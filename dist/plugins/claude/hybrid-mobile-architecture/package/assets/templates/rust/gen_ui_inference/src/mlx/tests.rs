use super::*;
use gen_ui_types::inference::{InferenceRequest, SampleParams};
use std::sync::Mutex;

trait LockExt<T> {
    fn lock_ok(&self) -> std::sync::MutexGuard<'_, T>;
}
impl<T> LockExt<T> for Mutex<T> {
    fn lock_ok(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A scripted bridge: `load` succeeds, `generate` replays a fixed set of
/// bridge events, and calls are recorded for assertions.
struct MockMlxBridge {
    events: Mutex<Vec<MlxBridgeEvent>>,
    load_calls: Mutex<Vec<(PathBuf, String)>>,
    generate_calls: Mutex<Vec<String>>,
    available: bool,
    fail_load: bool,
}

impl MockMlxBridge {
    fn with_events(events: Vec<MlxBridgeEvent>) -> Self {
        Self {
            events: Mutex::new(events),
            load_calls: Mutex::new(Vec::new()),
            generate_calls: Mutex::new(Vec::new()),
            available: true,
            fail_load: false,
        }
    }
}

impl MlxBridge for MockMlxBridge {
    fn load(&self, model_dir: &Path, config_json: &str) -> CoreResult<()> {
        self.load_calls
            .lock_ok()
            .push((model_dir.to_owned(), config_json.to_owned()));
        if self.fail_load {
            Err(CoreError::Terminal("scripted load failure".into()))
        } else {
            Ok(())
        }
    }

    fn generate(
        &self,
        request_json: &str,
        on_event: &mut dyn FnMut(MlxBridgeEvent),
    ) -> CoreResult<()> {
        self.generate_calls.lock_ok().push(request_json.to_owned());
        for event in self.events.lock_ok().drain(..) {
            on_event(event);
        }
        Ok(())
    }

    fn cancel(&self) {}
    fn close(&self) {}
    fn available(&self) -> bool {
        self.available
    }
}

fn text_request(user: &str) -> InferenceRequest {
    InferenceRequest {
        prompt: String::new(),
        messages: vec![gen_ui_types::inference::InferenceMessage {
            role: InferenceRole::User,
            content: user.into(),
            name: None,
            tool_call_id: None,
        }],
        tools: vec![],
        images: vec![],
        generation_settings: SampleParams::default(),
        grammar: None,
        originating_plan: None,
    }
}

/// The engine's generation path uses `gen_ui_runtime::spawn_blocking`, which
/// needs the process runtime initialized. Idempotent across tests.
fn init_runtime() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| gen_ui_runtime::init(Some(2)));
}

async fn collect(stream: futures::stream::BoxStream<'static, StreamEvent>) -> Vec<StreamEvent> {
    stream.collect::<Vec<_>>().await
}

#[tokio::test]
async fn generation_wraps_deltas_with_message_start_and_done() {
    init_runtime();
    let bridge = Arc::new(MockMlxBridge::with_events(vec![
        MlxBridgeEvent::Delta {
            text: "Hello ".into(),
        },
        MlxBridgeEvent::Delta {
            text: "world".into(),
        },
        MlxBridgeEvent::Final,
    ]));
    let engine = MlxEngine::with_bridge("/tmp/gen-ui-mlx-test", bridge.clone());
    let events = collect(engine.generate_request(&text_request("hi")).await.unwrap()).await;

    assert!(matches!(events.first(), Some(StreamEvent::MessageStart)));
    assert!(matches!(events.last(), Some(StreamEvent::Done)));
    let text: String = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "Hello world");
    // The request the bridge saw carried the user message.
    assert!(bridge.generate_calls.lock_ok()[0].contains("\"hi\""));
}

#[tokio::test]
async fn tool_call_in_stream_becomes_canonical_events() {
    init_runtime();
    let bridge = Arc::new(MockMlxBridge::with_events(vec![
        MlxBridgeEvent::Delta {
            text: "Let me check. ".into(),
        },
        MlxBridgeEvent::Delta {
            text: "<tool_call>{\"name\":\"get_weather\",\"arguments\":{\"city\":\"Paris\"}}</tool_call>".into(),
        },
        MlxBridgeEvent::Final,
    ]));
    let engine = MlxEngine::with_bridge("/tmp/gen-ui-mlx-test", bridge);
    let events = collect(
        engine
            .generate_request(&text_request("weather?"))
            .await
            .unwrap(),
    )
    .await;

    let started = events.iter().find_map(|event| match event {
        StreamEvent::ToolCallStarted { name, .. } => Some(name.clone()),
        _ => None,
    });
    assert_eq!(started.as_deref(), Some("get_weather"));
    assert!(events.iter().any(|event| matches!(
        event,
        StreamEvent::ToolCallDelta { delta, .. } if delta.contains("Paris")
    )));
    assert!(events
        .iter()
        .any(|event| matches!(event, StreamEvent::ToolCallComplete { .. })));
    // Text before the tag survives; the tag itself does not leak as text.
    let text: String = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "Let me check. ");
    assert!(!text.contains("tool_call"));
}

#[tokio::test]
async fn unavailable_bridge_ends_cleanly_with_error() {
    init_runtime();
    let mut bridge = MockMlxBridge::with_events(vec![]);
    bridge.available = false;
    let engine = MlxEngine::with_bridge("/tmp/gen-ui-mlx-test", Arc::new(bridge));
    let events = collect(engine.generate_request(&text_request("hi")).await.unwrap()).await;
    assert!(events
        .iter()
        .any(|event| matches!(event, StreamEvent::Error { .. })));
    assert!(matches!(events.last(), Some(StreamEvent::Done)));
}

// -------- tool-call parser --------

fn parse_all(deltas: &[&str]) -> Vec<StreamEvent> {
    let mut parser = ToolCallParser::new();
    let mut out = Vec::new();
    for delta in deltas {
        out.extend(parser.push(delta));
    }
    out.extend(parser.finish());
    out
}

/// Parse with Qwen3 `<think>` reasoning splitting enabled.
fn parse_all_reasoning(deltas: &[&str]) -> Vec<StreamEvent> {
    use gen_ui_types::inference::ReasoningFormat;
    let mut parser = ToolCallParser::with_reasoning(ReasoningFormat::Qwen3Think);
    let mut out = Vec::new();
    for delta in deltas {
        out.extend(parser.push(delta));
    }
    out.extend(parser.finish());
    out
}

fn thinking_of(events: &[StreamEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::ThinkingDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect()
}

fn text_of(events: &[StreamEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn parser_passes_plain_text_through() {
    let events = parse_all(&["hello ", "there"]);
    assert_eq!(text_of(&events), "hello there");
    assert!(!events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

#[test]
fn parser_handles_open_tag_split_across_deltas() {
    // "<tool" then "_call>{...}" then "</tool_call>" split awkwardly.
    let events = parse_all(&[
        "before <too",
        "l_call>{\"name\":\"f\",\"arguments\":{}}</tool",
        "_call> after",
    ]);
    assert_eq!(text_of(&events), "before  after");
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, StreamEvent::ToolCallStarted { .. }))
            .count(),
        1
    );
    // The split "<too"/"l_call>" must not leak as text.
    assert!(!text_of(&events).contains("tool"));
}

#[test]
fn parser_close_tag_split_across_deltas() {
    let events = parse_all(&[
        "<tool_call>{\"name\":\"g\",\"arguments\":{\"x\":1}}<",
        "/tool_call>done",
    ]);
    assert_eq!(text_of(&events), "done");
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallDelta { delta, .. } if delta.contains("\"x\":1")
    )));
}

#[test]
fn parser_malformed_tool_call_falls_back_to_diagnostic_and_text() {
    let events = parse_all(&["<tool_call>not json at all</tool_call>"]);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::Custom { event_name, .. } if event_name == "malformed_tool_call"
    )));
    assert!(text_of(&events).contains("not json"));
    assert!(!events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

#[test]
fn parser_unterminated_tool_call_flushes_on_finish() {
    // Stream ends mid-tool-call; the body must still surface, not vanish.
    let events = parse_all(&["<tool_call>{\"name\":\"h\",\"arguments\":{}}"]);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallStarted { name, .. } if name == "h"
    )));
}

#[test]
fn parser_text_after_tool_call_is_preserved() {
    let events =
        parse_all(&["<tool_call>{\"name\":\"a\",\"arguments\":{}}</tool_call>trailing text"]);
    assert_eq!(text_of(&events), "trailing text");
    assert!(events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

// -------- reasoning (`<think>`) splitting --------

#[test]
fn think_block_becomes_thinking_deltas_not_text() {
    let events =
        parse_all_reasoning(&["<think>the user wants a greeting</think>Hello there!"]);
    assert_eq!(thinking_of(&events), "the user wants a greeting");
    assert_eq!(text_of(&events), "Hello there!");
    // Reasoning must never leak into content.
    assert!(!text_of(&events).contains("the user wants"));
}

#[test]
fn think_disabled_passes_tags_through_as_text() {
    // ReasoningFormat::None: a literal <think> is ordinary content.
    let events = parse_all(&["<think>kept</think> body"]);
    assert!(thinking_of(&events).is_empty());
    assert_eq!(text_of(&events), "<think>kept</think> body");
}

#[test]
fn think_open_and_close_split_across_deltas() {
    let events = parse_all_reasoning(&[
        "prefix <thi",
        "nk>reason",
        "ing text</thi",
        "nk>answer",
    ]);
    assert_eq!(thinking_of(&events), "reasoning text");
    assert_eq!(text_of(&events), "prefix answer");
    // The split "<thi"/"nk>" must not leak as text.
    assert!(!text_of(&events).contains("think"));
}

#[test]
fn think_then_tool_call_interleave() {
    let events = parse_all_reasoning(&[
        "<think>need the weather tool</think>",
        "<tool_call>{\"name\":\"weather\",\"arguments\":{\"city\":\"SF\"}}</tool_call>",
        "Here you go.",
    ]);
    assert_eq!(thinking_of(&events), "need the weather tool");
    assert_eq!(text_of(&events), "Here you go.");
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallStarted { name, .. } if name == "weather"
    )));
}

#[test]
fn think_no_markup_is_plain_passthrough() {
    // Reasoning enabled, but the model emits no <think> — content is untouched.
    let events = parse_all_reasoning(&["just a normal answer with no reasoning"]);
    assert!(thinking_of(&events).is_empty());
    assert_eq!(text_of(&events), "just a normal answer with no reasoning");
}

#[test]
fn think_unterminated_flushes_as_thinking_on_finish() {
    // Stream ends mid-<think>; the remainder must surface as reasoning, not vanish.
    let events = parse_all_reasoning(&["<think>partial reasoning that never closes"]);
    assert_eq!(thinking_of(&events), "partial reasoning that never closes");
    assert!(text_of(&events).is_empty());
}
