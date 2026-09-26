// TJ-ARCH-MOB-001 compliant
import { main } from './portable/platform.mjs';
import { builder } from './portable/builder.mjs';
await main(() => {
  const args = process.argv.slice(2);
  if (args.length > 1 || args[0]?.startsWith('-')) throw new Error('Usage: node scripts/scaffold-packages.mjs [project-root]');
  builder(['add', 'module', 'shared-packages', '--path', args[0] ?? '.']);
});
