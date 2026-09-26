// TJ-ARCH-MOB-001 compliant
import { resolve } from 'node:path';
import { main, packageRoot } from './portable/platform.mjs';
import { verifyManifest } from './portable/manifest.mjs';
await main(() => {
  const args = process.argv.slice(2);
  if (args.length > 1 || args[0]?.startsWith('-')) throw new Error('Usage: node scripts/verify-skill-manifest.mjs [package-root]');
  const errors = verifyManifest(resolve(args[0] ?? packageRoot));
  if (errors.length) throw new Error(errors.join('\n'));
  process.stdout.write('Manifest structural install contract: 4/4 conditions hold.\n');
});
