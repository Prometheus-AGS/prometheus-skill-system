// TJ-ARCH-MOB-001 compliant
import { mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { main, packageRoot, run } from './portable/platform.mjs';
await main(() => {
  const [operation, ...extra] = process.argv.slice(2); if (extra.length) throw new Error('Unexpected CI arguments');
  const node = (script: string, ...args: string[]) => run(process.execPath, [join(packageRoot, script), ...args], { cwd: packageRoot });
  const cargo = (...args: string[]) => run('cargo', args);
  switch (operation) {
    case 'generate':
      for (const script of ['scripts/generate-builder-manifests.mjs', 'scripts/generate-skill-metadata.mjs', 'scripts/generate-skill-evals.mjs', 'scripts/generate-command-contract.mjs', 'site/scripts/generate-skill-reference.mjs']) node(script);
      break;
    case 'authority':
      node('scripts/check-builder-authority.mjs', '--release');
      for (const script of ['scripts/check-skill-contracts.mjs', 'scripts/check-runtime-security.mjs', 'scripts/check-prometheus-boundary.mjs']) node(script);
      node('scripts/sync-skill-resources.mjs', '--check'); node('scripts/sync-harness-skills.mjs', '--check');
      run(process.execPath, ['--check', join(packageRoot, '.opencode/plugins/knowme-builder.mjs')]);
      node('scripts/test-opencode-plugin.mjs'); node('scripts/audit.mjs', 'doc-consistency'); node('scripts/audit.mjs', 'generator-purity'); break;
    case 'exhaust': {
      const files = run('git', ['ls-files', '-z'], { cwd: packageRoot, capture: true }).stdout.split('\0');
      const bad = files.filter(path => /(^|\/)(node_modules|target|dist|build|\.docusaurus|\.DS_Store)(\/|$)|\.kbd-orchestrator\/(dispatch-logs|hook-logs|status)|\.prometheus\/events\.jsonl/.test(path));
      if (bad.length) throw new Error(`Tracked runtime exhaust:\n${bad.join('\n')}`); break;
    }
    case 'rust-gates':
      cargo('fmt', '--manifest-path', 'tools/knowme-builder/Cargo.toml', '--', '--check');
      cargo('clippy', '--locked', '--manifest-path', 'tools/knowme-builder/Cargo.toml', '--all-targets', '--', '-D', 'warnings');
      cargo('test', '--locked', '--manifest-path', 'tools/knowme-builder/Cargo.toml'); break;
    case 'profiles': {
      const work = mkdtempSync(join(process.env.RUNNER_TEMP ?? tmpdir(), 'builder-profile-proof-'));
      function inventory(directory: string, prefix = ''): string[] {
        return readdirSync(directory, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name)).flatMap(entry => entry.isDirectory() ? inventory(join(directory, entry.name), `${prefix}${entry.name}/`) : [`${prefix}${entry.name}\0${readFileSync(join(directory, entry.name)).toString('base64')}`]);
      }
      try {
        for (const profile of ['sovereign-hybrid', 'governed-web-shell']) {
          const first = join(work, profile, 'first/fixture'), second = join(work, profile, 'second/fixture');
          for (const target of [first, second]) cargo('run', '--quiet', '--manifest-path', join(packageRoot, 'tools/knowme-builder/Cargo.toml'), '--', 'new', target, '--profile', profile, '--mode', 'runnable');
          if (JSON.stringify(inventory(first)) !== JSON.stringify(inventory(second))) throw new Error(`Profile output differs between runs: ${profile}`);
        }
        process.stdout.write('Profile generation is byte-stable; this does not certify application execution.\n');
      } finally { rmSync(work, { recursive: true, force: true }); }
      break;
    }
    case 'manifest':
      cargo('run', '--quiet', '--manifest-path', 'tools/knowme-builder/Cargo.toml', '--', 'manifest', 'check'); node('scripts/generate-command-contract.mjs', '--check'); break;
    case 'inference-check':
      cargo('check', '-p', 'gen_ui_inference', '--features', 'local-litert-lm'); cargo('check', '-p', 'gen_ui_inference', '--features', 'local-mlx'); break;
    default: throw new Error('Expected generate, authority, exhaust, rust-gates, profiles, manifest or inference-check');
  }
});
