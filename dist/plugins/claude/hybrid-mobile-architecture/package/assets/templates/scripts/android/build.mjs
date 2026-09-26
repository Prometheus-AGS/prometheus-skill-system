// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/android-build.mts
import { existsSync, readdirSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

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

// src/native-helpers/android-build.mts
await main(() => {
  const profile = process.argv[2] ?? "release";
  assert(["release", "debug"].includes(profile), "profile must be release or debug", 64);
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "../.."), rust = join(root, "rust");
  const sdk = process.env.ANDROID_SDK_ROOT ?? process.env.ANDROID_HOME ?? (process.platform === "win32" ? join(process.env.LOCALAPPDATA ?? join(homedir(), "AppData/Local"), "Android/Sdk") : process.platform === "darwin" ? join(homedir(), "Library/Android/sdk") : join(homedir(), "Android/Sdk"));
  const ndks = join(sdk, "ndk");
  const latest = existsSync(ndks) ? readdirSync(ndks, { withFileTypes: true }).filter((entry) => entry.isDirectory()).map((entry) => entry.name).sort((a, b) => a.localeCompare(b, "en", { numeric: true })).at(-1) : void 0;
  const ndk = process.env.ANDROID_NDK ?? (latest ? join(ndks, latest) : "");
  assert(ndk && existsSync(ndk), `Android NDK not found. Set ANDROID_NDK or install it in ${ndks}.`);
  run("cargo", ["ndk", "--target", "arm64-v8a", "--platform", "29", "--", "build", "-p", "gen_ui_ffi", "--target", "aarch64-linux-android", ...profile === "release" ? ["--release"] : []], { cwd: rust, env: { ...process.env, ANDROID_NDK: ndk, NDK_ROOT: ndk } });
  console.log("\u2713 arm64-v8a");
  run("flutter_rust_bridge_codegen", ["generate", "--config-file", join(rust, "flutter_rust_bridge.yaml")], { cwd: root });
  console.log("\u2713 Dart bindings generated");
});
