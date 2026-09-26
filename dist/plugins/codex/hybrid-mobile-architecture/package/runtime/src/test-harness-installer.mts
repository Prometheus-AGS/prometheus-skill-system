// TJ-ARCH-MOB-001 compliant
import { join } from 'node:path';
import { main, packageRoot, run } from './portable/platform.mjs';
await main(() => {
  if (process.argv.length !== 2) throw new Error('Usage: node scripts/test-harness-installer.mjs');
  process.exitCode = run(process.execPath, ['--test', join(packageRoot, 'runtime/tests/harness-installer.test.mjs')], { allowFailure: true }).status;
});
