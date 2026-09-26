// TJ-ARCH-MOB-001 compliant
import { main } from './portable/platform.mjs';
import { scaffold } from './portable/builder.mjs';
await main(() => scaffold('flutter-mobile'));
