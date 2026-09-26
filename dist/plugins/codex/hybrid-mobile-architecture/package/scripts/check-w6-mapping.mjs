// TJ-ARCH-MOB-001 compliant
import { main, packageRoot } from './portable/platform.mjs';
import { checkW6 } from './portable/w6.mjs';
await main(() => { if (process.argv.length !== 2)
    throw new Error('No arguments accepted'); const errors = checkW6(packageRoot); if (errors.length)
    throw new Error(errors.join('\n')); process.stdout.write('W6/R6 mapping verified.\n'); });
