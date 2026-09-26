#!/usr/bin/env node

import { readFile, readdir } from "node:fs/promises";
import { extname, join, relative, resolve } from "node:path";

import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const scanRoots = [
  "assets/templates",
  "references",
  "skills",
];
const textExtensions = new Set([
  ".dart", ".json", ".kt", ".md", ".mjs", ".rs", ".swift", ".toml", ".ts", ".tsx", ".yaml", ".yml",
]);
const findings = [];

const prohibited = [
  {
    pattern: /\bServiceRole\b|\bservice_role\b/,
    message: "client-visible service role",
    codeOnly: true,
  },
  {
    pattern: /\bdecodeJwt\b|\bjwtDecode\b/,
    message: "decoded token used as an identity primitive",
    codeOnly: true,
  },
  {
    pattern: /Stream(?:<[^>]+>)?\.empty\(\)|const Stream\.empty\(\)/,
    message: "empty runnable event stream",
  },
  {
    pattern: /UnimplementedError|todo!\s*\(/,
    message: "runnable stub",
  },
];

async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.name === "vendor") continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await walk(path);
      continue;
    }
    if (!textExtensions.has(extname(path))) continue;
    const text = await readFile(path, "utf8");
    const codeLike = extname(path) !== ".md";
    for (const rule of prohibited) {
      if (rule.codeOnly && !codeLike) continue;
      if (rule.pattern.test(text)) {
        findings.push(`${relative(root, path)}: ${rule.message}`);
      }
    }
    if (
      text.includes('"shell:default"') &&
      !path.endsWith("command-contract/commands.json")
    ) {
      findings.push(`${relative(root, path)}: shell:default is not allowed`);
    }
  }
}

for (const scanRoot of scanRoots) await walk(join(root, scanRoot));

for (const finding of findings) process.stderr.write(`error: ${finding}\n`);
if (findings.length > 0) process.exitCode = 1;
else process.stdout.write("runtime authority and security patterns are clean\n");
