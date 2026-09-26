import { assert, main, run } from './common.mjs';
await main(() => {
  const container = process.argv[2] ?? 'prometheus-postgres18-verification';
  const actual = run('docker', ['exec', container, 'psql', '-U', 'flint', '-d', 'flint', '-Atc', 'select extname from pg_extension order by extname'], { capture: true }).trim().split(/\r?\n/);
  for (const extension of ['vector', 'pgcrypto', 'pg_net', 'pg_cron', 'flint_llm', 'flint_vault', 'flint_meta', 'flint_auth', 'flint_hooks']) assert(actual.includes(extension), `missing extension: ${extension}`);
  run('docker', ['exec', container, 'wal-g', '--version']);
  assert(run('docker', ['exec', container, 'psql', '-U', 'flint', '-d', 'flint', '-Atc', 'show wal_level'], { capture: true }).trim() === 'logical', 'wal_level must be logical');
  console.log('PostgreSQL extension and replication verification passed');
});
