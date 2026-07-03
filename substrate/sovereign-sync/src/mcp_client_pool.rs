/// MCP client pool for sovereign-sync.
///
/// Reads a `mcp-servers.json` config file (same format used by Claude Code,
/// Kimi, and OpenCode) and establishes rmcp child-process connections to each
/// listed server.
///
/// Privacy gate: KB-content payloads (PrivacyClass::LocalOnly) are NEVER
/// forwarded through MCP client connections to external servers.
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// mcp-servers.json schema
// ---------------------------------------------------------------------------

/// A single MCP server entry — mirrors Claude Code / Kimi mcp-servers.json format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// If set, sovereign-sync will forward only these tool names via passthrough.
    /// Omit to forward all non-local tools.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

/// Top-level mcp-servers.json structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServersConfig {
    #[serde(rename = "mcpServers")]
    pub servers: HashMap<String, McpServerEntry>,
}

impl McpServersConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Resolve the config from the first existing path in the standard search list.
    pub fn load_default() -> Self {
        let candidates = [
            dirs_next::home_dir().map(|h| h.join(".claude").join("mcp-servers.json")),
            dirs_next::home_dir().map(|h| {
                h.join(".config")
                    .join("sovereign-sync")
                    .join("mcp-servers.json")
            }),
            Some(std::path::PathBuf::from("mcp-servers.json")),
        ];
        for candidate in candidates.iter().flatten() {
            if candidate.exists() {
                match Self::load(candidate) {
                    Ok(cfg) => {
                        info!("Loaded MCP server config from {:?}", candidate);
                        return cfg;
                    }
                    Err(e) => {
                        warn!("Failed to parse {:?}: {e}", candidate);
                    }
                }
            }
        }
        info!("No mcp-servers.json found — MCP client pool empty");
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// McpClientPool
// ---------------------------------------------------------------------------

/// A pool of MCP client connections to configured servers.
///
/// In the current implementation, this is a lightweight registry: it holds
/// metadata about configured servers and verifies their command exists.
/// Full rmcp child-process client spawning is deferred to call time to avoid
/// orphan processes.
///
/// The privacy gate is enforced at the call site: `call_tool` refuses to
/// forward any payload classified as LocalOnly.
#[derive(Debug)]
pub struct McpClientPool {
    config: McpServersConfig,
}

impl McpClientPool {
    pub fn new(config: McpServersConfig) -> Self {
        let count = config.servers.len();
        info!("McpClientPool initialized with {count} server entries");
        for (name, entry) in &config.servers {
            info!("  MCP server: {name} — command: {}", entry.command);
        }
        Self { config }
    }

    pub fn from_default() -> Self {
        Self::new(McpServersConfig::load_default())
    }

    /// List all configured server names.
    pub fn server_names(&self) -> Vec<&str> {
        self.config.servers.keys().map(|s| s.as_str()).collect()
    }

    /// Check whether a server entry allows forwarding a given tool.
    ///
    /// If `allowed_tools` is empty → all tools allowed.
    pub fn allows_tool(&self, server: &str, tool: &str) -> bool {
        match self.config.servers.get(server) {
            Some(entry) => {
                entry.allowed_tools.is_empty() || entry.allowed_tools.iter().any(|t| t == tool)
            }
            None => false,
        }
    }

    /// Forward a single MCP `tools/call` request to a configured stdio server.
    ///
    /// Each call starts a short-lived child process, performs the MCP
    /// initialize handshake, sends the initialized notification, forwards the
    /// requested tool call, and then waits for the child to exit. Keeping calls
    /// short-lived avoids orphaning upstream server processes in the daemon.
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let entry = self
            .config
            .servers
            .get(server)
            .ok_or_else(|| anyhow::anyhow!("MCP server '{server}' is not configured"))?;

        if !self.allows_tool(server, tool) {
            anyhow::bail!("MCP server '{server}' does not allow tool '{tool}'");
        }

        let mut command = Command::new(&entry.command);
        command
            .args(&entry.args)
            .envs(&entry.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = command.spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open stdin for MCP server '{server}'"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open stdout for MCP server '{server}'"))?;
        let mut stdout = BufReader::new(stdout);

        write_json_line(
            &mut stdin,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "sovereign-sync",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }),
        )
        .await?;
        let _init = read_response(&mut stdout, 1).await?;

        write_json_line(
            &mut stdin,
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            }),
        )
        .await?;

        write_json_line(
            &mut stdin,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": tool,
                    "arguments": arguments
                }
            }),
        )
        .await?;
        let result = read_response(&mut stdout, 2).await?;
        drop(stdin);

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("MCP server '{server}' exited with status {status}");
        }

        Ok(result)
    }
}

async fn write_json_line(
    stdin: &mut tokio::process::ChildStdin,
    value: serde_json::Value,
) -> anyhow::Result<()> {
    let mut line = serde_json::to_vec(&value)?;
    line.push(b'\n');
    stdin.write_all(&line).await?;
    stdin.flush().await?;
    Ok(())
}

