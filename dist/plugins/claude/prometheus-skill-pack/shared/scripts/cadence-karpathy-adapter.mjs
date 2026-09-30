import { parseArgs } from '../lib/cadence-adapters/io.mjs';
import { karpathyMain } from '../lib/cadence-adapters/karpathy.mjs';

try {
  const result = await karpathyMain(parseArgs(process.argv.slice(2)));
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch {
  process.stdout.write(`${JSON.stringify({ status: 'degraded', reason: 'invalid-learning-adapter-input' })}\n`);
}
