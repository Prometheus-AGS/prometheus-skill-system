use std::sync::OnceLock;

use prometheus_exec_embedded::{EmbeddedExecutionAdapter, EmbeddedExecutionApi};

static EXEC_HOST: OnceLock<EmbeddedExecutionAdapter> = OnceLock::new();

/// Installs the process-global execution host from trusted Rust application code.
///
/// Flutter and webview callers cannot supply or replace signing material. The
/// host constructs `EmbeddedExecutionApi` after retrieving its device key from
/// the platform secure-key provider, then installs the API exactly once.
#[flutter_rust_bridge::frb(ignore)]
pub fn install_execution_api(api: EmbeddedExecutionApi) -> Result<(), &'static str> {
    EXEC_HOST
        .set(EmbeddedExecutionAdapter::new(api))
        .map_err(|_| "the Prometheus Exec host is already installed")
}

pub(crate) fn execution_adapter() -> Result<&'static EmbeddedExecutionAdapter, &'static str> {
    EXEC_HOST
        .get()
        .ok_or("the Prometheus Exec host has not been installed by trusted Rust application code")
}
