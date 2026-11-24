// Resolve an authoritative status snapshot through the same projection used by method-context.
import { existsSync, readFileSync, realpathSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const [root, phaseId] = process.argv.slice(2);
const modules = ['skills/process/kbd-process-orchestrator', 'skills/kbd-process-orchestrator']
  .map(skill => new URL(`../../../${skill}/shared/method-context.mjs`, import.meta.url));
try {
  const resolver = modules.find(url => existsSync(fileURLToPath(url)));
  if (!resolver) throw new Error('method-context resolver missing from pack');
  const { phaseDirectory } = await import(resolver);
  const status = JSON.parse(readFileSync(0, 'utf8'));
  // Preserve the caller's project-root spelling when it includes a symlink.
  const canonicalRoot = realpathSync(root);
  const directory = phaseDirectory(canonicalRoot, status, phaseId);
  process.stdout.write(path.join(root, path.relative(canonicalRoot, directory)) + '\n');
} catch (error) {
  process.stderr.write(`kbd-phase-directory: ${error.message}\n`);
  process.exitCode = 2;
}
