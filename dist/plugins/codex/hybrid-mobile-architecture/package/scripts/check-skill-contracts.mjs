#!/usr/bin/env node
import { access, readFile, readdir } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const manifest = JSON.parse(await readFile(resolve(root, "builder.manifest.json"), "utf8"));
const evalLines = (await readFile(resolve(root, "evals/builder-skills.jsonl"), "utf8"))
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
const failures = [];
const publicSkills = [manifest.distribution.packageSkill, ...manifest.skills];

async function exists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

for (const skill of publicSkills) {
  const directory = resolve(root, manifest.distribution.skillSourceRoot, skill);
  const markdown = await readFile(resolve(directory, "SKILL.md"), "utf8");
  const match = markdown.match(/^---\n([\s\S]*?)\n---\n/);
  if (!match) {
    failures.push(`${skill}: missing frontmatter`);
    continue;
  }
  const keys = match[1]
    .split("\n")
    .filter((line) => /^[A-Za-z][A-Za-z0-9_-]*:/.test(line))
    .map((line) => line.slice(0, line.indexOf(":")));
  if (keys.join(",") !== "name,description") {
    failures.push(`${skill}: frontmatter must contain only name and description`);
  }
  const name = match[1].match(/^name:\s*(.+)$/m)?.[1]?.trim();
  const description = match[1].match(/^description:\s*(.+)$/m)?.[1]?.trim();
  if (name !== skill) failures.push(`${skill}: frontmatter name mismatch`);
  if (!description || description.length > 1024) {
    failures.push(`${skill}: description must be 1..1024 characters`);
  }
  const referencedPaths = new Set([
    ...[...markdown.matchAll(/\]\((?!https?:|#|mailto:)([^)]+)\)/g)].map(
      (match) => match[1].split("#", 1)[0],
    ),
    ...[...markdown.matchAll(/\breferences\/[A-Za-z0-9_.\/-]+\.(?:md|json|toml|ya?ml)\b/g)].map(
      (match) => match[0],
    ),
  ]);
  for (const referencedPath of referencedPaths) {
    const target = resolve(directory, referencedPath);
    const fromSkill = relative(directory, target);
    if (fromSkill.startsWith("..") || fromSkill.startsWith("/")) {
      failures.push(`${skill}: resource escapes skill directory: ${referencedPath}`);
    } else if (!(await exists(target))) {
      failures.push(`${skill}: missing referenced resource: ${referencedPath}`);
    }
  }
  const openai = resolve(directory, "agents/openai.yaml");
  try {
    const yaml = await readFile(openai, "utf8");
    if (!yaml.includes("display_name:") || !yaml.includes("short_description:")) {
      failures.push(`${skill}: incomplete agents/openai.yaml`);
    }
  } catch {
    failures.push(`${skill}: missing agents/openai.yaml`);
  }
  const cases = evalLines.filter(
    (entry) =>
      entry.id.startsWith(`${skill}-`) &&
      (entry.expectedSkills.includes(skill) || entry.kind === "negative-near-miss"),
  );
  const kinds = new Set(cases.map((entry) => entry.kind));
  for (const kind of ["positive-explicit", "positive-implicit", "negative-near-miss"]) {
    if (!kinds.has(kind)) failures.push(`${skill}: missing ${kind} evaluation`);
  }
}

const skillDirectories = (
  await readdir(resolve(root, manifest.distribution.skillSourceRoot), { withFileTypes: true })
)
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();
const declared = [...publicSkills].sort();
if (JSON.stringify(skillDirectories) !== JSON.stringify(declared)) {
  failures.push("canonical skill directories differ from builder.manifest.json");
}

// Vendored internal authoring skills (openspec-*, source-command-opsx-*).
//
// These are written by an EXTERNAL generator (`openspec update`), not by this
// pack. `metadata.internal: true` is a repo-local invariant the OpenSpec CLI has
// no notion of, so it strips the key on every run; `scripts/normalize-vendored-skills.mjs`
// re-applies it. See openspec/changes/2026-08-22-c300-openspec-mirror-repair/design.md.
//
// Each managed harness is asserted INDIVIDUALLY. A union count across harnesses
// (the previous shape) could be satisfied by `.agents` alone, so a harness
// losing its entire mirror set passed silently — which is exactly what happened
// when 1.10.0 migrated `.kimi` to `.kimi-code`.
//
// Excluded by design:
//   .kimi  — migrated to .kimi-code by openspec 1.10.0; the CLI deletes it each run
//   .codex — reads shared skills from .agents/ (project.json: codex.skill_dir);
//            its own openspec-* copies were stale duplicates the CLI refuses to
//            overwrite and asks to have deleted
export const INTERNAL_SKILL_HARNESSES = {
  ".agents": { "openspec-": 12, "source-command-opsx-": 10 },
  ".claude": { "openspec-": 12, "source-command-opsx-": 0 },
  ".kimi-code": { "openspec-": 12, "source-command-opsx-": 0 },
  ".opencode": { "openspec-": 12, "source-command-opsx-": 0 },
};

for (const [harness, expected] of Object.entries(INTERNAL_SKILL_HARNESSES)) {
  const harnessRoot = resolve(root, harness, "skills");
  const seen = Object.fromEntries(Object.keys(expected).map((prefix) => [prefix, 0]));
  let entries;
  try {
    entries = await readdir(harnessRoot, { withFileTypes: true });
  } catch {
    failures.push(`${harness}/skills: managed harness directory is missing`);
    continue;
  }
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const prefix = Object.keys(expected).find((candidate) => entry.name.startsWith(candidate));
    if (!prefix) continue;
    seen[prefix] += 1;
    const markdown = await readFile(resolve(harnessRoot, entry.name, "SKILL.md"), "utf8");
    // Anchor to the YAML frontmatter. An unanchored match would accept a
    // `metadata:` block appearing anywhere in the body — including a prose
    // example — while the frontmatter the harnesses actually parse carries no
    // `internal: true`. These files are rewritten wholesale by an external
    // generator, so body content is uncontrolled input.
    const frontmatter = markdown.match(/^---\n([\s\S]*?)\n---(?:\n|$)/);
    if (!frontmatter) {
      failures.push(`${harness}/skills/${entry.name}: no YAML frontmatter`);
    } else if (!/^metadata:\n(?:  .+\n)*  internal: true$/m.test(`${frontmatter[1]}\n`)) {
      failures.push(`${harness}/skills/${entry.name}: missing metadata.internal: true`);
    }
  }
  for (const [prefix, count] of Object.entries(expected)) {
    if (seen[prefix] !== count) {
      failures.push(
        `${harness}/skills: expected ${count} ${prefix}* skills, found ${seen[prefix]}`,
      );
    }
  }
}

for (const failure of failures) process.stderr.write(`error: ${failure}\n`);
if (failures.length) process.exit(1);
process.stdout.write(
  `Validated ${publicSkills.length} public Agent Skills and ${evalLines.length} evaluation cases.\n`,
);
