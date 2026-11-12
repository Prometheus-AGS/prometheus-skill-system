//! Core building blocks for a DSPy-like system.
//!
//! Configuration in `dspy-core` is designed to support:
//! - YAML configuration files (with discovery)
//! - Environment variable overrides
//! - Command-line flag overrides
//!
//! The loader is implemented as a composed pipeline of sources (Strategy-style),
//! making it easy to extend with new sources later (e.g., remote config). [TBD]

pub mod error;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use error::{ConfigError, DspyError, Result};

/// Temporary placeholder type to prove the workspace compiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub name: &'static str,
    pub version: &'static str,
}

pub fn version() -> VersionInfo {
    VersionInfo {
        name: "dspy-core",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Effective application configuration for the `dspy-rs` Rust port.
///
/// This config is environment-aware (development vs production), supports
/// external integrations like an LLM gateway, and includes knobs commonly
/// needed for services and CLIs.
///
/// Notes from reference materials:
/// - We expect LLM gateway integration via a Supabase-based gateway client. [1]
/// - We anticipate external integrations like PromCraft agent builder. [1]
/// - Environment-aware configuration patterns (e.g., RUST_ENV switching behavior)
///   are useful for Rust services. [5]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Environment name (e.g., "development", "production"). [5]
    pub environment: Environment,

    /// Logging configuration (level, JSON vs pretty, etc.). [2]
    pub logging: LoggingConfig,

    /// HTTP server settings (for future agent/service APIs).
    pub server: ServerConfig,

    /// LLM access configuration.
    pub llm: LlmConfig,

    /// Optional integrations.
    pub integrations: IntegrationsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            logging: LoggingConfig::default(),
            server: ServerConfig::default(),
            llm: LlmConfig::default(),
            integrations: IntegrationsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level string like "debug", "info". [2]
    pub level: String,

    /// Whether to emit JSON logs (useful in containers).
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Bind address for HTTP server.
    pub bind: String,

    /// Port for HTTP server (common default 3000 in references). [6]
    pub port: u16,

    /// Whether to enable development-only features (hot reload, etc.). [2]
    pub dev_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0".to_string(),
            port: 3000,
            dev_mode: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Which provider to use.
    pub provider: LlmProvider,

    /// Default model name (provider-specific).
    pub model: String,

    /// Request timeout in milliseconds.
    pub timeout_ms: u64,

    /// Retry attempts for transient failures.
    pub retries: u32,

    /// Optional LLM gateway configuration (Supabase gateway). [1]
    pub gateway: Option<LlmGatewayConfig>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Gateway,
            model: "gpt-4o-mini".to_string(),
            timeout_ms: 60_000,
            retries: 2,
            gateway: Some(LlmGatewayConfig::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    /// Use a gateway service for distributed LLM access. [1]
    Gateway,
    /// Direct provider mode (future).
    Direct,
    /// Local runtime mode (future).
    Local,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Gateway
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmGatewayConfig {
    /// Base URL for the gateway service.
    pub base_url: String,

    /// Optional API key.
    pub api_key: Option<String>,

    /// Optional project/tenant identifier.
    pub project_id: Option<String>,
}

impl Default for LlmGatewayConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            project_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IntegrationsConfig {
    /// Optional PromCraft integration (agent builder toolkit). [1]
    pub promcraft: Option<PromCraftConfig>,
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self { promcraft: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PromCraftConfig {
    /// Path or URL to PromCraft service.
    pub endpoint: String,
}

impl Default for PromCraftConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:7070".to_string(),
        }
    }
}

/// A patch type used for merging layered configuration sources.
/// Each field is optional so that "unset" can be distinguished from an override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfigPatch {
    pub environment: Option<Environment>,
    pub logging: Option<LoggingConfig>,
    pub server: Option<ServerConfig>,
    pub llm: Option<LlmConfig>,
    pub integrations: Option<IntegrationsConfig>,
}

impl AppConfigPatch {
    pub fn merge_over(mut self, other: AppConfigPatch) -> AppConfigPatch {
        if other.environment.is_some() {
            self.environment = other.environment;
        }
        if other.logging.is_some() {
            self.logging = other.logging;
        }
        if other.server.is_some() {
            self.server = other.server;
        }
        if other.llm.is_some() {
            self.llm = other.llm;
        }
        if other.integrations.is_some() {
            self.integrations = other.integrations;
        }
        self
    }

    pub fn into_config(self) -> AppConfig {
        let mut cfg = AppConfig::default();
        if let Some(v) = self.environment {
            cfg.environment = v;
        }
        if let Some(v) = self.logging {
            cfg.logging = v;
        }
        if let Some(v) = self.server {
            cfg.server = v;
        }
        if let Some(v) = self.llm {
            cfg.llm = v;
        }
        if let Some(v) = self.integrations {
            cfg.integrations = v;
        }
        cfg
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigDiagnostics {
    pub yaml_files_used: Vec<PathBuf>,
    pub env_keys_used: Vec<&'static str>,
    pub cli_overrides_used: Vec<&'static str>,
}

pub trait ConfigSource {
    fn load(
        &self,
        diag: &mut ConfigDiagnostics,
    ) -> std::result::Result<AppConfigPatch, ConfigError>;
}

pub struct ConfigLoader<S1, S2, S3> {
    yaml: S1,
    env: S2,
    cli: S3,
}

impl<S1, S2, S3> ConfigLoader<S1, S2, S3>
where
    S1: ConfigSource,
    S2: ConfigSource,
    S3: ConfigSource,
{
    pub fn new(yaml: S1, env: S2, cli: S3) -> Self {
        Self { yaml, env, cli }
    }

    /// Load config in precedence order: YAML -> env -> CLI.
    pub fn load(&self) -> std::result::Result<(AppConfig, ConfigDiagnostics), ConfigError> {
        let mut diag = ConfigDiagnostics::default();

        let yaml_patch = self.yaml.load(&mut diag)?;
        let env_patch = self.env.load(&mut diag)?;
        let cli_patch = self.cli.load(&mut diag)?;

        let effective = AppConfigPatch::default()
            .merge_over(yaml_patch)
            .merge_over(env_patch)
            .merge_over(cli_patch)
            .into_config();

        Ok((effective, diag))
    }
}

/// YAML discovery + merge strategy.
///
/// Discovery order:
/// 1) ./config.yaml
/// 2) $HOME/.dspy/config.yaml
/// 3) CONFIG_FILE env var
/// 4) --config/-c cli flag value
pub struct YamlDiscoverySource {
    pub cwd_config: PathBuf,
    pub home_config: Option<PathBuf>,
    pub config_file_env: Option<PathBuf>,
    pub cli_config: Option<PathBuf>,
}

impl YamlDiscoverySource {
    pub fn new(cli_config: Option<PathBuf>) -> Self {
        let cwd_config = PathBuf::from("config.yaml");
        let home_config = dirs::home_dir().map(|h| h.join(".dspy").join("config.yaml"));
        let config_file_env = std::env::var_os("CONFIG_FILE").map(PathBuf::from);

        Self {
            cwd_config,
            home_config,
            config_file_env,
            cli_config,
        }
    }

    fn candidate_paths(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();

        if self.cwd_config.exists() {
            out.push(self.cwd_config.clone());
        }
        if let Some(p) = &self.home_config {
            if p.exists() {
                out.push(p.clone());
            }
        }
        if let Some(p) = &self.config_file_env {
            out.push(p.clone());
        }
        if let Some(p) = &self.cli_config {
            out.push(p.clone());
        }

        out
    }

    fn load_yaml_file(path: &Path) -> std::result::Result<AppConfigPatch, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadConfigFile {
            path: path.to_path_buf(),
            source: e,
        })?;

        serde_yaml::from_str::<AppConfigPatch>(&text).map_err(|e| ConfigError::ParseYaml {
            path: path.to_path_buf(),
            source: e,
        })
    }
}

impl ConfigSource for YamlDiscoverySource {
    fn load(
        &self,
        diag: &mut ConfigDiagnostics,
    ) -> std::result::Result<AppConfigPatch, ConfigError> {
        let mut merged = AppConfigPatch::default();

        for p in self.candidate_paths() {
            let patch = Self::load_yaml_file(&p)?;
            diag.yaml_files_used.push(p);
            merged = merged.merge_over(patch);
        }

        Ok(merged)
    }
}

/// Environment override strategy.
///
/// Env var naming convention:
/// - DSPY_ENVIRONMENT=development|production
/// - DSPY_LOG_LEVEL=debug|info|...
/// - DSPY_LOG_JSON=true|false
/// - DSPY_BIND=0.0.0.0
/// - DSPY_PORT=3000
/// - DSPY_LLM_PROVIDER=gateway|direct|local
/// - DSPY_LLM_MODEL=...
/// - DSPY_LLM_TIMEOUT_MS=...
/// - DSPY_LLM_RETRIES=...
/// - DSPY_GATEWAY_BASE_URL=...
/// - DSPY_GATEWAY_API_KEY=...
/// - DSPY_GATEWAY_PROJECT_ID=...
/// - DSPY_PROMCRAFT_ENDPOINT=...
pub struct EnvSource;

impl EnvSource {
    const ENVIRONMENT: &'static str = "DSPY_ENVIRONMENT";
    const LOG_LEVEL: &'static str = "DSPY_LOG_LEVEL";
    const LOG_JSON: &'static str = "DSPY_LOG_JSON";

    const BIND: &'static str = "DSPY_BIND";
    const PORT: &'static str = "DSPY_PORT";
    const DEV_MODE: &'static str = "DSPY_DEV_MODE";

    const LLM_PROVIDER: &'static str = "DSPY_LLM_PROVIDER";
    const LLM_MODEL: &'static str = "DSPY_LLM_MODEL";
    const LLM_TIMEOUT_MS: &'static str = "DSPY_LLM_TIMEOUT_MS";
    const LLM_RETRIES: &'static str = "DSPY_LLM_RETRIES";

    const GATEWAY_BASE_URL: &'static str = "DSPY_GATEWAY_BASE_URL";
    const GATEWAY_API_KEY: &'static str = "DSPY_GATEWAY_API_KEY";
    const GATEWAY_PROJECT_ID: &'static str = "DSPY_GATEWAY_PROJECT_ID";

    const PROMCRAFT_ENDPOINT: &'static str = "DSPY_PROMCRAFT_ENDPOINT";

    fn parse_bool(key: &'static str, raw: &str) -> std::result::Result<bool, ConfigError> {
        match raw.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "y" | "on" => Ok(true),
            "0" | "false" | "no" | "n" | "off" => Ok(false),
            other => Err(ConfigError::InvalidEnv {
                key,
                message: format!("expected boolean, got '{other}'"),
            }),
        }
    }

    fn parse_environment(raw: &str) -> std::result::Result<Environment, ConfigError> {
        match raw.to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            other => Err(ConfigError::InvalidEnv {
                key: Self::ENVIRONMENT,
                message: format!("expected development|production, got '{other}'"),
            }),
        }
    }

    fn parse_llm_provider(raw: &str) -> std::result::Result<LlmProvider, ConfigError> {
        match raw.to_ascii_lowercase().as_str() {
            "gateway" => Ok(LlmProvider::Gateway),
            "direct" => Ok(LlmProvider::Direct),
            "local" => Ok(LlmProvider::Local),
            other => Err(ConfigError::InvalidEnv {
                key: Self::LLM_PROVIDER,
                message: format!("expected gateway|direct|local, got '{other}'"),
            }),
        }
    }
}

