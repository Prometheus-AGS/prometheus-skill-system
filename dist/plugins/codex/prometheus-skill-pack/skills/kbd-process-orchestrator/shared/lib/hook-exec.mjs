import { spawnSync } from 'node:child_process';

// Exec-form registrations cross a tool-execution boundary. Never reinterpret argv as shell text.
try {
  const entry = JSON.parse(process.argv[2]);
  if (typeof entry.command !== 'string' || !entry.command || !Array.isArray(entry.args) ||
      entry.args.some(arg => typeof arg !== 'string') || !Number.isFinite(entry.timeout) || entry.timeout <= 0) {
    throw new Error('Exec hook requires a command, string args, and a positive timeout');
  }
  const result = spawnSync(entry.command === 'node' ? process.execPath : entry.command, entry.args,
    { shell: false, stdio: 'inherit', timeout: entry.timeout * 1000 });
  process.exitCode = result.status ?? (result.error?.code === 'ETIMEDOUT' ? 124 : 1);
} catch {
  process.stderr.write('Invalid or unavailable exec-form hook\n');
  process.exitCode = 1;
}
