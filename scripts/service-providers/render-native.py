"""Render the existing native service specification for the selected supervisor."""
import json
import os
import pathlib
import plistlib
import re
import sys

request = json.load(sys.stdin)
home, root, label, service, bindir = (request[k] for k in ("home", "root", "label", "service", "binDir"))
id_ = service["id"]
port = str(service.get("port", ""))
env = request["env"]
log = os.path.join(home, ".prometheus", "logs")
state = os.path.join(home, ".prometheus", "exec")
socket = env.get("PROMETHEUS_EXEC_SOCKET") or os.path.join(home, ".prometheus/run/prometheus-exec.sock")
is_mac = sys.platform == "darwin"
mlx = is_mac and os.uname().machine == "arm64"
pk_url = env.get("PK_MCP_URL") or "http://127.0.0.1:8942/mcp"
surreal_url = env.get("SURREAL_ENDPOINT") or "ws://127.0.0.1:28000"
values = {
    "__PROMETHEUS_ROOT__": root, "__PROMETHEUS_HOME__": home, "__HOME__": home,
    "__PROMETHEUS_USER__": request["user"], "__PROMETHEUS_PATH__": env["PATH"],
    "__PROMETHEUS_LOG_DIR__": log, "__PROMETHEUS_EXEC_BIN__": os.path.join(bindir, "prometheus-exec"),
    "__PROMETHEUS_EXEC_STATE__": state, "__PROMETHEUS_EXEC_SOCKET__": socket,
    "__PROMETHEUS_EXEC_IDENTITY__": os.path.join(state, "identity.json"),
    "__PROMETHEUS_PLUGIN_ROOT__": os.path.join(home, ".prometheus/plugins/prometheus-skill-pack"),
    "__SURREAL_BIN__": request.get("surreal") or os.path.join(home, ".prometheus/bin/surreal"),
    "__SURFACE_BRIDGE_BIN__": os.path.join(bindir, "surface-bridge"),
    "__SURREAL_MEMORY_BIN__": os.path.join(bindir, "surreal-memory-server"),
    "__SURREAL_MLX_EXECUTOR__": os.path.join(bindir, "surreal-memory-mlx-executor"),
    "__LOCAL_EMBEDDING_BACKEND__": env.get("PROMETHEUS_LOCAL_EMBEDDING_BACKEND", "mlx" if mlx else "candle"),
    "__LOCAL_EMBEDDING_DEVICE__": env.get("PROMETHEUS_LOCAL_EMBEDDING_DEVICE", "cpu" if is_mac and not mlx else "auto"),
    "__PK_CHERRY_BIN__": os.path.join(bindir, "pk-cherry"), "__FORGE_BIN__": os.path.join(bindir, "forge"),
    "__LEARNING_WORKER_BIN__": os.path.join(bindir, "prometheus-learning-worker"),
}


