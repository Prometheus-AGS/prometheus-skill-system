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

import { existsSync, mkdirSync, symlinkSync, unlinkSync, readdirSync, statSync, readlinkSync } from 'fs';
import { join, resolve, relative, basename } from 'path';
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
    globalSkillsDir: join(HOME, '.config', 'opencode', 'skills'),
    projectSkillsDir: '.opencode/skills',
    description: 'OpenCode (CLI, VS Code extension)',
    supportsPlugins: false,
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
    globalSkillsDir: join(HOME, '.agents', 'skills'),
    projectSkillsDir: '.agents/skills',
    description: 'OpenAI Codex / Agents',
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
];

function detectInstalledPlatforms(): Platform[] {
  return PLATFORMS.filter((p) => {
    const parentDir = resolve(p.globalSkillsDir, '..');
    return existsSync(parentDir);
  });
}

function createSymlink(target: string, linkPath: string): boolean {
  try {
    if (existsSync(linkPath)) {
      const stats = statSync(linkPath);
      if (stats.isSymbolicLink?.() || readlinkSync(linkPath)) {
        unlinkSync(linkPath);
      } else {
        console.warn(`  [skip] ${linkPath} exists and is not a symlink`);
        return false;
      }
    }
  } catch {
    // Not a symlink or doesn't exist — proceed
    try { unlinkSync(linkPath); } catch { /* noop */ }
  }

  mkdirSync(resolve(linkPath, '..'), { recursive: true });
  symlinkSync(target, linkPath, 'dir');
  return true;
}

function installPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir = scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  console.log(`\n  Installing to ${platform.name} (${scope})...`);
  console.log(`    Target: ${linkPath}`);

  if (createSymlink(REPO_ROOT, linkPath)) {
    console.log(`    ✅ Symlink created`);
  }

  // For OpenCode: also create .opencode/tools/ symlink if tools exist
  if (platform.name === 'opencode') {
    const toolsSource = join(REPO_ROOT, '.opencode', 'tools');
    if (existsSync(toolsSource)) {
      const toolsTarget = scope === 'global'
        ? join(HOME, '.config', 'opencode', 'tools')
        : join(process.cwd(), '.opencode', 'tools');

      if (createSymlink(toolsSource, join(toolsTarget, SKILL_NAME))) {
        console.log(`    ✅ OpenCode tools linked`);
      }
    }
  }
}

function uninstallPlatform(platform: Platform, scope: 'global' | 'project'): void {
  const targetDir = scope === 'global' ? platform.globalSkillsDir : join(process.cwd(), platform.projectSkillsDir);
  const linkPath = join(targetDir, SKILL_NAME);

  if (existsSync(linkPath)) {
    try {
      unlinkSync(linkPath);
      console.log(`  ✅ Removed: ${linkPath}`);
    } catch (e) {
      console.warn(`  ⚠️  Could not remove: ${linkPath}`);
    }
  }
}

function main(): void {
  const args = process.argv.slice(2);
  const scope = args.includes('--scope') ? (args[args.indexOf('--scope') + 1] as 'global' | 'project') : 'global';
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
    console.log(`\nAll platforms: ${PLATFORMS.map((p) => p.name).join(', ')}`);
    return;
  }

  const platforms = targetPlatform
    ? PLATFORMS.filter((p) => p.name === targetPlatform)
    : PLATFORMS;

  if (platforms.length === 0) {
    console.error(`\n❌ Unknown platform: ${targetPlatform}`);
    console.log(`   Available: ${PLATFORMS.map((p) => p.name).join(', ')}`);
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
