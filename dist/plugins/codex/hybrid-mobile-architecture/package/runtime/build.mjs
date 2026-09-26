import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
const runtime = dirname(fileURLToPath(import.meta.url)), root = resolve(runtime, '..');
const result = spawnSync(process.execPath, [join(runtime, 'node_modules/typescript/bin/tsc'), '-p', join(runtime, 'tsconfig.json')], { stdio: 'inherit', shell: false });
if (result.error) throw result.error;
if (result.status) process.exit(result.status);
for (const directory of ['templates/project-skills/hooks', '.claude/hooks', '.kimi-code/hooks']) {
  await mkdir(join(root, directory), { recursive: true });
  for (const hook of ['skill-activation', 'a11y-reminder']) await cp(join(root, 'scripts', `${hook}.mjs`), join(root, directory, `${hook}.mjs`));
}
const hooks = { hooks: {
  UserPromptSubmit: [{ matcher: '*', hooks: [{ type: 'command', command: 'node', args: ['.claude/hooks/skill-activation.mjs'], description: 'Advisory KnowMe Builder skill activation' }] }],
  PostToolUse: [{ matcher: 'Write|Edit', hooks: [{ type: 'command', command: 'node', args: ['.claude/hooks/a11y-reminder.mjs'], description: 'Advisory accessibility review reminder' }] }],
} };
await writeFile(join(root, 'templates/project-skills/settings.hooks.json'), `${JSON.stringify(hooks, null, 2)}\n`);
const { build } = await import('esbuild');
const helperOutputs = {
  'gen-design-tokens': ['scripts/gen-design-tokens.mjs'],
  'verify-scaffold': ['scripts/verify-scaffold.mjs'],
  'verify-tauri-boot': ['scripts/verify-tauri-boot.mjs'],
  'verify-tauri-ui-restart': ['scripts/verify-tauri-ui-restart.mjs'],
  'mark-generated-sources': ['scripts/mark-generated-sources.mjs'],
  'install-tauri-webdriver': ['scripts/install-tauri-webdriver.mjs'],
  'install-flutter': ['scripts/install-flutter.mjs'],
  'worktree-consolidation-inventory': ['scripts/worktree-consolidation-inventory.mjs'],
  'consolidate-prometheus-wikis': ['scripts/consolidate-prometheus-wikis.mjs'],
  'tray-templates-negative': ['scripts/test-fixtures/tray-templates-negative.mjs'],
  'cleanup-antigravity-ide-exts': ['cleanup_antigravity_ide_exts.mjs'],
  'scaffold-tauri-tray': ['scripts/scaffold-tauri-tray.mjs'],
  'verify-tray-templates': ['scripts/verify-tray-templates.mjs'],
  'patch-cargokit-ios': ['scripts/patch-cargokit-ios.mjs'],
  'verify-flutter-ios-restart': ['scripts/verify-flutter-ios-restart.mjs'],
  'render-supervisor-plist': ['scripts/render-supervisor-plist.mjs'],
  'validate-catalog': ['deploy/scripts/validate-catalog.mjs'],
  'validate-gitops': ['deploy/scripts/validate-gitops.mjs'],
  'verify-postgres': ['deploy/scripts/verify-postgres.mjs'],
  'android-build': ['assets/templates/scripts/android/build.mjs'],
  'ios-build-xcframework': ['assets/templates/scripts/ios/build-xcframework.mjs'],
  'android-device-gates': ['assets/templates/scripts/android/verify-device-runtime-gates.mjs'],
  'android-native-gates': ['assets/templates/scripts/android/verify-native-inference-gates.mjs'],
  'docusaurus-scaffold': ['skills/build-branded-docusaurus/scripts/scaffold.mjs', 'templates/project-skills/build-branded-docusaurus/scripts/scaffold.mjs'],
  'docusaurus-site-build': ['skills/build-branded-docusaurus/scripts/build-site.mjs', 'templates/project-skills/build-branded-docusaurus/scripts/build-site.mjs'],
  'docusaurus-verify': ['skills/build-branded-docusaurus/scripts/verify.mjs', 'templates/project-skills/build-branded-docusaurus/scripts/verify.mjs'],
  'record-progress': ['skills/karpathy-progress-memory/scripts/record-progress.mjs', 'templates/project-skills/karpathy-progress-memory/scripts/record-progress.mjs'],
};
for (const [entry, outputs] of Object.entries(helperOutputs)) {
  for (const output of outputs) await build({ absWorkingDir: runtime, entryPoints: [join(runtime, 'src/native-helpers', `${entry}.mts`)], outfile: join(root, output), bundle: true, platform: 'node', format: 'esm', target: 'node22', sourcemap: false, banner: { js: "// TJ-ARCH-MOB-001 compliant\nimport { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);" }, logLevel: 'warning' });
}
await cp(join(root, 'scripts/opencode-skill-activation.mjs'), join(root, '.opencode/hooks/skill-activation.mjs'));
await cp(join(root, 'scripts/opencode-plugin.mjs'), join(root, '.opencode/plugins/knowme-builder.mjs'));
await build({
  absWorkingDir: runtime,
  entryPoints: [join(runtime, 'src/ci-deployment.mts')],
  outfile: join(root, 'scripts/ci-deployment.mjs'),
  bundle: true,
  platform: 'node',
  format: 'esm',
  target: 'node22',
  banner: { js: "// TJ-ARCH-MOB-001 compliant\nimport { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);" },
  logLevel: 'warning',
});
