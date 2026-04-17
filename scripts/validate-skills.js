#!/usr/bin/env node

/**
 * Validates skills against agentskills.io specification
 * Recursively scans sub-skills within each skill directory
 */

import fs from 'fs/promises';
import path from 'path';
import yaml from 'js-yaml';
import Ajv from 'ajv';

const ajv = new Ajv();

// Schema for SKILL.md frontmatter
const frontmatterSchema = {
  type: 'object',
  required: ['name', 'description'],
  properties: {
    name: {
      type: 'string',
      pattern: '^[a-z0-9]+(-[a-z0-9]+)*$',
      maxLength: 64,
    },
    description: {
      type: 'string',
      minLength: 1,
      maxLength: 1024,
    },
    license: { type: 'string' },
    compatibility: { type: 'string', maxLength: 500 },
    metadata: { type: 'object' },
    'allowed-tools': { type: 'string' },
  },
};

const validateFrontmatter = ajv.compile(frontmatterSchema);

class SkillValidator {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.skillCount = 0;
  }

  async validateSkill(skillPath, { isSubSkill = false, visited = null } = {}) {
    // Initialize visited set on the first (top-level) call. Threaded through
    // recursion so we detect both symlink loops and nested-clone recursion.
    if (visited === null) {
      visited = new Set();
    }

    // Resolve to an absolute, canonical path for comparison. realpath follows
    // symlinks to their real target, so a symlink loop resolves to a path
    // already in the set on the second visit.
    let canonicalPath;
    try {
      canonicalPath = await fs.realpath(skillPath);
    } catch {
      canonicalPath = path.resolve(skillPath);
    }

    if (visited.has(canonicalPath)) {
      // Already validated (or currently validating) this path. Skip to prevent
      // infinite recursion from symlink loops or nested-clone structures.
      return;
    }
    visited.add(canonicalPath);

    const skillName = path.basename(skillPath);
    const prefix = isSubSkill ? '  ' : '';
    const label = isSubSkill ? `sub-skill: ${skillName}` : skillName;
    console.log(`${prefix}\n${prefix}🔍 Validating ${label}`);

    this.skillCount++;

    try {
      // Check SKILL.md exists
      const skillMdPath = path.join(skillPath, 'SKILL.md');
      const skillMdExists = await this.fileExists(skillMdPath);

      if (!skillMdExists) {
        this.addError(skillName, 'SKILL.md is required but not found');
        return;
      }

      // Read and parse SKILL.md
      const content = await fs.readFile(skillMdPath, 'utf-8');
      const { frontmatter, body } = this.parseFrontmatter(content);

      // Validate frontmatter
      if (!frontmatter) {
        this.addError(skillName, 'SKILL.md must have YAML frontmatter');
        return;
      }

      const valid = validateFrontmatter(frontmatter);
      if (!valid) {
        validateFrontmatter.errors.forEach(err => {
          this.addError(skillName, `Frontmatter validation: ${err.message}`);
        });
      }

      // Check name matches directory
      if (frontmatter.name !== skillName) {
        this.addWarning(
          skillName,
          `Frontmatter name "${frontmatter.name}" doesn't match directory "${skillName}"`
        );
      }

      // Check body content
      if (!body || body.trim().length === 0) {
        this.addError(skillName, 'SKILL.md body cannot be empty');
      }

      // Check for Windows-style paths (ignore backslashes inside code blocks)
      if (body) {
        const bodyWithoutCodeBlocks = body.replace(/```[\s\S]*?```/g, '');
        if (bodyWithoutCodeBlocks.includes('\\')) {
          this.addWarning(
            skillName,
            'Found backslashes in SKILL.md (outside code blocks) - use forward slashes for paths'
          );
        }
      }

      // Check line count
      const lineCount = content.split('\n').length;
      if (lineCount > 800) {
        this.addWarning(skillName, `SKILL.md exceeds 800-line maximum (${lineCount} lines)`);
      } else if (lineCount > 500) {
        this.addWarning(
          skillName,
          `SKILL.md exceeds 500-line recommendation (${lineCount} lines) - consider moving content to references/`
        );
      }

      // Check optional directories
      await this.checkOptionalDir(skillPath, 'scripts', skillName);
      await this.checkOptionalDir(skillPath, 'references', skillName);
      await this.checkOptionalDir(skillPath, 'assets', skillName);

      // Validate scripts if present
      await this.validateScripts(skillPath, skillName);

      console.log(`${prefix}✅ ${skillName} validation complete`);

      // Recursively validate sub-skills in skills/ directory
      const subSkillsDir = path.join(skillPath, 'skills');
      if (await this.fileExists(subSkillsDir)) {
        // lstat to avoid following a skills/ symlink that points back into us.
        const subStats = await fs.lstat(subSkillsDir);
        if (subStats.isDirectory()) {
          const subDirs = await fs.readdir(subSkillsDir);
          for (const subDir of subDirs) {
            const subPath = path.join(subSkillsDir, subDir);
            const subDirStats = await fs.lstat(subPath);
            // Skip symlinks entirely — they're the #1 source of validation loops.
            if (subDirStats.isSymbolicLink()) continue;
            if (subDirStats.isDirectory()) {
              await this.validateSkill(subPath, { isSubSkill: true, visited });
            }
          }
        }
      }

      // Also scan sibling directories with SKILL.md (for skill suites like
      // prometheus-entity-skills where sub-skills are siblings, not under skills/)
      const entries = await fs.readdir(skillPath);
      for (const entry of entries) {
        if (
          entry === 'skills' ||
          entry === 'scripts' ||
          entry === 'references' ||
          entry === 'assets' ||
          entry === 'agents' ||
          entry === 'hooks' ||
          entry === 'prompts' ||
          entry === 'workflows' ||
          entry === '.claude-plugin' ||
          entry === 'node_modules' ||
          entry === 'target' ||
          entry === 'dist' ||
          entry === 'crates' ||
          entry === 'examples' ||
          entry === 'docs' ||
          entry === skillName ||  // defense against self-named nested dir / recursive clone
          entry.startsWith('_') ||
          entry.startsWith('.')
        ) {
          continue;
        }
        const entryPath = path.join(skillPath, entry);
        const entryStats = await fs.lstat(entryPath);
        // Skip symlinks.
        if (entryStats.isSymbolicLink()) continue;
        if (entryStats.isDirectory()) {
          const hasSkillMd = await this.fileExists(path.join(entryPath, 'SKILL.md'));
          if (hasSkillMd) {
            await this.validateSkill(entryPath, { isSubSkill: true, visited });
          }
        }
      }
    } catch (error) {
      this.addError(skillName, `Validation failed: ${error.message}`);
    }
  }

  parseFrontmatter(content) {
    const frontmatterRegex = /^---\n([\s\S]*?)\n---\n([\s\S]*)$/;
    const match = content.match(frontmatterRegex);

    if (!match) {
      return { frontmatter: null, body: content };
    }

    try {
      const frontmatter = yaml.load(match[1]);
      const body = match[2];
      return { frontmatter, body };
    } catch (error) {
      return { frontmatter: null, body: content };
    }
  }

  async fileExists(filePath) {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  async checkOptionalDir(skillPath, dirName, skillName) {
    const dirPath = path.join(skillPath, dirName);
    const exists = await this.fileExists(dirPath);

    if (exists) {
      const stats = await fs.stat(dirPath);
      if (!stats.isDirectory()) {
        this.addError(skillName, `${dirName} exists but is not a directory`);
      }
    }
  }

  async validateScripts(skillPath, skillName) {
    const scriptsDir = path.join(skillPath, 'scripts');
    const exists = await this.fileExists(scriptsDir);

    if (!exists) return;

    try {
      const files = await fs.readdir(scriptsDir);

      for (const file of files) {
        const filePath = path.join(scriptsDir, file);
        const stats = await fs.stat(filePath);

        if (stats.isFile()) {
          // Check if script is executable (Unix-like systems)
          if (process.platform !== 'win32') {
            const mode = stats.mode;
            const isExecutable = (mode & 0o111) !== 0;

            if (!isExecutable && (file.endsWith('.sh') || file.endsWith('.py'))) {
              this.addWarning(
                skillName,
                `Script ${file} is not executable - run: chmod +x scripts/${file}`
              );
            }
          }
        }
      }
    } catch (error) {
      this.addWarning(skillName, `Could not validate scripts: ${error.message}`);
    }
  }

  addError(skill, message) {
    this.errors.push({ skill, message });
    console.error(`  ❌ ERROR: ${message}`);
  }

  addWarning(skill, message) {
    this.warnings.push({ skill, message });
    console.warn(`  ⚠️  WARNING: ${message}`);
  }

  printSummary() {
    console.log('\n' + '='.repeat(60));
    console.log('VALIDATION SUMMARY');
    console.log('='.repeat(60));

    console.log(`\n📊 ${this.skillCount} skill(s) validated (including sub-skills)`);

    if (this.errors.length === 0 && this.warnings.length === 0) {
      console.log('\n✨ All skills valid! No errors or warnings.');
      return true;
    }

    if (this.errors.length > 0) {
      console.log(`\n❌ ${this.errors.length} ERROR(S) FOUND:`);
      this.errors.forEach(({ skill, message }) => {
        console.log(`   ${skill}: ${message}`);
      });
    }

    if (this.warnings.length > 0) {
      console.log(`\n⚠️  ${this.warnings.length} WARNING(S) FOUND:`);
      this.warnings.forEach(({ skill, message }) => {
        console.log(`   ${skill}: ${message}`);
      });
    }

    return this.errors.length === 0;
  }
}

