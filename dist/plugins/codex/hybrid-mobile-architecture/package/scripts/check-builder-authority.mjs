#!/usr/bin/env node

import { access, readFile, readdir } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const release = process.argv.includes("--release");
const manifest = JSON.parse(
  await readFile(join(root, "builder.manifest.json"), "utf8"),
);
const failures = [];
const warnings = [];

function fail(message) {
  failures.push(message);
}

async function exists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

if (manifest.schemaVersion !== 1) fail("unsupported builder manifest schemaVersion");
if (!/^2\.0\.0-(alpha|beta)\.\d+$|^2\.0\.0$/.test(manifest.package.version)) {
  fail(`invalid Builder version: ${manifest.package.version}`);
}
if (new Set(manifest.skills).size !== manifest.skills.length) {
  fail("duplicate skill identifiers in builder.manifest.json");
}
const requiredHarnesses = [
  "claude-code",
  "codex",
  "opencode",
  "kimi-code",
  "minimax-code",
  "zed",
];
if (
  manifest.supportedHarnesses.length !== requiredHarnesses.length ||
  requiredHarnesses.some((harness) => !manifest.supportedHarnesses.includes(harness))
) {
  fail(`supported harnesses must be exactly: ${requiredHarnesses.join(", ")}`);
}
if (manifest.distribution?.skillSourceRoot !== "skills") {
  fail("skills must be the canonical public source root");
}
if (manifest.distribution?.packageSkill !== manifest.package.id) {
  fail("distribution packageSkill must match package.id");
}

for (const target of manifest.generatedTargets) {
  if (!(await exists(join(root, target)))) fail(`missing generated target: ${target}`);
}

for (const template of manifest.templates) {
  const templatePath = join(root, template.path);
  if (!(await exists(templatePath))) {
    fail(`declared template does not exist: ${template.path}`);
    continue;
  }
  const entries = await readdir(templatePath);
  if (entries.length === 0) fail(`declared template is empty: ${template.path}`);
}

for (const skill of manifest.skills) {
  const skillPath = join(root, manifest.distribution.skillSourceRoot, skill, "SKILL.md");
  if (!(await exists(skillPath))) {
    const message = `declared skill is not implemented: ${skill}`;
    if (release) fail(message);
    else warnings.push(message);
  }
}

const skillFiles = [];
for (const skill of manifest.skills) {
  const path = join(root, manifest.distribution.skillSourceRoot, skill, "SKILL.md");
  if (await exists(path)) skillFiles.push(path);
}
skillFiles.push(
  join(
    root,
    manifest.distribution.skillSourceRoot,
    manifest.distribution.packageSkill,
    "SKILL.md",
  ),
);
const skillText = (
  await Promise.all(skillFiles.map((path) => readFile(path, "utf8")))
).join("\n");

if (skillText.includes("../../AGENT_BASE_RULES.md")) {
  fail("layout-dependent AGENT_BASE_RULES.md link remains");
}

const referenceRoot = join(root, "references");
const referenceFiles = [];
async function collect(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) await collect(path);
    else referenceFiles.push(path);
  }
}
await collect(referenceRoot);
for (const path of referenceFiles) {
  const ref = relative(root, path);
  if (!skillText.includes(ref) && !skillText.includes(relative(referenceRoot, path))) {
    fail(`orphaned reference: ${ref}`);
  }
}

for (const warning of warnings) process.stderr.write(`warning: ${warning}\n`);
for (const failure of failures) process.stderr.write(`error: ${failure}\n`);
if (failures.length > 0) process.exitCode = 1;
else {
  process.stdout.write(
    `Builder authority valid (${manifest.skills.length} declared skills, ` +
      `${manifest.templates.length} template groups, ${warnings.length} planned items)\n`,
  );
}
