// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/ios-build-xcframework.mts
import { cpSync, mkdirSync, rmSync } from "node:fs";
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

// src/native-helpers/ios-build-xcframework.mts
await main(() => {
  assert(process.platform === "darwin", "iOS XCFramework builds require macOS with Xcode; Windows/Linux hosts cannot produce this artifact", 69);
  const profile = process.argv[2] ?? "release";
  assert(["release", "debug"].includes(profile), "profile must be release or debug", 64);
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "../.."), rust = join(root, "rust"), build = join(root, "scripts/ios/build");
  const target = process.env.CARGO_TARGET_DIR ? resolve(rust, process.env.CARGO_TARGET_DIR) : join(rust, "target");
  mkdirSync(build, { recursive: true });
  for (const triple of ["aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios"]) run("cargo", ["build", "--manifest-path", join(rust, "Cargo.toml"), "--target", triple, ...profile === "release" ? ["--release"] : []], { cwd: rust });
  const fat = join(build, "libgen_ui_core_sim.a");
  run("lipo", ["-create", join(target, "aarch64-apple-ios-sim", profile, "libgen_ui_core.a"), join(target, "x86_64-apple-ios", profile, "libgen_ui_core.a"), "-output", fat]);
  const xc = join(build, "GenUICore.xcframework");
  rmSync(xc, { recursive: true, force: true });
  run("xcodebuild", ["-create-xcframework", "-library", join(target, "aarch64-apple-ios", profile, "libgen_ui_core.a"), "-library", fat, "-output", xc]);
  const destination = join(root, "mobile/ios/Frameworks/GenUICore.xcframework");
  mkdirSync(dirname(destination), { recursive: true });
  rmSync(destination, { recursive: true, force: true });
  cpSync(xc, destination, { recursive: true });
  console.log("\u2713 XCFramework \u2192 mobile/ios/Frameworks/");
});
