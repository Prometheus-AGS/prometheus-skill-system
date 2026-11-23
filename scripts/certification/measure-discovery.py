#!/usr/bin/env python3
"""Measure synthetic skill descriptions in native model-visible discovery prompts."""
import argparse
import codex_projection
from datetime import datetime, timezone
import gzip
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import math
import os
from pathlib import Path
import queue
import shutil
import signal
import subprocess
import tempfile
import threading
import tomllib
import time

HARNESSES = {"claude-code": "claude", "codex": "codex", "opencode": "opencode", "kimi": "kimi"}
PROJECT = Path(__file__).resolve().parents[2]
PROMPT = "Reply briefly."
TIMEOUT = 45
CONFIGS = {"claude-code": ".claude/settings.json", "codex": ".codex/config.toml",
           "opencode": ".config/opencode/opencode.json", "kimi": ".kimi-code/config.toml"}


def entry(index):
    name = f"discovery-probe-{index:04d}"
    text = ("Use this skill to describe a synthetic example from the local catalogue. "
            "Identify the example by its unique reference and give a short summary. "
            f"The reference is atlas-{index:04d}.")
    return name, text


def prepare_home(home, harness, count):
    work = home / "work"
    work.mkdir()
    skill_dir = home / { "claude-code": ".claude/skills", "codex": ".agents/skills",
                         "opencode": ".config/opencode/skills", "kimi": ".kimi-code/skills" }[harness]
    for index in range(1, count + 1):
        name, description = entry(index)
        folder = skill_dir / name
        folder.mkdir(parents=True)
        (folder / "SKILL.md").write_text(
            f"---\nname: {name}\ndescription: {description}\n---\n\nDescribe the supplied example.\n")
    return work, skill_dir


def environment(home):
    env = {"HOME": str(home), "PATH": os.environ["PATH"], "LANG": "en_US.UTF-8",
           "CODEX_HOME": str(home / ".codex"), "KIMI_CODE_HOME": str(home / ".kimi-code"),
           "CLAUDE_CONFIG_DIR": str(home / ".claude"), "XDG_CONFIG_HOME": str(home / ".config"),
           "XDG_DATA_HOME": str(home / ".local/share"), "XDG_CACHE_HOME": str(home / ".cache"),
           "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1", "DO_NOT_TRACK": "1",
           "OPENCODE_DISABLE_AUTOUPDATE": "1", "OPENCODE_DISABLE_MODELS_FETCH": "true",
           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
           "NO_PROXY": "127.0.0.1,localhost", "no_proxy": "127.0.0.1,localhost"}
    for key in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"):
        env[key] = "http://127.0.0.1:1"
    (home / ".codex").mkdir()
    return env


def codex_options():
    file = Path.home() / CONFIGS["codex"]
    config = tomllib.loads(file.read_text()) if file.is_file() else {}
    options = []
    for key in ("model", "model_context_window"):
        if isinstance(config.get(key), (str, int)):
            options += ["-c", f"{key}={json.dumps(config[key])}"]
    tokens = config.get("skills", {}).get("max_context_tokens")
    if isinstance(tokens, int):
        options += ["-c", f"skills.max_context_tokens={tokens}"]
    return options


def inputs(harness, exe, home, url, skills, env):
    if harness == "codex":
        options = codex_options()
        return [exe, "debug", "prompt-input", *options, PROMPT], f"codex debug prompt-input {options}"
    if harness == "claude-code":
        env.update(ANTHROPIC_API_KEY="local-synthetic-key", ANTHROPIC_BASE_URL=url,
                   DISABLE_AUTOUPDATER="1")
        settings_file = Path.home() / ".claude/settings.json"
        settings = json.loads(settings_file.read_text()) if settings_file.is_file() else {}
        effective = {k:settings[k] for k in ("model", "skillListingBudgetFraction") if k in settings}
        limit = os.environ.get("SLASH_COMMAND_TOOL_CHAR_BUDGET", settings.get("env", {}).get("SLASH_COMMAND_TOOL_CHAR_BUDGET"))
        if limit is not None:
            env["SLASH_COMMAND_TOOL_CHAR_BUDGET"] = str(limit)
        (home / ".claude/settings.json").write_text(json.dumps(effective))
        source = f"native Anthropic messages request, model={effective.get('model', 'default')}, character override={limit}"
        return [exe, "-p", "--max-turns", "1", "--strict-mcp-config", "--mcp-config",
                '{"mcpServers":{}}', "--", PROMPT], source
    if harness == "opencode":
        config_file = Path.home() / ".config/opencode/opencode.json"
        settings = json.loads(config_file.read_text()) if config_file.is_file() else {}
        model = settings.get("model", "openai/gpt-5.4")
        provider, _, model_id = model.partition("/")
        limit = settings.get("provider", {}).get(provider, {}).get("models", {}).get(model_id, {}).get("limit", {})
        model_config = {"name": model_id}
        if limit:
            model_config["limit"] = limit
        env["OPENCODE_CONFIG_DIR"] = str(skills.parent)
        agents = home / ".agents"
        agents.mkdir()
        (agents / "skills").symlink_to(skills, target_is_directory=True)
        config = {"model": model, "provider": {provider: {"npm": "@ai-sdk/openai-compatible",
                  "name": "Local calibration", "options": {"baseURL": url + "/v1", "apiKey": "local-synthetic-key"},
                  "models": {model_id: model_config}}}}
        (home / ".config/opencode/opencode.json").write_text(json.dumps(config))
        return [exe, "run", "--standalone", "--title", "Discovery measurement", "--model", model, PROMPT], f"OpenAI-compatible request, {model} {limit}"
    config_file = Path.home() / CONFIGS["kimi"]
    settings = tomllib.loads(config_file.read_text()) if config_file.is_file() else {}
    configured = settings.get("models", {}).get(settings.get("default_model"), {})
    model = configured.get("model", "k3")
    window = int(configured.get("max_context_size", 1048576))
    (home / ".kimi-code/config.toml").write_text(
        'default_model="probe"\n[providers.loopback]\ntype="openai"\n'
        f'base_url="{url}/v1"\napi_key="local-synthetic-key"\n'
        f'[models.probe]\nprovider="loopback"\nmodel={json.dumps(model)}\nmax_context_size={window}\n')
    return [exe, "-p", PROMPT, "-m", "probe", "--skills-dir", str(skills)], f"OpenAI request, {model}/{window}"


