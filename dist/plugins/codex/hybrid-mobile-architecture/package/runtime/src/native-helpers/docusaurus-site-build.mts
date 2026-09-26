import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, run, text } from './common.mjs';
await main(() => {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  run(process.execPath, [join(root, 'scripts/sanitize.mjs')], { cwd: root });
  const manifest = createRequire(import.meta.url).resolve('@docusaurus/core/package.json');
  const pkg = JSON.parse(text(manifest)) as { bin?: string | Record<string, string> };
  const bin = typeof pkg.bin === 'string' ? pkg.bin : pkg.bin?.docusaurus;
  assert(bin, 'installed @docusaurus/core does not expose the docusaurus CLI');
  run(process.execPath, [resolve(dirname(manifest), bin), 'build', ...process.argv.slice(2)], { cwd: root });
});
