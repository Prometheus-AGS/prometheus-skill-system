// TJ-ARCH-MOB-001 compliant
import { resolve } from 'node:path';
import { main, packageRoot, json } from './portable/platform.mjs';
import { settingsFile, inspect, inspectFiles } from './portable/hook-config.mjs';
await main(() => {
  const args = process.argv.slice(2);
  if (args.length > 1 || args[0]?.startsWith('-')) throw new Error('Usage: node scripts/verify-hooks-reliability.mjs [root]');
  const root = resolve(args[0] ?? packageRoot), file = settingsFile(root);
  if (!file) { process.stdout.write('No hook configuration.\n'); return; }
  const issues = [...inspect(json(file)), ...inspectFiles(root)];
  if (issues.length) throw new Error(issues.join('\n'));
  process.stdout.write(`Hook reliability checks passed: ${file}\n`);
});
