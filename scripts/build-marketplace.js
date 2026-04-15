#!/usr/bin/env node

/**
 * Builds marketplace distribution and creates symlinks
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

async function createSymlink(target, linkPath) {
  try {
    // Remove existing symlink if present
    try {
      await fs.unlink(linkPath);
    } catch {
      // Ignore if doesn't exist
    }

    // Create directory if needed
    await fs.mkdir(path.dirname(linkPath), { recursive: true });

    // Create symlink
    const relativeTarget = path.relative(path.dirname(linkPath), target);
    await fs.symlink(relativeTarget, linkPath, 'dir');
    console.log(`✓ Created symlink: ${path.basename(linkPath)} -> ${relativeTarget}`);
  } catch (error) {
    console.error(`✗ Failed to create symlink ${linkPath}:`, error.message);
  }
}

async function buildMarketplace() {
  console.log('🏗️  Building Prometheus Skill Pack marketplace distribution\n');

  const pluginDir = path.join(rootDir, '.claude-plugin');
  const skillsDir = path.join(rootDir, 'skills');
  const agentsDir = path.join(rootDir, 'agents');
  const hooksDir = path.join(rootDir, 'hooks');

  // Create symlinks from .claude-plugin to main directories
  console.log('Creating symlinks...');

  const pluginSkillsDir = path.join(pluginDir, 'skills');
  const pluginAgentsDir = path.join(pluginDir, 'agents');
  const pluginHooksDir = path.join(pluginDir, 'hooks');

  // For skills, we need to symlink the entire skills directory structure
  await createSymlink(skillsDir, pluginSkillsDir);
  await createSymlink(agentsDir, pluginAgentsDir);
  await createSymlink(hooksDir, pluginHooksDir);

  console.log('\n✨ Marketplace build complete!');
  console.log('\nStructure:');
  console.log('  .claude-plugin/');
  console.log('  ├── plugin.json');
  console.log('  ├── skills/ -> ../skills/');
  console.log('  ├── agents/ -> ../agents/');
  console.log('  └── hooks/ -> ../hooks/');
  console.log('\n📦 Ready for distribution');
}

async function main() {
  try {
    await buildMarketplace();
  } catch (error) {
    console.error('Build failed:', error);
    process.exit(1);
  }
}

main();
