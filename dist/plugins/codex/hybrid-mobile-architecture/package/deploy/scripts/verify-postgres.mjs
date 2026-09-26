// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/common.mts
import { spawnSync } from "node:child_process";
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

// src/native-helpers/verify-postgres.mts
await main(() => {
  const container = process.argv[2] ?? "prometheus-postgres18-verification";
  const actual = run("docker", ["exec", container, "psql", "-U", "flint", "-d", "flint", "-Atc", "select extname from pg_extension order by extname"], { capture: true }).trim().split(/\r?\n/);
  for (const extension of ["vector", "pgcrypto", "pg_net", "pg_cron", "flint_llm", "flint_vault", "flint_meta", "flint_auth", "flint_hooks"]) assert(actual.includes(extension), `missing extension: ${extension}`);
  run("docker", ["exec", container, "wal-g", "--version"]);
  assert(run("docker", ["exec", container, "psql", "-U", "flint", "-d", "flint", "-Atc", "show wal_level"], { capture: true }).trim() === "logical", "wal_level must be logical");
  console.log("PostgreSQL extension and replication verification passed");
});