impl ConfigSource for EnvSource {
    fn load(
        &self,
        diag: &mut ConfigDiagnostics,
    ) -> std::result::Result<AppConfigPatch, ConfigError> {
        let mut patch = AppConfigPatch::default();

        // environment
        if let Ok(v) = std::env::var(Self::ENVIRONMENT) {
            diag.env_keys_used.push(Self::ENVIRONMENT);
            patch.environment = Some(Self::parse_environment(&v)?);
        }

        // logging
        let mut logging = None::<LoggingConfig>;
        if let Ok(v) = std::env::var(Self::LOG_LEVEL) {
            diag.env_keys_used.push(Self::LOG_LEVEL);
            logging.get_or_insert_with(LoggingConfig::default).level = v;
        }
        if let Ok(v) = std::env::var(Self::LOG_JSON) {
            diag.env_keys_used.push(Self::LOG_JSON);
            logging.get_or_insert_with(LoggingConfig::default).json =
                Self::parse_bool(Self::LOG_JSON, &v)?;
        }
        patch.logging = logging;

        // server
        let mut server = None::<ServerConfig>;
        if let Ok(v) = std::env::var(Self::BIND) {
            diag.env_keys_used.push(Self::BIND);
            server.get_or_insert_with(ServerConfig::default).bind = v;
        }
        if let Ok(v) = std::env::var(Self::PORT) {
            diag.env_keys_used.push(Self::PORT);
            let parsed = v.parse::<u16>().map_err(|e| ConfigError::InvalidEnv {
                key: Self::PORT,
                message: e.to_string(),
            })?;
            server.get_or_insert_with(ServerConfig::default).port = parsed;
        }
        if let Ok(v) = std::env::var(Self::DEV_MODE) {
            diag.env_keys_used.push(Self::DEV_MODE);
            server.get_or_insert_with(ServerConfig::default).dev_mode =
                Self::parse_bool(Self::DEV_MODE, &v)?;
        }
        patch.server = server;

        // llm
        let mut llm = None::<LlmConfig>;
        if let Ok(v) = std::env::var(Self::LLM_PROVIDER) {
            diag.env_keys_used.push(Self::LLM_PROVIDER);
            llm.get_or_insert_with(LlmConfig::default).provider = Self::parse_llm_provider(&v)?;
        }
        if let Ok(v) = std::env::var(Self::LLM_MODEL) {
            diag.env_keys_used.push(Self::LLM_MODEL);
            llm.get_or_insert_with(LlmConfig::default).model = v;
        }
        if let Ok(v) = std::env::var(Self::LLM_TIMEOUT_MS) {
            diag.env_keys_used.push(Self::LLM_TIMEOUT_MS);
            let parsed = v.parse::<u64>().map_err(|e| ConfigError::InvalidEnv {
                key: Self::LLM_TIMEOUT_MS,
                message: e.to_string(),
            })?;
            llm.get_or_insert_with(LlmConfig::default).timeout_ms = parsed;
        }
        if let Ok(v) = std::env::var(Self::LLM_RETRIES) {
            diag.env_keys_used.push(Self::LLM_RETRIES);
            let parsed = v.parse::<u32>().map_err(|e| ConfigError::InvalidEnv {
                key: Self::LLM_RETRIES,
                message: e.to_string(),
            })?;
            llm.get_or_insert_with(LlmConfig::default).retries = parsed;
        }

        // gateway
        let mut gateway = None::<LlmGatewayConfig>;
        if let Ok(v) = std::env::var(Self::GATEWAY_BASE_URL) {
            diag.env_keys_used.push(Self::GATEWAY_BASE_URL);
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .base_url = v;
        }
        if let Ok(v) = std::env::var(Self::GATEWAY_API_KEY) {
            diag.env_keys_used.push(Self::GATEWAY_API_KEY);
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .api_key = Some(v);
        }
        if let Ok(v) = std::env::var(Self::GATEWAY_PROJECT_ID) {
            diag.env_keys_used.push(Self::GATEWAY_PROJECT_ID);
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .project_id = Some(v);
        }
        if gateway.is_some() {
            llm.get_or_insert_with(LlmConfig::default).gateway = gateway;
        }

        patch.llm = llm;

        // integrations
        let mut integrations = None::<IntegrationsConfig>;
        if let Ok(v) = std::env::var(Self::PROMCRAFT_ENDPOINT) {
            diag.env_keys_used.push(Self::PROMCRAFT_ENDPOINT);
            integrations
                .get_or_insert_with(IntegrationsConfig::default)
                .promcraft = Some(PromCraftConfig { endpoint: v });
        }
        patch.integrations = integrations;

        Ok(patch)
    }
}

