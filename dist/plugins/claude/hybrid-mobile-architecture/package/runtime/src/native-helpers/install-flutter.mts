import { existsSync, mkdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, dirname, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { assert, executable, main, run, ToolError } from './common.mjs';
import { versions } from '../portable/versions.mjs';
// Only Flutter's vendor bootstrap batch files are supported. cmd metacharacters
// and expansion are rejected; there is no general-purpose shell execution API.
function vendor(file: string, args: string[], cwd?: string): void {
  if (!file.endsWith('.bat')) { run(file, args, { cwd }); return; }
  for (const token of [file, ...args]) assert(!/[&|<>^%!"\r\n\0]/.test(token), 'vendor batch adapter refuses command expansion characters in SDK path or arguments');
  assert(['flutter.bat', 'dart.bat'].some(name => file.endsWith(name)), 'unsupported vendor batch entrypoint');
  const result = spawnSync(process.env.ComSpec ?? 'cmd.exe', ['/d', '/v:off', '/s', '/c', `"${[file, ...args].map(token => `"${token}"`).join(' ')}"`], { cwd, stdio: 'inherit', shell: false, windowsVerbatimArguments: true });
  if (result.error || result.status !== 0) throw new ToolError(`Flutter vendor bootstrap failed: ${result.error?.message ?? result.status}`, result.status ?? 1);
}
await main(() => {
  const pins = versions(), flutterVersion = pins.toolchain?.flutter;
  assert(flutterVersion && /^\d+\.\d+\.\d+$/.test(flutterVersion), 'toolchain.flutter must be an exact stable x.y.z version');
  assert(process.argv.length <= 3 && (!process.argv[2] || ['--fvm', '--help'].includes(process.argv[2])), 'usage: node scripts/install-flutter.mjs [--fvm]');
  if (process.argv[2] === '--help') { console.log(`node scripts/install-flutter.mjs [--fvm]\nInstalls pinned Flutter ${flutterVersion}. Windows bootstrap uses the vendor flutter.bat with a restricted adapter.`); return; }
  if (process.argv[2] === '--fvm') {
    const dart = executable('dart') ?? (process.env.PATH ?? '').split(delimiter).map(dir => join(dir, 'dart.bat')).find(existsSync);
    assert(dart, 'Dart is required to install FVM');
    vendor(dart, ['pub', 'global', 'activate', 'fvm']);
    for (const args of [['install', flutterVersion], ['global', flutterVersion], ['flutter', '--version']]) vendor(dart, ['pub', 'global', 'run', 'fvm:fvm', ...args]);
    console.log(`Add ${join(homedir(), '.pub-cache/bin')} to PATH. Use fvm flutter <command>.`); return;
  }
  const sdk = join(homedir(), 'development/flutter'), flutter = join(sdk, 'bin', process.platform === 'win32' ? 'flutter.bat' : 'flutter');
  if (existsSync(sdk)) { assert(existsSync(join(sdk, '.git')), `existing Flutter directory is not a Git checkout: ${sdk}`); run('git', ['fetch', '--tags', 'origin', flutterVersion], { cwd: sdk }); run('git', ['checkout', '--detach', flutterVersion], { cwd: sdk }); }
  else { mkdirSync(dirname(sdk), { recursive: true }); run('git', ['clone', '--branch', flutterVersion, '--depth', '1', 'https://github.com/flutter/flutter.git', sdk]); }
  process.env.PATH = `${join(sdk, 'bin')}${delimiter}${process.env.PATH ?? ''}`;
  vendor(flutter, ['doctor', '--no-color'], sdk); vendor(flutter, ['--version'], sdk);
  console.log(`Permanently add ${join(sdk, 'bin')} to PATH. Then run flutter doctor --android-licenses.`);
});
