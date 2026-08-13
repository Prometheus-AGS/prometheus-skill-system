#!/usr/bin/env node
/**
 * Completeness gate: every skill, at every target, resolving to current content.
 *
 * WHY THIS EXISTS (2026-08-13)
 * ---------------------------
 * The installer verified the wrong thing. When a skill's canonical name was
 * occupied, placement diverted to `prometheus-<name>` and `verifyTargets()`
 * re-derived the SAME fallback path and validated *that* — so a run in which 19
 * skills were unreachable at the name every tool searches still printed
 * "Verified immutable generation installed to all supported user targets."
 *
 * An operator shipped believing all skills were available. A Codex session
 * blocked because `deep-research` was not where it looked.
 *
 * The deeper failure was in how completeness was checked at all: a loop scoped
 * to whichever directory happened to be under investigation, reported as though
 * it covered the whole. This gate is exhaustive BY CONSTRUCTION —
 * `<skills> x <targets>`, enumerated from the generation, denominator always
 * printed. It cannot be run against a subset by accident.
 *
 * WHAT IT CHECKS
 *   link targets : canonical name is a symlink resolving INTO the active
 *                  generation for that exact skill. A symlink into a source
 *                  checkout is NOT acceptable — that is how `artifact-refiner`
 *                  served April content while the generation held August.
 *   copy targets : canonical name exists, and EVERY file under it matches the
 *                  generation by hash. The old check hashed SKILL.md only, so a
 *                  half-copied skill passed.
 *
 * EXIT
 *   0  every placement resolves to current content
 *   1  at least one does not (each listed)
 *   2  cannot determine (no generation) — never silently pass
 */

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';

const TARGETS = [
  '.claude/skills',
  '.opencode/skills',
  '.kimi-code/skills',
  '.minimax/skills',
  '.cursor/skills',
  '.codex/skills',
  '.gemini/skills',
  '.roo/skills',
  '.windsurf/skills',
  '.codeium/windsurf/skills',
  '.agents/skills',
  '.config/zed/skills',
  '.zed/skills',
  '.cline/skills',
];
const COPY_TARGETS = new Set(['.minimax/skills', '.codex/skills']);

const HOME = process.env.VERIFY_HOME || os.homedir();
const PLUGIN_ROOT =
  process.env.VERIFY_PLUGIN_ROOT || path.join(HOME, '.prometheus/plugins/prometheus-skill-pack');
const CURRENT = path.join(PLUGIN_ROOT, 'current');
const JSON_OUT = process.argv.includes('--json');

const sha = (p) => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');

function filesUnder(root) {
  const out = [];
  const walk = (dir, rel = '') => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      // Installer-written receipts are not generation payload; skip them.
      if (rel === '' && (e.name === '.prometheus-generation' || e.name === '_meta.json')) continue;
      if (e.name === '.prometheus-pack') continue;
      const abs = path.join(dir, e.name);
      const r = rel ? `${rel}/${e.name}` : e.name;
      if (e.isDirectory()) walk(abs, r);
      else if (e.isFile()) out.push(r);
    }
  };
  walk(root);
  return out.sort();
}

/** The denominator: every generation directory carrying a SKILL.md. */
function generationSkills() {
  const skillsRoot = path.join(CURRENT, 'skills');
  if (!fs.existsSync(skillsRoot)) return null;
  return fs
    .readdirSync(skillsRoot, { withFileTypes: true })
    .filter((e) => e.isDirectory() && fs.existsSync(path.join(skillsRoot, e.name, 'SKILL.md')))
    .map((e) => e.name)
    .sort();
}

function checkLink(targetRoot, skill) {
  const p = path.join(targetRoot, skill);
  const st = fs.lstatSync(p, { throwIfNoEntry: false });
  if (!st) return 'absent';
  if (!st.isSymbolicLink()) {
    return fs.existsSync(path.join(p, '.git')) ? 'foreign git checkout' : 'unowned directory';
  }
  const resolved = path.resolve(path.dirname(p), fs.readlinkSync(p));
  if (!fs.existsSync(resolved)) return `dangling -> ${fs.readlinkSync(p)}`;
  // Must point INTO the plugin root, at this exact skill. A link to a source
  // checkout resolves fine and serves stale content — the failure that made
  // `artifact-refiner` four months out of date across six targets.
  const expected = path.join(CURRENT, 'skills', skill);
  if (path.resolve(resolved) !== path.resolve(expected))
    return `points outside the active generation -> ${fs.readlinkSync(p)}`;
  return null;
}

function checkCopy(targetRoot, skill) {
  const dst = path.join(targetRoot, skill);
  const src = path.join(CURRENT, 'skills', skill);
  if (!fs.existsSync(dst)) return 'absent';
  const st = fs.lstatSync(dst);
  if (st.isSymbolicLink()) {
    const resolved = path.resolve(path.dirname(dst), fs.readlinkSync(dst));
    if (path.resolve(resolved) !== path.resolve(src))
      return `symlink outside the active generation -> ${fs.readlinkSync(dst)}`;
    return null;
  }
  const want = filesUnder(src);
  for (const rel of want) {
    const f = path.join(dst, rel);
    if (!fs.existsSync(f)) return `missing file: ${rel}`;
    if (sha(f) !== sha(path.join(src, rel))) return `stale content: ${rel}`;
  }
  return null;
}

const skills = generationSkills();
if (!skills) {
  process.stderr.write(`verify-skill-install: no generation at ${CURRENT}\n`);
  process.exit(2);
}

const failures = [];
let checked = 0;
for (const target of TARGETS) {
  const targetRoot = path.join(HOME, ...target.split('/'));
  if (!fs.existsSync(targetRoot)) {
    failures.push({ target, skill: '(all)', reason: 'target directory does not exist' });
    continue;
  }
  for (const skill of skills) {
    checked += 1;
    const reason = COPY_TARGETS.has(target)
      ? checkCopy(targetRoot, skill)
      : checkLink(targetRoot, skill);
    if (reason) failures.push({ target, skill, reason });
  }
}

const total = skills.length * TARGETS.length;
if (JSON_OUT) {
  process.stdout.write(
    `${JSON.stringify({ skills: skills.length, targets: TARGETS.length, checked, total, failures }, null, 2)}\n`
  );
} else {
  process.stdout.write(
    `verify-skill-install: ${checked - failures.length}/${total} placements current ` +
      `(${skills.length} skills x ${TARGETS.length} targets)\n`
  );
  for (const f of failures) {
    process.stdout.write(`  FAIL  ~/${f.target}/${f.skill}\n        ${f.reason}\n`);
  }
  if (failures.length)
    process.stdout.write(
      `\n${failures.length} placement(s) not current. Reinstall:\n` +
        `  node scripts/install.js --scope user\n`
    );
}
process.exit(failures.length ? 1 : 0);
