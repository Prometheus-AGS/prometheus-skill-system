import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, run } from './common.mjs';
await main(() => {
  const profile = process.argv[2] ?? 'release';
  assert(['release', 'debug'].includes(profile), 'profile must be release or debug', 64);
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..'), rust = join(root, 'rust');
  const sdk = process.env.ANDROID_SDK_ROOT ?? process.env.ANDROID_HOME ?? (process.platform === 'win32' ? join(process.env.LOCALAPPDATA ?? join(homedir(), 'AppData/Local'), 'Android/Sdk') : process.platform === 'darwin' ? join(homedir(), 'Library/Android/sdk') : join(homedir(), 'Android/Sdk'));
  const ndks = join(sdk, 'ndk');
  const latest = existsSync(ndks) ? readdirSync(ndks, { withFileTypes: true }).filter(entry => entry.isDirectory()).map(entry => entry.name).sort((a, b) => a.localeCompare(b, 'en', { numeric: true })).at(-1) : undefined;
  const ndk = process.env.ANDROID_NDK ?? (latest ? join(ndks, latest) : '');
  assert(ndk && existsSync(ndk), `Android NDK not found. Set ANDROID_NDK or install it in ${ndks}.`);
  run('cargo', ['ndk', '--target', 'arm64-v8a', '--platform', '29', '--', 'build', '-p', 'gen_ui_ffi', '--target', 'aarch64-linux-android', ...(profile === 'release' ? ['--release'] : [])], { cwd: rust, env: { ...process.env, ANDROID_NDK: ndk, NDK_ROOT: ndk } });
  console.log('✓ arm64-v8a');
  // A stale Dart bridge is not a successful build: codegen is required.
  run('flutter_rust_bridge_codegen', ['generate', '--config-file', join(rust, 'flutter_rust_bridge.yaml')], { cwd: root });
  console.log('✓ Dart bindings generated');
});
