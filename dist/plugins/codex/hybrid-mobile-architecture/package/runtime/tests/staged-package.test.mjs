import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, existsSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const run = (file, args = [], cwd = root) => spawnSync(process.execPath, [file, ...args], { cwd, encoding: 'utf8', shell: false });
test('full and mini stage identical isolated payloads that install and uninstall without source dependencies', t => {
  const work = mkdtempSync(join(tmpdir(), 'builder-staging ü-')); t.after(() => rmSync(work, { recursive: true, force: true }));
  const receipts = [];
  for (const variant of ['full', 'mini']) {
    const output = join(work, variant), staged = join(output, 'package'), project = join(work, `${variant} application`);
    let result = run(join(root, 'scripts/stage-skill-package.mjs'), ['--variant', variant, '--output', output]);
    assert.equal(result.status, 0, result.stderr);
    receipts.push(JSON.parse(readFileSync(join(output, 'receipt.json'), 'utf8')));
    assert.equal(existsSync(join(staged, 'node_modules')), false);
    result = run(join(staged, 'scripts/verify-skill-manifest.mjs'), [staged]); assert.equal(result.status, 0, result.stderr);
    mkdirSync(project); mkdirSync(join(project, '.agents/skills/user-skill'), { recursive: true });
    writeFileSync(join(project, '.agents/skills/user-skill/SKILL.md'), 'user-owned');
    const installer = join(staged, 'scripts/install-harness-package.mjs');
    result = run(installer, ['--harness', 'opencode', '--scope', 'project'], project); assert.equal(result.status, 0, result.stderr);
    assert.ok(existsSync(join(project, '.opencode/plugins/knowme-builder.mjs')));
    assert.ok(existsSync(join(project, '.opencode/commands/knowme-builder-new.md')));
    result = run(installer, ['--harness', 'opencode', '--scope', 'project', '--uninstall'], project); assert.equal(result.status, 0, result.stderr);
    assert.equal(readFileSync(join(project, '.agents/skills/user-skill/SKILL.md'), 'utf8'), 'user-owned');
  }
  assert.equal(receipts[0].payloadSha256, receipts[1].payloadSha256);
  assert.deepEqual(receipts[0].files, receipts[1].files);
});
test('staging rejects a source child beginning with two dots before creating it', () => {
  const target = join(root, '..payload-portability-test');
  assert.equal(existsSync(target), false);
  const result = run(join(root, 'scripts/stage-skill-package.mjs'), ['--variant', 'mini', '--output', target]);
  assert.notEqual(result.status, 0); assert.match(result.stderr, /outside the source checkout/);
  assert.equal(existsSync(target), false);
});
