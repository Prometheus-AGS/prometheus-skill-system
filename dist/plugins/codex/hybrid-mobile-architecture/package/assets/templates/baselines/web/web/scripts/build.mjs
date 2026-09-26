// TJ-ARCH-MOB-001 compliant
import { spawnSync } from 'node:child_process';
import { build } from 'vite';
const result = spawnSync(process.execPath, ['node_modules/typescript/bin/tsc', '--noEmit'], { stdio: 'inherit', shell: false });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
await build();
