use crate::config::AgentConfig;

/// Detect which AI coding agents are installed on this system.
///
/// Checks for the existence of each agent's configuration directory.
pub fn detect_installed_agents() -> Vec<AgentConfig> {
    AgentConfig::all()
        .into_iter()
        .filter(|agent| agent.is_installed())
        .collect()
}

/// Get a specific agent's config by name.
pub fn get_agent(name: &str) -> Option<AgentConfig> {
    AgentConfig::all()
        .into_iter()
        .find(|a| a.kind.name() == name)
}
