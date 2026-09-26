// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/record-progress.mts
import { randomUUID } from "node:crypto";
import { appendFileSync, existsSync, mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { basename, isAbsolute, join as join2, relative, resolve as resolve2 } from "node:path";

// src/native-helpers/common.mts
import { spawnSync } from "node:child_process";
import { delimiter, dirname, join, resolve } from "node:path";
var ToolError = class extends Error {
  constructor(message, code = 1) {
    super(message);
    this.code = code;
  }
  code;
};
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: options.cwd, env: options.env ?? process.env, encoding: "utf8", stdio: options.capture ? ["ignore", "pipe", "pipe"] : "inherit", shell: false, maxBuffer: 64 * 1024 * 1024 });
  if (result.error) throw new ToolError(`required tool failed: ${command}: ${result.error.message}`, 127);
  if (result.status !== 0 && !options.allowFailure) throw new ToolError(`${command} failed (${result.status ?? result.signal})${result.stderr ? `: ${result.stderr.trim()}` : ""}`, result.status ?? 1);
  return result.status === 0 ? result.stdout ?? "" : "";
}
function repoRoot() {
  return resolve(run("git", ["rev-parse", "--show-toplevel"], { capture: true }).trim());
}
function assert(condition, message, code = 1) {
  if (!condition) throw new ToolError(message, code);
}
async function main(fn) {
  try {
    await fn();
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = error instanceof ToolError ? error.code : 1;
  }
}

// src/native-helpers/record-progress.mts
await main(() => {
  const args = process.argv.slice(2), values = { status: "in-progress" };
  const fields = ["phase", "title", "summary", "evidence", "next", "status"];
  for (let i = 0; i < args.length; i += 2) {
    const field = args[i].replace(/^--/, "");
    assert(args[i].startsWith("--") && fields.includes(field) && args[i + 1], "usage: node record-progress.mjs --phase SLUG --title TEXT --summary TEXT --evidence TEXT --next TEXT [--status STATUS]", 2);
    values[field] = args[i + 1];
  }
  for (const field of fields) assert(values[field], `missing --${field}`, 2);
  assert(/^[a-z0-9][a-z0-9-]*$/.test(values.phase), "phase must be a lowercase slug", 2);
  const combined = [values.title, values.summary, values.evidence, values.next].join(" ");
  assert(!/(api[_ -]?key|token|password|secret|private[_ -]?key)\s*[:=]\s*[^$<{\[]/i.test(combined), "refusing to record text that appears to contain a secret value", 3);
  const root = repoRoot(), slug = basename(root).replace(/[^a-zA-Z0-9-]+/g, "-").replace(/-$/, "");
  const utc = (/* @__PURE__ */ new Date()).toISOString().replace(/\.\d{3}Z$/, "Z"), stamp = utc.replace(/[-:]/g, "");
  const id = `karpathy-progress-${stamp}-${values.phase}-${randomUUID().slice(0, 8)}`;
  const roots = /* @__PURE__ */ new Set([root, root.replaceAll("\\", "/")]);
  if (process.platform === "darwin" && /^\/private\/(?:var|tmp)\//.test(root)) roots.add(root.slice("/private".length));
  const pwd = process.env.PWD;
  if (pwd && existsSync(pwd) && realpathSync(pwd) === realpathSync(root)) roots.add(pwd);
  const cwd = resolve2();
  if (existsSync(cwd) && realpathSync(cwd) === realpathSync(root)) roots.add(cwd);
  const canonicalRoot = realpathSync(root);
  for (const base of [tmpdir(), homedir(), process.env.TEMP, process.env.TMP, process.env.USERPROFILE]) {
    if (!base || !existsSync(base)) continue;
    const suffix = relative(realpathSync(base), canonicalRoot);
    if (!isAbsolute(suffix) && suffix !== ".." && !suffix.startsWith(`..${process.platform === "win32" ? "\\" : "/"}`)) roots.add(join2(base, suffix));
  }
  for (const path of [...roots]) {
    roots.add(path.replaceAll("\\", "/"));
    roots.add(path.replaceAll("/", "\\"));
  }
  for (const field of ["title", "summary", "evidence", "next"]) for (const path of [...roots].sort((a, b) => b.length - a.length)) values[field] = values[field].replaceAll(path, "$REPO_ROOT");
  const projectWiki = join2(root, ".prometheus/knowledge/wiki"), privateProject = join2(process.env.PROMETHEUS_PRIVATE_ROOT ?? join2(homedir(), ".prometheus"), "knowledge/private", slug), privateWiki = join2(privateProject, "wiki");
  mkdirSync(projectWiki, { recursive: true });
  mkdirSync(privateWiki, { recursive: true });
  const content = `---
type: Reference
id: ${id}
title: ${JSON.stringify(values.title)}
tags:
- karpathy-progress
- ${values.phase}
- ${JSON.stringify(values.status)}
sources:
- conversation:operator-agent
timestamp: ${utc}
created_at: ${utc}
updated_at: ${utc}
revision: 1
---

## Intent

${values.summary}

## Observed state and verification

${values.evidence}

## Decision and lesson

Status: ${values.status}. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

${values.next}
`;
  const event = JSON.stringify({ id: randomUUID(), kind: "compiled", session_id: "karpathy-progress-memory", project_root: "$REPO_ROOT", scope: "project", timestamp: utc, payload: { entry_id: id, tags: ["karpathy-progress", values.phase, values.status], title: values.title, ts: utc, type: "compiled" }, affects: [id] }) + "\n";
  for (const path of [join2(projectWiki, `${id}.md`), join2(privateWiki, `${id}.md`)]) {
    writeFileSync(path, content, { flag: "wx" });
    console.log(path);
  }
  appendFileSync(join2(root, ".prometheus/events.jsonl"), event);
  appendFileSync(join2(privateProject, "events.jsonl"), event);
});
