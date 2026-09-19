#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline/promises';

import { compareVersions, readSkillSystem, targetsById } from './lib/skill-system.js';

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {
    sourceRoot: process.cwd(),
    profile: null,
    targets: 'detected',
    nonInteractive: false,
    yes: false,
    dryRun: false,
    verify: false,
    uninstall: false,
    bestEffort: false,
    home: os.homedir(),
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--profile') args.profile = argv[++index];
    else if (value === '--targets') args.targets = argv[++index];
    else if (value === '--source-root') args.sourceRoot = argv[++index];
    else if (value === '--home') args.home = argv[++index];
    else if (value === '--non-interactive') args.nonInteractive = true;
    else if (value === '--yes') args.yes = true;
    else if (value === '--dry-run') args.dryRun = true;
    else if (value === '--verify') args.verify = true;
    else if (value === '--uninstall') args.uninstall = true;
    else if (value === '--best-effort') args.bestEffort = true;
    else if (value === '--help' || value === '-h') args.help = true;
    else fail(`unknown argument: ${value}`);
  }
  args.sourceRoot = path.resolve(args.sourceRoot);
  args.home = path.resolve(args.home);
  if (args.profile && !['skills', 'full'].includes(args.profile)) fail('--profile must be skills or full');
  if (args.verify && args.uninstall) fail('--verify and --uninstall are mutually exclusive');
  if (args.bestEffort && args.verify) fail('--best-effort cannot certify --verify');
  return args;
}

function help() {
  process.stdout.write(`Usage: ./install.sh [options]\n\n` +
    `  --profile skills|full       Installation profile (default: skills)\n` +
    `  --targets detected|all|IDs  Detected clients, all 14, or comma-separated IDs\n` +
    `  --non-interactive           Disable prompts\n` +
    `  --yes                       Approve the displayed mutation summary\n` +
    `  --dry-run                   Display exact planned mutations only\n` +
    `  --verify                    Verify the selected installed surfaces\n` +
    `  --uninstall                 Remove only receipt-owned selected surfaces\n` +
    `  --best-effort               Continue after failures; never reports certification\n`);
}

function commandExists(command) {
  return spawnSync(process.platform === 'win32' ? 'where' : 'sh', process.platform === 'win32' ? [command] : ['-c', `command -v "$1" >/dev/null 2>&1`, 'sh', command]).status === 0;
}

function detectedTargets(contract, home) {
  return contract.targets.filter(target => target.detect.some(probe => {
    if (probe.startsWith('command:')) return commandExists(probe.slice('command:'.length));
    return fs.existsSync(path.join(home, ...probe.split('/')));
  }));
}

function resolveTargets(contract, selection, home) {
  if (selection === 'detected') return detectedTargets(contract, home);
  return targetsById(contract, selection);
}

function rejectObsoleteNativeUmbrella(contract, targets, home) {
  const ids = new Set(targets.map(target => target.id));
  const env = { ...process.env, HOME: home, CODEX_HOME: path.join(home, '.codex') };
  if (ids.has('claude') && commandExists('claude')) {
    const result = spawnSync('claude', ['plugin', 'list', '--json'], { encoding: 'utf8', env });
    if (result.status === 0) {
      const row = JSON.parse(result.stdout).find(candidate => candidate.id === 'prometheus-skill-pack@prometheus-skill-pack' && candidate.enabled);
      if (row && compareVersions(row.version, contract.minimumActiveVersion) < 0) fail(`enabled Claude umbrella ${row.version} is below minimum ${contract.minimumActiveVersion}`);
    }
  }
  if (ids.has('codex') && commandExists('codex')) {
    const result = spawnSync('codex', ['plugin', 'list', '--json'], { encoding: 'utf8', env });
    if (result.status === 0) {
      const row = (JSON.parse(result.stdout).installed ?? []).find(candidate => candidate.pluginId === 'prometheus-skill-pack@prometheus-skill-pack' && candidate.enabled);
      if (row && compareVersions(row.version, contract.minimumActiveVersion) < 0) fail(`enabled Codex umbrella ${row.version} is below minimum ${contract.minimumActiveVersion}`);
    }
  }
}

