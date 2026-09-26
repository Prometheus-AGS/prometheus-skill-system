import { copyFileSync, existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, npm, text } from './common.mjs';
await main(() => {
  assert(process.argv.length === 6, 'usage: node scaffold.mjs SITE_DIR SITE_NAME SITE_URL BASE_URL', 64);
  const [siteArg, name, url, base] = process.argv.slice(2) as [string, string, string, string];
  const site = resolve(siteArg), assets = resolve(dirname(fileURLToPath(import.meta.url)), '../assets');
  assert(!existsSync(site), `refusing to overwrite existing path: ${site}`);
  mkdirSync(dirname(site), { recursive: true });
  npm(['exec', '--yes', '--package=create-docusaurus@3.10.1', '--', 'create-docusaurus', basename(site), 'classic', '--javascript', '--package-manager', 'npm', '--skip-install'], dirname(site));
  mkdirSync(join(site, 'scripts'), { recursive: true });
  rmSync(join(site, 'docusaurus.config.js'), { force: true });
  for (const [source, destination] of [['docusaurus.config.mjs', 'docusaurus.config.mjs'], ['flat2.css', 'src/css/custom.css'], ['index.jsx', 'src/pages/index.js'], ['content-sources.yaml', 'content-sources.yaml'], ['sanitize.mjs', 'scripts/sanitize.mjs']]) copyFileSync(join(assets, source!), join(site, destination!));
  copyFileSync(resolve(assets, '../scripts/build-site.mjs'), join(site, 'scripts/build.mjs'));
  const configPath = join(site, 'docusaurus.config.mjs');
  // Values live in quoted JS literals in this template. Escape literal content.
  const literal = (value: string) => JSON.stringify(value).slice(1, -1).replaceAll("'", "\\'");
  writeFileSync(configPath, text(configPath).replaceAll('__SITE_NAME__', () => literal(name)).replaceAll('__SITE_URL__', () => literal(url)).replaceAll('__BASE_URL__', () => literal(base)));
  const packagePath = join(site, 'package.json');
  const pkg = JSON.parse(text(packagePath)) as { scripts?: Record<string, string>; overrides?: Record<string, string> };
  pkg.scripts = { ...pkg.scripts, sanitize: 'node scripts/sanitize.mjs', build: 'node scripts/build.mjs' };
  pkg.overrides = { ...pkg.overrides, 'serialize-javascript': '7.0.5' };
  writeFileSync(packagePath, JSON.stringify(pkg, null, 2) + '\n');
  npm(['install', '--save-exact', '@docusaurus/core@3.10.1', '@docusaurus/preset-classic@3.10.1', '@docusaurus/theme-mermaid@3.10.1', '@mdx-js/react@3.1.1', '@easyops-cn/docusaurus-search-local@0.55.2', 'mermaid@11.16.0'], site);
  npm(['install', '--package-lock-only'], site);
  console.log(`scaffolded ${site}; classify sources before replacing starter content`);
});
