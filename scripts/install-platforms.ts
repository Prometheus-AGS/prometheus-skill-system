#!/usr/bin/env npx tsx
/**
 * Multi-platform installer for prometheus-skill-pack.
 * Activates the immutable generation: skills land once, as real directories, in
 * the canonical root ~/.agents/skills (read by Codex, OpenCode, Cursor, Kimi,
 * MiniMax, Gemini, ...), plus symlink shims for Claude Code and Qwen Code.
 *
 * Usage:
 *   npx tsx scripts/install-platforms.ts                     # install all platforms
 *   npx tsx scripts/install-platforms.ts --platform opencode  # specific platform
 *   npx tsx scripts/install-platforms.ts --scope project      # project-local install
 *   npx tsx scripts/install-platforms.ts --uninstall          # remove symlinks
 *   npx tsx scripts/install-platforms.ts --list               # show detected platforms
 */

import {
  existsSync,
  mkdirSync,
  symlinkSync,
  unlinkSync,
  lstatSync,
  readlinkSync,
  readFileSync,
  writeFileSync,
  copyFileSync,
} from 'fs';
import { dirname, join, resolve } from 'path';
import { homedir } from 'os';
import { execFileSync, execSync } from 'child_process';

const REPO_ROOT = resolve(import.meta.dirname, '..');
const SKILL_NAME = 'prometheus-skill-pack';
const HOME = homedir();

interface Platform {
  name: string;
  /** Where this harness reads user skills after owner decision D3. */
  globalSkillsDir: string;
  projectSkillsDir: string;
  /** Directory whose presence means the harness is installed. */
  detectDir: string;
  /** Retired per-harness skill dir, scanned only to remove a legacy bundle link. */
  legacySkillsDir?: string;
  description: string;
  supportsPlugins: boolean;
  pluginDir?: string;
}

// Install target matrix (owner decision D3, see skill-system.json `targets`):
// skills are installed ONCE as real directories into the canonical root
// ~/.agents/skills, which every harness below reads directly. Only Claude Code
// and Qwen Code do not read it, so they get a symlink shim. The former
// per-harness skill dirs are retired; the generation installer removes only the
// pack-owned entries there.
const CANONICAL_SKILLS_DIR = join(HOME, '.agents', 'skills');
const CANONICAL_PROJECT_SKILLS_DIR = '.agents/skills';

function canonicalPlatform(
  name: string,
  detectDir: string,
  legacySkillsDir: string | undefined,
  description: string,
  extra: Partial<Platform> = {}
): Platform {
  return {
    name,
    globalSkillsDir: CANONICAL_SKILLS_DIR,
    projectSkillsDir: CANONICAL_PROJECT_SKILLS_DIR,
    detectDir,
    legacySkillsDir,
    description,
    supportsPlugins: false,
    ...extra,
  };
}

