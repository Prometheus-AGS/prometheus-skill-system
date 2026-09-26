// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/install-flutter.mts
import { existsSync as existsSync2, mkdirSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter as delimiter3, dirname as dirname3, join as join4 } from "node:path";
import { spawnSync as spawnSync2 } from "node:child_process";

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

// src/portable/versions.mts
import { readFileSync as readFileSync2 } from "node:fs";
import { join as join3 } from "node:path";

// src/portable/platform.mts
import { dirname as dirname2, delimiter as delimiter2, join as join2, resolve as resolve2, extname } from "node:path";
import { fileURLToPath } from "node:url";
var packageRoot = resolve2(dirname2(fileURLToPath(import.meta.url)), "../..");

// src/portable/versions.mts
function versions(file = process.env.VERSIONS_TOML ?? join3(packageRoot, "versions.toml")) {
  const result = {};
  let section = "";
  for (const line of readFileSync2(file, "utf8").split(/\r?\n/)) {
    const heading = line.match(/^\s*\[([^\]]+)\]\s*(?:#.*)?$/);
    if (heading) {
      section = heading[1];
      result[section] ??= {};
      continue;
    }
    const pair = line.match(/^\s*([\w-]+)\s*=\s*"([^"\n]*)"\s*(?:#.*)?$/);
    if (pair && section) result[section][pair[1]] = pair[2];
  }
  return result;
}

// src/native-helpers/install-flutter.mts
function vendor(file, args, cwd) {
  if (!file.endsWith(".bat")) {
    run(file, args, { cwd });
    return;
  }
  for (const token of [file, ...args]) assert(!/[&|<>^%!"\r\n\0]/.test(token), "vendor batch adapter refuses command expansion characters in SDK path or arguments");
  assert(["flutter.bat", "dart.bat"].some((name) => file.endsWith(name)), "unsupported vendor batch entrypoint");
  const result = spawnSync2(process.env.ComSpec ?? "cmd.exe", ["/d", "/v:off", "/s", "/c", `"${[file, ...args].map((token) => `"${token}"`).join(" ")}"`], { cwd, stdio: "inherit", shell: false, windowsVerbatimArguments: true });
  if (result.error || result.status !== 0) throw new ToolError(`Flutter vendor bootstrap failed: ${result.error?.message ?? result.status}`, result.status ?? 1);
}
await main(() => {
  const pins = versions(), flutterVersion = pins.toolchain?.flutter;
  assert(flutterVersion && /^\d+\.\d+\.\d+$/.test(flutterVersion), "toolchain.flutter must be an exact stable x.y.z version");
  assert(process.argv.length <= 3 && (!process.argv[2] || ["--fvm", "--help"].includes(process.argv[2])), "usage: node scripts/install-flutter.mjs [--fvm]");
  if (process.argv[2] === "--help") {
    console.log(`node scripts/install-flutter.mjs [--fvm]
Installs pinned Flutter ${flutterVersion}. Windows bootstrap uses the vendor flutter.bat with a restricted adapter.`);
    return;
  }
  if (process.argv[2] === "--fvm") {
    const dart = executable("dart") ?? (process.env.PATH ?? "").split(delimiter3).map((dir) => join4(dir, "dart.bat")).find(existsSync2);
    assert(dart, "Dart is required to install FVM");
    vendor(dart, ["pub", "global", "activate", "fvm"]);
    for (const args of [["install", flutterVersion], ["global", flutterVersion], ["flutter", "--version"]]) vendor(dart, ["pub", "global", "run", "fvm:fvm", ...args]);
    console.log(`Add ${join4(homedir(), ".pub-cache/bin")} to PATH. Use fvm flutter <command>.`);
    return;
  }
  const sdk = join4(homedir(), "development/flutter"), flutter = join4(sdk, "bin", process.platform === "win32" ? "flutter.bat" : "flutter");
  if (existsSync2(sdk)) {
    assert(existsSync2(join4(sdk, ".git")), `existing Flutter directory is not a Git checkout: ${sdk}`);
    run("git", ["fetch", "--tags", "origin", flutterVersion], { cwd: sdk });
    run("git", ["checkout", "--detach", flutterVersion], { cwd: sdk });
  } else {
    mkdirSync(dirname3(sdk), { recursive: true });
    run("git", ["clone", "--branch", flutterVersion, "--depth", "1", "https://github.com/flutter/flutter.git", sdk]);
  }
  process.env.PATH = `${join4(sdk, "bin")}${delimiter3}${process.env.PATH ?? ""}`;
  vendor(flutter, ["doctor", "--no-color"], sdk);
  vendor(flutter, ["--version"], sdk);
  console.log(`Permanently add ${join4(sdk, "bin")} to PATH. Then run flutter doctor --android-licenses.`);
});