/// CLI override strategy takes already-parsed values (the CLI crate parses flags).
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub environment: Option<Environment>,
    pub log_level: Option<String>,
    pub log_json: Option<bool>,

    pub bind: Option<String>,
    pub port: Option<u16>,
    pub dev_mode: Option<bool>,

    pub llm_provider: Option<LlmProvider>,
    pub llm_model: Option<String>,
    pub llm_timeout_ms: Option<u64>,
    pub llm_retries: Option<u32>,

    pub gateway_base_url: Option<String>,
    pub gateway_api_key: Option<String>,
    pub gateway_project_id: Option<String>,

    pub promcraft_endpoint: Option<String>,
}

pub struct CliSource {
    pub overrides: CliOverrides,
}

impl ConfigSource for CliSource {
    fn load(
        &self,
        diag: &mut ConfigDiagnostics,
    ) -> std::result::Result<AppConfigPatch, ConfigError> {
        let mut patch = AppConfigPatch::default();

        if self.overrides.environment.is_some() {
            diag.cli_overrides_used.push("environment");
            patch.environment = self.overrides.environment.clone();
        }

        let mut logging = None::<LoggingConfig>;
        if let Some(v) = self.overrides.log_level.clone() {
            diag.cli_overrides_used.push("logging.level");
            logging.get_or_insert_with(LoggingConfig::default).level = v;
        }
        if let Some(v) = self.overrides.log_json {
            diag.cli_overrides_used.push("logging.json");
            logging.get_or_insert_with(LoggingConfig::default).json = v;
        }
        patch.logging = logging;

        let mut server = None::<ServerConfig>;
        if let Some(v) = self.overrides.bind.clone() {
            diag.cli_overrides_used.push("server.bind");
            server.get_or_insert_with(ServerConfig::default).bind = v;
        }
        if let Some(v) = self.overrides.port {
            diag.cli_overrides_used.push("server.port");
            server.get_or_insert_with(ServerConfig::default).port = v;
        }
        if let Some(v) = self.overrides.dev_mode {
            diag.cli_overrides_used.push("server.dev_mode");
            server.get_or_insert_with(ServerConfig::default).dev_mode = v;
        }
        patch.server = server;

        let mut llm = None::<LlmConfig>;
        if let Some(v) = self.overrides.llm_provider.clone() {
            diag.cli_overrides_used.push("llm.provider");
            llm.get_or_insert_with(LlmConfig::default).provider = v;
        }
        if let Some(v) = self.overrides.llm_model.clone() {
            diag.cli_overrides_used.push("llm.model");
            llm.get_or_insert_with(LlmConfig::default).model = v;
        }
        if let Some(v) = self.overrides.llm_timeout_ms {
            diag.cli_overrides_used.push("llm.timeout_ms");
            llm.get_or_insert_with(LlmConfig::default).timeout_ms = v;
        }
        if let Some(v) = self.overrides.llm_retries {
            diag.cli_overrides_used.push("llm.retries");
            llm.get_or_insert_with(LlmConfig::default).retries = v;
        }

        let mut gateway = None::<LlmGatewayConfig>;
        if let Some(v) = self.overrides.gateway_base_url.clone() {
            diag.cli_overrides_used.push("llm.gateway.base_url");
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .base_url = v;
        }
        if let Some(v) = self.overrides.gateway_api_key.clone() {
            diag.cli_overrides_used.push("llm.gateway.api_key");
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .api_key = Some(v);
        }
        if let Some(v) = self.overrides.gateway_project_id.clone() {
            diag.cli_overrides_used.push("llm.gateway.project_id");
            gateway
                .get_or_insert_with(LlmGatewayConfig::default)
                .project_id = Some(v);
        }
        if gateway.is_some() {
            llm.get_or_insert_with(LlmConfig::default).gateway = gateway;
        }

        patch.llm = llm;

        let mut integrations = None::<IntegrationsConfig>;
        if let Some(v) = self.overrides.promcraft_endpoint.clone() {
            diag.cli_overrides_used
                .push("integrations.promcraft.endpoint");
            integrations
                .get_or_insert_with(IntegrationsConfig::default)
                .promcraft = Some(PromCraftConfig { endpoint: v });
        }
        patch.integrations = integrations;

        Ok(patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_yaml(path: &Path, yaml: &str) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, yaml).unwrap();
    }

    #[test]
    fn env_overrides_yaml_and_cli_overrides_env() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        write_yaml(
            &dir.path().join("config.yaml"),
            r#"
server:
  port: 3001
logging:
  level: info
"#,
        );

        // SAFETY: set_var is unsafe in multi-threaded environments (like tests) because it modifies global state.
        // We wrap it in unsafe to satisfy the compiler/safety checks.
        unsafe {
            std::env::set_var("DSPY_PORT", "3002");
        }

        let cli = CliSource {
            overrides: CliOverrides {
                port: Some(3003),
                ..CliOverrides::default()
            },
        };

        let (cfg, _diag) = ConfigLoader::new(YamlDiscoverySource::new(None), EnvSource, cli)
            .load()
            .unwrap();

        assert_eq!(cfg.server.port, 3003);
    }
}