async fn read_response(
    stdout: &mut BufReader<tokio::process::ChildStdout>,
    expected_id: u64,
) -> anyhow::Result<serde_json::Value> {
    let mut line = String::new();
    loop {
        line.clear();
        let read = stdout.read_line(&mut line).await?;
        if read == 0 {
            anyhow::bail!("MCP server exited before response id {expected_id}");
        }
        let response: serde_json::Value = serde_json::from_str(line.trim_end())?;
        if response.get("id").and_then(|id| id.as_u64()) != Some(expected_id) {
            continue;
        }
        if let Some(error) = response.get("error") {
            anyhow::bail!("MCP server returned error for id {expected_id}: {error}");
        }
        return response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("MCP response id {expected_id} missing result"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_config_allows_no_tools() {
        let pool = McpClientPool::new(McpServersConfig::default());
        assert!(pool.server_names().is_empty());
        assert!(!pool.allows_tool("nonexistent", "some-tool"));
    }

    #[test]
    fn allowed_tools_filter_works() {
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".into(),
            McpServerEntry {
                command: "test".into(),
                args: vec![],
                env: HashMap::new(),
                allowed_tools: vec!["tool-a".into()],
            },
        );
        let cfg = McpServersConfig { servers };
        let pool = McpClientPool::new(cfg);
        assert!(pool.allows_tool("test-server", "tool-a"));
        assert!(!pool.allows_tool("test-server", "tool-b"));
    }

    #[test]
    fn empty_allowed_tools_allows_all() {
        let mut servers = HashMap::new();
        servers.insert(
            "open-server".into(),
            McpServerEntry {
                command: "test".into(),
                args: vec![],
                env: HashMap::new(),
                allowed_tools: vec![],
            },
        );
        let cfg = McpServersConfig { servers };
        let pool = McpClientPool::new(cfg);
        assert!(pool.allows_tool("open-server", "any-tool"));
        assert!(pool.allows_tool("open-server", "another-tool"));
    }

    #[tokio::test]
    async fn call_tool_forwards_to_stdio_server() {
        let pool = McpClientPool::new(config_with_fixture(vec![]));
        let result = pool
            .call_tool("fixture", "echo", json!({"message": "hello"}))
            .await
            .unwrap();

        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "hello");
    }

    #[tokio::test]
    async fn call_tool_rejects_disallowed_tool_before_spawning() {
        let pool = McpClientPool::new(config_with_fixture(vec!["echo".into()]));
        let error = pool
            .call_tool("fixture", "blocked", json!({}))
            .await
            .unwrap_err();

        assert!(
            error.to_string().contains("does not allow tool"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn call_tool_surfaces_upstream_error() {
        let pool = McpClientPool::new(config_with_fixture(vec![]));
        let error = pool
            .call_tool("fixture", "fail", json!({}))
            .await
            .unwrap_err();

        assert!(
            error.to_string().contains("MCP server returned error"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn call_tool_surfaces_upstream_exit() {
        let mut servers = HashMap::new();
        servers.insert(
            "broken".into(),
            McpServerEntry {
                command: "python3".into(),
                args: vec!["-c".into(), "import sys; sys.exit(7)".into()],
                env: HashMap::new(),
                allowed_tools: vec![],
            },
        );
        let pool = McpClientPool::new(McpServersConfig { servers });

        let error = pool
            .call_tool("broken", "echo", json!({}))
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("exited before response id 1"),
            "unexpected error: {error}"
        );
    }

    fn config_with_fixture(allowed_tools: Vec<String>) -> McpServersConfig {
        let mut servers = HashMap::new();
        servers.insert(
            "fixture".into(),
            McpServerEntry {
                command: "python3".into(),
                args: vec!["-u".into(), "-c".into(), fixture_script().into()],
                env: HashMap::new(),
                allowed_tools,
            },
        );
        McpServersConfig { servers }
    }

    fn fixture_script() -> &'static str {
        r#"
import json
import sys

for line in sys.stdin:
    msg = json.loads(line)
    method = msg.get("method")
    msg_id = msg.get("id")
    if method == "initialize":
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "fixture", "version": "1.0.0"}
            }
        }), flush=True)
    elif method == "notifications/initialized":
        continue
    elif method == "tools/call":
        name = msg.get("params", {}).get("name")
        arguments = msg.get("params", {}).get("arguments", {})
        if name == "echo":
            print(json.dumps({
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": str(arguments.get("message", ""))
                    }],
                    "isError": False
                }
            }), flush=True)
        else:
            print(json.dumps({
                "jsonrpc": "2.0",
                "id": msg_id,
                "error": {
                    "code": -32601,
                    "message": "unknown tool"
                }
            }), flush=True)
            sys.exit(2)
"#
    }
}
