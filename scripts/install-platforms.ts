#!/usr/bin/env npx tsx
/**
 * Multi-platform installer for prometheus-skill-pack.
 * Installs skills to Claude Code, OpenCode, Cursor, Codex, and other AI tool directories.
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

const REPO_ROOT = resolve(import.meta.dirname, '..');
const SKILL_NAME = 'prometheus-skill-pack';
const HOME = homedir();

interface Platform {
  name: string;
  globalSkillsDir: string;
  projectSkillsDir: string;
  description: string;
  supportsPlugins: boolean;
  pluginDir?: string;
}

const PLATFORMS: Platform[] = [
  {
    name: 'claude-code',
    globalSkillsDir: join(HOME, '.claude', 'skills'),
    projectSkillsDir: '.claude/skills',
    description: 'Claude Code (CLI, Desktop, IDE extensions)',
    supportsPlugins: true,
    pluginDir: join(HOME, '.claude', 'plugins'),
  },
  {
    name: 'opencode',
    globalSkillsDir: join(HOME, '.opencode', 'skills'),
    projectSkillsDir: '.opencode/skills',
    description: 'OpenCode (CLI, VS Code extension)',
    supportsPlugins: true,
    pluginDir: join(HOME, '.opencode'),
  },
  {
    name: 'cursor',
    globalSkillsDir: join(HOME, '.cursor', 'skills'),
    projectSkillsDir: '.cursor/skills',
    description: 'Cursor AI Editor',
    supportsPlugins: false,
  },
  {
    name: 'codex',
    globalSkillsDir: join(HOME, '.codex', 'skills'),
    projectSkillsDir: '.codex/skills',
    description: 'OpenAI Codex CLI',
    supportsPlugins: false,
  },
  {
    name: 'gemini-cli',
    globalSkillsDir: join(HOME, '.gemini', 'skills'),
    projectSkillsDir: '.gemini/skills',
    description: 'Google Gemini CLI',
    supportsPlugins: false,
  },
  {
    name: 'roo-code',
    globalSkillsDir: join(HOME, '.roo', 'skills'),
    projectSkillsDir: '.roo/skills',
    description: 'Roo Code (VS Code extension)',
    supportsPlugins: false,
  },
  {
    name: 'windsurf',
    globalSkillsDir: join(HOME, '.windsurf', 'skills'),
    projectSkillsDir: '.windsurf/skills',
    description: 'Windsurf Cascade',
    supportsPlugins: false,
  },
  {
    name: 'amp',
    globalSkillsDir: join(HOME, '.agents', 'skills'),
    projectSkillsDir: '.agents/skills',
    description: 'Amp AI Agent',
    supportsPlugins: false,
  },
  {
    name: 'zed',
    globalSkillsDir: join(HOME, '.config', 'zed', 'skills'),
    projectSkillsDir: '.zed/skills',
    description: 'Zed Editor',
    supportsPlugins: false,
  },
  {
    name: 'antigravity',
    globalSkillsDir: join(HOME, '.zed', 'skills'),
    projectSkillsDir: '.zed/skills',
    description: 'Antigravity (Zed fork)',
    supportsPlugins: false,
  },
];

function detectInstalledPlatforms(): Platform[] {
  return PLATFORMS.filter(p => {
    const parentDir = resolve(p.globalSkillsDir, '..');
    return existsSync(parentDir);
  });
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

function installPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir =
    scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  console.log(`\n  Installing to ${platform.name} (${scope})...`);
  console.log(`    Target: ${linkPath}`);

  if (createSymlink(REPO_ROOT, linkPath)) {
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

    // Merge prometheus plugin entry into opencode.json without clobbering existing config
    const opencodeJsonPath =
      scope === 'global'
        ? join(HOME, '.opencode', 'opencode.json')
        : join(process.cwd(), 'opencode.json');

    // Global scope: opencode.json resolves paths relative to ~/.opencode/, so we
    // must register as an absolute path. Project scope: relative ./.opencode is fine.
    const pluginPath = scope === 'global' ? join(REPO_ROOT, '.opencode') : './.opencode';

    try {
      let opencodeConfig: Record<string, unknown> = {};
      if (existsSync(opencodeJsonPath)) {
        const backup = `${opencodeJsonPath}.bak.${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14)}`;
        copyFileSync(opencodeJsonPath, backup);
        console.log(`    ✅ Backed up opencode.json: ${backup}`);
        try {
          opencodeConfig = JSON.parse(readFileSync(opencodeJsonPath, 'utf-8'));
        } catch {
          // Malformed JSON — start fresh rather than clobbering silently
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
        console.log(`    ✅ opencode.json already contains plugin entry`);
      }
    } catch (e) {
      console.warn(`    ⚠️  Could not update opencode.json: ${e}`);
    }
  }
}

function uninstallPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir =
    scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  if (pathExistsOrIsSymlink(linkPath) && isRepoOwnedSymlink(linkPath)) {
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
