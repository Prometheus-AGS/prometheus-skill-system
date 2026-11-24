#!/usr/bin/env python3
"""Local acceptance of signed KBD session exchange and team handoff."""
import argparse
from contextlib import ExitStack, closing
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import http.client
import http.server
import json
import os
from pathlib import Path
import re
import select
import shutil
import signal
import socket
import socketserver
import stat
import struct
import subprocess
import sys
import tempfile
import threading
import time
import uuid


SCRIPT = Path(__file__).resolve()
PACK = SCRIPT.parents[2]
TEAM_CLI = PACK / "skills/process/agent-team-creator/scripts/cli.mjs"
SPEC = PACK / ".kbd-orchestrator/changes/change-kis-001-live-sessions"
ASK = "Review is blocking release"
ACK = "Reviewed and ready"


class Blocked(RuntimeError):
    """A required platform capability or external executable is unavailable."""


def ident():
    return str(uuid.uuid4())


def stamp():
    return {"utc": datetime.now(timezone.utc).isoformat(), "monotonicNs": time.monotonic_ns()}


def sha(value):
    return hashlib.sha256(value).hexdigest()


def hash_file(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def lines(data):
    return [json.loads(line) for line in data.splitlines() if line.strip()]


def executable(value):
    path = shutil.which(value)
    if not path:
        raise Blocked(f"required executable unavailable: {value}")
    return str(Path(path).resolve())


def isolated_env(root):
    env = {"PATH": os.defpath, "NO_PROXY": "*", "USER": "kbd-fixture",
           "LOGNAME": "kbd-fixture", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
           "GIT_TERMINAL_PROMPT": "0", "PYTHONDONTWRITEBYTECODE": "1"}
    for var, directory in {"HOME": "home", "TMPDIR": "tmp", "XDG_CONFIG_HOME": "config",
                           "XDG_DATA_HOME": "data", "XDG_STATE_HOME": "state",
                           "XDG_CACHE_HOME": "cache", "PROMETHEUS_DATA_DIR": "data",
                           "PROMETHEUS_SKILLS_DIR": "skills"}.items():
        path = root / directory
        path.mkdir(parents=True, exist_ok=True)
        env[var] = str(path)
    return env


def run_process(argv, cwd, env, timeout):
    started = stamp()
    with subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) as process:
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            stdout, stderr = process.communicate()
            return {"argv": argv, "pid": process.pid, "started": started, "finished": stamp(),
                    "exitCode": process.returncode, "stdout": stdout, "stderr": stderr,
                    "timedOut": True}
        return {"argv": argv, "pid": process.pid, "started": started, "finished": stamp(),
                "exitCode": process.returncode, "stdout": stdout, "stderr": stderr}


def stop(process):
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=6)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=4)
    return process.returncode


def wait_until(description, predicate, timeout, interval=0.08):
    deadline = time.monotonic() + timeout
    while True:
        found = predicate()
        if found:
            return found
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f"deadline exceeded: {description}")
        time.sleep(min(interval, remaining))


class Evidence:
    def __init__(self, report, timeout, stack):
        self.report = report
        self.timeout = timeout
        self.stack = stack
        self.processes = []

    def check(self, description, condition):
        self.report["assertions"].append({"description": description, "passed": bool(condition)})
        if not condition:
            raise AssertionError(description)

    def command(self, argv, cwd, env, expect=None):
        result = run_process([str(arg) for arg in argv], cwd, env, self.timeout)
        return self.result(result, expect)

    def result(self, result, expect=None):
        self.report["commands"].append(result)
        if result.get("timedOut"):
            raise TimeoutError(f"command exceeded deadline: {result['argv'][:5]}")
        if expect is not None:
            self.check(f"expected exit {expect} for {result['argv'][0]} {result['argv'][1:4]}",
                       result["exitCode"] == expect)
        return result

    def register(self, pid, kind):
        row = {"pid": pid, "kind": kind, "started": stamp()}
        self.processes.append(row)
        return row

    def observe_cleanup(self):
        remaining = []
        for row in self.processes:
            try:
                os.kill(row["pid"], 0)
            except ProcessLookupError:
                row.setdefault("stopped", stamp())
            else:
                remaining.append(row["pid"])
        return remaining


def session_worker():
    """Own one persistent command channel and the watcher it starts."""
    watchers = {}
    try:
        for line in sys.stdin:
            action, *args = json.loads(line)
            if action == "identify":
                result = {"pid": os.getpid(), "at": stamp(), "label": os.environ["CODEX_THREAD_ID"]}
            elif action == "run":
                argv, cwd, overrides, timeout, start_barrier = args
                if start_barrier:
                    print(json.dumps({"armed": True}), flush=True)
                    wait_until("concurrent command start barrier", Path(start_barrier).is_file, timeout, 0.002)
                result = run_process(argv, cwd, dict(os.environ) | overrides, timeout)
            elif action == "watch":
                argv, cwd, overrides, stdout, stderr = args
                with open(stdout, "wb") as out, open(stderr, "wb") as err:
                    process = subprocess.Popen(argv, cwd=cwd, env=dict(os.environ) | overrides,
                                               stdin=subprocess.DEVNULL, stdout=out, stderr=err)
                watchers[process.pid] = process
                result = {"pid": process.pid, "started": stamp(), "argv": argv}
            elif action == "watch-status":
                result = watchers[args[0]].poll()
            elif action == "watch-stop":
                result = stop(watchers.pop(args[0]))
            elif action == "close":
                return
            else:
                raise ValueError(f"unknown fixture action {action}")
            print(json.dumps(result), flush=True)
    finally:
        for process in watchers.values():
            stop(process)


