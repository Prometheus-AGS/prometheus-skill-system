import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, run } from './common.mjs';
await main(() => {
  const args = process.argv.slice(2); assert(args.every(arg => ['--fast', '--keep', '--help', '-h'].includes(arg)), 'unknown option');
  if (args.includes('--help') || args.includes('-h')) { console.log('node scripts/verify-tray-templates.mjs [--fast] [--keep]\n--fast verifies only aggregator and returns 2 (PARTIAL).'); return; }
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..'), work = mkdtempSync(join(tmpdir(), 'hma-tray-'));
  let failures = 0;
  try {
    const target = join(work, 'app'), crate = join(target, 'crates/health-aggregator'), tray = join(target, 'src-tauri/src/tray.rs');
    run(process.execPath, [join(root, 'scripts/scaffold-tauri-tray.mjs'), target], { capture: true });
    assert(existsSync(join(crate, 'src/lib.rs')) && existsSync(tray), 'scaffold did not emit required templates');
    const check = (cwd: string, command: string[], label: string, log: string) => {
      const result = spawnSync('cargo', command, { cwd, encoding: 'utf8', env: { ...process.env, CARGO_TARGET_DIR: join(work, 'target') }, maxBuffer: 64 * 1024 * 1024, shell: false });
      const output = `${result.stdout ?? ''}${result.stderr ?? ''}${result.error?.message ?? ''}`; writeFileSync(join(work, log), output);
      if (result.status !== 0) { failures++; console.error(`${label}\n${output.split(/\r?\n/).slice(-25).join('\n')}`); } else console.log(`✓ ${label.replace('failed', 'passed').replace('does not compile', 'compiles cleanly')}`);
    };
    check(crate, ['clippy', '--all-targets', '--', '-D', 'warnings'], 'health-aggregator clippy failed', 'agg.out');
    check(crate, ['test'], 'health-aggregator tests failed', 'agg-test.out');
    if (!args.includes('--fast')) {
      const probe = join(work, 'tray-probe'); mkdirSync(join(probe, 'src'), { recursive: true });
      writeFileSync(join(probe, 'Cargo.toml'), '[package]\nname = "tray-probe"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ntauri = { version = "2", features = ["tray-icon"] }\n');
      copyFileSync(tray, join(probe, 'src/lib.rs'));
      check(probe, ['clippy', '--', '-D', 'warnings'], 'tray.rs does not compile', 'tray.out');
    }
    assert(failures === 0, `verify-tray-templates: FAIL — ${failures} problem(s); scaffold ships code that does not build`);
    assert(!args.includes('--fast'), 'verify-tray-templates: PARTIAL — health-aggregator only; tray.rs was NOT built (--fast). Run without --fast to gate the templates.', 2);
    console.log('verify-tray-templates: PASS — every rendered template builds');
  } finally { if (args.includes('--keep')) console.log(`scratch kept at ${work}`); else rmSync(work, { recursive: true, force: true }); }
});
