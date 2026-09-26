// TJ-ARCH-MOB-001 compliant
import { resolve } from 'node:path';
import { main, packageRoot } from './portable/platform.mjs';
import { settingsFile, normalize } from './portable/hook-config.mjs';
await main(async () => {
  const args = process.argv.slice(2), positional = args.filter(arg => arg !== '--dry-run');
  if (positional.length > 1 || positional[0]?.startsWith('-')) throw new Error('Usage: node scripts/install-hooks-reliability.mjs [--dry-run] [root]');
  const file = settingsFile(resolve(positional[0] ?? packageRoot));
  if (file) await normalize(file, args.includes('--dry-run'));
  process.stdout.write(file ? `${args.includes('--dry-run') ? 'Checked' : 'Normalized'} ${file}\n` : 'No hook configuration.\n');
});
