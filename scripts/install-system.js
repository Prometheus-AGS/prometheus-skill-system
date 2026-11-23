#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline/promises';
import { fileURLToPath } from 'node:url';

import { planServices, applyServices } from './service-providers/index.mjs';
import {syncInstalledMcp} from './service-providers/mcp-config.mjs';
import native from './service-providers/native.mjs';
import { compareVersions, readSkillSystem, targetsById } from './lib/skill-system.js';
import { resolveCodexHome } from './lib/store-paths.js';

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {
    sourceRoot: process.cwd(),
    profile: null,
    selections: {}, endpoints: {}, ports: {},
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
    if (['--service','--endpoint','--port'].includes(value)) {
      const match = /^([a-z0-9-]+)=(.+)$/.exec(argv[++index] || '');
      if (!match) fail(value + ' requires id=value');
      const field = {'--service':'selections','--endpoint':'endpoints','--port':'ports'}[value];
      args[field][match[1]] = match[2];
    }
    else if (value === '--profile') args.profile = argv[++index];
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
  args.codexHome = resolveCodexHome(args.home);
  if (args.profile && !['skills', 'full'].includes(args.profile)) fail('--profile must be skills or full');
  if (args.verify && args.uninstall) fail('--verify and --uninstall are mutually exclusive');
  if (args.bestEffort && args.verify) fail('--best-effort cannot certify --verify');
  return args;
}

function help() {
  process.stdout.write(`Usage: ./install.sh [options]\n\n` +
    `  --profile skills|full       Installation profile (default: skills)\n` +
    `  --targets detected|all|IDs  Detected clients, all (agents,claude,qwen), or IDs\n` +
    `  --service ID=MODE           Repeat for auto, native, docker, reuse, or off\n` +
    `  --endpoint ID=URL           Use an existing service endpoint\n` +
    `  --port ID=PORT              Use an alternate local port\n` +
    `  --home PATH                 Fallback home; explicit CODEX_HOME takes precedence\n` +
    `  --non-interactive           Disable prompts\n` +
    `  --yes                       Approve the displayed mutation summary\n` +
    `  --dry-run                   Display exact planned mutations only\n` +
    `  --verify                    Verify the selected installed surfaces\n` +
    `  --uninstall                 Remove only receipt-owned selected surfaces\n` +
    `  --best-effort               Continue after failures; never reports certification\n`);
}

const CANONICAL_TARGET_ID = 'agents';

function commandExists(command) {
  return spawnSync(process.platform === 'win32' ? 'where' : 'sh', process.platform === 'win32' ? [command] : ['-c', `command -v "$1" >/dev/null 2>&1`, 'sh', command]).status === 0;
}

function probeDetected(probe, home) {
  if (probe.startsWith('command:')) return commandExists(probe.slice('command:'.length));
  const destination = probe === '.codex' || probe.startsWith('.codex/')
    ? path.join(resolveCodexHome(home), ...probe.split('/').slice(1))
    : path.join(home, ...probe.split('/'));
  return fs.existsSync(destination);
}

// `.agents/skills` is the canonical root every non-shim harness reads, so it is
// always part of a detected install; the shims follow their own detection.
function detectedTargets(contract, home) {
  return contract.targets.filter(target =>
    target.id === CANONICAL_TARGET_ID || target.detect.some(probe => probeDetected(probe, home)));
}

// Codex is served by the canonical root, not a target of its own, so its
// policies follow its presence on this machine.
function codexPresent(home) {
  return probeDetected('command:codex', home) || probeDetected('.codex', home);
}

function servesCodex(targets, home) {
  return targets.some(target => target.id === CANONICAL_TARGET_ID) && codexPresent(home);
}

function resolveTargets(contract, selection, home) {
  if (selection === 'detected') return detectedTargets(contract, home);
  return targetsById(contract, selection);
}

