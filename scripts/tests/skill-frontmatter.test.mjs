/**
 * Fixtures for the shared SKILL.md frontmatter parser.
 *
 * The regression this exists for: a negative lookahead placed after `\s*` is
 * evaluated at the position after the colon -- where the next character is a
 * space, not `>` -- so a folded `description: >` was captured as the literal
 * string ">" and the folded branch was never reached. 231 of 232 skills using
 * a folded description were affected, and because that description feeds
 * `searchText` in the skill index, those skills shipped undiscoverable.
 */

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseFrontmatter, parseSkillFrontmatter } from '../lib/skill-frontmatter.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const results = [];
const check = (name, body) => {
  body();
  results.push(name);
};

const skill = (body) => `---\n${body}\n---\n\n# Heading\n`;

check('a folded description is read, not captured as ">"', () => {
  const parsed = parseSkillFrontmatter(
    skill('name: demo\ndescription: >\n  First line of the description.\n  Second line of it.')
  );
  assert.equal(parsed.name, 'demo');
  assert.equal(parsed.description, 'First line of the description. Second line of it.');
  assert.notEqual(parsed.description, '>');
});

check('every block scalar introducer is handled', () => {
  // `>` folds, `|` keeps newlines, and both take chomping and indent hints.
  for (const introducer of ['>', '|', '>-', '|-', '>+', '|+', '>2', '|2']) {
    const parsed = parseSkillFrontmatter(
      skill(`name: demo\ndescription: ${introducer}\n  Body text here.`)
    );
    assert.equal(parsed.description, 'Body text here.', `failed for ${introducer}`);
  }
});

check('an inline description is unchanged', () => {
  assert.equal(
    parseSkillFrontmatter(skill('name: demo\ndescription: A plain inline description.')).description,
    'A plain inline description.'
  );
});

check('quotes are stripped, and only matching ones', () => {
  assert.equal(parseFrontmatter('description: "quoted value"').description, 'quoted value');
  assert.equal(parseFrontmatter("description: 'quoted value'").description, 'quoted value');
  // A description that merely contains an apostrophe keeps it.
  assert.equal(
    parseFrontmatter("description: it's fine").description,
    "it's fine"
  );
});

check('a description containing > is not mistaken for a block scalar', () => {
  assert.equal(
    parseFrontmatter('description: use a > b to compare').description,
    'use a > b to compare'
  );
});

check('a blank line inside a block scalar is a paragraph break', () => {
  const parsed = parseFrontmatter('description: >\n  First paragraph.\n\n  Second paragraph.');
  assert.equal(parsed.description, 'First paragraph. Second paragraph.');
});

check('the block body stops at the next key', () => {
  const parsed = parseFrontmatter(
    'description: >\n  Body of the description.\nlicense: MIT\nname: demo'
  );
  assert.equal(parsed.description, 'Body of the description.');
  assert.equal(parsed.name, 'demo');
});

check('a missing description is empty, not undefined', () => {
  assert.equal(parseFrontmatter('name: demo').description, '');
  assert.equal(parseFrontmatter('').name, '');
});

// ---------------------------------------------------------------------------
// Against the real corpus, which is where the defect actually lived
// ---------------------------------------------------------------------------

function walk(directory, out = []) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) walk(absolute, out);
    else if (entry.name === 'SKILL.md') out.push(absolute);
  }
  return out;
}

check('no skill in the repository parses to a bare block-scalar introducer', () => {
  const broken = [];
  let folded = 0;
  for (const file of walk(path.join(repoRoot, 'skills'))) {
    const text = fs.readFileSync(file, 'utf8');
    if (/^description:[ \t]*[>|]/m.test(text.replace(/\r\n/g, '\n'))) folded += 1;
    const { description } = parseSkillFrontmatter(text);
    if (/^[>|][-+]?\d*$/.test(description.trim())) {
      broken.push(path.relative(repoRoot, file));
    }
  }
  assert.ok(folded > 100, `expected the folded corpus to be large, saw ${folded}`);
  assert.deepEqual(broken, [], `${broken.length} skills still parse as a bare introducer`);
  results.push(`  (checked ${folded} folded descriptions)`);
});

process.stdout.write(`skill-frontmatter: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