class Session:
    def __init__(self, fixture, label):
        self.fixture = fixture
        self.evidence = fixture.evidence
        self.label = f"{label}-{ident()}"
        env = fixture.env | {"CODEX_THREAD_ID": self.label, "PROMETHEUS_HARNESS": label}
        self.error = (fixture.root / f"{self.label}.log").open("wb")
        self.evidence.stack.callback(self.error.close)
        self.process = subprocess.Popen([sys.executable, str(SCRIPT), "--session-worker"], env=env,
                                        cwd=fixture.root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.error, text=True, start_new_session=True)
        self.row = self.evidence.register(self.process.pid, "session")
        self.evidence.stack.callback(self.close)
        identity = self.request("identify")
        self.evidence.check("persistent session reports its child PID and unique session label",
                            (identity["pid"], identity["label"]) == (self.process.pid, self.label))
        self.row["label"] = self.label

    def send(self, action, *args):
        if self.process.poll() is not None:
            raise RuntimeError(f"session {self.label} exited before {action}: {self.process.returncode}")
        self.process.stdin.write(json.dumps([action, *args]) + "\n")
        self.process.stdin.flush()

    def receive(self):
        if not select.select([self.process.stdout], [], [], self.evidence.timeout + 2)[0]:
            raise TimeoutError(f"session {self.label} stopped responding")
        response = self.process.stdout.readline()
        if not response:
            raise RuntimeError(f"session {self.label} exited before sending its receipt")
        return json.loads(response)

    def request(self, action, *args):
        self.send(action, *args)
        return self.receive()

    def start_command(self, argv, cwd, overrides=None, start_barrier=None):
        self.send("run", [str(arg) for arg in argv], str(cwd), overrides or {}, self.evidence.timeout,
                  str(start_barrier) if start_barrier else None)

    def command(self, argv, cwd, overrides=None, expect=None):
        self.start_command(argv, cwd, overrides)
        return self.evidence.result(self.receive(), expect)

    def close(self):
        if self.process.poll() is None:
            try:
                self.send("close")
                self.process.wait(timeout=9)
            except (OSError, subprocess.TimeoutExpired):
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait(timeout=4)
        if any(row.get("sessionPid") == self.process.pid and "stopped" not in row
               for row in self.evidence.processes):
            try:
                os.killpg(self.process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        self.process.stdin.close()
        self.process.stdout.close()
        self.row["exitCode"] = self.process.returncode
        self.row["stopped"] = stamp()


class Watcher:
    def __init__(self, session, project, endpoint=None, after=None, strict=False):
        self.session = session
        self.evidence = session.evidence
        tag = ident()
        self.out = session.fixture.root / f"watch-{tag}.jsonl"
        self.err = session.fixture.root / f"watch-{tag}.log"
        flags = (["--after-event", after] if after else []) + (["--receiver-only"] if strict else [])
        argv = session.fixture.kbd_argv(project, "watch", *flags)
        started = session.request("watch", argv, str(project), control_env(endpoint), str(self.out), str(self.err))
        self.pid = started["pid"]
        self.row = self.evidence.register(self.pid, "watcher")
        self.row.update(started | {"sessionPid": session.process.pid, "sessionLabel": session.label,
                                   "afterEvent": after, "controlSocket": str(endpoint) if endpoint else None})
        self.evidence.stack.callback(self.close)

    def events(self):
        # The writer may still be completing the last JSONL record.
        return [json.loads(line) for line in self.out.read_bytes().split(b"\n")[:-1] if line.strip()]

    def sources(self):
        return [line.removeprefix("KBD source: ") for line in self.err.read_text().splitlines()
                if line.startswith("KBD source: ")]

    def ensure_running(self):
        code = self.session.request("watch-status", self.pid)
        if code is not None:
            raise RuntimeError(f"watcher {self.pid} exited {code}: {self.err.read_text()[-500:]}")

    def wait_event(self, command_id):
        def observed():
            self.ensure_running()
            return next((event for event in self.events() if event.get("commandId") == command_id), None)

        event = wait_until(f"watcher {self.pid} sees command {command_id}", observed, self.evidence.timeout)
        self.row.setdefault("observations", []).append({"event": event, "observed": stamp()})
        return event

    def wait_source(self, wanted, after=0):
        def observed():
            self.ensure_running()
            return wanted in self.sources()[after:]

        wait_until(f"watcher {self.pid} reports source {wanted}", observed, self.evidence.timeout)
        self.row.setdefault("sourceObservations", []).append({"source": wanted, "observed": stamp()})

    def close(self):
        if "stopped" not in self.row:
            code = self.session.request("watch-stop", self.pid)
            self.row.update({"exitCode": code, "stopped": stamp(), "stdout": self.out.read_text(),
                             "stderr": self.err.read_text()})


class UnixHTTP(http.client.HTTPConnection):
    def __init__(self, path, timeout):
        super().__init__("localhost", timeout=timeout)
        self.path = str(path)

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.connect(self.path)


def socket_request(path, route, method="GET", body=None, timeout=2):
    headers = {"Content-Type": "application/json"} if body is not None else {}
    with closing(UnixHTTP(path, timeout)) as connection:
        connection.request(method, route, body, headers)
        response = connection.getresponse()
        return response.status, response.getheader("Content-Type"), response.read()


def caller_pid(connection):
    if sys.platform == "darwin":
        # Darwin's <sys/un.h> defines SOL_LOCAL=0 and LOCAL_PEERPID=0x002.
        return connection.getsockopt(0, 0x002)
    if hasattr(socket, "SO_PEERCRED"):
        credentials = connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i"))
        return struct.unpack("3i", credentials)[0]
    raise Blocked("Unix peer PID attribution is required for the socket observation")


class ControlHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.forward("GET")

    def do_POST(self):
        self.forward("POST")

    def forward(self, method):
        proxy = self.server.proxy
        started = stamp()
        pid = caller_pid(self.connection)
        request = None if method == "GET" else self.rfile.read(int(self.headers["Content-Length"]))
        try:
            status, content_type, response = socket_request(proxy.target, self.path, method, request, proxy.timeout)
        except (OSError, http.client.HTTPException):
            self.send_error(502, "fixture backend transport failed")
            return
        observed = {"pid": pid, "method": method, "route": self.path, "status": status,
                    "requestAt": started, "responseAt": stamp(), "responseSha256": sha(response)}
        if request is not None:
            envelope = json.loads(request)
            observed["requestSha256"] = sha(request)
            observed["commandId"] = envelope["command"]["commandId"]
            observed["signed"] = bool(envelope.get("signature"))
        elif self.path.endswith("/audit") and status == 200:
            observed["events"] = [event_identity(event) for event in lines(response) if event.get("commandId")]
        if method == "POST" and status == 200 and observed.get("commandId") == proxy.drop_command:
            proxy.drop_command = None
            observed["responseDroppedAfterCommit"] = True
            proxy.observations.append(observed)
            return
        proxy.observations.append(observed)
        self.send_response(status)
        if content_type is not None:
            self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        try:
            self.wfile.write(response)
        except BrokenPipeError:
            pass


class ControlProxy:
    """Relay real Companion traffic, with one optional post-commit response loss."""
    def __init__(self, fixture):
        self.path = fixture.root / "p" / "control.sock"
        self.path.parent.mkdir()
        self.target = fixture.socket
        self.timeout = fixture.evidence.timeout
        self.observations = []
        self.drop_command = None
        self.server = None
        fixture.evidence.stack.callback(self.stop)

    def start(self):
        self.server = socketserver.UnixStreamServer(str(self.path), ControlHandler)
        self.server.proxy = self
        self.thread = threading.Thread(target=self.server.serve_forever, kwargs={"poll_interval": 0.05})
        self.thread.start()

    def stop(self):
        if self.server is not None:
            self.server.shutdown()
            self.thread.join()
            self.server.server_close()
            self.path.unlink(missing_ok=True)
            self.server = None


def control_env(socket_path=None, endpoint=None):
    if endpoint:
        return {"PROMETHEUS_KBD_CONTROL_PLANE": "1", "PROMETHEUS_CONTROL_ENDPOINT": endpoint}
    if socket_path:
        return {"PROMETHEUS_KBD_CONTROL_PLANE": "1", "SOVEREIGN_SYNC_SOCKET": str(socket_path)}
    return {}


def event_identity(event):
    return {key: event.get(key) for key in ("projectId", "commandId", "eventId", "integrityHash",
                                            "signature", "signerKeyId", "revision", "timestamp")}


class Fixture:
    def __init__(self, root, args, evidence, with_key=True):
        self.root = root
        self.args = args
        self.evidence = evidence
        self.env = isolated_env(root)
        self.socket = root / "s" / "control.sock"
        self.env["SOVEREIGN_SYNC_SOCKET"] = str(self.socket)
        self.headless = None
        if with_key:
            config = root / "config/config.toml"
            self.env["SOVEREIGN_SYNC_CONFIG"] = str(config)
            self.env["PROMETHEUS_DEVICE_KEY_FILE"] = str(config.parent / "device-key.json")
            config.write_text(
                f'[node]\np2p_identity_file = {json.dumps(str(config.parent / "p2p.json"))}\n'
                f'skills_dir = {json.dumps(self.env["PROMETHEUS_SKILLS_DIR"])}\n'
                '[peers]\nlocal_only = true\n[discovery]\nmdns = false\ndht = false\n'
            )
            initialized = evidence.command([args.sync_cli, "--mode", "init"], root, self.env, 0)
            identities = lines(initialized["stdout"])
            self.public = {"keyId": next(row["keyId"] for row in identities if "keyId" in row),
                           "endpointId": next(row["endpointId"] for row in identities if "endpointId" in row)}
            evidence.report["publicFixtures"].append(self.public)

    def git(self, directory, *args):
        return self.evidence.command([self.args.git, "-c", "core.hooksPath=/dev/null",
                                      "-c", "user.name=KBD fixture",
                                      "-c", "user.email=kbd-fixture@example.invalid",
                                      *args], directory, self.env, 0)["stdout"].strip()

    def git_project(self, name, files):
        checkout = self.root / name
        checkout.mkdir()
        self.git(checkout, "init", "--initial-branch=fixture")
        for relative, data in files.items():
            path = checkout / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.git(checkout, "add", "--", ".")
        self.git(checkout, "commit", "-m", "fixture baseline")
        return checkout

    def project(self, name, project_id=None):
        project_id = project_id or ident()
        manifest = {"schemaVersion": "1", "projectId": project_id,
                    "repositoryFingerprint": "sha256:" + sha(project_id.encode())}
        checkout = self.git_project(name, {".prometheus/project.json": encode(manifest)})
        registration = self.register(checkout)
        self.evidence.check("registered checkout maps to the declared UUID",
                            registration["projectId"] == project_id)
        self.evidence.report["fixtureManifests"].append({
            "checkout": str(checkout), "manifest": manifest, "sha256": sha(encode(manifest)),
            "registration": registration, "head": self.git(checkout, "rev-parse", "HEAD"),
        })
        return checkout, project_id

    def worktree(self, checkout, name):
        duplicate = self.root / name
        self.git(checkout, "worktree", "add", "--detach", str(duplicate))
        return duplicate

    def register(self, checkout):
        (checkout / ".kbd-orchestrator").mkdir(exist_ok=True)
        result = self.kbd(checkout, "register", str(checkout), expect=0)
        return json.loads(result["stdout"])["registration"]

    def kbd_argv(self, checkout, *args):
        return [self.args.pack_cli, "kbd", "--path", str(checkout), *args]

    def kbd(self, checkout, *args, session=None, endpoint=None, socket_path=None, expect=None):
        argv = self.kbd_argv(checkout, *args)
        override = control_env(socket_path, endpoint)
        if session:
            return session.command(argv, checkout, override, expect)
        return self.evidence.command(argv, checkout, self.env | override, expect)

    def blocker(self, checkout, action, command_id, blocker, *, session=None, socket_path=None):
        args = ("--summary", ASK) if action == "record" else ("--resolution", ACK)
        return self.kbd(checkout, "blocker", action, "--command-id", command_id, "--id", blocker,
                        *args, session=session, socket_path=socket_path, expect=0)

    def baseline(self, checkout):
        command_id = ident()
        self.blocker(checkout, "record", command_id, ident())
        return self.one(checkout, command_id)

    def audit(self, checkout, **options):
        return json.loads(self.kbd(checkout, "audit", "--json", expect=0, **options)["stdout"])

    def one(self, checkout, command_id, **options):
        events = [event for event in self.audit(checkout, **options) if event.get("commandId") == command_id]
        self.evidence.check(f"independent signed audit has one effect for {command_id}",
                            len(events) == 1 and bool(events[0].get("signature"))
                            and bool(events[0].get("integrityHash")))
        return events[0]

    def background(self, argv, cwd, kind):
        logfile = self.root / f"{kind}-{ident()}.log"
        with logfile.open("wb") as log:
            process = subprocess.Popen(argv, env=self.env, cwd=cwd, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=subprocess.STDOUT)
        row = self.evidence.register(process.pid, kind)
        row["argv"] = argv

        def cleanup():
            if "stopped" not in row:
                row.update({"stopped": stamp(), "exitCode": stop(process)})
        self.evidence.stack.callback(cleanup)
        return process, row, logfile, cleanup

    def start_headless(self, checkout, project_id):
        self.evidence.check("headless control socket is absent before daemon startup", not self.socket.exists())
        self.headless = self.background([self.args.headless], checkout, "headless")
        process, row, log, _ = self.headless

        def ready():
            if process.poll() is not None:
                raise RuntimeError(f"headless failed: {log.read_text(errors='replace')[-1000:]}")
            try:
                status, _, data = socket_request(self.socket, f"/api/v1/kbd/projects/{project_id}/status")
            except (OSError, http.client.HTTPException):
                return None
            return json.loads(data) if status == 200 else None

        status = wait_until("headless Unix socket serves exact project", ready, self.evidence.timeout)
        self.evidence.check("actual Companion serves the registered authority", status["projectId"] == project_id)
        row["ready"] = stamp()
        return status

    def stop_headless(self):
        if self.headless is not None:
            self.headless[3]()
            self.headless = None
        self.socket.unlink(missing_ok=True)

    def socket_audit(self, project_id):
        route = f"/api/v1/kbd/projects/{project_id}/audit"
        status, content_type, body = socket_request(self.socket, route, timeout=self.evidence.timeout)
        self.evidence.check("actual headless socket returns the signed audit media type",
                            status == 200 and content_type == "application/x-ndjson")
        self.evidence.report["directSocketReads"].append(
            {"at": stamp(), "socket": str(self.socket), "route": route,
             "status": status, "responseSha256": sha(body), "events": [event_identity(event) for event in lines(body)]}
        )
        return lines(body)


def unique_effects(events, *ids):
    return [event.get("commandId") for event in events if event.get("commandId") in ids]


def assert_exchange(expected, observed):
    for name, signed in expected.items():
        reads = observed[name]
        if len(reads) != 1 or reads[0] != signed:
            raise AssertionError(f"missing or duplicate exact signed cross-session observation: {name}")


def blocker_shape(event, command_id, action, project_id, blocker, session):
    if event["projectId"] != project_id or event["commandId"] != command_id:
        return False
    actor = event["actor"]
    if actor["session"] != session.label or not event.get("signature"):
        return False
    kind = event["kind"]
    payload = kind["payload"]
    if action == "record":
        return kind["type"] == "blocker_recorded" and payload["blocker"]["id"] == blocker \
            and payload["blocker"]["summary"] == ASK
    return kind["type"] == "blocker_cleared" and payload["blocker_id"] == blocker \
        and payload["resolution"] == ACK


def assert_socket_recording(evidence, proxy, project_id, effects, watchers, writers, *, after=0):
    route = f"/api/v1/kbd/projects/{project_id}"
    captured = proxy.observations[after:]
    for effect, watcher, writer in zip(effects, watchers, writers):
        event = event_identity(effect)
        watched = next((row for row in captured if row["pid"] == watcher.pid
                        and row["method"] == "GET" and row["route"] == route + "/audit"
                        and row["status"] == 200 and event in row.get("events", [])), None)
        evidence.check(f"watcher PID {watcher.pid} read the exact signed event from Companion socket",
                       bool(watched))
        posts = [row for row in captured if row["method"] == "POST" and row["route"] == route + "/commands"
                 and row["status"] == 200 and row.get("commandId") == effect["commandId"]
                 and row.get("signed")]
        evidence.check("same endpoint accepted a signed command from the fixture writer",
                       len(posts) == 1 and posts[0]["pid"] == writer["pid"])
        observation = next(row for row in watcher.row["observations"] if row["event"] == effect)
        evidence.check("socket and watcher observations occurred after command dispatch in this case",
                       posts[0]["requestAt"]["monotonicNs"] >= writer["started"]["monotonicNs"]
                       and watched["responseAt"]["monotonicNs"] <= observation["observed"]["monotonicNs"])


def live_exchange(root, args, evidence, remote, negative):
    fixture = Fixture(root, args, evidence)
    checkout, project_id = fixture.project("primary")
    baseline = fixture.baseline(checkout)
    target = checkout
    endpoint = fixture.socket
    proxy = None
    if remote:
        target = fixture.worktree(checkout, "worktree")
        second = fixture.register(target)
        first = evidence.report["fixtureManifests"][-1]["registration"]
        evidence.check("two supported registrations share one UUID and have distinct replica identities",
                       second["projectId"] == project_id and second["replicaId"] != first["replicaId"]
                       and not second["readOnly"])
        evidence.report["fixtureManifests"].append({"checkout": str(target), "registration": second,
                                                   "head": fixture.git(target, "rev-parse", "HEAD")})
    outsider, outsider_id = fixture.project("unrelated")
    outsider_base = fixture.baseline(outsider)
    if remote:
        fixture.start_headless(checkout, project_id)
        proxy = ControlProxy(fixture)
        proxy.start()
        endpoint = proxy.path
    else:
        evidence.check("standalone variant has no Companion and selected socket is absent",
                       not fixture.socket.exists() and fixture.headless is None)
    a, b = Session(fixture, "a"), Session(fixture, "b")
    # The standalone readers inherit the absent socket path but local mode never selects it.
    use_socket = endpoint if remote else None
    watch_a, watch_b = Watcher(a, checkout, use_socket), Watcher(b, target, use_socket)
    outsider_watch = Watcher(Session(fixture, "unrelated"), outsider, use_socket)
    evidence.check("independent live sessions and watcher PIDs",
                   len({a.process.pid, b.process.pid, watch_a.pid, watch_b.pid}) == 4 and a.label != b.label)
    for watcher, event in ((watch_a, baseline), (watch_b, baseline), (outsider_watch, outsider_base)):
        evidence.check("reader consumed a common pre-write baseline from its own project",
                       watcher.wait_event(event["commandId"]) == event)
        watcher.wait_source("companion" if remote else "local")
    barrier = stamp()
    blocker, ask, ack, other = ident(), ident(), ident(), ident()
    if negative:
        fixture.blocker(outsider, "record", other, blocker, socket_path=use_socket)
        outsider_watch.wait_event(other)
    request = fixture.blocker(checkout, "record", ask, blocker, session=a, socket_path=use_socket)
    seen_ask = watch_b.wait_event(ask)
    response = fixture.blocker(target, "clear", ack, blocker, session=b, socket_path=use_socket)
    seen_ack = watch_a.wait_event(ack)
    expected_ask, expected_ack = (fixture.one(checkout, command) for command in (ask, ack))
    evidence.check("signed request and reply match the newly issued blocker and actors",
                   blocker_shape(expected_ask, ask, "record", project_id, blocker, a)
                   and blocker_shape(expected_ack, ack, "clear", project_id, blocker, b))
    evidence.check("writes and observations followed the baseline and original listeners stayed live",
                   barrier["monotonicNs"] <= request["started"]["monotonicNs"]
                   < watch_b.row["observations"][-1]["observed"]["monotonicNs"]
                   <= response["started"]["monotonicNs"]
                   < watch_a.row["observations"][-1]["observed"]["monotonicNs"])
    for watcher in (watch_a, watch_b):
        watcher.wait_event(ack)
        evidence.check("each watcher saw each signed effect once in order",
                       unique_effects(watcher.events(), ask, ack) == [ask, ack])
        watcher.ensure_running()
    observed = {"request": [seen_ask], "reply": [seen_ack]}
    expected = {"request": expected_ask, "reply": expected_ack}
    assert_exchange(expected, observed)
    evidence.check("independent exchange oracle accepted both exact signed observations", True)
    if remote:
        socket_audit = fixture.socket_audit(project_id)
        evidence.check("the actual Companion audit contains the fresh exact effects once",
                       [event for event in socket_audit if event.get("commandId") in (ask, ack)] ==
                       [expected_ask, expected_ack])
        evidence.check("both successful write receipts came from Companion",
                       all(not json.loads(write["stdout"]).get("committedLocally")
                           for write in (request, response)))
        assert_socket_recording(evidence, proxy, project_id, [expected_ask, expected_ack],
                                [watch_b, watch_a], [request, response])
    if negative:
        fixture.blocker(outsider, "record", ident(), ident(), socket_path=use_socket)
        outsider_end = fixture.one(outsider, other)
        evidence.check("separate UUID does not observe either effect and neither recipient saw the outsider event",
                       outsider_end["projectId"] == outsider_id
                       and not unique_effects(fixture.audit(outsider), ask, ack)
                       and not unique_effects(outsider_watch.events(), ask, ack)
                       and all(not unique_effects(watcher.events(), other) for watcher in (watch_a, watch_b)))
        omitted = deepcopy(observed)
        omitted["request"].clear()
        try:
            assert_exchange(expected, omitted)
        except AssertionError as error:
            evidence.check("removing a live update makes the independent exchange oracle fail",
                           "exact signed cross-session observation: request" in str(error))
        else:
            evidence.check("removing a live update makes the independent exchange oracle fail", False)
        if remote:
            proxy.stop()
            fixture.stop_headless()
            strict = fixture.kbd(checkout, "audit", "--json", "--receiver-only", socket_path=endpoint)
            evidence.check("receiver-only read fails on stopped Companion despite populated local audit",
                           strict["exitCode"] != 0 and not strict["stdout"]
                           and "connect to control socket" in strict["stderr"])
    else:
        watch_b.close()
        missed, next_reply, next_blocker = ident(), ident(), ident()
        missed_write = fixture.blocker(checkout, "record", missed, next_blocker, session=a, socket_path=use_socket)
        fixture.blocker(checkout, "clear", next_reply, next_blocker, session=a, socket_path=use_socket)
        resumed = Watcher(b, target, use_socket, after=expected_ack["eventId"])
        for command in (missed, next_reply):
            evidence.check("resumed watcher sees each missed signed event",
                           resumed.wait_event(command) == fixture.one(checkout, command))
        evidence.check("resume is exclusive and preserves canonical audit order with exactly one effect per ID",
                       unique_effects(resumed.events(), ask, ack, missed, next_reply) == [missed, next_reply]
                       and resumed.pid != watch_b.pid
                       and watch_b.row["stopped"]["monotonicNs"] <= missed_write["started"]["monotonicNs"]
                       < resumed.row["started"]["monotonicNs"])
    evidence.report["exchanges"].append({"mode": "companion" if remote else "local", "projectId": project_id,
                                         "barrier": barrier, "baseline": event_identity(baseline),
                                         "request": event_identity(expected_ask), "reply": event_identity(expected_ack)})
    if proxy:
        evidence.report["relayedSocketRequests"].extend(proxy.observations)


def strict_read(fixture, checkout, *args, socket_path=None, endpoint=None, error):
    result = fixture.kbd(checkout, *args, socket_path=socket_path, endpoint=endpoint)
    fixture.evidence.check(f"{args[0]} fails closed for {error}",
                           result["exitCode"] != 0 and not result["stdout"]
                           and "local-fallback" not in result["stderr"] and error in result["stderr"])
    return result


def fallback_and_claim(root, args, evidence):
    fixture = Fixture(root, args, evidence)
    checkout, project_id = fixture.project("project")
    baseline = fixture.baseline(checkout)
    a, b = Session(fixture, "a"), Session(fixture, "b")
    fixture.start_headless(checkout, project_id)
    proxy = ControlProxy(fixture)
    proxy.start()
    watcher = Watcher(a, checkout, proxy.path)
    evidence.check("watcher starts with an exact Companion baseline", watcher.wait_event(baseline["commandId"]) == baseline)
    watcher.wait_source("companion")
    proxy.stop()
    fixture.stop_headless()
    evidence.check("both fixture control sockets are absent during fallback",
                   not proxy.path.exists() and not fixture.socket.exists())
    watcher.wait_source("local-fallback")
    fallback_status = fixture.kbd(checkout, "status", "--json", socket_path=proxy.path, expect=0)
    fallback_audit = fixture.kbd(checkout, "audit", "--json", socket_path=proxy.path, expect=0)
    evidence.check("optional status identifies local fallback in state JSON and stderr",
                   json.loads(fallback_status["stdout"])["readSource"] == "local-fallback"
                   and "KBD source: local-fallback" in fallback_status["stderr"])
    evidence.check("optional audit keeps an unwrapped event array and labels the fallback",
                   baseline in json.loads(fallback_audit["stdout"])
                   and "KBD source: local-fallback" in fallback_audit["stderr"])
    strict_read(fixture, checkout, "status", "--json", "--receiver-only", socket_path=proxy.path,
                error="connect to control socket")
    strict_read(fixture, checkout, "audit", "--json", "--receiver-only", socket_path=proxy.path,
                error="connect to control socket")
    strict_read(fixture, checkout, "watch", "--receiver-only", socket_path=proxy.path,
                error="connect to control socket")
    ordinary = fixture.kbd(checkout, "status", "--json", expect=0)
    evidence.check("ordinary local mode stays distinguishable from explicit Companion fallback",
                   json.loads(ordinary["stdout"])["readSource"] == "local")
    ask, ack, blocker = ident(), ident(), ident()
    local = fixture.blocker(checkout, "record", ask, blocker, session=b, socket_path=proxy.path)
    local_receipt = json.loads(local["stdout"])
    evidence.check("connection refusal before dispatch makes a clearly identified local commit",
                   local_receipt.get("committedLocally") is True
                   and local_receipt.get("remoteStatusUnknown") is False
                   and local_receipt.get("commandId") == ask
                   and "unreachable before dispatch" in local["stderr"])
    offline_ask = watcher.wait_event(ask)
    fixture.blocker(checkout, "clear", ack, blocker, session=b, socket_path=proxy.path)
    offline_ack = watcher.wait_event(ack)
    source_count = len(watcher.sources())
    fixture.start_headless(checkout, project_id)
    restoration = stamp()
    restored_at = len(proxy.observations)
    proxy.start()
    watcher.wait_source("companion", after=source_count)
    watch_cursor = offline_ack["eventId"]
    lost, lost_blocker = ident(), ident()
    proxy.drop_command = lost
    uncertain = fixture.blocker(checkout, "record", lost, lost_blocker, session=b, socket_path=proxy.path)
    reconciled = json.loads(uncertain["stdout"])
    remote_event = watcher.wait_event(lost)
    evidence.check("post-commit transport loss yields a distinct audit-confirmed receipt",
                   reconciled.get("commandId") == lost
                   and reconciled.get("reconciledFromAudit") is True and reconciled.get("duplicate") is True
                   and reconciled.get("eventId") == remote_event["eventId"]
                   and reconciled.get("integrityHash") == remote_event["integrityHash"]
                   and "submission ambiguous" in uncertain["stderr"])
    captured = proxy.observations[restored_at:]
    dropped = [row for row in captured if row.get("commandId") == lost and row.get("responseDroppedAfterCommit")]
    evidence.check("fault control dropped one successful real POST response after the backend commit",
                   len(dropped) == 1 and dropped[0]["status"] == 200)
    assert_socket_recording(evidence, proxy, project_id, [remote_event], [watcher], [uncertain], after=restored_at)
    restored_watcher_reads = [row for row in captured if row["method"] == "GET" and row["pid"] == watcher.pid
                              and event_identity(remote_event) in row.get("events", [])]
    evidence.check("fresh restored Companion response came from the same original watcher PID",
                   bool(restored_watcher_reads)
                   and restored_watcher_reads[0]["requestAt"]["monotonicNs"] >= restoration["monotonicNs"]
                   and watcher.sources() == ["companion", "local-fallback", "companion"])
    evidence.check("restored watcher kept ordered catch-up and no duplicate effect at its cursor",
                   unique_effects(watcher.events(), ask, ack, lost) == [ask, ack, lost]
                   and watcher.events().index(offline_ask) < watcher.events().index(offline_ack))
    replay_result = fixture.blocker(checkout, "record", lost, lost_blocker, session=b, socket_path=proxy.path)
    replay = json.loads(replay_result["stdout"])
    evidence.check("same logical command replays without another signed effect",
                   replay.get("duplicate") is True and fixture.one(checkout, lost) == remote_event)
    watcher.ensure_running()

    scope = f"task:{ident()}"
    claim_ids = [ident(), ident()]
    holders = ["session-a", "session-b"]
    start_barrier = root / "claim-start"
    for session, command, holder in zip((a, b), claim_ids, holders):
        argv = fixture.kbd_argv(checkout, "claim", "acquire", scope, "--holder", holder,
                                 "--command-id", command, "--ttl", "600")
        session.start_command(argv, checkout, control_env(proxy.path), start_barrier)
    for session in (a, b):
        evidence.check("claim session armed before release", session.receive() == {"armed": True})
    start_barrier.touch(exist_ok=False)
    claims = [evidence.result(session.receive()) for session in (a, b)]
    evidence.check("both claim processes were live concurrently after the release barrier",
                   max(result["started"]["monotonicNs"] for result in claims)
                   < min(result["finished"]["monotonicNs"] for result in claims))
    evidence.check("concurrent independent claim clients report a single success",
                   sum(result["exitCode"] == 0 for result in claims) == 1)
    winner = next(index for index, result in enumerate(claims) if result["exitCode"] == 0)
    loser = claims[1 - winner]
    evidence.check("losing client reports an explicit claim or frontier conflict",
                   bool(re.search(r"(frontier|claim.*(?:blocked|held|conflict))", loser["stderr"], re.I)))
    canonical = fixture.socket_audit(project_id)
    evidence.check("Companion audit confirms only the winning claim command",
                   unique_effects(canonical, *claim_ids) == [claim_ids[winner]])
    for session in (a, b):
        state = json.loads(fixture.kbd(checkout, "claim", "list", "--json", session=session,
                                       socket_path=proxy.path, expect=0)["stdout"])
        matches = [claim for claim in state["claims"].values() if claim["scope"] == scope]
        evidence.check("each session agrees on the exclusive canonical owner",
                       len(matches) == 1 and matches[0]["holderId"] == holders[winner]
                       and matches[0]["mode"] == "exclusive" and not matches[0]["released"])
    fixture.kbd(checkout, "claim", "acquire", scope, "--holder", holders[winner], "--command-id",
                claim_ids[winner], "--ttl", "600", session=(a, b)[winner], socket_path=proxy.path, expect=0)
    evidence.check("replaying the winning claim keeps one signed effect",
                   unique_effects(fixture.socket_audit(project_id), *claim_ids) == [claim_ids[winner]])
    evidence.report["fallback"] = {"watcherPid": watcher.pid, "restoredAt": restoration,
                                    "cursorEventId": watch_cursor, "observedAfterRestore": event_identity(remote_event),
                                    "winner": holders[winner], "winningCommandId": claim_ids[winner]}
    evidence.report["relayedSocketRequests"].extend(proxy.observations)


def incompatible_server(state, baseline, stack, report):
    bad_state = encode(state | {"schemaVersion": "999"})
    bad_audit = encode(baseline | {"schemaVersion": "999"}) + b"\n"

    class IncompatibleHandler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            audit = self.path.endswith("/audit")
            body = bad_audit if audit else bad_state
            report["incompatibleRequests"].append({"path": self.path, "at": stamp(), "sha256": sha(body)})
            self.send_response(200)
            self.send_header("Content-Type", "application/x-ndjson" if audit else "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    server = http.server.HTTPServer(("127.0.0.1", 0), IncompatibleHandler)
    thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.05})
    thread.start()

    def cleanup():
        server.shutdown()
        thread.join()
        server.server_close()
    stack.callback(cleanup)
    return f"http://127.0.0.1:{server.server_port}"


