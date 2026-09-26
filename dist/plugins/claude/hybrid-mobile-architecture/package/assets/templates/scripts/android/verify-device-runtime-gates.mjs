// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/android-device-gates.mts
import { existsSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout } from "node:timers/promises";

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

// src/native-helpers/android-device-gates.mts
await main(async () => {
  assert(process.argv.length >= 3 && process.argv.length <= 4, "usage: node verify-device-runtime-gates.mjs /path/to/app.apk [package.name]", 64);
  const apk = process.argv[2], pkg = process.argv[3] ?? "__APP_ID__";
  assert(/^[a-zA-Z][a-zA-Z0-9_]*(?:\.[a-zA-Z][a-zA-Z0-9_]*)+$/.test(pkg), "package must be an Android application identifier", 64);
  assert(existsSync(apk), `APK not found: ${apk}`, 66);
  const wait = Number(process.env.LAUNCH_WAIT_SECONDS ?? 8);
  assert(Number.isFinite(wait) && wait >= 0, "LAUNCH_WAIT_SECONDS must be non-negative", 64);
  const log = process.env.LOG_PATH ?? join(tmpdir(), "__APP_NAME__-device-runtime-gates.log");
  assert(run("adb", ["get-state"], { capture: true, allowFailure: true }).trim() === "device", "no adb device available; connect one device or set ANDROID_SERIAL", 69);
  const prop = (name) => run("adb", ["shell", "getprop", name], { capture: true }).trim() || "unknown";
  console.log(`Device: ${prop("ro.product.manufacturer")} ${prop("ro.product.model")}, Android ${prop("ro.build.version.release")}/API ${prop("ro.build.version.sdk")}, ABI ${prop("ro.product.cpu.abi")}, SoC ${prop("ro.soc.model")}`);
  console.log(`Installing: ${apk}`);
  run("adb", ["install", "-r", apk], { capture: true });
  run("adb", ["shell", "am", "force-stop", pkg], { capture: true, allowFailure: true });
  run("adb", ["logcat", "-c"]);
  console.log(`Launching: ${pkg}`);
  run("adb", ["shell", "monkey", "-p", pkg, "-c", "android.intent.category.LAUNCHER", "1"], { capture: true });
  await setTimeout(wait * 1e3);
  const output = run("adb", ["logcat", "-d", "-v", "time"], { capture: true });
  writeFileSync(log, output);
  const pid = run("adb", ["shell", "pidof", pkg], { capture: true, allowFailure: true }).trim();
  const gate = (condition, message) => assert(condition, `G4 FAIL: ${message}
logcat: ${log}`);
  gate(Boolean(pid), "app process is not alive after launch");
  gate(/Loaded gen_ui_ffi through System\.loadLibrary/.test(output), "libgen_ui_ffi load confirmation was not found in logcat");
  gate(/mobile migrations ready/.test(output), "Rust mobile boot/migration ready signal was not found in logcat");
  const fatal = /FATAL EXCEPTION|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed/;
  const failures = output.split(/\r?\n/).filter((line) => fatal.test(line) || line.includes(`${pkg} E AndroidRuntime`));
  gate(failures.length === 0, `fatal or app native-load failure found in logcat
${failures.join("\n")}`);
  console.log(`G3 PASS: APK installed on attached Android device
G4 PASS: app launched and process stayed alive pid=${pid}
G4 PASS: libgen_ui_ffi loaded through System.loadLibrary
G4 PASS: Rust mobile boot reached migrations-ready state
G5-G6 remain model/runtime self-test gates; run a chat/model certification workflow separately
logcat: ${log}`);
});
