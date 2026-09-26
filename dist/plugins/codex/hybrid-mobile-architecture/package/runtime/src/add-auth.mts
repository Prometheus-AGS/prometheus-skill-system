// TJ-ARCH-MOB-001 compliant
import { main } from './portable/platform.mjs';
import { builder } from './portable/builder.mjs';
await main(() => {
  const args = process.argv.slice(2);
  if (!args.length || args.length > 2 || args.some(a => a.startsWith('-'))) throw new Error('Usage: node scripts/add-auth.mjs <name> [project-root]. Legacy platform arguments are not accepted.');
  builder(['add', 'auth', args[0], '--path', args[1] ?? '.']);
});