function rejectObsoleteNativeUmbrella(contract, targets, home) {
  const ids = new Set(targets.map(target => target.id));
  const env = { ...process.env, HOME: home, CODEX_HOME: resolveCodexHome(home) };
  if (ids.has('claude') && commandExists('claude')) {
    const result = spawnSync('claude', ['plugin', 'list', '--json'], { encoding: 'utf8', env });
    if (result.status === 0) {
      const row = JSON.parse(result.stdout).find(candidate => candidate.id === 'prometheus-skill-pack@prometheus-skill-pack' && candidate.enabled);
      if (row && compareVersions(row.version, contract.minimumActiveVersion) < 0) fail(`enabled Claude umbrella ${row.version} is below minimum ${contract.minimumActiveVersion}`);
    }
  }
  if (commandExists('codex')) {
    const result = spawnSync('codex', ['plugin', 'list', '--json'], { encoding: 'utf8', env });
    if (result.status === 0) {
      const row = (JSON.parse(result.stdout).installed ?? []).find(candidate => candidate.pluginId === 'prometheus-skill-pack@prometheus-skill-pack' && candidate.enabled);
      if (row && compareVersions(row.version, contract.minimumActiveVersion) < 0) fail(`enabled Codex umbrella ${row.version} is below minimum ${contract.minimumActiveVersion}`);
    }
  }
}

// Apply all three Codex memory settings and archive known summaries. The caller
// handles false through the normal installer failure policy. CODEX_MEMORIES_SCRIPT
// overrides the script path for fault injection in tests only.
export function applyCodexMemories({ home = os.homedir() } = {}) {
  try {
    const script = process.env.CODEX_MEMORIES_SCRIPT
      || fileURLToPath(new URL('../shared/scripts/codex-memories-config.sh', import.meta.url));
    const env = { ...process.env, HOME: home, CODEX_HOME: resolveCodexHome(home) };
    const result = spawnSync('bash', [script, '--create'], { encoding: 'utf8', env });
    if (result.status !== 0) {
      const detail = (result.stderr || result.error?.message || `exit ${result.status}`).trim();
      process.stderr.write(`codex memories policy failed: ${detail}\n`);
      return false;
    }
    return true;
  } catch (error) {
    process.stderr.write(`codex memories policy failed: ${error.message}\n`);
    return false;
  }
}

