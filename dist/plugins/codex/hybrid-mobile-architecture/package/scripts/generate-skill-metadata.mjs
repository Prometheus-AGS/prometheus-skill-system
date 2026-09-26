#!/usr/bin/env node

import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const skillsRoot = join(root, "skills");
const check = process.argv.includes("--check");

function title(name) {
  return name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function frontmatter(markdown) {
  const match = markdown.match(/^---\n([\s\S]*?)\n---\n/);
  if (!match) throw new Error("missing YAML frontmatter");
  const name = match[1].match(/^name:\s*(.+)$/m)?.[1]?.trim();
  const descriptionMatch = match[1].match(/^description:\s*(.+)$/m);
  if (!name || !descriptionMatch) {
    throw new Error("frontmatter must contain name and single-line description");
  }
  return { name, description: descriptionMatch[1].trim() };
}

let drift = false;
const directories = (await readdir(skillsRoot, { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .sort((a, b) => a.name.localeCompare(b.name));

for (const directory of directories) {
  const markdown = await readFile(join(skillsRoot, directory.name, "SKILL.md"), "utf8");
  const { name, description } = frontmatter(markdown);
  if (name !== directory.name) {
    throw new Error(`${directory.name}: frontmatter name differs from directory`);
  }
  const short = description
    .split(/[.!?](?:\s|$)/)[0]
    .slice(0, 96)
    .trim()
    .replace(/["\\]/g, "");
  const output =
    "interface:\n" +
    `  display_name: ${JSON.stringify(title(name))}\n` +
    `  short_description: ${JSON.stringify(short)}\n` +
    `  default_prompt: ${JSON.stringify(`Use $${name} to apply its contract to this task and verify the required evidence.`)}\n`;
  const target = join(skillsRoot, directory.name, "agents/openai.yaml");
  let current = "";
  try {
    current = await readFile(target, "utf8");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  if (current === output) continue;
  drift = true;
  if (check) {
    process.stderr.write(`skill metadata drift: ${directory.name}\n`);
  } else {
    await mkdir(join(skillsRoot, directory.name, "agents"), { recursive: true });
    await writeFile(target, output);
    process.stdout.write(`generated ${directory.name}/agents/openai.yaml\n`);
  }
}

if (check && drift) process.exitCode = 1;
