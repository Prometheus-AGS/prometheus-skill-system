// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/worktree-consolidation-inventory.mts
import { existsSync, lstatSync, mkdirSync, readlinkSync, readFileSync as readFileSync2, writeFileSync } from "node:fs";
import { basename, dirname as dirname2, join as join3, relative, resolve as resolve2 } from "node:path";

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

// src/native-helpers/wiki-common.mts
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { join as join2 } from "node:path";
var scopes = { root: ".prometheus", desktop: "apps/knowme-poc/desktop/.prometheus", "src-tauri": "apps/knowme-poc/desktop/src-tauri/.prometheus", rust: "apps/knowme-poc/rust/.prometheus" };
var sha = (data) => createHash("sha256").update(data).digest("hex");
var fileHash = (path) => sha(readFileSync(path));
function scopeFiles(root) {
  return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).flatMap((entry) => entry.isDirectory() ? scopeFiles(join2(root, entry.name)) : [join2(root, entry.name)]);
}
var git = (root, ...args) => run("git", args, { cwd: root, capture: true });
function frontmatter(text) {
  if (!text.startsWith("---\n")) return {};
  const end = text.indexOf("\n---\n", 4);
  if (end < 0) return {};
  return Object.fromEntries(text.slice(4, end).split("\n").flatMap((line) => {
    const match = /^(type|id|title|revision|timestamp|created_at|updated_at):\s*(.*?)\s*$/.exec(line);
    return match ? [[match[1], match[2].replace(/^["']+|["']+$/g, "")]] : [];
  }));
}
function worktrees(root) {
  return git(root, "worktree", "list", "--porcelain", "-z").split("\0\0").filter(Boolean).map((block) => Object.fromEntries(block.split("\0").filter(Boolean).map((line) => {
    const at = line.indexOf(" ");
    return at < 0 ? [line, ""] : [line.slice(0, at), line.slice(at + 1)];
  })));
}

// src/native-helpers/worktree-consolidation-inventory.mts
await main(() => {
  assert(process.argv.length === 3, "node scripts/worktree-consolidation-inventory.mjs OUTPUT", 2);
  const root = repoRoot();
  const records = worktrees(root).map((raw) => {
    const worktree = raw.worktree;
    const parts = git(worktree, "status", "--porcelain=v1", "-z", "--untracked-files=all").split("\0"), dirty = [];
    for (let i = 0; parts[i]; i++) {
      const line = parts[i], status = line.slice(0, 2), path = line.slice(3), entry = { status, path };
      if (/[RC]/.test(status)) entry.source_path = parts[++i];
      const candidate = join3(worktree, path);
      try {
        const stat = lstatSync(candidate);
        if (stat.isSymbolicLink()) entry.symlink_target = readlinkSync(candidate);
        else if (stat.isFile()) {
          entry.sha256 = fileHash(candidate);
          entry.size = stat.size;
        }
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
      }
      dirty.push(entry);
    }
    const inventory = Object.fromEntries(Object.entries(scopes).map(([name, rel]) => {
      const scope = join3(worktree, rel), present = existsSync(scope);
      const entries = present ? scopeFiles(scope).map((path) => {
        const scopePath = relative(scope, path).replaceAll("\\", "/");
        const entry = { path: relative(worktree, path).replaceAll("\\", "/"), scope_path: scopePath, sha256: lstatSync(path).isSymbolicLink() ? null : fileHash(path), size: lstatSync(path).size };
        if (lstatSync(path).isSymbolicLink()) entry.symlink_target = readlinkSync(path);
        else if (path.endsWith(".md")) {
          const fm = frontmatter(readFileSync2(path, "utf8"));
          delete fm.type;
          delete fm.title;
          if (Object.keys(fm).length) entry.frontmatter = fm;
        }
        return entry;
      }) : [];
      return [name, { exists: present, file_count: entries.length, wiki_markdown_count: entries.filter((entry) => String(entry.scope_path).startsWith("knowledge/wiki/") && String(entry.scope_path).endsWith(".md")).length, files: entries }];
    }));
    return { name: resolve2(worktree) === root ? "primary" : basename(worktree), path: resolve2(worktree) === root ? "$REPO_ROOT" : `$REPO_ROOT/${relative(root, worktree).replaceAll("\\", "/")}`, head: raw.HEAD ?? "", branch: (raw.branch ?? "").replace(/^refs\/heads\//, ""), locked: raw.locked ?? null, dirty_files: dirty, prometheus_scopes: inventory };
  });
  const branches = git(root, "for-each-ref", "--format=%(refname)|%(objectname)|%(upstream)|%(upstream:track)", "refs/heads", "refs/remotes").trim().split("\n").filter(Boolean).map((line) => {
    const [ref, sha2, upstream, tracking] = line.split("|");
    return { ref, sha: sha2, upstream, tracking };
  });
  const manifest = { schema_version: 1, purpose: "Pre-consolidation loss-prevention inventory", repository: "$REPO_ROOT", baseline_main: git(root, "rev-parse", "main").trim(), branches, worktrees: records };
  const output = resolve2(root, process.argv[2]);
  mkdirSync(dirname2(output), { recursive: true });
  writeFileSync(output, JSON.stringify(manifest, null, 2) + "\n");
  console.log(output);
});