function platformKind() {
  if (process.platform === 'darwin') return 'darwin';
  if (process.platform === 'linux') return process.env.WSL_DISTRO_NAME ? 'windows-wsl' : 'linux';
  if (process.platform === 'win32' && process.env.MSYSTEM) return 'windows-git-bash';
  return process.platform;
}

function run(command, argv, options = {}) {
  const result = spawnSync(command, argv, { cwd: options.cwd, env: options.env, encoding: 'utf8', stdio: options.capture ? 'pipe' : 'inherit' });
  if (result.status !== 0) fail(`${command} ${argv.join(' ')} failed${result.stderr ? `: ${result.stderr.trim()}` : ''}`);
  return options.capture ? result.stdout.trim() : '';
}

async function chooseProfile(args) {
  if (args.profile) return args.profile;
  if (args.nonInteractive || !process.stdin.isTTY) return 'skills';
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const answer = (await rl.question('Profile [skills (recommended)/full]: ')).trim().toLowerCase();
  rl.close();
  if (!answer || answer === 'skills') return 'skills';
  if (answer === 'full') return 'full';
  fail(`unknown profile: ${answer}`);
}

async function chooseTargets(args, contract) {
  if (args.targets !== 'detected' || args.nonInteractive || !process.stdin.isTTY) {
    return resolveTargets(contract, args.targets, args.home);
  }
  const detected = detectedTargets(contract, args.home);
  if (!detected.length) return [];
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const answer = (await rl.question(
    `Detected targets [${detected.map(target => target.id).join(',')}]. Press Enter to keep all, or enter a comma-separated selection: `
  )).trim();
  rl.close();
  return answer ? targetsById(contract, answer) : detected;
}

async function confirm(args, summary) {
  process.stdout.write(`${summary.join('\n')}\n`);
  if (args.dryRun) return false;
  if (args.yes) return true;
  if (args.nonInteractive || !process.stdin.isTTY) fail('mutation requires --yes in non-interactive mode');
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const answer = (await rl.question('Apply these changes? [y/N]: ')).trim().toLowerCase();
  rl.close();
  return answer === 'y' || answer === 'yes';
}

function verifyImportCommit(sourceRoot, entry) {
  const actual = run('git', ['-C', path.join(sourceRoot, entry.path), 'rev-parse', 'HEAD'], { capture: true });
  if (actual !== entry.commit) fail(`import ${entry.path} is ${actual}; expected ${entry.commit}`);
}

function initializeImports(sourceRoot, contract, profile) {
  const required = contract.imports.filter(entry => entry.requiredFor?.includes(profile));
  const paths = required.map(entry => entry.path);
  run('git', ['-C', sourceRoot, 'submodule', 'update', '--init', '--recursive', '--', ...paths]);
  for (const entry of required) verifyImportCommit(sourceRoot, entry);
}

function generationArgs(args, targets) {
  return [
    path.join(args.sourceRoot, 'scripts/install-plugin-generation.js'),
    '--source-root', args.sourceRoot,
    '--home', args.home,
    '--plugin-root', path.join(args.home, '.prometheus/plugins/prometheus-skill-pack'),
    '--targets', targets.map(target => target.id).join(','),
  ];
}

