// Build-only overlay for the pinned upstream cowork CLI. Its config installer
// needs the Windows directory-link API already used by its skill installer.
import { execFileSync } from 'node:child_process';
import { mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const source = 'tools/cowork-skills';
const file = 'cli/src/commands/config.rs';
const preimage = '7017cad1c4c2b21134d1c8a3414091f5a5d4df01';
const patch = 'scripts/release/patches/cowork-windows-dev-link.patch';
const git = (cwd, ...args) => execFileSync('git', args, { cwd, encoding: 'utf8' }).trim();

function stage(destination) {
  const commit = git(root, 'rev-parse', 'HEAD');
  const link = git(root, 'ls-tree', commit, '--', source);
  const match = /^160000 commit ([a-f0-9]{40})\ttools\/cowork-skills$/.exec(link);
  if (!match) throw new Error('The pack commit must pin tools/cowork-skills');
  const revision = match[1];
  const original = path.join(root, source);
  if (git(original, 'rev-parse', revision + ':' + file) !== preimage) {
    throw new Error('Pinned cowork source changed; review the Windows dev-link overlay');
  }
  mkdirSync(destination); // Caller supplies a private parent and an unused destination.
  try {
    git(root, 'clone', '--quiet', '--shared', '--no-checkout', '-c', 'core.autocrlf=false', '--', original, destination);
    git(destination, 'sparse-checkout', 'set', '--cone', 'cli');
    git(destination, 'checkout', '--quiet', '--detach', revision);
    // A CRLF checkout can change patch syntax, including empty context lines.
    // Preserve every byte from the same commit that selected the source gitlink.
    const bytes = execFileSync('git', ['cat-file', 'blob', commit + ':' + patch], { cwd: root });
    execFileSync('git', ['apply', '--index', '-'], { cwd: destination, input: bytes });
  } catch (error) {
    rmSync(destination, { recursive: true, force: true, maxRetries: 3 });
    throw error;
  }
}

if (process.argv.length !== 3) throw new Error('Usage: node stage-windows-cowork.mjs <new-directory>');
stage(path.resolve(process.argv[2]));
