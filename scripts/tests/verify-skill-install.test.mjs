#!/usr/bin/env node
/**
 * Red/green tests for the skill-install completeness gate.
 *
 * A gate that has only ever passed is indistinguishable from one that always
 * passes. On 2026-08-13 an installer printed
 * "Verified … installed to all supported user targets" while 19 skills were
 * unreachable at their canonical names, and a hand-written freshness check
 * reported "0 drift" from a 25-of-163 sample. Both were green and both were
 * wrong.
 *
 * Every failure mode below is planted in an isolated fixture, observed failing,
 * then repaired and observed passing. Nothing here touches the live install.
 */

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const GATE = path.join(REPO, 'scripts/verify-skill-install.js');
const PLUGIN_ROOT = path.join(os.homedir(), '.prometheus/plugins/prometheus-skill-pack');
const GEN = path.join(PLUGIN_ROOT, 'current/skills');

const TARGETS = [
  '.claude/skills',
  '.opencode/skills',
  '.kimi-code/skills',
  '.minimax/skills',
  '.cursor/skills',
  '.codex/skills',
  '.gemini/skills',
  '.roo/skills',
  '.devin/skills',
  '.codeium/windsurf/skills',
  '.agents/skills',
  '.config/zed/skills',
  '.zed/skills',
  '.cline/skills',
];
const COPY = new Set(['.minimax/skills', '.codex/skills']);

if (!fs.existsSync(GEN)) {
  process.stderr.write(`SKIP: no installed generation at ${GEN}\n`);
  process.exit(0);
}

const skills = fs
  .readdirSync(GEN, { withFileTypes: true })
  .filter((e) => e.isDirectory() && fs.existsSync(path.join(GEN, e.name, 'SKILL.md')))
  .map((e) => e.name);

const home = fs.mkdtempSync(path.join(os.tmpdir(), 'skillgate-'));
for (const t of TARGETS) {
  const root = path.join(home, ...t.split('/'));
  fs.mkdirSync(root, { recursive: true });
  for (const s of skills) {
    if (COPY.has(t)) fs.cpSync(path.join(GEN, s), path.join(root, s), { recursive: true });
    else fs.symlinkSync(path.join(GEN, s), path.join(root, s));
  }
}

const run = () => {
  try {
    execFileSync(process.execPath, [GATE], {
      env: { ...process.env, VERIFY_HOME: home, VERIFY_PLUGIN_ROOT: PLUGIN_ROOT },
      stdio: 'pipe',
    });
    return { code: 0, out: '' };
  } catch (e) {
    return { code: e.status, out: `${e.stdout}${e.stderr}` };
  }
};

let pass = 0;
let fail = 0;
const check = (name, ok, detail = '') => {
  if (ok) {
    pass += 1;
    process.stdout.write(`  PASS  ${name}\n`);
  } else {
    fail += 1;
    process.stdout.write(`  FAIL  ${name}${detail ? ` — ${detail}` : ''}\n`);
  }
};

process.stdout.write(`skill-install gate: ${skills.length} skills x ${TARGETS.length} targets\n`);

// GREEN — a complete, correct install must pass. A gate that refuses everything
// is as useless as one that refuses nothing.
check('complete install passes', run().code === 0);

const cases = [
  {
    name: 'unowned directory at canonical name',
    at: () => path.join(home, '.claude/skills/deep-research'),
    break: (p) => {
      fs.rmSync(p);
      fs.mkdirSync(p);
    },
    repair: (p) => {
      fs.rmSync(p, { recursive: true });
      fs.symlinkSync(path.join(GEN, 'deep-research'), p);
    },
    expect: /unowned directory/,
  },
  {
    name: 'skill absent from a target',
    at: () => path.join(home, '.roo/skills/kbd-assess'),
    break: (p) => fs.rmSync(p),
    repair: (p) => fs.symlinkSync(path.join(GEN, 'kbd-assess'), p),
    expect: /absent/,
  },
  {
    // The failure that served four-month-old artifact-refiner across six
    // targets: a symlink that resolves fine, but into a source checkout.
    name: 'symlink resolves outside the active generation',
    at: () => path.join(home, '.cursor/skills/artifact-refiner'),
    break: (p) => {
      fs.rmSync(p);
      fs.symlinkSync(path.join(REPO, 'skills/imported/artifact-refiner'), p);
    },
    repair: (p) => {
      fs.rmSync(p);
      fs.symlinkSync(path.join(GEN, 'artifact-refiner'), p);
    },
    expect: /outside the active generation/,
  },
  {
    name: 'dangling symlink',
    at: () => path.join(home, '.gemini/skills/kbd-plan'),
    break: (p) => {
      fs.rmSync(p);
      fs.symlinkSync(path.join(GEN, 'no-such-skill-xyz'), p);
    },
    repair: (p) => {
      fs.rmSync(p);
      fs.symlinkSync(path.join(GEN, 'kbd-plan'), p);
    },
    expect: /dangling/,
  },
  {
    // Old check hashed SKILL.md only, so a half-copied skill passed.
    name: 'copy target stale in a non-SKILL.md file',
    at: () => path.join(home, '.codex/skills/deep-research'),
    break: (p) => {
      const f = fs
        .readdirSync(path.join(p, 'references'), { withFileTypes: true })
        .find((e) => e.isFile());
      fs.appendFileSync(path.join(p, 'references', f.name), '\nDRIFT\n');
    },
    repair: (p) => {
      fs.rmSync(p, { recursive: true });
      fs.cpSync(path.join(GEN, 'deep-research'), p, { recursive: true });
    },
    expect: /stale content/,
  },
  {
    name: 'copy target missing a file',
    at: () => path.join(home, '.minimax/skills/kbd-assess'),
    break: (p) => fs.rmSync(path.join(p, 'SKILL.md')),
    repair: (p) => {
      fs.rmSync(p, { recursive: true });
      fs.cpSync(path.join(GEN, 'kbd-assess'), p, { recursive: true });
    },
    expect: /missing file/,
  },
];

for (const c of cases) {
  const p = c.at();
  c.break(p);
  const red = run();
  check(`RED  ${c.name}`, red.code === 1 && c.expect.test(red.out), `exit=${red.code}`);
  c.repair(p);
  check(`GREEN ${c.name} repaired`, run().code === 0);
}

fs.rmSync(home, { recursive: true, force: true });
process.stdout.write(`\n  ${pass} passed, ${fail} failed\n`);
process.exit(fail ? 1 : 0);