def loopback_denial(fixture, checkout):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    # This is sovereign-sync's actual TCP bearer middleware, with a private token file.
    token = fixture.root / "config/acceptance-token"
    command = [fixture.args.sync_cli, "--mode", "server", "--tcp", "--port", str(port),
               "--token-file", str(token)]
    process, row, log, _ = fixture.background(command, checkout, "bearer-protected-server")

    def ready():
        if process.poll() is not None:
            raise RuntimeError(f"bearer-protected service exited {process.returncode}: " + log.read_text()[-500:])
        try:
            with closing(http.client.HTTPConnection("127.0.0.1", port, timeout=1)) as client:
                client.request("GET", "/ready")
                return client.getresponse().status == 401
        except (OSError, http.client.HTTPException):
            return False

    wait_until("actual loopback bearer gate returns Unauthorized", ready, fixture.evidence.timeout)
    row["ready"] = stamp()
    return f"http://127.0.0.1:{port}"


def refusal_controls(root, args, evidence):
    local = Fixture(root / "a", args, evidence)
    receiver = Fixture(root / "r", args, evidence)
    checkout, project_id = local.project("local")
    baseline = local.baseline(checkout)
    receiver_checkout, _ = receiver.project("remote", project_id)
    remote_baseline = receiver.baseline(receiver_checkout)
    other_checkout, other_id = local.project("other")
    local.baseline(other_checkout)
    evidence.check("denying endpoint uses a real different signing authority for the same requested UUID",
                   baseline["signerKeyId"] != remote_baseline["signerKeyId"]
                   and baseline["projectId"] == remote_baseline["projectId"] == project_id)
    receiver.start_headless(receiver_checkout, project_id)
    proxy = ControlProxy(receiver)
    proxy.start()
    refused_id = ident()
    before_local, before_remote = local.audit(checkout), receiver.socket_audit(project_id)
    denied = local.kbd(checkout, "blocker", "record", "--command-id", refused_id, "--id", ident(),
                       "--summary", ASK, socket_path=proxy.path)
    refused_posts = [item for item in proxy.observations if item.get("commandId") == refused_id]
    evidence.check("real Companion rejects an unenrolled signed mutation and both journals stay unchanged",
                   denied["exitCode"] != 0 and not denied["stdout"] and "401 Unauthorized" in denied["stderr"]
                   and len(refused_posts) == 1 and refused_posts[0]["status"] == 401 and refused_posts[0]["signed"]
                   and local.audit(checkout) == before_local and receiver.socket_audit(project_id) == before_remote)
    denied_endpoint = loopback_denial(receiver, receiver_checkout)
    for action in (("status", "--json"), ("audit", "--json"), ("watch",)):
        strict_read(local, checkout, *action, endpoint=denied_endpoint, error="401 Unauthorized")
        wrong = strict_read(local, other_checkout, *action, socket_path=proxy.path, error="404 Not Found")
        evidence.check("reachable Companion reported the exact unknown project UUID", other_id in wrong["stderr"])
    real_state = json.loads(local.kbd(checkout, "status", "--json", expect=0)["stdout"])
    malformed = incompatible_server(real_state, baseline, evidence.stack, evidence.report)
    strict_read(local, checkout, "status", "--json", endpoint=malformed, error="incompatible Companion status")
    for action in (("audit", "--json"), ("watch",)):
        strict_read(local, checkout, *action, endpoint=malformed, error="invalid signed KBD audit")
    evidence.check("negative protocol listener served precisely the three requested production read routes",
                   [row["path"].rsplit("/", 1)[1] for row in evidence.report["incompatibleRequests"]]
                   == ["status", "audit", "audit"])
    evidence.check("source data remains valid after the refusal controls", local.audit(checkout) == before_local)
    evidence.report["relayedSocketRequests"].extend(proxy.observations)


