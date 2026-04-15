use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported AI coding agent platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    ClaudeCode,
    OpenCode,
    Cursor,
    Codex,
    GeminiCli,
    RooCode,
    Windsurf,
    Amp,
    Cline,
    KiloCode,
}

impl AgentKind {
    pub fn all() -> &'static [AgentKind] {
        &[
            Self::ClaudeCode,
            Self::OpenCode,
            Self::Cursor,
            Self::Codex,
            Self::GeminiCli,
            Self::RooCode,
            Self::Windsurf,
            Self::Amp,
            Self::Cline,
            Self::KiloCode,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::OpenCode => "opencode",
            Self::Cursor => "cursor",
            Self::Codex => "codex",
            Self::GeminiCli => "gemini-cli",
            Self::RooCode => "roo-code",
            Self::Windsurf => "windsurf",
            Self::Amp => "amp",
            Self::Cline => "cline",
            Self::KiloCode => "kilo-code",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::OpenCode => "OpenCode",
            Self::Cursor => "Cursor",
            Self::Codex => "OpenAI Codex",
            Self::GeminiCli => "Gemini CLI",
            Self::RooCode => "Roo Code",
            Self::Windsurf => "Windsurf",
            Self::Amp => "Amp",
            Self::Cline => "Cline",
            Self::KiloCode => "Kilo Code",
        }
    }
}

/// Configuration for a specific AI agent platform.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub kind: AgentKind,
    pub global_skills_dir: PathBuf,
    pub project_skills_dir: &'static str,
    pub supports_plugins: bool,
    pub plugin_dir: Option<PathBuf>,
    pub tools_dir: Option<PathBuf>,
}

impl AgentConfig {
    pub fn all() -> Vec<Self> {
        let home = dirs::home_dir().unwrap_or_default();

        vec![
            Self {
                kind: AgentKind::ClaudeCode,
                global_skills_dir: home.join(".claude").join("skills"),
                project_skills_dir: ".claude/skills",
                supports_plugins: true,
                plugin_dir: Some(home.join(".claude").join("plugins")),
                tools_dir: None,
            },
            Self {
                kind: AgentKind::OpenCode,
                global_skills_dir: home.join(".config").join("opencode").join("skills"),
                project_skills_dir: ".opencode/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: Some(home.join(".config").join("opencode").join("tools")),
            },
            Self {
                kind: AgentKind::Cursor,
                global_skills_dir: home.join(".cursor").join("skills"),
                project_skills_dir: ".cursor/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::Codex,
                global_skills_dir: home.join(".agents").join("skills"),
                project_skills_dir: ".agents/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::GeminiCli,
                global_skills_dir: home.join(".gemini").join("skills"),
                project_skills_dir: ".gemini/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::RooCode,
                global_skills_dir: home.join(".roo").join("skills"),
                project_skills_dir: ".roo/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::Windsurf,
                global_skills_dir: home.join(".codeium").join("windsurf").join("skills"),
                project_skills_dir: ".windsurf/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::Amp,
                global_skills_dir: home.join(".agents").join("skills"),
                project_skills_dir: ".agents/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::Cline,
                global_skills_dir: home.join(".cline").join("skills"),
                project_skills_dir: ".cline/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
            Self {
                kind: AgentKind::KiloCode,
                global_skills_dir: home.join(".kilo-code").join("skills"),
                project_skills_dir: ".kilo-code/skills",
                supports_plugins: false,
                plugin_dir: None,
                tools_dir: None,
            },
        ]
    }

    /// Check if this agent's configuration directory exists on the system.
    pub fn is_installed(&self) -> bool {
        let parent = self.global_skills_dir.parent().unwrap_or(&self.global_skills_dir);
        parent.exists()
    }
}
