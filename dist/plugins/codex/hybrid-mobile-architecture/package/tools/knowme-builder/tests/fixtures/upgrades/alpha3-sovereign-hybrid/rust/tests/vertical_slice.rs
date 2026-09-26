use output_core::{
    DeterministicUarRuntime, RunRequest, UarEvent, UarRuntimeFacade,
};

#[test]
fn message_to_uar_to_tool_to_a2ui_to_recovery() {
    let mut runtime = DeterministicUarRuntime::default();
    let request = RunRequest {
        run_id: "run-1".to_owned(),
        message: "hello".to_owned(),
        idempotency_id: "command-1".to_owned(),
        max_turns: 4,
        max_retries: 1,
        max_duration_ms: 5_000,
        max_output_bytes: 64_000,
    };
    let events = runtime.run(request).expect("run");
    assert!(events.iter().any(|event| matches!(event, UarEvent::ToolApprovalRequired { .. })));
    assert!(events.iter().any(|event| matches!(event, UarEvent::A2uiSurface { .. })));
    let recovered = runtime.recover("run-1").expect("recover").expect("projection");
    assert!(recovered.completed);
    assert_eq!(recovered.events, events);
}

#[test]
fn rejects_zero_budgets_and_handles_idempotent_retries() {
    let mut runtime = DeterministicUarRuntime::default();
    let invalid = RunRequest {
        run_id: "invalid".to_owned(),
        message: "hello".to_owned(),
        idempotency_id: "invalid-command".to_owned(),
        max_turns: 0,
        max_retries: 0,
        max_duration_ms: 0,
        max_output_bytes: 0,
    };
    assert!(runtime.run(invalid).is_err());

    let valid = RunRequest {
        run_id: "idempotent-run".to_owned(),
        message: "hello".to_owned(),
        idempotency_id: "command-2".to_owned(),
        max_turns: 2,
        max_retries: 1,
        max_duration_ms: 2_000,
        max_output_bytes: 4_096,
    };
    let first = runtime.run(valid.clone()).expect("first run");
    let retry = runtime.run(valid).expect("idempotent retry");
    assert_eq!(first, retry);
}
