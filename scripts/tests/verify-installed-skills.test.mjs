#!/usr/bin/env node

import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const verifier = join(repository, 'scripts/verify-installed-skills.js');
const fixture = mkdtempSync(join(tmpdir(), 'verify-installed-skills-'));
const fixtureRepository = join(fixture, 'repository');
const fixtureHome = join(fixture, 'home');

function write(path, contents) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
}

try {
  const liveSkill = join(fixtureRepository, 'skills/process/live-skill');
  write(join(liveSkill, 'SKILL.md'), '---\nname: live-skill\n---\n# Live skill\n');
  write(
    join(
      fixtureRepository,
      'skills/process/reviewer/tests/fixtures/flawed-skill/SKILL.md'
    ),
    '---\nname: flawed-skill\n---\n# Test fixture\n'
  );
  write(
    join(fixtureRepository, 'skills/process/reviewer/fixtures/clean-skill/SKILL.md'),
    '---\nname: clean-skill\n---\n# Test fixture\n'
  );

  const installed = join(fixtureHome, '.minimax/skills/live-skill');
  write(join(installed, 'SKILL.md'), '---\nname: live-skill\n---\n# Live skill\n');
  write(join(installed, '_meta.json'), '{"platform":"minimax"}\n');

  const output = execFileSync(
    process.execPath,
    [verifier, '--platform', 'minimax', '--json'],
    {
      env: {
        ...process.env,
        VERIFY_REPO_ROOT: fixtureRepository,
        VERIFY_HOME: fixtureHome,
      },
      encoding: 'utf8',
    }
  );
  const result = JSON.parse(output);
  if (!result.ok || result.skills_discovered !== 1) {
    throw new Error(`fixture directories entered the install contract: ${output}`);
  }
  if (result.results[0]?.payloads_verified !== 1 || result.results[0]?.failures.length !== 0) {
    throw new Error(`installed MiniMax payload was not verified: ${output}`);
  }

  console.log('installed skill verifier: test and fixture trees are excluded consistently');
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
