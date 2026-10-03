import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { refresh, run } from './lifecycle.mjs';

export async function main(argv = process.argv.slice(2)) {
  const [command, ...rest] = argv;
  if (!['refresh', 'run'].includes(command)) throw new Error('Usage: refresh --project <root> [--timeout-ms N] | run --project <root> -- <openspec args>');
  const options = { project: process.cwd(), timeoutMs: 120000, args: [] };
  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === '--') { options.args = rest.slice(i + 1); break; }
    if (rest[i] === '--project' && rest[i + 1]) options.project = rest[++i];
    else if (rest[i] === '--timeout-ms' && rest[i + 1]) options.timeoutMs = Number(rest[++i]);
    else throw new Error(`Unknown or incomplete OpenSpec runner option: ${rest[i]}`);
  }
  if (!Number.isSafeInteger(options.timeoutMs) || options.timeoutMs < 1000) throw new Error('--timeout-ms must be an integer of at least 1000');
  if (command === 'run' && !options.args.length) throw new Error('run requires OpenSpec arguments after --');
  const result = await (command === 'refresh' ? refresh(options) : run(options));
  // Keep run stdout exclusively for the CLI, including its machine-readable JSON.
  process.stderr.write(`${JSON.stringify(result)}\n`);
  return result.code;
}

if (process.argv[1] && fs.realpathSync(process.argv[1]) === fs.realpathSync(fileURLToPath(import.meta.url))) {
  try { process.exitCode = await main(); }
  catch (error) { process.stderr.write(`openspec-runner: ${error.message}\n`); process.exitCode = 2; }
}