def native_capture(harness, exe, count):
    with tempfile.TemporaryDirectory(prefix="catalogue-") as temp:
        home = Path(temp)
        work, skills = prepare_home(home, harness, count)
        env = environment(home)
        requests = queue.Queue()

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_POST(self):
                data = self.rfile.read(int(self.headers.get("Content-Length", "0")))
                try:
                    payload = json.loads(gzip.decompress(data) if self.headers.get("Content-Encoding") == "gzip" else data)
                    if (self.path.split("?")[0].endswith(("/messages", "/chat/completions", "/responses"))
                            and ("messages" in payload or "input" in payload)):
                        requests.put(payload)
                finally:
                    self.send_response(400)
                    self.end_headers()

        with ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server:
            url = f"http://127.0.0.1:{server.server_port}"
            command, source = inputs(harness, exe, home, url, skills, env)
            if harness == "codex":
                run = subprocess.run(command, env=env, cwd=work, capture_output=True, timeout=TIMEOUT)
                if run.returncode:
                    raise RuntimeError(run.stderr.decode(errors="replace")[-1200:])
                return json.loads(run.stdout), source

            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            with tempfile.TemporaryFile() as log:
                process = subprocess.Popen(command, cwd=work, env=env, stdout=log, stderr=log,
                                           stdin=subprocess.DEVNULL, start_new_session=True)
                try:
                    deadline = time.monotonic() + TIMEOUT
                    while time.monotonic() < deadline:
                        try:
                            payload = requests.get(timeout=0.2)
                        except queue.Empty:
                            if process.poll() is not None:
                                break
                            continue
                        if "discovery-probe-" in json.dumps(payload):
                            return payload, source
                    log.seek(0)
                    raise RuntimeError("no matching discovery-bearing model request captured: " + log.read().decode(errors="replace")[-1400:])
                finally:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    process.wait()
                    server.shutdown()
                    thread.join()


def observe(harness, exe, count):
    payload, source = native_capture(harness, exe, count)
    raw = json.dumps(payload, sort_keys=True, ensure_ascii=False).encode()
    text = raw.decode()
    names = descriptions = chars = 0
    for index in range(1, count + 1):
        name, description = entry(index)
        names += name in text
        descriptions += description in text
        chars += len(name) + len(description) + 2
    sample = {"count": count, "inputChars": chars, "namesSeen": names,
              "completeDescriptions": descriptions, "promptSha256": hashlib.sha256(raw).hexdigest()}
    print(harness, count, f"{names} names, {descriptions} complete descriptions", flush=True)
    return sample, source


def measure(args):
    harness = args.harness
    exe = shutil.which(HARNESSES[harness])
    if not exe:
        raise ValueError(f"{HARNESSES[harness]} unavailable")
    version = subprocess.check_output([exe, "--version"], text=True).strip()
    budget = json.loads(subprocess.check_output([
        args.prometheus, "skill", "--path", str(PROJECT / "skills"), "budget", "--harness", harness, "--json"]))
    row_size = sum(map(len, entry(1))) + 2
    high = max(9, math.ceil(budget["inventoryChars"] * 1.25 / row_size))
    low = 1
    samples = {}
    samples[low], source = observe(harness, exe, low)
    if samples[low]["completeDescriptions"] != low:
        raise ValueError("native prompt omitted the calibration description")
    samples[high], _ = observe(harness, exe, high)
    if samples[high]["completeDescriptions"] < high:
        while high > low + 1:
            mid = (high + low) // 2
            samples[mid], _ = observe(harness, exe, mid)
            if samples[mid]["completeDescriptions"] == mid:
                low = mid
            else:
                high = mid
    config_file = Path.home() / CONFIGS[harness]
    config_digest = hashlib.sha256(config_file.read_bytes()).hexdigest() if config_file.is_file() else None
    receipt = {"schemaVersion": 2, "configSha256": config_digest, "harness": harness, "harnessVersion": version,
               "capturedAt": datetime.now(timezone.utc).isoformat(), "source": source,
               "inventorySha256": budget["inventorySha256"], "inventoryChars": budget["inventoryChars"],
               "inventorySkills": budget["skills"], "samples": [samples[i] for i in sorted(samples)]}
    if harness == "codex":
        receipt["projection"] = codex_projection.assess(PROJECT, exe, environment, codex_options())
    if harness == "claude-code":
        receipt["environmentBudget"] = os.environ.get("SLASH_COMMAND_TOOL_CHAR_BUDGET")
    args.receipt_dir.mkdir(parents=True, exist_ok=True)
    file = args.receipt_dir / f"{harness}.json"
    with tempfile.NamedTemporaryFile(dir=file.parent, mode="w", delete=False) as out:
        json.dump(receipt, out, indent=2)
        out.write("\n")
    os.replace(out.name, file)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--harness", required=True, choices=HARNESSES)
    parser.add_argument("--prometheus", default="prometheus")
    parser.add_argument("--receipt-dir", type=Path,
                        default=Path.home() / ".prometheus/skill-activation/harness-budgets")
    measure(parser.parse_args())
