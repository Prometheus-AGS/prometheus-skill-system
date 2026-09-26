#!/usr/bin/env node

import { cp, mkdir, readFile, rm, stat } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");

const mappings = [
  ["references", "skills/hybrid-mobile-architecture/references"],
  ["references/sync/client-rag.md", "skills/client-rag/references/client-rag.md"],
  ["references/rust/new-block-type.md", "skills/content-block-ui/references/rust/new-block-type.md"],
  ["references/generator-placeholders.md", "skills/dependency-pin-discipline/references/generator-placeholders.md"],
  ["references/sync/peer-crdt.md", "skills/peer-profile-sync/references/peer-crdt.md"],
  ["references/sync/decisions.md", "skills/pem-local-first/references/decisions.md"],
  ["references/tauri/patterns.md", "skills/pem-local-first/references/tauri-patterns.md"],
  ["references/rust/patterns.md", "skills/pem-local-first/references/rust-patterns.md"],
  ["references/sync/doctrine.md", "skills/sync-doctrine/references/doctrine.md"],
  ["references/sync/partial-replication.md", "skills/sync-doctrine/references/partial-replication.md"],
  ["references/sync/decisions.md", "skills/sync-doctrine/references/decisions.md"],
];

async function files(path) {
  const info = await stat(path);
  if (info.isFile()) return [path];
  const { readdir } = await import("node:fs/promises");
  const entries = await readdir(path, { withFileTypes: true });
  const nested = await Promise.all(
    entries
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((entry) => files(join(path, entry.name))),
  );
  return nested.flat();
}

async function equal(source, target) {
  try {
    const sourceInfo = await stat(source);
    const targetInfo = await stat(target);
    if (sourceInfo.isFile() !== targetInfo.isFile()) return false;
    if (sourceInfo.isFile()) {
      return (await readFile(source)).equals(await readFile(target));
    }
    const sourceFiles = await files(source);
    const targetFiles = await files(target);
    const sourceRelative = sourceFiles.map((path) => relative(source, path));
    const targetRelative = targetFiles.map((path) => relative(target, path));
    if (JSON.stringify(sourceRelative) !== JSON.stringify(targetRelative)) return false;
    for (let index = 0; index < sourceFiles.length; index += 1) {
      if (!(await readFile(sourceFiles[index])).equals(await readFile(targetFiles[index]))) {
        return false;
      }
    }
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

let drift = false;
for (const [sourceName, targetName] of mappings) {
  const source = join(root, sourceName);
  const target = join(root, targetName);
  if (await equal(source, target)) continue;
  drift = true;
  if (checkOnly) {
    process.stderr.write(`skill resource drift: ${targetName}\n`);
    continue;
  }
  await rm(target, { recursive: true, force: true });
  await mkdir(dirname(target), { recursive: true });
  await cp(source, target, { recursive: true });
  process.stdout.write(`generated ${targetName}\n`);
}

if (checkOnly && drift) process.exitCode = 1;
