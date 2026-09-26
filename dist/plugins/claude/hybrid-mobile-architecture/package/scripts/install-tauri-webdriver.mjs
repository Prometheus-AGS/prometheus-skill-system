// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/install-tauri-webdriver.mts
import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join as join2 } from "node:path";

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

// src/native-helpers/install-tauri-webdriver.mts
function value(source, key, pattern) {
  const match = source.match(new RegExp(`^${key}\\s*=\\s*"(${pattern.source})"`, "m"))?.[1];
  assert(Boolean(match), `versions.toml has no valid ${key}`);
  return match;
}
await main(() => {
  const args = process.argv.slice(2);
  assert(
    args.length <= 1 && (args.length === 0 || args[0] === "--check"),
    "node scripts/install-tauri-webdriver.mjs [--check]",
    2
  );
  const source = readFileSync(join2(repoRoot(), "versions.toml"), "utf8");
  const driver = value(source, "tauri_driver", /\d+\.\d+\.\d+(?:[-+][\w.-]+)?/);
  const edgeTool = value(source, "msedgedriver_tool_rev", /[0-9a-f]{40}/);
  if (args[0] === "--check") {
    process.stdout.write(`tauri-driver=${driver}
msedgedriver-tool=${edgeTool}
`);
    return;
  }
  assert(process.platform === "win32", "Tauri WebDriver installation is supported on Windows runners only", 2);
  run("cargo", ["install", "tauri-driver", "--version", driver, "--locked"]);
  run("cargo", ["install", "--git", "https://github.com/chippers/msedgedriver-tool", "--rev", edgeTool, "--locked"]);
  const cargoHome = process.env.CARGO_HOME ?? join2(homedir(), ".cargo");
  const installer = join2(cargoHome, "bin", "msedgedriver-tool.exe");
  assert(existsSync(installer), `msedgedriver-tool binary not found after installation: ${installer}`);
  run(installer, [], { cwd: process.cwd() });
  if (process.env.GITHUB_PATH) appendFileSync(process.env.GITHUB_PATH, `${process.cwd()}
`);
  process.stdout.write(`PASS: installed pinned native Tauri and Edge WebDriver tools
`);
});
