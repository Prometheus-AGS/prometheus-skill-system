/** The runtime-only Claude plugin: hooks and MCP, no skills, dispatching from an installed generation. */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const full = path.join(root, 'dist/plugins/claude/prometheus-skill-pack');
const runtime = path.join(root, 'dist/plugins/claude/prometheus-runtime');
const FAILURE_MARKERS = [/ERR_MODULE_NOT_FOUND/, /Cannot find module/, /HOOK_RUNTIME_ERROR/, /NOT_ACTIVATED/];

// 1. Shape: the same hooks as the full plugin, an MCP config, and nothing a harness would list as a skill.
const manifest = JSON.parse(fs.readFileSync(path.join(runtime, '.claude-plugin/plugin.json'), 'utf8'));
assert.equal(manifest.name, 'prometheus-runtime');
assert(!('skills' in manifest), 'the runtime plugin declares no skills');
assert.equal(
  fs.readFileSync(path.join(runtime, 'hooks/hooks.json'), 'utf8'),
  fs.readFileSync(path.join(full, 'hooks/hooks.json'), 'utf8'),
  'the runtime plugin carries the full plugin\'s hook file unchanged'
);
assert(fs.existsSync(path.join(runtime, '.mcp.json')), 'the runtime plugin carries the MCP config');
const skillFiles = [];
(function walk(directory) {
  for (const name of fs.readdirSync(directory)) {
    const entry = path.join(directory, name);
    if (fs.lstatSync(entry).isDirectory()) walk(entry);
    else if (name === 'SKILL.md') skillFiles.push(path.relative(runtime, entry));
  }
})(runtime);
assert.deepEqual(skillFiles, [], 'the runtime plugin ships no SKILL.md');
const marketplace = JSON.parse(fs.readFileSync(path.join(root, '.claude-plugin/marketplace.json'), 'utf8'));
assert(marketplace.plugins.some(entry => entry.name === 'prometheus-runtime'), 'the marketplace offers the runtime plugin');

// 2. Behaviour: with a generation installed from the full payload, every hook
//    the runtime plugin declares dispatches when IT is the plugin root.
const base = fs.mkdtempSync(path.join(os.tmpdir(), 'runtime-plugin-'));
try {
  const home = path.join(base, 'home');
  const project = path.join(base, 'project');
  const payload = path.join(base, 'cache/prometheus-skill-pack');
  const plugin = path.join(base, 'cache/prometheus-runtime');
  for (const directory of [home, project, path.dirname(payload)]) fs.mkdirSync(directory, { recursive: true });
  fs.cpSync(full, payload, { recursive: true, verbatimSymlinks: true });
  fs.cpSync(runtime, plugin, { recursive: true, verbatimSymlinks: true });
  const env = {
    ...process.env, HOME: home, CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
    CLAUDE_PLUGIN_ROOT: plugin, PROMETHEUS_PLUGIN_ROOT: path.join(home, '.prometheus/plugins/prometheus-skill-pack'),
    XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'), PYTHONDONTWRITEBYTECODE: '0',
  };
  delete env.PROMETHEUS_SKILL_DOMAINS;
  const installed = spawnSync(process.execPath, [path.join(payload, 'scripts/install-plugin-generation.js'),
    '--source-root', payload, '--home', home, '--targets', 'all'], { env, cwd: project, encoding: 'utf8', timeout: 240_000 });
  assert.equal(installed.status, 0, installed.stderr);

  const hooks = JSON.parse(fs.readFileSync(path.join(plugin, 'hooks/hooks.json'), 'utf8')).hooks;
  const failures = [];
  let count = 0;
  for (const [event, entries] of Object.entries(hooks)) {
    for (const entry of entries) {
      for (const hook of entry.hooks ?? []) {
        if (hook.type !== 'command') continue;
        count += 1;
        const args = (hook.args ?? []).map(text => text.replaceAll('${CLAUDE_PLUGIN_ROOT}', plugin));
        const run = spawnSync(hook.command, args, {
          cwd: project, env, encoding: 'utf8', timeout: (hook.timeout ?? 90) * 1000,
          input: JSON.stringify({ session_id: 'runtime-plugin', cwd: project, hook_event_name: event, stop_hook_active: false,
            source: 'startup', prompt: 'hello', tool_name: 'Bash', tool_input: { command: 'true' } }),
        });
        const output = `${run.stdout ?? ''}\n${run.stderr ?? ''}`;
        const marker = FAILURE_MARKERS.find(pattern => pattern.test(output));
        if (run.status !== 0 || run.error || marker) {
          failures.push(`${event}/${args[args.indexOf('--hook') + 1]}: exit ${run.status} ${marker?.source ?? ''} ${output.slice(0, 300)}`);
        }
      }
    }
  }
  assert(count >= 30, `expected the full hook set, ran ${count}`);
  assert.deepEqual(failures, [], 'hooks failed from the runtime plugin');
  console.log(`PASS: the runtime plugin ships no skills and dispatches all ${count} hooks from an installed generation`);
} finally {
  fs.rmSync(base, { recursive: true, force: true });
}
