use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "knowme-builder", version, about)]
pub struct Cli {
    /// Emit a machine-readable JSON result.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a validated application in a new destination.
    New(NewArgs),
    /// Adopt an evolved application without re-scaffolding it.
    Adopt(AdoptArgs),
    /// Upgrade Builder-owned files.
    Upgrade(UpgradeArgs),
    /// Add a typed application capability.
    Add(AddArgs),
    /// Install or verify the Builder skill bundle.
    Skills(SkillsArgs),
    /// Audit an application against a Builder profile.
    Audit(AuditArgs),
    /// Check Builder, UAR, Prometheus, and harness compatibility.
    Doctor(DoctorArgs),
    /// Generate or check package manifests.
    Manifest(ManifestArgs),
    /// Generate shell completion output.
    Completions(CompletionsArgs),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    SovereignHybrid,
    GovernedWebShell,
    FlutterMobile,
    TauriDesktop,
    AxumWeb,
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SovereignHybrid => "sovereign-hybrid",
            Self::GovernedWebShell => "governed-web-shell",
            Self::FlutterMobile => "flutter-mobile",
            Self::TauriDesktop => "tauri-desktop",
            Self::AxumWeb => "axum-web",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum GenerationMode {
    Runnable,
    Skeleton,
}

impl GenerationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::Skeleton => "skeleton",
        }
    }
}

#[derive(Debug, Args)]
pub struct NewArgs {
    pub path: PathBuf,
    #[arg(long, value_enum)]
    pub profile: Profile,
    #[arg(long, value_enum)]
    pub mode: GenerationMode,
    /// Preview the generated paths without changing the destination.
    #[arg(long)]
    pub check: bool,
    /// Adopt a non-empty destination instead of generating into it.
    #[arg(long, conflicts_with = "force")]
    pub adopt: bool,
    /// Replace a destination only after an explicit request.
    #[arg(long, conflicts_with = "adopt")]
    pub force: bool,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("operation")
        .required(true)
        .args(["check", "apply"])
))]
pub struct AdoptArgs {
    pub path: PathBuf,
    #[arg(long, value_enum)]
    pub profile: Profile,
    #[arg(long)]
    pub check: bool,
    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("operation")
        .required(true)
        .args(["check", "apply", "rollback"])
))]
pub struct UpgradeArgs {
    pub path: PathBuf,
    /// Original rendering name for legacy state that did not persist identity.
    #[arg(long)]
    pub app_name: Option<String>,
    #[arg(long)]
    pub check: bool,
    #[arg(long)]
    pub apply: bool,
    /// Restore the last managed-file upgrade, preserving files edited afterward.
    #[arg(long, conflicts_with_all = ["check", "apply", "app_name"])]
    pub rollback: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AddKind {
    Feature,
    Auth,
    Module,
    LegacyEmbed,
}

impl AddKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Auth => "auth",
            Self::Module => "module",
            Self::LegacyEmbed => "legacy-embed",
        }
    }
}

#[derive(Debug, Args)]
pub struct AddArgs {
    #[arg(value_enum)]
    pub kind: AddKind,
    pub name: Option<String>,
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Args)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Debug, Subcommand)]
pub enum SkillsCommand {
    Install {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        check: bool,
        /// Replace drifted Builder skill payloads after preserving a backup.
        #[arg(long, conflicts_with = "check")]
        force: bool,
    },
    Check {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct AuditArgs {
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long, value_enum)]
    pub profile: Option<Profile>,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    /// Inspect native tooling without requiring the Prometheus control plane.
    #[arg(long)]
    pub native_only: bool,
    /// Windows MSVC target to inspect (repeat for both architectures).
    #[arg(long, value_parser = ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"])]
    pub target: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ManifestArgs {
    #[command(subcommand)]
    pub command: ManifestCommand,
}

#[derive(Debug, Subcommand)]
pub enum ManifestCommand {
    Generate,
    Check,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: CompletionShell,
}
