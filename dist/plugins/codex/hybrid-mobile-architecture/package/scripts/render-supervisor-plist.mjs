// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/render-supervisor-plist.mts
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname as dirname2, resolve as resolve2 } from "node:path";
import { fileURLToPath } from "node:url";

// src/native-helpers/common.mts
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from "node:fs";
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
function executable(name) {
  for (const dir of (process.env.PATH ?? "").split(delimiter)) {
    for (const suffix of process.platform === "win32" ? [".exe", ""] : [""]) {
      const candidate = join(dir, name + suffix);
      if (existsSync(candidate) && statSync(candidate).isFile()) return candidate;
    }
  }
  return void 0;
}
function text(path) {
  return readFileSync(path, "utf8");
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

// src/native-helpers/render-supervisor-plist.mts
await main(() => {
  const values = {}, args = [], env = [], input = process.argv.slice(2);
  for (let i = 0; i < input.length; i++) {
    const flag = input[i];
    if (flag === "--help" || flag === "-h") {
      console.log("node scripts/render-supervisor-plist.mjs --label NAME --program PATH [--arg ARG] [--env KEY=VALUE] [--format launchd|systemd] [--throttle 15] [--description TEXT] [--log-dir DIR] [--working-dir DIR] [--out FILE]");
      return;
    }
    assert(["--label", "--program", "--arg", "--env", "--format", "--throttle", "--description", "--log-dir", "--working-dir", "--out"].includes(flag) && input[i + 1] !== void 0, `unknown option or missing value: ${flag}`);
    const value = input[++i];
    assert(!/[\r\n\0]/.test(value), `${flag} must be a single line`);
    if (flag === "--arg") args.push(value);
    else if (flag === "--env") {
      assert(/^[A-Za-z_][A-Za-z0-9_]*=/.test(value), "--env requires KEY=VALUE");
      env.push(value);
    } else values[flag.slice(2)] = value;
  }
  const label = values.label, program = values.program, throttle = values.throttle ?? "15";
  assert(label && /^[A-Za-z0-9._-]+$/.test(label), "a valid --label is required");
  assert(program, "--program is required");
  assert(/^\d+$/.test(throttle) && Number.isSafeInteger(Number(throttle)) && Number(throttle) >= 10, "--throttle must be an integer at or above the 10s floor (R1.1)");
  assert(process.platform !== "win32" || values.format, "Windows has no launchd/systemd default; choose --format explicitly for a deployment target");
  const format = values.format ?? (process.platform === "darwin" ? "launchd" : "systemd");
  assert(["launchd", "systemd"].includes(format), "unknown --format (want launchd or systemd)");
  const log = values["log-dir"] ?? "$HOME/.prometheus/logs", work = values["working-dir"] ?? "$HOME";
  const xml = (value) => value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  const sd = (value) => value.replaceAll("%", "%%").replace("$HOME", "%h");
  const quote = (value, command = true) => '"' + value.replaceAll("\\", "\\\\").replaceAll('"', '\\"').replaceAll("$", command ? "$$" : "$").replaceAll("%", "%%") + '"';
  const substitutions = { LABEL: label, THROTTLE: throttle, DESCRIPTION: sd(values.description ?? label), STDOUT_PATH: `${log}/${label}.log`, STDERR_PATH: `${log}/${label}.err`, WORKING_DIRECTORY: work };
  if (format === "launchd") {
    for (const key of ["STDOUT_PATH", "STDERR_PATH", "WORKING_DIRECTORY"]) substitutions[key] = xml(substitutions[key]);
    substitutions.PROGRAM_ARGUMENTS = [program, ...args].map((arg) => `    <string>${xml(arg)}</string>`).join("\n");
    substitutions.ENVIRONMENT = ["PATH=/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin", ...env].map((pair) => {
      const index = pair.indexOf("=");
      return `    <key>${xml(pair.slice(0, index))}</key>
    <string>${xml(pair.slice(index + 1))}</string>`;
    }).join("\n");
  } else {
    for (const key of ["STDOUT_PATH", "STDERR_PATH", "WORKING_DIRECTORY"]) substitutions[key] = sd(substitutions[key]);
    substitutions.EXEC_START = [program, ...args].map((value) => quote(value)).join(" ");
    substitutions.ENVIRONMENT = ["PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin", ...env].map((pair) => `Environment=${quote(pair, false)}`).join("\n");
  }
  const template = resolve2(dirname2(fileURLToPath(import.meta.url)), "../assets/templates/launchagent-supervisor", format === "launchd" ? "supervisor.plist.template" : "supervisor.service.template");
  let rendered = text(template);
  for (const [key, value] of Object.entries(substitutions)) rendered = rendered.replaceAll(`@@${key}@@`, () => value);
  rendered = rendered.trimEnd() + "\n";
  if (values.out) {
    mkdirSync(dirname2(resolve2(values.out)), { recursive: true });
    writeFileSync(values.out, rendered);
    if (format === "launchd" && executable("plutil")) run("plutil", ["-lint", values.out], { capture: true });
    console.log(`render-supervisor-plist: wrote ${values.out} (${format})`);
  } else process.stdout.write(rendered);
});
