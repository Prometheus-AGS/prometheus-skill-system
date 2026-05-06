use prometheus_rust_auditor::reporter::{Finding, OutputFormat, Report, Severity};

#[test]
fn exit_code_zero_when_no_findings() {
    let report = Report {
        phase: "test".into(),
        findings: vec![],
    };
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn exit_code_one_when_findings_present() {
    let report = Report {
        phase: "test".into(),
        findings: vec![Finding {
            severity: Severity::High,
            phase: "test".into(),
            crate_name: "my-crate".into(),
            message: "something is wrong".into(),
            file: None,
            line: None,
        }],
    };
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn json_output_contains_severity_lowercase() {
    let report = Report {
        phase: "test".into(),
        findings: vec![Finding {
            severity: Severity::Critical,
            phase: "test".into(),
            crate_name: "my-crate".into(),
            message: "critical issue found".into(),
            file: None,
            line: None,
        }],
    };
    let json = report.to_json().unwrap();
    assert!(json.contains("\"critical\""), "Expected lowercase 'critical' in JSON output, got: {json}");
}

#[test]
fn output_format_from_str() {
    assert!("text".parse::<OutputFormat>().is_ok());
    assert!("json".parse::<OutputFormat>().is_ok());
    assert!("sarif".parse::<OutputFormat>().is_ok());
    assert!("invalid".parse::<OutputFormat>().is_err());
}
