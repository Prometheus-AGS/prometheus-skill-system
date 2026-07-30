#!/usr/bin/env node

import { createHash } from 'crypto';
import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  rmSync,
  writeFileSync,
} from 'fs';
import { dirname, join, relative, resolve } from 'path';
import { fileURLToPath } from 'url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const defaultRepoRoot = resolve(scriptDir, '..');

function option(args, name, fallback) {
  const index = args.indexOf(name);
  return index >= 0 && args[index + 1] ? resolve(args[index + 1]) : fallback;
}

function parseFrontmatter(skillMd) {
  const source = readFileSync(skillMd, 'utf8');
  const frontmatter = source.match(/^---\n([\s\S]*?)\n---/);
  const body = frontmatter?.[1] ?? '';
  const name = body.match(/^name:\s*['"]?([^'"\n]+)['"]?/m)?.[1]?.trim();
  const version = body.match(/^version:\s*['"]?([^'"\n]+)['"]?/m)?.[1]?.trim();
  return {
    name: name || skillMd.split('/').at(-2),
    version: version || '1.0.0',
  };
}

function collectSkills(repoRoot) {
  const skillsRoot = join(repoRoot, 'skills');
  const skills = [];

  function walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      const rel = relative(skillsRoot, path).split('\\').join('/');
      if (rel === 'imported' || rel.startsWith('imported/')) continue;
      // Skip test fixtures. A skill's tests/ may hold deliberately BROKEN
      // SKILL.md trees used to prove a review gate discriminates (see
      // skills/process/adversarial-review/tests/fixtures/). Without this,
      // `flawed-skill` installs as a real, invocable MiniMax skill.
      if (entry.isDirectory() && (entry.name === 'tests' || entry.name === 'fixtures')) continue;
      if (entry.isDirectory()) walk(path);
      else if (entry.name === 'SKILL.md') {
        skills.push({ dir, ...parseFrontmatter(path) });
      }
    }
  }

  walk(skillsRoot);
  skills.sort((a, b) => a.name.localeCompare(b.name));

  const seen = new Map();
  for (const skill of skills) {
    if (seen.has(skill.name)) {
      throw new Error(
        `duplicate skill name '${skill.name}': ${seen.get(skill.name)} and ${skill.dir}`
      );
    }
    seen.set(skill.name, skill.dir);
  }
  return skills;
}

function isPackOwned(dest) {
  const metaPath = join(dest, '_meta.json');
  if (!existsSync(metaPath)) return false;
  try {
    return JSON.parse(readFileSync(metaPath, 'utf8')).platform === 'minimax';
  } catch {
    return false;
  }
}

function removeLegacyBundle(targetDir, repoRoot) {
  const legacy = join(targetDir, 'prometheus-skill-pack');
  if (!existsSync(legacy) && !lstatSafe(legacy)) return false;
  try {
    if (!lstatSync(legacy).isSymbolicLink()) return false;
    const target = resolve(dirname(legacy), readlinkSync(legacy));
    if (target !== repoRoot) return false;
    rmSync(legacy);
    return true;
  } catch {
    return false;
  }
}

function lstatSafe(path) {
  try {
    return lstatSync(path);
  } catch {
    return null;
  }
}

function main() {
  const args = process.argv.slice(2);
  const repoRoot = option(args, '--repo-root', defaultRepoRoot);
  const targetDir = option(args, '--target-dir', join(process.env.HOME, '.minimax', 'skills'));
  const uninstall = args.includes('--uninstall');
  const quiet = args.includes('--quiet');
  const json = args.includes('--json');
  const skills = collectSkills(repoRoot);

  mkdirSync(targetDir, { recursive: true });
  const legacyRemoved = removeLegacyBundle(targetDir, repoRoot);
  let installed = 0;
  let removed = 0;
  let skipped = 0;

  for (const skill of skills) {
    const preferredDest = join(targetDir, skill.name);
    const fallbackDest = join(targetDir, `prometheus-${skill.name}`);

    if (uninstall) {
      for (const dest of [preferredDest, fallbackDest]) {
        const destStat = lstatSafe(dest);
        if (destStat?.isDirectory() && isPackOwned(dest)) {
          rmSync(dest, { recursive: true, force: true });
          removed += 1;
        }
      }
      continue;
    }

    let dest = preferredDest;
    if (lstatSafe(preferredDest) && !isPackOwned(preferredDest)) {
      dest = fallbackDest;
      if (!quiet) {
        console.warn(
          `  [collision] preserved ${preferredDest}; installing pack payload as prometheus-${skill.name}`
        );
      }
    }
    if (lstatSafe(dest) && !isPackOwned(dest)) {
      if (!quiet) console.warn(`  [skip] ${dest} also exists and is not pack-owned`);
      skipped += 1;
      continue;
    }

    rmSync(dest, { recursive: true, force: true });
    cpSync(skill.dir, dest, { recursive: true, preserveTimestamps: true });
    const numericId = Number.parseInt(
      createHash('md5').update(skill.name).digest('hex').slice(0, 8),
      16
    );
    writeFileSync(
      join(dest, '_meta.json'),
      JSON.stringify(
        {
          id: numericId,
          version: skill.version,
          name: skill.name,
          updated_at: Date.now(),
          platform: 'minimax',
        },
        null,
        2
      ) + '\n'
    );
    installed += 1;
  }

  const summary = {
    platform: 'minimax',
    target_dir: targetDir,
    discovered: skills.length,
    installed,
    removed,
    skipped,
    legacy_bundle_removed: legacyRemoved,
  };
  if (json) console.log(JSON.stringify(summary));
  else if (!quiet) {
    const action = uninstall ? `${removed} skills removed` : `${installed} skills installed`;
    console.log(`  ✅ minimax: ${action} as complete copies (${skipped} skipped)`);
  }
}

try {
  main();
} catch (error) {
  console.error(`install-minimax-skills: ${error.message}`);
  process.exit(1);
}