def team_case(root, args, evidence, negative):
    fixture = Fixture(root, args, evidence, with_key=False)
    artifact = f"Evidence retained by both sessions: {ident()}\n".encode()
    source = fixture.git_project("source", {"docs/review.txt": artifact})
    destination = fixture.worktree(source, "destination")
    state_path = root / "state.json"
    a, b = Session(fixture, "implementer"), Session(fixture, "reviewer")
    private_ref = "role:implementer/private-review"
    private_path = source / private_ref
    private_path.parent.mkdir()
    with os.fdopen(os.open(private_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as file:
        file.write(ident().encode())
    private_digest = hash_file(private_path)

    def call(command, request, session=None, cwd=source, expect=0):
        input_path = root / f"{command}-{ident()}.json"
        input_path.write_bytes(encode(request))
        argv = [args.node, str(TEAM_CLI), command, "--input", str(input_path)]
        receipt = session.command(argv, cwd, expect=expect) if session else evidence.command(argv, cwd, fixture.env, expect)
        receipt["inputSha256"] = hash_file(input_path)
        receipt["input"] = request
        return json.loads(receipt["stdout"] if expect == 0 else receipt["stderr"])

    team = {"schemaVersion": 1, "id": "live-acceptance", "outcome": "Exchange reviewed work",
            "scope": "project", "harness": "codex",
            "roles": [{"id": role, "description": f"{role} role", "prompt": "Report precise evidence.",
                       "owns": ["docs/"] if role == "implementer" else [], "skills": [],
                       "inputs": ["Requirements"], "outputs": ["Evidence"], "dependsOn": []}
                      for role in ("implementer", "reviewer")]}
    def read(session=None, path=state_path):
        return call("status", {"state": str(path)}, session=session, cwd=destination if session is b else source)

    def transition(state, action, **extra):
        old = state["tasks"][0]
        return call("task", {"state": str(state_path), "expectedRevision": state["revision"],
                             "task": {"action": action, "id": old["id"], "owner": old["owner"],
                                      "expectedTaskRevision": old["revision"], **extra}}, a)

    def create(state):
        return call("handoff-create", {"state": str(state_path), "expectedRevision": state["revision"],
                                       "cwd": str(source), "handoff": {
                                           "taskId": task_id, "owner": "implementer",
                                           "expectedTaskRevision": state["tasks"][0]["revision"],
                                           "toOwner": "reviewer", "toHarness": "claude",
                                           "context": "Review the exact Git artifact from the destination.",
                                           "evidence": ["docs/review.txt"],
                                           "remaining": ["Read the artifact and report the result"],
                                           "memoryRefs": [private_ref],
                                           "provenance": {"evidenceFiles": ["docs/review.txt"]},
                                       }}, a)

    def unchanged_rejection(request, expected_error):
        before = state_path.read_bytes()
        rejected = call("handoff-accept", request, b, destination, expect=1)
        evidence.check("handoff rejection leaves authoritative state and lock unchanged",
                       expected_error in rejected["error"] and state_path.read_bytes() == before
                       and not state_path.with_suffix(".json.lock").exists()
                       and read(b) == json.loads(before))

    state = call("init", {"state": str(state_path), "team": team}, a)
    task_id = f"task-{ident()}"
    state = call("task", {"state": str(state_path), "expectedRevision": state["revision"],
                          "task": {"action": "add", "id": task_id, "title": "Review local evidence",
                                   "owner": "implementer"}}, a)
    state = transition(state, "start")
    state = create(state)
    stale = state["handoffs"][-1]
    if negative:
        state = transition(state, "block", reason="Advance the task revision before retry")
        state = create(state)
    packet = state["handoffs"][-1]
    request = {"state": str(state_path), "expectedRevision": state["revision"], "id": packet["id"],
               "destination": {"owner": "reviewer", "harness": "claude"}}
    before_accept = read(b)
    evidence.check("two active role sessions observe source ownership and unchanged task revision before acceptance",
                   before_accept == read(a) and before_accept["tasks"][0]["owner"] == "implementer"
                   and before_accept["tasks"][0]["revision"] == packet["taskRevision"]
                   and a.process.poll() is None and b.process.poll() is None and a.process.pid != b.process.pid)
    evidence.check("packet binds exact task, destination, next action, and source Git identity",
                   packet["taskId"] == task_id and packet["to"] == request["destination"]
                   and "Read the artifact and report the result" in packet["remaining"]
                   and packet["git"]["head"] == fixture.git(destination, "rev-parse", "HEAD"))
    inspected = call("handoff-inspect", {"state": str(state_path), "id": packet["id"], "cwd": str(destination)},
                     b, destination)
    artifact_in_destination = destination / packet["evidence"][0]
    evidence.check("recipient opens the exact tracked evidence from its distinct Git checkout",
                   artifact_in_destination.read_bytes() == artifact
                   and hash_file(artifact_in_destination) == packet["provenance"]["evidence"][0]["sha256"]
                   and inspected["current"]["evidence"][0]["comparison"] == "matches"
                   and inspected["ownership"]["owner"] == "implementer")
    evidence.report["artifact"] = {"relativePath": packet["evidence"][0], "sha256": sha(artifact),
                                   "commit": packet["git"]["head"],
                                   "permissionScope": "same-user file mode and reference preservation"}
    if negative:
        unchanged_rejection(request | {"destination": {"owner": "reviewer", "harness": "codex"}},
                            "targeted destination")
        unchanged_rejection(request | {"id": stale["id"]}, "Task revision conflict")
        unchanged_rejection(request | {"expectedRevision": request["expectedRevision"] - 1},
                            "State revision conflict")
        copied_state = root / "independent-state.json"
        copied_state.write_bytes(state_path.read_bytes())
        before_copy_acceptance = state_path.read_bytes()
        copied = call("handoff-accept", request | {"state": str(copied_state)}, b, destination)
        evidence.check("accepting a copied packet on independent team state cannot alter original authority",
                       copied["tasks"][0]["owner"] == "reviewer"
                       and state_path.read_bytes() == before_copy_acceptance
                       and read(b)["tasks"][0]["owner"] == "implementer")
    else:
        accepted = call("handoff-accept", request, b, destination)
        before_replay = state_path.read_bytes()
        replay = call("handoff-accept", request, b, destination)
        os_state = json.loads(state_path.read_bytes())
        evidence.check("explicit acceptance advances both revisions once and both original sessions agree",
                       read(a) == read(b) == read() == accepted == os_state
                       and accepted["revision"] == state["revision"] + 1
                       and accepted["tasks"][0]["revision"] == packet["taskRevision"] + 1
                       and accepted["tasks"][0]["owner"] == "reviewer"
                       and accepted["tasks"][0]["harness"] == "claude"
                       and accepted["tasks"][0]["status"] == "pending")
        confirmations = [event for event in accepted["events"] if event["kind"] == "handoff.accepted"]
        evidence.check("same original request returns the same receipt and leaves exact state bytes unchanged",
                       replay == accepted and state_path.read_bytes() == before_replay
                       and len(confirmations) == 1
                       and confirmations[0]["detail"]["handoffId"] == packet["id"]
                       and confirmations[0]["detail"]["expectedRevision"] == request["expectedRevision"]
                       and bool(accepted["handoffs"][-1].get("acceptedAt")))
        evidence.report["acceptance"] = {"receipt": accepted["handoffs"][-1], "event": confirmations[0],
                                          "stateSha256": sha(before_replay)}
    evidence.check("private reference remains a reference with its original file mode and content",
                   packet["memoryRefs"] == [private_ref] and not (destination / private_ref).exists()
                   and stat.S_IMODE(private_path.stat().st_mode) == 0o600
                   and hash_file(private_path) == private_digest
                   and stat.S_IMODE(state_path.stat().st_mode) == 0o600)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=("LOCAL-001", "LOCAL-002", "LOCAL-003"), required=True)
    parser.add_argument("--negative-only", action="store_true")
    parser.add_argument("--pack-cli", required=True)
    parser.add_argument("--headless", required=True)
    parser.add_argument("--sync-cli", required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30, help="maximum seconds per observation")
    args = parser.parse_args()
    report = {"schemaVersion": "1", "case": args.case, "negativeOnly": args.negative_only,
              "timeoutSeconds": args.timeout, "started": stamp(), "passed": False, "result": "FAIL",
              "assertions": [], "commands": [], "processes": [], "publicFixtures": [], "fixtureManifests": [],
              "exchanges": [], "directSocketReads": [], "relayedSocketRequests": [], "incompatibleRequests": []}
    scratch = None
    evidence = None
    try:
        if args.timeout <= 0:
            raise ValueError("timeout must be positive")
        if sys.version_info < (3, 9):
            raise Blocked("Python 3.9 or newer is required")
        if sys.platform not in ("darwin", "linux"):
            raise Blocked("Unix sockets with peer PID inspection require Darwin or Linux")
        args.git = executable("git")
        if args.case == "LOCAL-003":
            args.node = executable("node")
            bins = {"node": args.node, "git": args.git}
        else:
            for name in ("pack_cli", "headless", "sync_cli"):
                setattr(args, name, executable(getattr(args, name)))
            bins = {name: getattr(args, name) for name in ("pack_cli", "headless", "sync_cli", "git")}
        report["binaries"] = {name: {"path": value, "sha256": hash_file(value)} for name, value in bins.items()}
        source_paths = [SCRIPT, SPEC / "spec.md", SPEC / "verification.md"]
        if args.case == "LOCAL-003":
            source_paths.extend(TEAM_CLI.parent / file for file in ("cli.mjs", "state.mjs", "handoff.mjs"))
        else:
            cli_source = PACK / "tools/prometheus-cli/crates/prometheus-cli/src"
            source_paths.extend((cli_source / "main.rs", cli_source / "commands/kbd.rs",
                                 cli_source / "commands/control_transport.rs"))
        report["sourceDigests"] = {str(path.relative_to(PACK)): hash_file(path) for path in source_paths}
        # Keep paths below the Darwin Unix-domain sockaddr limit.
        with tempfile.TemporaryDirectory(prefix="kbd-live-", dir="/tmp") as directory:
            scratch = Path(directory)
            with ExitStack() as stack:
                evidence = Evidence(report, args.timeout, stack)
                report["processes"] = evidence.processes
                if args.case == "LOCAL-001":
                    live_exchange(scratch / "l", args, evidence, False, args.negative_only)
                    live_exchange(scratch / "c", args, evidence, True, args.negative_only)
                elif args.case == "LOCAL-002":
                    runner = refusal_controls if args.negative_only else fallback_and_claim
                    runner(scratch, args, evidence)
                else:
                    version = evidence.command([args.node, "--version"], scratch,
                                               isolated_env(scratch / "node"), 0)["stdout"].strip()
                    if int(version.removeprefix("v").split(".")[0]) < 22:
                        raise Blocked(f"Node.js 22 or newer required, found {version}")
                    team_case(scratch, args, evidence, args.negative_only)
        report["result"] = "PASS"
    except Blocked as error:
        report.update({"result": "BLOCKED", "error": str(error)})
    except Exception as error:
        # The failure receipt is always retained, including partial assertions and cleanup.
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        live = evidence.observe_cleanup() if evidence is not None else []
        cleanup = {"scratchRemoved": scratch is None or not scratch.exists(), "survivingFixturePids": live}
        report["cleanup"] = cleanup
        if live or not cleanup["scratchRemoved"]:
            report["result"] = "FAIL"
            report["error"] = "fixture cleanup failed"
        report["assertionCount"] = len(report["assertions"])
        report["fixtureManifestSha256"] = sha(encode(report["fixtureManifests"]))
        report["finished"] = stamp()
        report["passed"] = report["result"] == "PASS" and report["assertionCount"] > 0
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"case": args.case, "result": report["result"], "report": str(args.report),
                      "assertionCount": report["assertionCount"]}))
    return 0 if report["passed"] else 2 if report["result"] == "BLOCKED" else 1


if __name__ == "__main__":
    if sys.argv[1:] == ["--session-worker"]:
        session_worker()
    else:
        raise SystemExit(main())
