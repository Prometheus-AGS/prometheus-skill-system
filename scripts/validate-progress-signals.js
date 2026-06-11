#!/usr/bin/env node

/**
 * Validates that process skills declare Progress Signals.
 *
 * The progress-signal rule (CLAUDE.md → "Progress Signaling (MANDATORY)") was
 * prose-only; this lint makes it a merge gate. Every SKILL.md under
 * skills/process/ must contain a "## Progress Signals" section (or an explicit
 * Starting/Completed signal declaration).
 *
 * Ratchet baseline: skills that predate the gate are listed in
 * scripts/progress-signals-baseline.json. The baseline can only shrink —
 * a baselined skill that gains signals must be removed from the baseline,
 * and new skills can never be added to it (edit the skill, not the list).
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..');
const PROCESS_DIR = path.join(REPO_ROOT, 'skills', 'process');
const BASELINE_PATH = path.join(__dirname, 'progress-signals-baseline.json');

const SIGNAL_HEADING = /^##\s+Progress Signals/m;
const SIGNAL_DECLARATION = /^(Starting|Completed)\s+\S[^\n]*(—|--|\bout of\b|\bof\b)/m;

async function findSkillFiles(dir) {
  const out = [];
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name === 'node_modules' || entry.name.startsWith('.')) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await findSkillFiles(full)));
    } else if (entry.isFile() && entry.name === 'SKILL.md') {
      out.push(full);
    }
  }
  return out;
}

function hasSignals(content) {
  return SIGNAL_HEADING.test(content) || SIGNAL_DECLARATION.test(content);
}

async function main() {
  let baseline = [];
  try {
    baseline = JSON.parse(await fs.readFile(BASELINE_PATH, 'utf8'));
  } catch {
    // No baseline file → empty baseline (full enforcement).
  }
  const baselineSet = new Set(baseline);

  const files = await findSkillFiles(PROCESS_DIR);
  const errors = [];
  const seenBaselined = new Set();

  for (const file of files) {
    const rel = path.relative(REPO_ROOT, file);
    const content = await fs.readFile(file, 'utf8');
    const ok = hasSignals(content);

    if (baselineSet.has(rel)) {
      seenBaselined.add(rel);
      if (ok) {
        errors.push(
          `${rel}: now declares Progress Signals — remove it from scripts/progress-signals-baseline.json (ratchet only shrinks)`
        );
      }
      continue;
    }

    if (!ok) {
      errors.push(
        `${rel}: missing "## Progress Signals" section. Declare the Starting/Completed signal contract (see CLAUDE.md → Progress Signaling). New skills may not be added to the baseline.`
      );
    }
  }

  // Baseline entries that no longer exist on disk are stale.
  for (const rel of baselineSet) {
    if (!seenBaselined.has(rel)) {
      errors.push(
        `${rel}: listed in progress-signals-baseline.json but not found on disk — remove the stale entry`
      );
    }
  }

  const checked = files.length;
  const exempt = seenBaselined.size;
  if (errors.length > 0) {
    console.error(`✗ progress-signal lint failed (${errors.length} problem(s)):\n`);
    for (const err of errors) console.error(`  - ${err}`);
    console.error(
      `\nChecked ${checked} process skills (${exempt} baselined/exempt).`
    );
    process.exit(1);
  }
  console.log(
    `✓ progress-signal lint passed — ${checked} process skills checked, ${exempt} baselined (ratchet)`
  );
}

main().catch((err) => {
  console.error('validate-progress-signals failed:', err);
  process.exit(1);
});