def expand(obj):
    if isinstance(obj, dict):
        return {k: expand(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [expand(v) for v in obj]
    if isinstance(obj, str):
        return re.sub(r"__[A-Z_]+__", lambda m: values[m[0]], obj)
    return obj


if id_ == "openai-proxy":
    job = {"Label": label, "ProgramArguments": [os.path.join(bindir, id_), "serve", "--host", "127.0.0.1", "--port", port],
           "RunAtLoad": True, "KeepAlive": True, "ThrottleInterval": 10}
else:
    template = pathlib.Path(root, "shared/launchagents", label + ".plist")
    if id_ == "prometheus-research":
        template = pathlib.Path(root, "substrate/prometheus-research/com.prometheus.research.plist")
    # Some historical XML comments document long CLI options containing '--'.
    job = expand(plistlib.loads(re.sub(r"<!--.*?-->", "", template.read_text(), flags=re.S).encode()))
args = job["ProgramArguments"]
job_env = job.setdefault("EnvironmentVariables", {})
job_env.update({"HOME": home, "USER": request["user"], "PATH": env["PATH"]})
job.setdefault("WorkingDirectory", root)
job["StandardOutPath"] = os.path.join(log, id_ + ".stdout.log")
job["StandardErrorPath"] = os.path.join(log, id_ + ".stderr.log")
if id_ == "surrealdb":
    user = env.get("SURREAL_ROOT_USERNAME") or env.get("SURREAL_USERNAME", "root")
    password = env.get("SURREAL_ROOT_PASSWORD") or env.get("SURREAL_PASSWORD", "root")
    args[:] = [a for a in args if not a.startswith(("--user=", "--pass="))]
    args[args.index("--bind=127.0.0.1:28000")] = "--bind=127.0.0.1:" + port
    job_env.update(SURREAL_USER=user, SURREAL_PASS=password)
elif id_ == "surreal-memory":
    job_env.update(API_PORT=port, SURREAL_ENDPOINT=surreal_url)
    for k in ("SURREAL_USERNAME", "SURREAL_PASSWORD", "SURREAL_NAMESPACE", "SURREAL_DATABASE", "SURREAL_AUTH_LEVEL", "SURREAL_REMOTE_WIRE_ALLOW_EXTERNAL", "AUTH_MODE", "AUTH_JWT_ISSUER", "AUTH_JWT_AUDIENCE", "AUTH_JWT_ALGORITHM", "AUTH_JWT_KEY", "AUTH_JWT_KEY_FILE", "AUTH_JWT_MAX_AGE_SECS", "EMBEDDING_PROVIDER", "LOCAL_EMBEDDING_MODEL", "LOCAL_EMBEDDING_MODEL_REVISION", "LOCAL_EMBEDDING_DIMENSIONS", "OPENAI_API_KEY", "OPENAI_BASE_URL"):
        if k in env:
            job_env[k] = env[k]
    job_env["LOCAL_EMBEDDING_DIMENSIONS"] = service["profile"]["dimensions"]
    job_env["SURREAL_USERNAME"] = env.get("MEMORY_USERNAME", job_env["SURREAL_USERNAME"])
    job_env["SURREAL_PASSWORD"] = env.get("MEMORY_PASSWORD", job_env["SURREAL_PASSWORD"])
elif id_ == "pk-cherry":
    job_env.update(PK_BIND="127.0.0.1:" + port)
    if not is_mac:
        job_env["PK_KB_DIR"] = os.path.join(home, ".prometheus/knowledge")
elif id_ == "forge-mcp":
    args[args.index("--port") + 1] = port
    args[args.index("--pk-mcp-url") + 1] = pk_url
    job_env["PK_MCP_URL"] = pk_url
elif id_ == "liter-llm":
    job_env.update(LITER_LLM_BIN=os.path.join(bindir, id_), LITER_LLM_PORT=port)
    if env.get("LITER_LLM_CONFIG"):
        job_env["LITER_LLM_CONFIG"] = env["LITER_LLM_CONFIG"]
elif id_ == "prometheus-research":
    args[0] = os.path.join(bindir, id_)
    args.extend(["--port", port])
elif id_ == "learning-worker":
    default_queue = os.path.join(home, ".prometheus/learning-queue")
    queue = service["endpoint"]
    job_env["PROMETHEUS_LEARNING_QUEUE"] = queue
    job["WatchPaths"] = [p.replace(default_queue, queue, 1) for p in job["WatchPaths"]]
    for k in ("PK_MCP_URL", "SURREAL_MEMORY_URL", "SURREAL_MEMORY_MCP_URL"):
        if env.get(k):
            job_env[k] = env[k]
    if env.get("PROMETHEUS_WORKER_MEMORY_GATEWAY"):
        job_env["SURREAL_MEMORY_URL"] = env["PROMETHEUS_WORKER_MEMORY_GATEWAY"]


def quote(value):
    if any(ord(c) < 32 for c in value):
        raise ValueError("control character in service definition")
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%") + '"'


if is_mac:
    files = {label + ".plist": plistlib.dumps(job).decode()}
else:
    learning = id_ == "learning-worker"
    lines = ["[Unit]", "Description=Prometheus " + id_, "After=network.target", "",
             "[Service]", "Type=" + ("oneshot" if learning else "simple"),
             "WorkingDirectory=" + job["WorkingDirectory"].replace("%", "%%")]
    lines.extend("Environment=" + quote(k + "=" + v) for k, v in job_env.items())
    # The ':' prefix disables systemd's separate dollar-variable expansion.
    lines.append("ExecStart=:" + " ".join(quote(a) for a in args))
    if not learning:
        lines += ["Restart=on-failure", "RestartSec=10s", "", "[Install]", "WantedBy=default.target"]
    files = {label + ".service": "\n".join(lines) + "\n"}
    if learning:
        for extension in ("timer", "path"):
            unit = expand(pathlib.Path(root, "shared/systemd", label + "." + extension).read_text())
            if extension == "path":
                unit = unit.replace(default_queue, queue.replace("%", "%%"))
            files[label + "." + extension] = unit
print(json.dumps(files))