// Trim Codex's skill catalog (redundant and unselected copies) through the same
// guarded helper the shell installer uses. Failures follow the installer policy.
export function applyCodexCatalog({ home = os.homedir(), sourceRoot } = {}) {
  try {
    const script = fileURLToPath(new URL('./lib/install-codex-catalog.sh', import.meta.url));
    const root = sourceRoot || fileURLToPath(new URL('..', import.meta.url));
    const env = { ...process.env, HOME: home, CODEX_HOME: resolveCodexHome(home) };
    const result = spawnSync('bash', [script, root], { encoding: 'utf8', env });
    if (result.status !== 0) {
      const detail = (result.stderr || result.error?.message || `exit ${result.status}`).trim();
      process.stderr.write(`codex catalog policy failed: ${detail}\n`);
      return false;
    }
    return true;
  } catch (error) {
    process.stderr.write(`codex catalog policy failed: ${error.message}\n`);
    return false;
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

const serviceImports = {'tools/liter-llm':'liter-llm','tools/openai-proxy':'openai-proxy','tools/surreal-memory-server':'surreal-memory'};
function initializeImports(sourceRoot, contract, profile, hasBinaries, sourceImports, plan) {
  const required = contract.imports.filter(entry => entry.requiredFor?.includes(profile)
    && (!plan || !serviceImports[entry.path] || plan.services.some(s => s.provider === 'native' && s.id === serviceImports[entry.path]))
    && (!hasBinaries || !entry.path.startsWith('tools/') || sourceImports.includes(entry.path)));
  const paths = required.map(entry => entry.path);
  if (!paths.length) return;
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

// MCP configuration is per harness. Claude follows its shim target; harnesses
// served by the canonical root follow their own detection.
function mcpTools(targets, home) {
  const tools = [];
  if (targets.some(target => target.id === 'claude')) tools.push('claude-code');
  if (!targets.some(target => target.id === CANONICAL_TARGET_ID)) return tools;
  for (const [tool, probes] of [
    ['codex', ['command:codex', '.codex']],
    ['opencode', ['command:opencode', '.opencode']],
    ['kimi-code', ['command:kimi', '.kimi-code']],
  ]) {
    if (probes.some(probe => probeDetected(probe, home))) tools.push(tool);
  }
  return tools;
}

async function configureFull(args, targets, contract, sourceImports, options, plan) {
  const supported = contract.platforms.full.includes(platformKind());
  if (!supported) fail('full installation is supported on macOS and Linux only. On Windows, use --profile skills from Git Bash or WSL.');
  const env = {...process.env,...plan.env, PROMETHEUS_NATIVE_SERVICES: plan.services.filter(s => s.provider === 'native').map(s => s.id).join(',')};
  const prereqArgs = [path.join(args.sourceRoot, 'scripts/check-prerequisites.sh'), '--core-only'];
  if (!plan.services.some(s => s.provider === 'docker')) prereqArgs.push('--skip-docker');
  if (process.env.PROMETHEUS_BINARY_RELEASE_STAGE && !sourceImports.length) prereqArgs.push('--prebuilt');
  const prereq = spawnSync('bash', prereqArgs, { cwd: args.sourceRoot, stdio: 'inherit' });
  if (prereq.status !== 0) {
    if (!args.yes) fail('missing prerequisites; rerun with --yes to approve local prerequisite installation');
    run('bash', [...prereqArgs, '--install'], { cwd: args.sourceRoot });
  }
  run('bash', [path.join(args.sourceRoot, 'scripts/install-binaries.sh')], {cwd:args.sourceRoot,env});
  const applied = await applyServices({...options,nativeBinariesReady:true,env},{native});
  if (applied.applicable === false) fail('Service installation blocked: ' + JSON.stringify(applied.conflicts));
  run('bash', [path.join(args.sourceRoot, 'scripts/install-mcp-services.sh'), '--maintenance-only'], {cwd:args.sourceRoot,env});
  for (const tool of mcpTools(targets, args.home)) {
    run('bash', [path.join(args.sourceRoot, 'scripts/configure-mcp-all-tools.sh'), '--tool', tool], {cwd:args.sourceRoot,env});
  }
  syncInstalledMcp({home:args.home,env});
  run(process.execPath,[path.join(args.sourceRoot,'scripts/service-providers/doctor.mjs')],{cwd:args.sourceRoot,env});
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return help();
  // Every descendant, including full-profile shell launchers and doctor, sees
  // the selected home and the same absolute Codex root before changing cwd.
  process.env.HOME = args.home;
  process.env.CODEX_HOME = args.codexHome;
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
  const explicitServices = [args.selections,args.endpoints,args.ports].some(x => Object.keys(x).length) ||
    Object.keys(process.env).some(k => /^PROMETHEUS_SERVICE_/.test(k));
  const useServices = profile === 'full' || explicitServices;
  const serviceOptions = {root:args.sourceRoot,home:args.home,selections:args.selections,endpoints:args.endpoints,
    ports:args.ports,interactive:!args.nonInteractive && !args.yes && process.stdin.isTTY,
    ...(profile === 'skills' ? {enabled:[]} : {})};
  let servicePlan;
  if (useServices && operation === 'install') {
    servicePlan = await planServices(serviceOptions,{native});
    if (!servicePlan.applicable) fail('Service conflicts: ' + JSON.stringify(servicePlan.conflicts));
    serviceOptions.interactive = false;
    serviceOptions.selections = Object.fromEntries(servicePlan.services.map(s => [s.id,s.requested]));
  }
  const imports = contract.imports.filter(entry => entry.requiredFor?.includes(profile)).map(entry => `${entry.path}@${entry.commit}`);
  const summary = [
    `Prometheus skill system ${operation} plan`,
    `  release: ${contract.releaseVersion} (minimum active ${contract.minimumActiveVersion})`,
    `  profile: ${profile}`,
    ...(servicePlan ? servicePlan.services.map(s => `  service ${s.id}: ${s.provider}${s.endpoint ? ' ' + s.endpoint : ''}`) : []),
    `  targets: ${targets.map(target => `${target.id}:${target.mode}`).join(', ')}`,
    `  imports${profile === 'full' ? ' (tool sources only on fallback)' : ''}: ${args.verify || args.uninstall ? 'none' : imports.join(', ')}`,
    `  plugin root: ${path.join(args.home, '.prometheus/plugins/prometheus-skill-pack')}`,
    `  Codex root: ${args.codexHome}`,
    ...(operation === 'install' && servesCodex(targets, args.home)
      ? ['  Codex memory policy: disable startup/generation/use; archive known v1/v2 summaries'] : []),
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
  const attempt = async (label, action) => {
    try { return await action(); }
    catch (error) {
      if (!args.bestEffort) throw error;
      failures.push(`${label}: ${error.message}`);
      process.stderr.write(`WARNING: ${label} failed; continuing only because --best-effort is active\n`);
      return null;
    }
  };

  const useBinaries = operation === 'install' && profile === 'full';
  let sourceImports = [];
  const stage = useBinaries ? fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-full-release-')) : null;
  try {
    if (stage) {
      const fetcher = path.join(args.sourceRoot, 'scripts/binary-release.mjs');
      const resolve = spawnSync(process.execPath, [fetcher, 'prepare', stage], { stdio: 'inherit' });
      if (resolve.error) throw resolve.error;
      if (resolve.status === 0) {
        sourceImports = run(process.execPath, [fetcher, 'source-imports', stage], {capture: true}).split(/\r?\n/).filter(Boolean)
          .filter(dir => !serviceImports[dir] || servicePlan.services.some(s => s.provider === 'native' && s.id === serviceImports[dir]));
        process.env.PROMETHEUS_BINARY_RELEASE_STAGE = stage;
        process.env.PROMETHEUS_RELEASE_HOOK_BIN = path.join(stage, 'bin/prometheus-hook');
      } else if (resolve.status === 3) {
        delete process.env.PROMETHEUS_BINARY_RELEASE_STAGE;
        delete process.env.PROMETHEUS_RELEASE_HOOK_BIN;
        process.env.PROMETHEUS_BINARY_RELEASE_PROBED = '1';
      } else fail('binary release verification failed');
    }
    const installerArgs = generationArgs(args, targets);
    if (args.verify) installerArgs.push('--verify');
    else if (args.uninstall) installerArgs.push('--uninstall');
    else await attempt('submodule initialization', () => initializeImports(args.sourceRoot, contract, profile, useBinaries && Boolean(process.env.PROMETHEUS_BINARY_RELEASE_STAGE), sourceImports, servicePlan));
    const generation = await attempt(operation, () => run(process.execPath, installerArgs, { capture: true }));
    if (!args.verify && !args.uninstall && servesCodex(targets, args.home)) {
      await attempt('Codex memory policy', () => {
        if (!applyCodexMemories({ home: args.home })) fail('could not apply Codex memory policy');
      });
      await attempt('Codex catalog policy', () => {
        if (!applyCodexCatalog({ home: args.home, sourceRoot: args.sourceRoot })) fail('could not apply Codex catalog policy');
      });
    }
    if (!args.verify && !args.uninstall && profile === 'full') {
      await attempt('full profile', () => configureFull(args,targets,contract,sourceImports,serviceOptions,servicePlan));
    } else if (useServices && operation === 'install') {
      await attempt('services', async () => {
        const applied = await applyServices(serviceOptions,{native});
        if (!applied.applicable) fail('Service conflicts: ' + JSON.stringify(applied.conflicts));
      });
    } else if (useServices && !args.dryRun) {
      const action = args.verify ? 'status' : 'uninstall';
      const selectedIds = [...new Set([...Object.keys(args.selections),...Object.keys(args.ports),...Object.keys(args.endpoints)])];
      await attempt('services', () => run(process.execPath,[path.join(args.sourceRoot,'scripts/service-providers/cli.mjs'),action,'--home',args.home,
        ...(selectedIds.length ? ['--enabled',selectedIds.join(',')] : [])],{cwd:args.sourceRoot}));
    }

    if (failures.length) {
      process.stdout.write(`Best-effort run completed without certification (${failures.length} failure(s)).\n`);
    } else {
      process.stdout.write(`Prometheus ${operation} verified${generation ? `: ${generation}` : ''}\n`);
    }
  } finally {
    if (stage) fs.rmSync(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && fs.realpathSync(process.argv[1]) === fs.realpathSync(fileURLToPath(import.meta.url))) {
  main().catch(error => {
    process.stderr.write(`install: ${error.message}\n`);
    process.exitCode = 1;
  });
}
