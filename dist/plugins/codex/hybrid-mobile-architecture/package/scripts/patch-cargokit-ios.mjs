// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/patch-cargokit-ios.mts
import { existsSync as existsSync2, writeFileSync } from "node:fs";
import { dirname as dirname2, join as join3, resolve as resolve2 } from "node:path";
import { fileURLToPath as fileURLToPath2 } from "node:url";

// src/native-helpers/common.mts
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from "node:fs";
var ToolError = class extends Error {
  constructor(message, code = 1) {
    super(message);
    this.code = code;
  }
  code;
};
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

// src/portable/versions.mts
import { readFileSync as readFileSync2 } from "node:fs";
import { join as join2 } from "node:path";

// src/portable/platform.mts
import { dirname, delimiter, join, resolve, extname } from "node:path";
import { fileURLToPath } from "node:url";
var packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

// src/portable/versions.mts
function versions(file = process.env.VERSIONS_TOML ?? join2(packageRoot, "versions.toml")) {
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

// src/native-helpers/patch-cargokit-ios.mts
await main(() => {
  const dir = dirname2(fileURLToPath2(import.meta.url));
  const pinFile = process.env.VERSIONS_TOML ?? [resolve2(dir, "../versions.toml"), resolve2(dir, "../rust/rust-toolchain.toml"), resolve2(dir, "../rust-toolchain.toml")].find(existsSync2);
  assert(pinFile, "canonical Rust pin not found; supply versions.toml, generated rust-toolchain.toml, or VERSIONS_TOML");
  const pins = versions(pinFile), rust = pins.toolchain?.rust ?? pins.toolchain?.channel;
  assert(rust && /^\d+\.\d+\.\d+$/.test(rust), "canonical Rust pin must be an exact x.y.z version");
  const template = existsSync2(join3(dir, "dedup_archive.dart")) ? join3(dir, "dedup_archive.dart") : resolve2(dir, "../assets/templates/cargokit/dedup_archive.dart");
  const target = join3(process.argv[2] ?? "mobile", "rust_builder/cargokit/build_tool/lib/src");
  assert(existsSync2(join3(target, "build_pod.dart")), "Cargokit build_pod.dart not found; run flutter_rust_bridge_codegen first.");
  let options = text(join3(target, "options.dart"));
  options = options.replace("enum Toolchain {\n  stable,\n  beta,\n  nightly,\n}\n\n", "").replace("final Toolchain toolchain;", "final String toolchain;");
  const oldParser = `static Toolchain _toolchainFromNode(YamlNode node) {
    if (node case YamlScalar(value: String name)) {
      final toolchain =
          Toolchain.values.firstWhereOrNull((element) => element.name == name);
      if (toolchain != null) {
        return toolchain;
      }
    }
    throw SourceSpanException(
        'Unknown toolchain. Must be one of \${Toolchain.values.map((e) => e.name)}.',
        node.span);
  }`;
  const newParser = `static String _toolchainFromNode(YamlNode node) {
    if (node case YamlScalar(value: String name)) {
      if (name.isNotEmpty && RegExp(r'^[A-Za-z0-9._-]+$').hasMatch(name)) {
        return name;
      }
    }
    throw SourceSpanException(
        'Invalid rustup toolchain. Expected a version, channel, or named toolchain.',
        node.span);
  }`;
  options = options.replace(oldParser, () => newParser).replace(/static Toolchain _toolchainFromNode\(YamlNode node\) \{[\s\S]*?node\.span\);\s*\}/, () => newParser).replace(/(?:Toolchain toolchain = Toolchain\.stable|String toolchain = '[^']+');/, `String toolchain = '${rust}';`);
  assert(options.includes(`String toolchain = '${rust}';`) && options.includes("static String _toolchainFromNode"), "failed to pin Cargokit options toolchain; unsupported vendor source");
  const builder = text(join3(target, "builder.dart")).replace(/String get _toolchain => _buildOptions\?\.toolchain(?:\.name)? \?\? '[^']+';/, `String get _toolchain => _buildOptions?.toolchain ?? '${rust}';`);
  assert(builder.includes(`?? '${rust}';`), "failed to pin Cargokit builder toolchain");
  const rustup = text(join3(target, "rustup.dart")).replace('Pattern nonCustom = RegExp(r"^(stable|beta|nightly)");', 'Pattern nonCustom = RegExp(r"^(stable|beta|nightly|[0-9])");');
  assert(rustup.includes("nightly|[0-9]"), "failed to retain version-pinned rustup toolchains");
  let pod = text(join3(target, "build_pod.dart"));
  const marker = "import 'builder.dart';\n", importLine = "import 'dedup_archive.dart';\n", callMarker = "    if (staticLibs.isNotEmpty) {\n";
  if (!pod.includes(importLine)) {
    assert(pod.includes(marker), "build_pod import marker missing");
    pod = pod.replace(marker, marker + importLine);
  }
  if (!pod.includes("dedupArchiveMembers(lib.path)")) {
    assert(pod.includes(callMarker), "build_pod static library marker missing");
    pod = pod.replace(callMarker, `${callMarker}      for (final lib in staticLibs) {
        if (dedupArchiveMembers(lib.path) > 0) {
          runCommand("ranlib", [lib.path]);
        }
      }
`);
  }
  const dedup = text(template);
  for (const [file, content] of [["options.dart", options], ["builder.dart", builder], ["rustup.dart", rustup], ["build_pod.dart", pod], ["dedup_archive.dart", dedup]]) writeFileSync(join3(target, file), content);
  console.log(`Patched Cargokit Rust ${rust} pin and archive dedup: ${join3(target, "build_pod.dart")}`);
});
