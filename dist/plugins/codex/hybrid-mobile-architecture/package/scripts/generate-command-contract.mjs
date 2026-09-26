#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const valueAfter = (flag) => {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : undefined;
};
const manifestPath = resolve(
  valueAfter("--manifest") ?? join(root, "assets/templates/command-contract/commands.json"),
);
const outputRoot = resolve(
  valueAfter("--out") ?? join(root, "generated/command-contract"),
);
const check = args.includes("--check");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const manifestLabel = relative(root, manifestPath);
if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.commands)) {
  throw new Error("unsupported command manifest");
}

const commandNames = manifest.commands.map(({ name }) => name);
if (new Set(commandNames).size !== commandNames.length) {
  throw new Error("duplicate command name");
}
if (manifest.forbiddenDefaults?.includes("shell:default") !== true) {
  throw new Error("command manifest must explicitly forbid shell:default");
}

const rust = `// Generated from ${manifestLabel}; do not edit.\n` +
  `pub const GENERATED_COMMANDS: &[&str] = &[\n` +
  commandNames.map((name) => `    "${name}",\n`).join("") +
  `];\n\n` +
  `#[macro_export]\nmacro_rules! generated_invoke_handler {\n` +
  `    () => { tauri::generate_handler![${commandNames.join(", ")}] };\n}\n`;

const typescript = `// Generated from the command manifest; do not edit.\n` +
  `import { invoke } from "@tauri-apps/api/core";\n\n` +
  manifest.commands.map(({ name, input, output }) =>
    `export const ${name} = (input: ${input}): Promise<${output}> => ` +
    `invoke<${output}>("${name}", { input });\n`
  ).join("");

const dart = `// Generated from the command manifest; do not edit.\n` +
  `abstract interface class GeneratedUarCommands {\n` +
  manifest.commands.map(({ name, input, output }) =>
    `  Future<${output}> ${name}(${input} input);\n`
  ).join("") +
  `}\n`;

const permissions = `# Generated from the command manifest; do not edit.\n` +
  manifest.commands.map(({ permission }) => `[[permission]]\nidentifier = "${permission}"\n`).join("\n");

const contractTest = `// Generated parity test.\n` +
  `import assert from "node:assert/strict";\n` +
  `import test from "node:test";\n` +
  `const expected = ${JSON.stringify(commandNames)};\n` +
  `test("command names remain unique and shell is not granted", () => {\n` +
  `  assert.equal(new Set(expected).size, expected.length);\n` +
  `  assert.equal(${JSON.stringify(manifest.defaultPermissions)}.includes("shell:default"), false);\n` +
  `});\n`;

const targets = new Map([
  ["rust/generated_commands.rs", rust],
  ["typescript/commands.ts", typescript],
  ["dart/commands.dart", dart],
  ["permissions/generated.toml", permissions],
  ["tests/command-contract.test.mjs", contractTest],
]);

let drift = false;
for (const [relative, expected] of targets) {
  const target = join(outputRoot, relative);
  let actual = "";
  try {
    actual = await readFile(target, "utf8");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  if (actual === expected) continue;
  drift = true;
  if (check) {
    process.stderr.write(`command contract drift: ${relative}\n`);
  } else {
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, expected);
    process.stdout.write(`generated ${relative}\n`);
  }
}
if (check && drift) process.exitCode = 1;
