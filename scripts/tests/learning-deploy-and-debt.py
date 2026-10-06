#!/usr/bin/env python3
"""One local batch, real production entrypoints, explicit private runtime inputs.

The approved initial stage certifies actual candidates, not installable final
parent pins. Final-stage publication/install graph evidence is a separate gate.
"""
import argparse
from datetime import datetime, timezone
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


class Blocked(Exception):
    pass


def absolute(raw):
    if not Path(raw).is_absolute():
        raise argparse.ArgumentTypeError("explicit absolute paths required")
    return Path(raw).resolve()


def utc():
    return datetime.now(timezone.utc).isoformat()


class Links(HTMLParser):
    def __init__(self):
        super().__init__(); self.links = []; self.ids = set()
    def handle_starttag(self, tag, attributes):
        self.ids.update(value for key,value in attributes if key == "id" and value)
        if tag == "a":
            self.links.extend(value for key,value in attributes if key == "href" and value)


def main():
    ap = argparse.ArgumentParser()
    for key in ("full", "mini", "surreal", "companion", "evidence", "inputs"):
        ap.add_argument("--"+key, type=absolute, required=True)
    ap.add_argument("--stage", choices=("initial", "final"), default="initial")
    ap.add_argument("--no-build", action="store_true", help="explicitly supplied prebuilt candidate artifacts only")
    ap.add_argument("--cases", help="comma-separated affected case names; partial confirmation never certifies a release")
    args = ap.parse_args()
    selected = set(args.cases.split(",")) if args.cases else None
    for root in (args.full,args.mini,args.surreal,args.companion):
        if not root.is_dir():
            ap.error("candidate directory unavailable: "+str(root))
    inputs = json.loads(args.inputs.read_text())
    args.evidence.mkdir(parents=True,exist_ok=True)
    evidence = args.evidence / (args.stage+"-"+datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
    evidence.mkdir()
    # macOS's default TMPDIR is too long for real Unix control socket paths.
    scratch = Path(tempfile.mkdtemp(prefix="ldd-local-",dir="/tmp")).resolve()
    env = {"PATH":os.environ.get("PATH",""), "HOME":str(scratch/"home"),
           "CODEX_HOME":str(scratch/"codex"), "CORTEX_DATA_DIR":str(scratch/"cortex"),
           "CLAUDE_CONFIG_DIR":str(scratch/"claude"), "PROMETHEUS_PLUGIN_ROOT":str(scratch/"plugins"),
           "PROMETHEUS_LEARNING_QUEUE":str(scratch/"queue"),
           "PROMETHEUS_LEARNING_LOG_DIR":str(scratch/"learning-log"),
           "PROMETHEUS_LEARNING_INDEX_DIR":str(scratch/"learning-index"),
           "PROMETHEUS_TEAM_DIGEST_DIR":str(scratch/"digest"),
           "PROMETHEUS_LEARNING_CORTEX":"0", "PROMETHEUS_LEARNING_PK":"0",
           "PROMETHEUS_PROJECT_ID_SKIP_RUNTIME":"1", "PROMETHEUS_DATA_DIR":str(scratch/"data"),
           "XDG_CONFIG_HOME":str(scratch/"config"), "XDG_DATA_HOME":str(scratch/"data"),
           "XDG_CACHE_HOME":str(scratch/"cache"), "TMPDIR":str(scratch/"tmp"),
           "PYTHONDONTWRITEBYTECODE":"1", "GIT_CONFIG_GLOBAL":os.devnull,
           "GIT_CONFIG_NOSYSTEM":"1", "HF_HUB_OFFLINE":"1", "TRANSFORMERS_OFFLINE":"1"}
    for key in ("HOME","CODEX_HOME","CORTEX_DATA_DIR","CLAUDE_CONFIG_DIR","TMPDIR"):
        Path(env[key]).mkdir(parents=True,exist_ok=True)
    records, started = [], utc()

    def provenance(root, seen=None):
        seen = set() if seen is None else seen
        if root in seen:
            raise AssertionError("cyclic candidate Git graph: "+str(root))
        seen = seen | {root}
        def git(*words):
            return subprocess.check_output(["git","-C",str(root),*words],text=True).strip()
        rows = []
        files = subprocess.check_output(["git","-C",str(root),"ls-files","--cached","--others","--exclude-standard","-z"]).decode().split("\0")
        for name in sorted(set(filter(None,files))):
            file = root/name
            # Exclude only this invocation's newly owned evidence. Every other
            # tracked or untracked candidate byte remains source-bound.
            if file == evidence or evidence in file.parents:
                continue
            try:
                mode = file.lstat().st_mode
            except FileNotFoundError:
                rows.append([name,"deleted"]); continue
            if stat.S_ISDIR(mode):
                child_top = subprocess.check_output(["git","-C",str(file),"rev-parse","--show-toplevel"],text=True).strip()
                if Path(child_top).resolve() != file.resolve():
                    rows.append([name,"uninitialized-gitlink"]); continue
                child = provenance(file.resolve(), seen)
                rows.append([name,"gitlink",child]); continue
            data = os.readlink(file).encode() if stat.S_ISLNK(mode) else file.read_bytes()
            rows.append([name,stat.S_IMODE(mode),hashlib.sha256(data).hexdigest()])
        return {"path":str(root),"head":git("rev-parse","HEAD"),
                "trackedDiffSha256":hashlib.sha256(subprocess.check_output(["git","-C",str(root),"diff","HEAD","--binary"])).hexdigest(),
                "sourceBytesAndModesSha256":hashlib.sha256(json.dumps(rows).encode()).hexdigest(),
                "sourceEntryCount":len(rows)}

    def record(name, code, **data):
        row = {"name":name,"exitCode":code,"status":{0:"PASS",1:"FAIL",2:"BLOCKED"}.get(code,"FAIL"),**data}
        records.append(row); print(json.dumps(row),flush=True)
        return row

    def run(name, argv, cwd=None, extra=None, timeout=1800):
        if selected is not None and name not in selected:
            return {"name":name,"exitCode":None,"status":"NOT_REQUESTED"}
        began = utc()
        stdout = evidence/(name+".stdout"); stderr = evidence/(name+".stderr")
        command = list(map(str,argv))
        with stdout.open("w") as out,stderr.open("w") as err:
            proc = subprocess.Popen(command,cwd=cwd or scratch,env=dict(env,**(extra or {})),
                                    stdout=out,stderr=err,start_new_session=True)
            try:
                code = proc.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid,signal.SIGTERM)
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid,signal.SIGKILL); proc.wait()
                code = 1; err.write("\nFAIL: bounded production entrypoint deadline exceeded\n")
        row = record(name,code,argv=command,cwd=str(cwd or scratch),startedAt=began,endedAt=utc(),
                     stdout=str(stdout),stderr=str(stderr),scratch=str(scratch))
        return row

    def path_input(name):
        value = inputs.get(name)
        if not value or not Path(value).is_absolute() or not Path(value).exists():
            raise Blocked("explicit existing absolute input missing: "+name)
        return Path(value).resolve()

    def cargo(name, words, cwd):
        if subprocess.run(["pgrep","-x","cargo"],stdout=subprocess.DEVNULL).returncode == 0 or subprocess.run(["pgrep","-x","rustc"],stdout=subprocess.DEVNULL).returncode == 0:
            record(name,2,reason="another Cargo/rustc process owns the machine")
            return False
        build_env = {"CARGO_HOME":str(path_input("cargoHome")),"RUSTUP_HOME":str(path_input("rustupHome")),
                     "RUSTUP_TOOLCHAIN":inputs.get("rustToolchain","1.99.0")}
        if inputs.get("sccacheBinary"):
            build_env["RUSTC_WRAPPER"] = str(path_input("sccacheBinary"))
        if selected is not None:
            selected.add(name)
        return run(name,["cargo",*words],cwd,build_env,14400)["exitCode"] == 0

    def guarded(name, action):
        try:
            action()
        except Blocked as error:
            record(name,2,reason=str(error))
        except Exception as error:
            record(name,1,reason=str(error))

    def endpoint_port():
        with socket.socket() as sock:
            sock.bind(("127.0.0.1",0)); port=sock.getsockname()[1]
        assert port != 23001
        return port

    def site(name, root):
        if selected is not None and name+"-production-build" not in selected:
            return
        site_root = root/"site"
        binary = site_root/"node_modules/@docusaurus/core/bin/docusaurus.mjs"
        if not binary.is_file():
            raise Blocked("locked Docusaurus installation missing: "+str(site_root))
        output = scratch/(name+"-site")
        if name == "mini":
            generated = run(name+"-catalog-generation",["node",site_root/"scripts/generate-skills-catalog.mjs"],site_root,timeout=1800)
            if generated["exitCode"]:
                return
        result = run(name+"-production-build",["node",binary,"build","--out-dir",output],site_root,timeout=1800)
        if result["exitCode"]:
            return
        port = endpoint_port()
        log = (evidence/(name+"-serve.log")).open("w")
        proc = subprocess.Popen(["node",str(binary),"serve","--dir",str(output),"--host","127.0.0.1",
                                 "--port",str(port),"--no-open"],cwd=site_root,env=env,
                                stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        try:
            pages = sorted(output.rglob("*.html"))
            if not pages:
                raise AssertionError("production build emitted no pages")
            # Derive baseUrl from emitted sitemap, not an invented localhost path.
            sitemap = output/"sitemap.xml"
            base = "/"
            if sitemap.is_file():
                import xml.etree.ElementTree as ET
                urls = [urllib.parse.urlparse(n.text).path for n in ET.parse(sitemap).getroot().iter() if n.tag.endswith("loc") and n.text]
                if urls:
                    base = min(urls,key=len)
                    if not base.endswith("/"):
                        base = base.rsplit("/",1)[0]+"/"
            host = "http://127.0.0.1:"+str(port)
            deadline = time.monotonic()+60
            while time.monotonic()<deadline:
                try:
                    with urllib.request.urlopen(host+base,timeout=2) as response:
                        if response.status == 200:
                            break
                except (OSError,urllib.error.URLError):
                    if proc.poll() is not None:
                        raise AssertionError("actual production site server exited")
                    time.sleep(0.2)
            else:
                raise AssertionError("actual production site server never became ready")
            checked, links, texts = [], set(), []
            for file in pages:
                rel = file.relative_to(output).as_posix()
                route = base+rel
                with urllib.request.urlopen(host+route,timeout=10) as response:
                    assert response.status == 200, route
                    body = response.read().decode("utf-8")
                parsed = Links(); parsed.feed(body)
                texts.append(body); checked.append(route)
                for link in parsed.links:
                    url = urllib.parse.urlparse(urllib.parse.urljoin(host+route,link))
                    if url.netloc == "127.0.0.1:"+str(port):
                        links.add((url.path,urllib.parse.unquote(url.fragment)))
            for route,fragment in sorted(links):
                with urllib.request.urlopen(host+route,timeout=10) as response:
                    assert response.status == 200, route
                    body = response.read().decode("utf-8")
                if fragment:
                    target = Links(); target.feed(body)
                    assert fragment in target.ids, "missing served anchor: "+route+"#"+fragment
            all_text = "\n".join(texts).lower()
            for topic in ("agent", "team", "model", "handoff", "companion"):
                assert topic in all_text, "new product topic absent: "+topic
            record(name+"-served-production-navigation",0,pageCount=len(checked),internalLinkCount=len(links),
                   routes=checked,baseUrl=base,endpoint=host)
        finally:
            os.killpg(proc.pid,signal.SIGTERM)
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid,signal.SIGKILL); proc.wait()
            log.close()

    source = {name:provenance(root) for name,root in (("full",args.full),("mini",args.mini),("surreal",args.surreal),("companion",args.companion))}
    artifacts = {}
    inputs["memorySourceId"] = source["surreal"]["sourceBytesAndModesSha256"]
    inputs["miniSourceId"] = source["mini"]["sourceBytesAndModesSha256"]
    try:
        if not args.no_build:
            builds = [
                ("build-native-cli",args.full/"tools/prometheus-cli",["build","--locked","-p","prometheus-cli","--bin","prometheus"],"doctorBinary",args.full/"tools/prometheus-cli/target/debug/prometheus"),
                ("build-hook-runtime",args.full/"crates/prometheus-hook",["build","--locked","--bin","prometheus-hook"],"hookRuntimeBinary",args.full/"crates/prometheus-hook/target/debug/prometheus-hook"),
                ("build-memory-candidate",args.surreal,["build","--locked","--no-default-features","--features","server-only,local-embeddings","--bin","surreal-memory-server"],"memoryBinary",args.surreal/"target/debug/surreal-memory-server"),
                ("build-memory-prior",path_input("priorMemorySource"),["build","--locked","--no-default-features","--features","server-only,local-embeddings","--bin","surreal-memory-server"],"priorMemoryBinary",path_input("priorMemorySource")/"target/debug/surreal-memory-server"),
                ("build-companion-hosts",args.companion,["build","--locked","-p","prometheus-substrate","-p","sovereign-sync","--no-default-features","--features","prometheus-substrate/sovereign","--bins"],"companionHeadlessBinary",args.companion/"target/debug/prometheus-companion-headless")]
            for name,cwd,words,key,binary in builds:
                guarded(name,lambda name=name,cwd=cwd,words=words,key=key,binary=binary:
                        artifacts.update({key:str(binary)}) if cargo(name,words,cwd) else None)
            inputs.update(artifacts)
            inputs["companionSyncBinary"] = str(args.companion/"target/debug/sovereign-sync")
            for name,cwd,words,target,key in [
                ("build-cache-integration",args.surreal,["test","--locked","--no-default-features","--features","server-only,local-embeddings","--test","query_cache_production","--no-run","--message-format=json"],"query_cache_production","cacheIntegrationBinary"),
                ("build-companion-integration",args.companion,["test","--locked","-p","prometheus-substrate","--no-default-features","--features","sovereign","--test","connected_headless_processes","--no-run","--message-format=json"],"connected_headless_processes","companionIntegrationBinary")]:
                def compile_target(name=name,cwd=cwd,words=words,target=target,key=key):
                    if not cargo(name,words,cwd):
                        return
                    for line in (evidence/(name+".stdout")).read_text().splitlines():
                        try:
                            row = json.loads(line)
                        except ValueError:
                            continue
                        if row.get("reason") == "compiler-artifact" and row.get("target",{}).get("name") == target and row.get("executable"):
                            inputs[key] = row["executable"]
                    if not inputs.get(key):
                        raise AssertionError("Cargo did not emit explicit integration executable")
                guarded(name,compile_target)
        # Supplemental drift checks are direct production commands, not legacy
        # package aliases that transitively add module/unit suites.
        for name,root,words in [
            ("full-distribution",args.full,["node","scripts/generate-skill-system-distribution.js","--check"]),
            ("full-harnesses",args.full,["node","scripts/check-harness-adapters.js"]),
            ("full-release-version",args.full,["node","scripts/check-release-version-matrix.mjs"]),
            ("full-workflow-policy",args.full,["node","scripts/check-workflow-policy.mjs"]),
            ("full-docs-sync",args.full,["node","scripts/docs-sync.mjs","--check"]),
            ("mini-distribution",args.mini,["node","scripts/generate-skill-system-distribution.mjs","--check"])]:
            run(name,words,root)
        prep = scratch/"install-fixture"; prep.mkdir()
        install_source, legacy = prep/"candidate",prep/"legacy"
        prepared = run("prepare-install-source",["node",args.full/"scripts/tests/prepare-learning-deploy-candidate.mjs",
            "--full",args.full,"--scratch",prep,"--output",install_source,"--legacy-output",legacy],args.full)
        def protected_candidates():
            if prepared["exitCode"]:
                raise Blocked("private committed candidate requires scratch-contained prepared full graph")
            run("initial-committed-protected-integrity",["node",args.full/"scripts/tests/certify-initial-protected-candidate.mjs",
                "--full",args.full,"--mini",args.mini,"--surreal",args.surreal,"--companion",args.companion,
                "--prepared-full",install_source,"--scratch",scratch,"--evidence",evidence],timeout=1800)
        guarded("protected-candidate-prerequisites",protected_candidates)
        common = ["--full",args.full,"--scratch",scratch,"--evidence",evidence]
        def hook_cases():
            run("hook-bytecode",["bash",args.full/"scripts/tests/test-hook-bytecode-integration.sh",*common,
                "--hook-runtime-bin",path_input("hookRuntimeBinary"),"--legacy-payload",legacy])
        guarded("hook-bytecode-prerequisites",hook_cases)
        def installer_cases():
            if prepared["exitCode"]:
                raise Blocked("scratch-contained actual installer source preparation failed")
            for name in ("codex-memory","codex-home"):
                command = ["bash",args.full/("scripts/tests/test-"+name+"-integration.sh"),*common,
                           "--doctor-bin",path_input("doctorBinary"),"--install-source",install_source]
                if name == "codex-memory" and inputs.get("codexBinary"):
                    command += ["--codex-bin",path_input("codexBinary")]
                    if inputs.get("codexProviderConfig"):
                        command += ["--codex-provider-config",path_input("codexProviderConfig")]
                run(name,command,timeout=1200)
        guarded("installer-prerequisites",installer_cases)
        run("partition",["bash",args.full/"scripts/tests/test-memory-partition.sh"],args.full)
        # Real authored merge/assembler processes over private filesystem
        # packages; no substitute workers, model fixtures or driver mocks.
        for case in ("merge-threads", "report-assembly"):
            run("research-"+case,["bash",args.full/("skills/research/deep-research/tests/"+case+".sh")],args.full)
        run("rebase-generated-ownership",["bash",args.full/"scripts/tests/test-rebase-regenerate.sh"],args.full)
        cortex = ["bash",args.full/"shared/scripts/tests/test-cortex-mirror.sh",*common]
        if inputs.get("cortexPackage"):
            cortex += ["--cortex-package",path_input("cortexPackage")]
        run("real-cortex",cortex,timeout=900)
        def mini_cases():
            extra = {"LDD_MINI_ROOT":str(args.mini),"LDD_SCRATCH_ROOT":str(scratch),
                     "LDD_EVIDENCE_DIR":str(evidence),"LDD_MINI_SOURCE_ID":inputs["miniSourceId"],"LDD_MINI_HISTORICAL_BASELINE":str(path_input("miniHistoricalBaseline"))}
            run("mini-production",["node",args.mini/"scripts/tests/learning-deploy-integration.test.mjs"],extra=extra)
        guarded("mini-prerequisites",mini_cases)
        def cache_cases():
            extra = {"LDD_SERVER_BIN":str(path_input("memoryBinary")),"LDD_PRIOR_SERVER_BIN":str(path_input("priorMemoryBinary")),
                     "LDD_SERVER_SOURCE_ID":inputs["memorySourceId"],"LDD_PRIOR_SOURCE_ID":inputs["priorMemorySourceId"],
                     "LDD_MLX_EXECUTOR_BIN":str(path_input("mlxExecutorBinary")),"LDD_MODEL_SNAPSHOT":str(path_input("modelSnapshot")),
                     "LDD_DOCKER_BIN":str(path_input("dockerBinary")),"LDD_SURREAL_IMAGE":inputs["surrealImage"],
                     "LDD_SCRATCH_ROOT":str(scratch),"LDD_EVIDENCE_DIR":str(evidence)}
            if inputs.get("dockerHost"):
                extra["LDD_DOCKER_HOST"]=inputs["dockerHost"]
            extra.update({"LDD_TEAM_SCENARIO_BIN":str(args.full/"scripts/tests/test-team-control-integration.mjs"),
                          "LDD_TEAM_NODE_BIN":shutil.which("node"),
                          "LDD_TEAM_INSTALLED_ROOT":str(args.full/"dist/plugins/codex/prometheus-skill-pack"),
                          "LDD_TEAM_RUNTIME_BIN":str(path_input("doctorBinary")),
                          "LDD_TEAM_CONTROL_HOST_BIN":str(path_input("companionHeadlessBinary")),
                          "LDD_TEAM_MODEL_INPUT":str(path_input("teamModelInput"))})
            run("real-cache-services",[path_input("cacheIntegrationBinary"),"--test-threads=1","--nocapture"],extra=extra,timeout=3600)
        guarded("cache-prerequisites",cache_cases)
        def companion_cases():
            run("real-companion-connected",["bash",args.companion/"scripts/tests/test-companion-connected-integration.sh",
                "--companion",args.companion,"--scratch",scratch,"--evidence",evidence,
                "--headless-bin",path_input("companionHeadlessBinary"),"--sync-bin",path_input("companionSyncBinary"),
                "--prometheus-bin",path_input("doctorBinary"),"--companion-integration-bin",path_input("companionIntegrationBinary")],timeout=1200)
        guarded("companion-prerequisites",companion_cases)
        guarded("full-site-prerequisites",lambda:site("full",args.full))
        guarded("mini-site-prerequisites",lambda:site("mini",args.mini))
        guarded("materialization-prerequisites",lambda:run("generated-byte-mode-certification",
            ["node",args.full/"scripts/tests/certify-generated-materializations.mjs",
             "--manifest",path_input("generatedMaterializations"),"--scratch",scratch,"--evidence",evidence]))
        if args.stage == "initial":
            records.append({"name":"final-parent-remote-install-graph","status":"PENDING",
                            "reason":"owner-approved dependency certification/merge precedes final exact parent pin approval and fresh-clone certification"})
        else:
            def final_graph():
                manifest=path_input("finalRemoteGraph")
                token_name=json.loads(manifest.read_text()).get("sourceReadTokenEnv")
                extra={token_name:os.environ[token_name]} if token_name and token_name.startswith("LDD_") and token_name in os.environ else {}
                run("final-remote-install-graph",[sys.executable,"-B",args.full/"scripts/tests/certify-remote-install-graph.py",
                    "--manifest",manifest,"--scratch",scratch,"--evidence",evidence,
                    "--doctor-bin",path_input("doctorBinary")],extra=extra,timeout=4800)
            guarded("final-graph-prerequisites",final_graph)
    except Blocked as error:
        record("batch-prerequisites",2,reason=str(error))
    except Exception as error:
        record("batch-failure",1,reason=str(error))
    finally:
        ending = {name:provenance(root) for name,root in (("full",args.full),("mini",args.mini),("surreal",args.surreal),("companion",args.companion))}
        if source != ending:
            record("source-binding",1,reason="candidate source changed during gate; affected evidence invalid")
        if selected is not None:
            missing = selected - {row["name"] for row in records}
            if missing:
                record("requested-case-coverage",2,reason="requested cases did not execute",missing=sorted(missing))
        result = 1 if any(r.get("exitCode",0) not in (0,2) for r in records) else 2 if any(r.get("exitCode") == 2 for r in records) else 0
        receipt = {"schemaVersion":1,"stage":args.stage,"startedAt":started,"endedAt":utc(),"exitCode":result,
                   "source":source,"finalSource":ending,"records":records,"scratch":str(scratch),
                   "acceptanceScope":"affected cases only; no release certification" if selected is not None else "actual candidate initial integration only" if args.stage=="initial" else "final graph",
                   "requestedCases":sorted(selected) if selected is not None else "all",
                   "publication":False,"deployment":False,"scratchCleanup":"owned runtime files removed after cases"}
        (evidence/"batch.json").write_text(json.dumps(receipt,indent=2)+"\n")
        shutil.rmtree(scratch)
        print("Local integration result:",receipt["exitCode"],"receipt",evidence/"batch.json",flush=True)
    return result


if __name__ == "__main__":
    raise SystemExit(main())