async function findSkills(rootDir) {
  const skills = [];
  const skillsDir = path.join(rootDir, 'skills');

  try {
    const categories = await fs.readdir(skillsDir);

    for (const category of categories) {
      const categoryPath = path.join(skillsDir, category);
      const stats = await fs.stat(categoryPath);

      if (stats.isDirectory()) {
        const skillDirs = await fs.readdir(categoryPath);

        for (const skillDir of skillDirs) {
          // Skip README files and non-directories
          const skillPath = path.join(categoryPath, skillDir);
          const skillStats = await fs.stat(skillPath);

          if (skillStats.isDirectory()) {
            skills.push(skillPath);
          }
        }
      }
    }
  } catch (error) {
    console.error(`Error finding skills: ${error.message}`);
  }

  return skills;
}

async function main() {
  const args = process.argv.slice(2);
  const rootDir = process.cwd();
  const validator = new SkillValidator();

  console.log('🚀 Prometheus Skill Pack - Validator');
  console.log('='.repeat(60));

  let skillsToValidate;

  if (args.length > 0) {
    // Validate specific skill
    skillsToValidate = args.map(arg => path.resolve(rootDir, arg));
  } else {
    // Validate all skills
    skillsToValidate = await findSkills(rootDir);
  }

  if (skillsToValidate.length === 0) {
    console.log('No skills found to validate.');
    process.exit(0);
  }

  console.log(`Found ${skillsToValidate.length} top-level skill(s) to validate\n`);

  for (const skillPath of skillsToValidate) {
    await validator.validateSkill(skillPath);
  }

  const success = validator.printSummary();
  process.exit(success ? 0 : 1);
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
