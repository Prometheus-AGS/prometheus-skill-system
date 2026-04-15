use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

struct Finding {
    file: String,
    line: usize,
    pattern: &'static str,
    severity: Severity,
}

#[derive(Clone, Copy)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    fn label(&self) -> colored::ColoredString {
        match self {
            Self::Critical => "CRITICAL".red().bold(),
            Self::High => "HIGH".red(),
            Self::Medium => "MEDIUM".yellow(),
            Self::Low => "LOW".dimmed(),
        }
    }
}

static PATTERNS: &[(&str, &str, Severity)] = &[
    // Dangerous commands
    (r"rm\s+-rf\s+/", "Dangerous rm -rf with root path", Severity::Critical),
    (r"format\s+[cC]:", "Format drive command", Severity::Critical),
    // Credential leaks
    (r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"][^'"]{8,}"#, "Hardcoded credential", Severity::Critical),
    (r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY", "Private key in source", Severity::Critical),
    // Code injection
    (r"\beval\s*\(", "eval() usage — potential code injection", Severity::High),
    (r"\bexec\s*\(", "exec() usage — potential code injection", Severity::High),
    // Prompt injection
    (r"(?i)ignore\s+(all\s+)?previous\s+instructions", "Prompt injection pattern", Severity::High),
    (r"(?i)you\s+are\s+now\s+in\s+", "Role manipulation pattern", Severity::Medium),
    // Direct deployment (TJ-CICD-001 violation)
    (r"kubectl\s+apply", "Direct kubectl apply — use GitOps", Severity::Medium),
    (r"helm\s+(upgrade|install)", "Direct helm command — use GitOps", Severity::Medium),
];

pub fn run(path: Option<&str>) -> Result<()> {
    println!("{}", "🔒 Security Audit".bold());

    let scan_dir = path.unwrap_or(".");
    let mut findings: Vec<Finding> = Vec::new();

    let compiled: Vec<(Regex, &str, Severity)> = PATTERNS
        .iter()
        .filter_map(|(pat, desc, sev)| {
            Regex::new(pat).ok().map(|r| (r, *desc, *sev))
        })
        .collect();

    for entry in WalkDir::new(scan_dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "node_modules" && name != "target"
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "md" | "sh" | "yaml" | "yml" | "json" | "ts" | "js" | "py" | "rs" | "toml") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                for (regex, desc, severity) in &compiled {
                    if regex.is_match(line) {
                        findings.push(Finding {
                            file: path.display().to_string(),
                            line: line_num + 1,
                            pattern: desc,
                            severity: *severity,
                        });
                    }
                }
            }
        }
    }

    if findings.is_empty() {
        println!("\n  {} No security issues found", "✅".green());
        return Ok(());
    }

    findings.sort_by_key(|f| match f.severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    });

    println!("\n  Found {} finding(s):\n", findings.len());
    for f in &findings {
        println!("  {} {}:{} — {}", f.severity.label(), f.file, f.line, f.pattern);
    }

    let critical = findings.iter().filter(|f| matches!(f.severity, Severity::Critical)).count();
    if critical > 0 {
        println!("\n  {} {} critical finding(s) require immediate attention", "❌".red(), critical);
        std::process::exit(2);
    }

    Ok(())
}