function configureFull(args, targets, contract) {
  const supported = contract.platforms.full.includes(platformKind());
  if (!supported) fail('full installation is supported on macOS and Linux only. On Windows, use --profile skills from Git Bash or WSL.');
  const prereq = spawnSync('bash', [path.join(args.sourceRoot, 'scripts/check-prerequisites.sh')], { cwd: args.sourceRoot, stdio: 'inherit' });
  if (prereq.status !== 0) {
    if (!args.yes) fail('missing prerequisites; rerun with --yes to approve local prerequisite installation');
    run('bash', [path.join(args.sourceRoot, 'scripts/check-prerequisites.sh'), '--install'], { cwd: args.sourceRoot });
  }
  run('bash', [path.join(args.sourceRoot, 'scripts/check-prerequisites.sh'), '--build-tools'], { cwd: args.sourceRoot });
  const mcpTargets = targets.filter(target => ['claude', 'codex', 'opencode', 'kimi-code'].includes(target.id));
  run('bash', [path.join(args.sourceRoot, 'scripts/install-mcp-services.sh')], { cwd: args.sourceRoot });
  const mcpNames = new Map([['claude', 'claude-code'], ['codex', 'codex'], ['opencode', 'opencode'], ['kimi-code', 'kimi-code']]);
  for (const target of mcpTargets) {
    run('bash', [path.join(args.sourceRoot, 'scripts/configure-mcp-all-tools.sh'), '--tool', mcpNames.get(target.id)], { cwd: args.sourceRoot });
  }
  run('bash', [path.join(args.sourceRoot, 'scripts/check-mcp-health.sh')], { cwd: args.sourceRoot });
  run('npm', ['run', 'doctor'], { cwd: args.sourceRoot });
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return help();
  const contract = readSkillSystem(args.sourceRoot);
  const profile = await chooseProfile(args);
  const platform = platformKind();
  if (profile === 'full' && ['windows-git-bash', 'windows-wsl', 'win32'].includes(platform)) {
    fail('full installation is supported on macOS and Linux only. On Windows, use --profile skills from Git Bash or WSL.');
  }
  if (!contract.platforms[profile].includes(platform)) {
    fail(`${profile} installation is not supported on ${platform}`);
  }
  const targets = await chooseTargets(args, contract);
  if (!targets.length) fail('no target clients selected or detected; pass --targets all or a comma-separated target list');
  if (!args.uninstall) rejectObsoleteNativeUmbrella(contract, targets, args.home);
  const operation = args.verify ? 'verify' : args.uninstall ? 'uninstall' : 'install';
  const imports = contract.imports.filter(entry => entry.requiredFor?.includes(profile)).map(entry => `${entry.path}@${entry.commit}`);
  const summary = [
    `Prometheus skill system ${operation} plan`,
    `  release: ${contract.releaseVersion} (minimum active ${contract.minimumActiveVersion})`,
    `  profile: ${profile}`,
    `  targets: ${targets.map(target => `${target.id}:${target.mode}`).join(', ')}`,
    `  imports: ${args.verify || args.uninstall ? 'none' : imports.join(', ')}`,
    `  plugin root: ${path.join(args.home, '.prometheus/plugins/prometheus-skill-pack')}`,
    `  certification: ${args.bestEffort ? 'disabled (--best-effort)' : 'required'}`,
  ];
  let approved = true;
  if (args.verify) process.stdout.write(`${summary.join('\n')}\n`);
  else approved = await confirm(args, summary);
  if (!approved) {
    if (!args.dryRun) process.stdout.write('No changes made.\n');
    return;
  }

  const failures = [];
  const attempt = (label, action) => {
    try { return action(); }
    catch (error) {
      if (!args.bestEffort) throw error;
      failures.push(`${label}: ${error.message}`);
      process.stderr.write(`WARNING: ${label} failed; continuing only because --best-effort is active\n`);
      return null;
    }
  };

  const installerArgs = generationArgs(args, targets);
  if (args.verify) installerArgs.push('--verify');
  else if (args.uninstall) installerArgs.push('--uninstall');
  else attempt('submodule initialization', () => initializeImports(args.sourceRoot, contract, profile));
  const generation = attempt(operation, () => run(process.execPath, installerArgs, { capture: true }));
  if (!args.verify && !args.uninstall && profile === 'full') attempt('full profile', () => configureFull(args, targets, contract));

  if (failures.length) {
    process.stdout.write(`Best-effort run completed without certification (${failures.length} failure(s)).\n`);
  } else {
    process.stdout.write(`Prometheus ${operation} verified${generation ? `: ${generation}` : ''}\n`);
  }
}

main().catch(error => {
  process.stderr.write(`install: ${error.message}\n`);
  process.exitCode = 1;
});
