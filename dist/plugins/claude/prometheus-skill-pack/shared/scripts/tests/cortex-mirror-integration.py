#!/usr/bin/env python3
"""Production writers, real Cortex, and OS-owned cross-process capacity."""
import argparse
import concurrent.futures
import fcntl
import hashlib
import json
import os
from pathlib import Path
import selectors
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time


class Blocked(Exception):
    pass


def absolute(raw):
    if not Path(raw).is_absolute():
        raise argparse.ArgumentTypeError("explicit absolute path required")
    return Path(raw).resolve()


def main():
    ap = argparse.ArgumentParser()
    for flag in ("full", "scratch", "evidence"):
        ap.add_argument("--"+flag, type=absolute, required=True)
    ap.add_argument("--cortex-package", type=absolute)
    ap.add_argument("--expected-version", default="2.0.3")
    args = ap.parse_args()
    args.scratch.mkdir(parents=True, exist_ok=True)
    args.evidence.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="cortex-", dir=args.scratch))
    writer = args.full / "shared/scripts/lib/learning_write.py"
    env = {"PATH":os.environ.get("PATH", ""), "HOME":str(work/"home"),
           "CODEX_HOME":str(work/"codex"), "CORTEX_DATA_DIR":str(work/"cortex-data"),
           "PROMETHEUS_PLUGIN_ROOT":str(args.full), "PYTHONDONTWRITEBYTECODE":"1",
           "PROMETHEUS_LEARNING_QUEUE":str(work/"queue"),
           "PROMETHEUS_LEARNING_LOG_DIR":str(work/"log"),
           "PROMETHEUS_LEARNING_INDEX_DIR":str(work/"index"),
           "PROMETHEUS_PROJECT_ID_SKIP_RUNTIME":"1", "PROMETHEUS_TEAM_DIGEST":"0",
           "PROMETHEUS_LEARNING_PK":"0", "HF_HUB_OFFLINE":"1",
           "TRANSFORMERS_OFFLINE":"1", "TRANSFORMERS_CACHE":str(work/"model-cache"),
           "HF_HOME":str(work/"hf"), "TMPDIR":str(work/"tmp")}
    for key in ("HOME", "CODEX_HOME", "CORTEX_DATA_DIR", "TMPDIR"):
        Path(env[key]).mkdir(parents=True, exist_ok=True)
    repo = work/"project"
    (repo/".prometheus").mkdir(parents=True)
    (repo/".prometheus/project.json").write_text('{"projectId":"project:ldd-cortex"}\n')
    pool = work/"queue/cortex-feeders"
    results, started, code = [], time.time(), 0

    def write(text, overrides=None):
        result = subprocess.run([sys.executable,"-B",str(writer),"--cwd",str(repo),
                                 "--text",text,"--visibility","project"],
                                env=dict(env,**(overrides or {})), cwd=repo,
                                capture_output=True, text=True, timeout=30)
        assert result.returncode == 0 and not result.stderr.strip(), result.stderr
        summary = json.loads(result.stdout)
        assert summary["written"] >= 1, summary
        assert list((work/"queue").rglob("*.json")), "durable outbox lost"
        assert (work/"log/lessons.jsonl").is_file(), "backup log lost"
        return summary

    def owners():
        slots = list(pool.glob("slot-*.lock"))
        if not slots or not shutil.which("lsof"):
            return {}
        result = subprocess.run(["lsof","-Fpc","--",*map(str,slots)],
                                capture_output=True,text=True,timeout=10)
        found, pid = {}, None
        for line in result.stdout.splitlines():
            if line.startswith("p"):
                pid = int(line[1:]); found[pid] = ""
            elif line.startswith("c") and pid is not None:
                found[pid] = line[1:]
        return found

    def live_slots():
        count = 0
        for slot in pool.glob("slot-*.lock"):
            with slot.open("r+") as stream:
                try:
                    fcntl.flock(stream,fcntl.LOCK_EX|fcntl.LOCK_NB)
                except BlockingIOError:
                    count += 1
        return count

    def drain():
        deadline = time.monotonic()+190
        while time.monotonic() < deadline:
            if not live_slots():
                return
            time.sleep(0.1)
        raise AssertionError("real server retained capacity beyond teardown deadline")

    def recall(argv, nonce):
        proc = subprocess.Popen(argv,stdin=subprocess.PIPE,stdout=subprocess.PIPE,
                                stderr=subprocess.DEVNULL,env=env,cwd=repo)
        selector = selectors.DefaultSelector()
        selector.register(proc.stdout,selectors.EVENT_READ)
        pending = b""
        try:
            requests = [{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}},
                        {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
                            "name":"cortex_recall","arguments":{"query":nonce,
                            "projectId":"project:ldd-cortex","limit":10}}}]
            proc.stdin.write(("\n".join(map(json.dumps,requests))+"\n").encode())
            proc.stdin.flush()
            deadline = time.monotonic()+120
            while time.monotonic() < deadline:
                for key,_ in selector.select(min(1,max(0,deadline-time.monotonic()))):
                    chunk = os.read(key.fileobj.fileno(),65536)
                    assert chunk, "real Cortex closed before reply"
                    pending += chunk
                    lines = pending.split(b"\n"); pending = lines.pop()
                    for line in lines:
                        try:
                            reply = json.loads(line)
                        except ValueError:
                            continue
                        if reply.get("id") == 2:
                            assert "error" not in reply, reply
                            assert nonce in json.dumps(reply.get("result")), "mirrored lesson absent from actual recall"
                            return
            raise AssertionError("real Cortex recall timed out")
        finally:
            selector.close(); proc.stdin.close()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait()

    try:
        absent = write("absent-"+str(started))
        assert absent["cortex_mirror"]["status"] == "absent", absent
        disabled = write("disabled-"+str(started),{"PROMETHEUS_CORTEX_MAX_FEEDERS":"0"})
        assert disabled["cortex_mirror"]["status"] == "disabled", disabled
        results.append({"case":"durable-primary-absent-disabled","status":"PASS"})
        if not args.cortex_package:
            raise Blocked("explicit real Cortex package required; no installed/live-home fallback")
        package = args.cortex_package
        metadata = json.loads((package/"package.json").read_text())
        server = package/"dist/mcp-server.js"
        cache = package/"node_modules/@xenova/transformers/.cache"
        if metadata.get("version") != args.expected_version or not server.is_file() or not cache.is_dir():
            raise Blocked("recorded Cortex version, actual entrypoint or cached model unavailable")
        node = shutil.which("node")
        if not node or not shutil.which("lsof"):
            raise Blocked("node/lsof missing for real process evidence")
        # The real package may itself cache models beside node_modules. Execute
        # an exact private copy so its cache strategy cannot mutate installation.
        for entry in package.rglob('*'):
            if entry.is_symlink() and not entry.resolve().is_relative_to(package.resolve()):
                raise Blocked("Cortex package has an escaping symlink")
        private_package=work/'package'
        shutil.copytree(package,private_package,symlinks=True,ignore=shutil.ignore_patterns('.git'))
        shutil.copytree(cache,work/"model-cache",dirs_exist_ok=True)
        argv = [node,str(private_package/'dist/mcp-server.js')]
        env["PROMETHEUS_CORTEX_MCP"] = shlex.join(argv)
        nonce = "ldd-real-"+str(time.time_ns())
        assert write(nonce)["cortex_mirror"]["status"] == "accepted"
        drain(); recall(argv,nonce)
        results.append({"case":"actual-mirror-recall","status":"PASS",
                        "serverSha256":hashlib.sha256(server.read_bytes()).hexdigest()})
        with concurrent.futures.ThreadPoolExecutor(max_workers=12) as workers:
            futures = [workers.submit(write,"pressure-%d-%s"%(i,nonce)) for i in range(12)]
            maximum = 0
            while not all(f.done() for f in futures):
                maximum = max(maximum,live_slots())
                assert maximum <= 4, "actual live feeder/server capacity exceeded four"
                time.sleep(0.02)
            summaries = [f.result() for f in futures]
        assert any(s["cortex_mirror"]["status"] == "saturated" for s in summaries), "real pressure did not exercise saturation"
        drain()
        results.append({"case":"real-concurrent-capacity-durable-saturation","status":"PASS","maximumLiveSlots":maximum})
        with concurrent.futures.ThreadPoolExecutor(max_workers=1) as workers:
            future = workers.submit(write,"crash-"+nonce,{"PROMETHEUS_CORTEX_MAX_FEEDERS":"1"})
            held_node = held_feeder = None
            deadline = time.monotonic()+10
            while time.monotonic()<deadline:
                for pid,command in owners().items():
                    if "node" in command:
                        held_node = pid
                    elif "python" in command.lower():
                        held_feeder = pid
                if held_node and held_feeder:
                    break
                time.sleep(0.02)
            if not held_node or not held_feeder:
                raise Blocked("actual server completed before crash ownership was observed")
            current_owners=owners()
            if "node" not in current_owners.get(held_node,"") or "python" not in current_owners.get(held_feeder,"").lower():
                raise Blocked("owned lease holders changed before crash injection")
            os.kill(held_node,signal.SIGSTOP)
            os.kill(held_feeder,signal.SIGKILL)
            future.result()
            assert live_slots() == 1, "live actual child lost inherited capacity on feeder crash"
            saturated = write("post-crash-"+nonce,{"PROMETHEUS_CORTEX_MAX_FEEDERS":"1"})
            assert saturated["cortex_mirror"]["status"] == "saturated", saturated
            os.kill(held_node,signal.SIGCONT)
        drain()
        assert write("recovered-"+nonce,{"PROMETHEUS_CORTEX_MAX_FEEDERS":"1"})["cortex_mirror"]["status"] == "accepted"
        drain()
        results.append({"case":"real-child-crash-lease-retention-recovery","status":"PASS"})
        invalid = write("invalid-"+nonce,{"PROMETHEUS_CORTEX_MAX_FEEDERS":"invalid"})
        assert invalid["cortex_mirror"]["limit"] == 4 and invalid["cortex_mirror"].get("diagnostic"), invalid
        drain()
    except Blocked as error:
        code = 2; results.append({"status":"BLOCKED","reason":str(error)})
    except Exception as error:
        code = 1; results.append({"status":"FAIL","reason":str(error)})
    finally:
        # Only holders of FDs into this private owned lease pool are signalled.
        for pid in owners():
            if pid != os.getpid():
                try:
                    os.kill(pid,signal.SIGCONT); os.kill(pid,signal.SIGTERM)
                except ProcessLookupError:
                    pass
        deadline = time.monotonic()+6
        while owners() and time.monotonic()<deadline:
            time.sleep(0.1)
        for pid in owners():
            if pid != os.getpid():
                try:
                    os.kill(pid,signal.SIGKILL)
                except ProcessLookupError:
                    pass
        receipt = {"schemaVersion":1,"exitCode":code,"status":{0:"PASS",1:"FAIL",2:"BLOCKED"}[code],
                   "scratch":str(work),"startedUnix":started,"endedUnix":time.time(),"results":results,
                   "isolation":"scratch homes/data/queue/model-cache; readonly actual executable/model inputs; no real-home assertions"}
        (args.evidence/"cortex-mirror.json").write_text(json.dumps(receipt,indent=2)+"\n")
        print(json.dumps(receipt)); shutil.rmtree(work)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
