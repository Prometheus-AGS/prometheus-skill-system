use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    ReadConfigFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse yaml config file {path}: {source}")]
    ParseYaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("invalid environment variable {key}: {message}")]
    InvalidEnv { key: &'static str, message: String },
}

#[derive(Debug, Error)]
pub enum DspyError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, DspyError>;
