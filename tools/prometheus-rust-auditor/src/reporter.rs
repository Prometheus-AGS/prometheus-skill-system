use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub phase: String,
    pub crate_name: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub phase: String,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn exit_code(&self) -> i32 {
        if self.findings.is_empty() { 0 } else { 1 }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    pub fn print(&self, fmt: &OutputFormat) {
        match fmt {
            OutputFormat::Json => {
                if let Ok(s) = self.to_json() {
                    println!("{s}");
                }
            }
            OutputFormat::Text => {
                if self.findings.is_empty() {
                    println!("✓ Phase [{}]: no findings", self.phase);
                } else {
                    println!("✗ Phase [{}]: {} finding(s)", self.phase, self.findings.len());
                    for f in &self.findings {
                        let loc = match (&f.file, &f.line) {
                            (Some(file), Some(line)) => format!(" @ {file}:{line}"),
                            (Some(file), None) => format!(" @ {file}"),
                            _ => String::new(),
                        };
                        println!("  [{:?}] {}{}", f.severity, f.message, loc);
                    }
                }
            }
            OutputFormat::Sarif => {
                // SARIF 2.1 stub — full implementation in Phase 6-9
                println!("{{\"version\":\"2.1.0\",\"runs\":[]}}");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            other => anyhow::bail!("unknown output format: {other}; use text|json|sarif"),
        }
    }
}
