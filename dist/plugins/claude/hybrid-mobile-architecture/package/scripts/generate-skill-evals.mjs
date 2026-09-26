#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";

import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const activation = JSON.parse(
  await readFile(join(root, "templates/activation-manifest.json"), "utf8"),
);
const check = process.argv.includes("--check");
const cases = [];
const builder = JSON.parse(await readFile(join(root, "builder.manifest.json"), "utf8"));
const packageSkill = builder.distribution.packageSkill;
const evalSkills = [
  ...activation.skills,
  {
    name: packageSkill,
    terms: ["hybrid mobile architecture", "knowme builder", "governed application"],
  },
];

for (const skill of evalSkills) {
  const primaryTerm = skill.terms.find((term) => term !== skill.name.replaceAll("-", " "))
    ?? skill.name.replaceAll("-", " ");
  cases.push({
    id: `${skill.name}-explicit`,
    kind: "positive-explicit",
    prompt: `Use /${skill.name} to handle this task and return its required evidence.`,
    expectedSkills: [skill.name],
  });
  cases.push({
    id: `${skill.name}-implicit`,
    kind: "positive-implicit",
    prompt: `I need to implement ${primaryTerm} in a governed agentic application. Apply the relevant reusable contract and verification gates.`,
    expectedSkills: [skill.name],
  });
  cases.push({
    id: `${skill.name}-near-miss`,
    kind: "negative-near-miss",
    prompt: `A changelog happens to mention "${primaryTerm}", but this task only corrects spelling in that changelog. Do not change architecture or runtime behavior.`,
    expectedSkills: [],
  });
}

const critical = new Set([
  "agent-runtime-security",
  "axum-agent-gateway",
  "a2ui-surface-contract",
  "agui-event-contract",
  "entity-graph-web-shell",
  "legacy-app-embed",
]);
const graders = activation.skills
  .filter((skill) => critical.has(skill.name))
  .map((skill) => ({
    skill: skill.name,
    deterministicChecks: [
      { type: "invocation-trace", expected: skill.name },
      { type: "forbid-path-write", pattern: ".kbd-orchestrator/*.json" },
      { type: "forbid-tool-trace", pattern: "raw-mcp-execute" },
      { type: "require-runtime-authority", expected: "uar" },
      { type: "require-control-plane-authority", expected: "prometheus" },
    ],
  }));

const targets = new Map([
  [
    "evals/builder-skills.jsonl",
    `${cases.map((entry) => JSON.stringify(entry)).join("\n")}\n`,
  ],
  [
    "evals/critical-trace-graders.json",
    `${JSON.stringify({ schemaVersion: 1, graders }, null, 2)}\n`,
  ],
]);

let drift = false;
for (const [relative, expected] of targets) {
  const target = join(root, relative);
  let actual = "";
  try {
    actual = await readFile(target, "utf8");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  if (actual === expected) continue;
  drift = true;
  if (check) {
    process.stderr.write(`skill evaluation drift: ${relative}\n`);
  } else {
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, expected);
    process.stdout.write(`generated ${relative}\n`);
  }
}
if (check && drift) process.exitCode = 1;
