import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fixture } from './lib/kbd-phase-closure-adapter.mjs';

test('control doctor distinguishes standalone projects, unreachable sockets and existing rollouts', {
  skip: process.platform === 'win32' && 'The Unix control socket is a POSIX transport',
}, t => {
  const binary = fs.realpathSync(process.env.PROMETHEUS_CLI_TEST_BINARY ?? 'tools/prometheus-cli/target/release/prometheus');
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'kbd-control-doctor-'));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const { project, env } = fixture(scratch, 'local-project', binary);
  delete env.PROMETHEUS_CONTROL_ENDPOINT;

  const run = (args, overrides = {}) => execFileSync(binary, args, {
    cwd: project, env: { ...env, ...overrides }, encoding: 'utf8', timeout: 30000,
  });
  const control = overrides => {
    const { checks } = JSON.parse(run(['doctor', '--json', '--check', 'control'], overrides));
    return Object.fromEntries(checks.map(check => [check.id, check]));
  };

  const standalone = control({});
  assert.equal(standalone['control.kbd-runtime'].status, 'skip');
  assert.equal(standalone['control.kbd-rollout'].status, 'skip');

  const socket = path.join(scratch, 'unbound.sock');
  const configured = { SOVEREIGN_SYNC_SOCKET: socket };
  const contract = JSON.parse(run(['contract', 'show', '--json'], configured));
  assert.equal(contract.endpoint, socket);
  assert.equal(contract.endpoint_source, 'env:SOVEREIGN_SYNC_SOCKET');
  const unreachable = control(configured);
  assert.equal(unreachable['control.kbd-runtime'].status, 'warn');
  assert.ok(unreachable['control.kbd-runtime'].details.some(detail => detail.includes(socket)));
  assert.equal(unreachable['control.kbd-rollout'].status, 'warn');

  // Starting a rollout preserves its gates even after removing explicit control settings.
  run(['kbd', '--path', project, 'rollout', 'observe', '--observation-id', 'local-inspection']);
  const started = control({});
  assert.equal(started['control.kbd-runtime'].status, 'skip');
  assert.equal(started['control.kbd-rollout'].status, 'warn');
  assert.match(started['control.kbd-rollout'].summary, /shadow.*blocked/i);
});