const PLATFORMS: Platform[] = [
  {
    name: 'claude-code',
    globalSkillsDir: join(HOME, '.claude', 'skills'),
    projectSkillsDir: '.claude/skills',
    detectDir: join(HOME, '.claude'),
    description: 'Claude Code (CLI, Desktop, IDE extensions) — symlink shim',
    supportsPlugins: true,
    pluginDir: join(HOME, '.claude', 'plugins'),
  },
  {
    name: 'qwen-code',
    globalSkillsDir: join(HOME, '.qwen', 'skills'),
    projectSkillsDir: '.qwen/skills',
    detectDir: join(HOME, '.qwen'),
    description: 'Qwen Code — symlink shim',
    supportsPlugins: false,
  },
  canonicalPlatform(
    'opencode',
    join(HOME, '.opencode'),
    join(HOME, '.opencode', 'skills'),
    'OpenCode (CLI, VS Code extension)',
    { supportsPlugins: true, pluginDir: join(HOME, '.opencode') }
  ),
  canonicalPlatform(
    'kimi-code',
    join(HOME, '.kimi-code'),
    join(HOME, '.kimi-code', 'skills'),
    'Kimi Code CLI (Moonshot AI)'
  ),
  canonicalPlatform(
    'minimax',
    join(HOME, '.minimax'),
    join(HOME, '.minimax', 'skills'),
    'MiniMax Desktop / Mavis CLI'
  ),
  canonicalPlatform(
    'cursor',
    join(HOME, '.cursor'),
    join(HOME, '.cursor', 'skills'),
    'Cursor AI Editor'
  ),
  canonicalPlatform(
    'codex',
    join(HOME, '.codex'),
    join(HOME, '.codex', 'skills'),
    'OpenAI Codex CLI (reads ~/.agents/skills; real directories)'
  ),
  canonicalPlatform(
    'gemini-cli',
    join(HOME, '.gemini'),
    join(HOME, '.gemini', 'skills'),
    'Google Gemini CLI'
  ),
  canonicalPlatform(
    'roo-code',
    join(HOME, '.roo'),
    join(HOME, '.roo', 'skills'),
    'Roo Code (VS Code extension)'
  ),
  canonicalPlatform('devin', join(HOME, '.devin'), join(HOME, '.devin', 'skills'), 'Devin'),
  canonicalPlatform('amp', join(HOME, '.agents'), undefined, 'Amp AI Agent'),
  canonicalPlatform(
    'zed',
    join(HOME, '.config', 'zed'),
    join(HOME, '.config', 'zed', 'skills'),
    'Zed Editor'
  ),
  canonicalPlatform(
    'antigravity',
    join(HOME, '.zed'),
    join(HOME, '.zed', 'skills'),
    'Antigravity (Zed fork)'
  ),
];

function detectInstalledPlatforms(): Platform[] {
  return PLATFORMS.filter(p => existsSync(p.detectDir));
}

function isRepoOwnedSymlink(linkPath: string): boolean {
  try {
    const stats = lstatSync(linkPath);
    if (!stats.isSymbolicLink()) return false;
    const target = resolve(dirname(linkPath), readlinkSync(linkPath));
    return target === REPO_ROOT || target.startsWith(`${REPO_ROOT}/`);
  } catch {
    return false;
  }
}

function pathExistsOrIsSymlink(linkPath: string): boolean {
  try {
    lstatSync(linkPath);
    return true;
  } catch {
    return false;
  }
}

function createSymlink(target: string, linkPath: string): boolean {
  if (pathExistsOrIsSymlink(linkPath)) {
    try {
      const stats = lstatSync(linkPath);
      if (stats.isSymbolicLink() && isRepoOwnedSymlink(linkPath)) {
        unlinkSync(linkPath);
      } else {
        console.warn(`  [skip] ${linkPath} exists and is not a repo-owned symlink`);
        return false;
      }
    } catch {
      console.warn(`  [skip] could not inspect ${linkPath}`);
      return false;
    }
  }

  mkdirSync(resolve(linkPath, '..'), { recursive: true });
  symlinkSync(target, linkPath, 'dir');
  return true;
}

function removeLegacyBundleLink(targetDir: string): void {
  const linkPath = join(targetDir, SKILL_NAME);
  if (pathExistsOrIsSymlink(linkPath) && isRepoOwnedSymlink(linkPath)) {
    unlinkSync(linkPath);
    console.log(`    ✅ Removed legacy bundle symlink: ${linkPath}`);
  }
}

function installPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir =
    scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  console.log(`\n  Installing to ${platform.name} (${scope})...`);
  console.log(`    Target: ${linkPath}`);

  if (scope === 'global') {
    if (platform.legacySkillsDir) removeLegacyBundleLink(platform.legacySkillsDir);
    console.log(`    ✅ Skills managed by the verified immutable generation`);
  } else if (createSymlink(REPO_ROOT, linkPath)) {
    console.log(`    ✅ Symlink created`);
  }

  // For OpenCode: also create .opencode/tools/ symlink and write opencode.json
  if (platform.name === 'opencode') {
    const toolsSource = join(REPO_ROOT, '.opencode', 'tools');
    if (existsSync(toolsSource)) {
      const toolsTarget =
        scope === 'global'
          ? join(HOME, '.opencode', 'tools')
          : join(process.cwd(), '.opencode', 'tools');

      if (createSymlink(toolsSource, join(toolsTarget, SKILL_NAME))) {
        console.log(`    ✅ OpenCode tools linked`);
      }
    }

    // Global scope: opencode.json resolves paths relative to ~/.opencode/, so we
    // must register as an absolute path. Project scope: relative ./.opencode is fine.
    const pluginPath = scope === 'global' ? join(REPO_ROOT, '.opencode') : './.opencode';
    const opencodeJsonPaths =
      scope === 'global'
        ? [
            join(HOME, '.opencode', 'opencode.json'),
            join(HOME, '.config', 'opencode', 'opencode.json'),
            join(HOME, '.config', 'opencode', 'tui.json'),
          ]
        : [join(process.cwd(), 'opencode.json')];

    for (const opencodeJsonPath of opencodeJsonPaths) {
      try {
        let opencodeConfig: Record<string, unknown> = {};
        mkdirSync(dirname(opencodeJsonPath), { recursive: true });
        if (existsSync(opencodeJsonPath)) {
          const backup = `${opencodeJsonPath}.bak.${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)}`;
          copyFileSync(opencodeJsonPath, backup);
          console.log(`    ✅ Backed up opencode.json: ${backup}`);
          try {
            opencodeConfig = JSON.parse(readFileSync(opencodeJsonPath, 'utf-8'));
          } catch {
            console.warn(`    ⚠️  Could not parse existing ${opencodeJsonPath} — will overwrite`);
          }
        }

        const existing = Array.isArray(opencodeConfig.plugin)
          ? (opencodeConfig.plugin as string[])
          : [];

        if (!existing.includes(pluginPath)) {
          opencodeConfig.plugin = [...existing, pluginPath];
          writeFileSync(opencodeJsonPath, JSON.stringify(opencodeConfig, null, 2) + '\n', 'utf-8');
          console.log(`    ✅ opencode.json updated: ${opencodeJsonPath}`);
        } else {
          console.log(`    ✅ ${opencodeJsonPath} already contains plugin entry`);
        }
      } catch (e) {
        console.warn(`    ⚠️  Could not update ${opencodeJsonPath}: ${e}`);
      }
    }
  }

  // For Kimi Code: create ~/.kimi-code/config.toml with MCP server entries
  if (platform.name === 'kimi-code' && scope === 'global') {
    const kimiConfigDir = join(HOME, '.kimi-code');
    const kimiConfigPath = join(kimiConfigDir, 'config.toml');

    try {
      mkdirSync(kimiConfigDir, { recursive: true });

      let existingConfig = '';
      if (existsSync(kimiConfigPath)) {
        existingConfig = readFileSync(kimiConfigPath, 'utf-8');
        const backup = `${kimiConfigPath}.bak.${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)}`;
        writeFileSync(backup, existingConfig, 'utf-8');
        console.log(`    ✅ Backed up config.toml: ${backup}`);
      }

      const mcpEntriesToAdd: string[] = [];

      if (!existingConfig.includes('[mcp_servers.surreal-memory]')) {
        mcpEntriesToAdd.push(
          '[mcp_servers.surreal-memory]\ntype = "sse"\nurl = "http://localhost:23001/mcp/sse"'
        );
      }
      if (!existingConfig.includes('[mcp_servers.sycophancy-correction]')) {
        mcpEntriesToAdd.push(
          '[mcp_servers.sycophancy-correction]\ncommand = "sycophancy-correction"'
        );
      }
      if (!existingConfig.includes('[mcp_servers.liter-llm]')) {
        // The pack's MCP launcher (scripts/install-liter-llm-mcp-launcher.mjs
        // installs it): it sources ~/.prometheus/kbd/secrets.env before starting
        // `liter-llm mcp`, so the server starts without the keys in Kimi's
        // environment. A literal path — TOML performs no shell expansion, and
        // Kimi Code rejects ${VAR} forms in these fields.
        const literLauncher = `${HOME}/.local/bin/liter-llm-mcp`;
        mcpEntriesToAdd.push(`[mcp_servers.liter-llm]\ncommand = "${literLauncher}"`);
      }

      if (mcpEntriesToAdd.length > 0) {
        const newConfig =
          (existingConfig ? existingConfig.trimEnd() + '\n\n' : '') +
          mcpEntriesToAdd.join('\n\n') +
          '\n';
        writeFileSync(kimiConfigPath, newConfig, 'utf-8');
        console.log(
          `    ✅ config.toml updated: ${kimiConfigPath} (${mcpEntriesToAdd.length} MCP entries added)`
        );
      } else {
        console.log(`    ✅ config.toml already contains all required MCP entries`);
      }
    } catch (e) {
      console.warn(`    ⚠️  Could not update Kimi config.toml: ${e}`);
    }
  }

  // For MiniMax: merge surreal-memory entry into ~/.minimax/mcp/mcp.json
  if (platform.name === 'minimax' && scope === 'global') {
    const minimaxMcpPath = join(HOME, '.minimax', 'mcp', 'mcp.json');
    if (existsSync(minimaxMcpPath)) {
      try {
        const backup = `${minimaxMcpPath}.bak.${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)}`;
        copyFileSync(minimaxMcpPath, backup);

        let mcpConfig: Record<string, unknown> = {};
        try {
          mcpConfig = JSON.parse(readFileSync(minimaxMcpPath, 'utf-8'));
        } catch {
          console.warn(`    ⚠️  Could not parse ${minimaxMcpPath} — skipping MCP merge`);
        }

        const servers = (mcpConfig.mcpServers as Record<string, unknown>) ?? {};
        let changed = false;

        if (!servers['surreal-memory']) {
          servers['surreal-memory'] = {
            type: 'sse',
            url: 'http://localhost:23001/mcp/sse',
            enabled: true,
            configured: true,
            description: 'Surreal-memory distributed knowledge graph and task tracking',
          };
          changed = true;
        }
        if (!servers['sycophancy-correction']) {
          servers['sycophancy-correction'] = {
            command: 'sycophancy-correction',
            enabled: true,
            configured: true,
          };
          changed = true;
        }

        if (changed) {
          mcpConfig.mcpServers = servers;
          writeFileSync(minimaxMcpPath, JSON.stringify(mcpConfig, null, 2) + '\n', 'utf-8');
          console.log(`    ✅ MiniMax mcp.json updated: ${minimaxMcpPath}`);
        } else {
          console.log(`    ✅ MiniMax mcp.json already contains required MCP entries`);
        }
      } catch (e) {
        console.warn(`    ⚠️  Could not update MiniMax mcp.json: ${e}`);
      }
    } else {
      console.log(`    ℹ️  MiniMax mcp.json not found at ${minimaxMcpPath} — skipping MCP config`);
    }
  }

  // For Codex: regenerate the plugin + marketplace artifacts (idempotent) so
  // `codex plugin marketplace add .` sees current output. Codex reads skills from
  // the canonical ~/.agents/skills root, where the generation installer places
  // real directories (Codex ignores symlinked skill dirs).
  if (platform.name === 'codex') {
    try {
      const command = scope === 'global' ? 'npm run validate:codex' : 'npm run build:codex';
      execSync(command, { cwd: REPO_ROOT, stdio: 'inherit' });
      console.log(
        scope === 'global'
          ? `    ✅ Codex plugin artifacts verified`
          : `    ✅ Codex plugin artifacts regenerated (.codex-plugin/, .agents/plugins/)`
      );
    } catch (e) {
      console.warn(`    ⚠️  Could not run build:codex: ${e}`);
    }
  }
}

function uninstallPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir =
    scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  if (scope === 'global') {
    removeLegacyBundleLink(targetDir);
    if (platform.legacySkillsDir) removeLegacyBundleLink(platform.legacySkillsDir);
  } else if (pathExistsOrIsSymlink(linkPath) && isRepoOwnedSymlink(linkPath)) {
    try {
      unlinkSync(linkPath);
      console.log(`  ✅ Removed: ${linkPath}`);
    } catch (e) {
      console.warn(`  ⚠️  Could not remove: ${linkPath}`);
    }
  }

  if (platform.name === 'opencode') {
    const toolsLink =
      scope === 'global'
        ? join(HOME, '.opencode', 'tools', SKILL_NAME)
        : join(process.cwd(), '.opencode', 'tools', SKILL_NAME);
    if (pathExistsOrIsSymlink(toolsLink) && isRepoOwnedSymlink(toolsLink)) {
      unlinkSync(toolsLink);
      console.log(`  ✅ Removed: ${toolsLink}`);
    }
  }
}

function main(): void {
  const args = process.argv.slice(2);
  const scope = args.includes('--scope')
    ? (args[args.indexOf('--scope') + 1] as 'global' | 'project')
    : 'global';
  const targetPlatform = args.includes('--platform') ? args[args.indexOf('--platform') + 1] : null;
  const uninstall = args.includes('--uninstall');
  const list = args.includes('--list');

  console.log('🔥 Prometheus Skill Pack — Multi-Platform Installer');
  console.log('='.repeat(55));

  if (list) {
    const detected = detectInstalledPlatforms();
    console.log(`\nDetected ${detected.length} platform(s):\n`);
    for (const p of PLATFORMS) {
      const installed = detected.includes(p);
      const marker = installed ? '✅' : '  ';
      console.log(`  ${marker} ${p.name.padEnd(15)} ${p.description}`);
    }
    console.log(`\nAll platforms: ${PLATFORMS.map(p => p.name).join(', ')}`);
    return;
  }

  const platforms = targetPlatform ? PLATFORMS.filter(p => p.name === targetPlatform) : PLATFORMS;

  if (platforms.length === 0) {
    console.error(`\n❌ Unknown platform: ${targetPlatform}`);
    console.log(`   Available: ${PLATFORMS.map(p => p.name).join(', ')}`);
    process.exit(1);
  }

  const action = uninstall ? 'Uninstalling from' : 'Installing to';
  console.log(`\n${action} ${platforms.length} platform(s) (scope: ${scope})\n`);

  if (scope === 'global') {
    execFileSync(
      process.execPath,
      [join(REPO_ROOT, 'scripts', 'build-codex-plugin.js'), '--check'],
      { stdio: 'inherit' }
    );
    const generationInstaller = join(REPO_ROOT, 'scripts', 'install-plugin-generation.js');
    const generationArgs = [generationInstaller, '--source-root', REPO_ROOT, '--home', HOME];
    if (uninstall) generationArgs.push('--uninstall');
    const generation = execFileSync(process.execPath, generationArgs, { encoding: 'utf8' }).trim();
    console.log(
      uninstall
        ? `  ✅ Removed immutable-generation projections`
        : `  ✅ Activated verified immutable generation: ${generation}`
    );
  }

  for (const platform of platforms) {
    if (uninstall) {
      uninstallPlatform(platform, scope);
    } else {
      installPlatform(platform, scope);
    }
  }

  console.log('\n' + '='.repeat(55));
  if (uninstall) {
    console.log('✨ Uninstallation complete');
  } else {
    console.log('✨ Installation complete');
    console.log('\nVerify with: npx tsx scripts/install-platforms.ts --list');
  }
}

main();
