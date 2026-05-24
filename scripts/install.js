#!/usr/bin/env node

/**
 * Installs skills to user or project scope
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { homedir } from 'os';
import { execFileSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

async function copyDirectory(src, dest) {
  await fs.mkdir(dest, { recursive: true });
  const entries = await fs.readdir(src, { withFileTypes: true });

  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);

    if (entry.isDirectory()) {
      await copyDirectory(srcPath, destPath);
    } else {
      await fs.copyFile(srcPath, destPath);
    }
  }
}

async function installSkills(scope) {
  console.log(`📦 Installing Prometheus Skill Pack to ${scope} scope\n`);

  const skillsDir = path.join(rootDir, 'skills');
  let targetDir;

  if (scope === 'user') {
    targetDir = path.join(homedir(), '.claude', 'skills', 'prometheus');
  } else if (scope === 'project') {
    targetDir = path.join(process.cwd(), '.claude', 'skills', 'prometheus');
  } else {
    console.error(`Invalid scope: ${scope}. Use 'user' or 'project'`);
    process.exit(1);
  }

  try {
    console.log(`Source: ${skillsDir}`);
    console.log(`Target: ${targetDir}\n`);

    // Copy skills directory
    await copyDirectory(skillsDir, targetDir);

    console.log('✅ Skills installed!');
    console.log(`\nSkills installed to: ${targetDir}`);

    // Install native Claude Code slash commands to ~/.claude/commands/
    if (scope === 'user') {
      const commandsDir = path.join(homedir(), '.claude', 'commands');
      console.log(`\n📋 Generating slash commands → ${commandsDir}`);
      const generateScript = path.join(rootDir, 'scripts', 'generate-commands.js');
      execFileSync(process.execPath, [generateScript, '--output', commandsDir], {
        stdio: 'inherit',
      });
    }

    console.log('\nAvailable skill categories:');
    const categories = await fs.readdir(targetDir);
    categories.forEach(cat => console.log(`  - ${cat}`));

    console.log('\n💡 Tip: Restart Claude Code or run /reload-plugins to use the new skills');
  } catch (error) {
    console.error('Installation failed:', error.message);
    process.exit(1);
  }
}

async function main() {
  const args = process.argv.slice(2);
  const scopeIdx = args.indexOf('--scope');
  const scopeEqArg = args.find(arg => arg.startsWith('--scope='));

  let scope;
  if (scopeIdx !== -1 && args[scopeIdx + 1]) {
    scope = args[scopeIdx + 1];
  } else if (scopeEqArg) {
    scope = scopeEqArg.split('=')[1];
  } else {
    console.error('Usage: npm run install:user OR npm run install:project');
    process.exit(1);
  }

  await installSkills(scope);
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
