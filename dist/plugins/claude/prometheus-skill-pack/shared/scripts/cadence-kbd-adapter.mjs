import { fileURLToPath } from 'node:url';
import { parseArgs } from '../lib/cadence-adapters/io.mjs';
import { kbdMain } from '../lib/cadence-adapters/kbd.mjs';

try {
  const result = await kbdMain(parseArgs(process.argv.slice(2)), { entry: fileURLToPath(import.meta.url) });
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch (error) {
  process.stderr.write(`${JSON.stringify({ error: error.message })}\n`);
  process.exitCode = 1;
}
