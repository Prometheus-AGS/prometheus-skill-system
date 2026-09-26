import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, text } from './common.mjs';
await main(() => {
  const args = process.argv.slice(2); let target = '', crate = 'health-aggregator', tray = 'supervisor-tray', force = false;
  const usage = 'node scripts/scaffold-tauri-tray.mjs <target-root> [--crate-name NAME] [--tray-id ID] [--force]';
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    if (arg === '--help' || arg === '-h') { console.log(usage); return; }
    if (arg === '--force') force = true;
    else if (arg === '--crate-name' || arg === '--tray-id') { assert(args[i + 1], `${arg} needs a value`); const value = args[++i]!; if (arg === '--crate-name') crate = value; else tray = value; }
    else { assert(!arg.startsWith('-') && !target, `unknown option or extra target: ${arg}`); target = arg; }
  }
  assert(target, usage, 2); assert(/^[a-z0-9_-]+$/.test(crate), '--crate-name must be lowercase kebab or snake');
  assert(/^[A-Za-z0-9._-]+$/.test(tray), '--tray-id must contain letters, numbers, dots, underscores or hyphens');
  target = resolve(target);
  const templates = resolve(dirname(fileURLToPath(import.meta.url)), '../assets/templates/tauri-tray');
  for (const [template, destination] of [['health-aggregator/Cargo.toml.template', `crates/${crate}/Cargo.toml`], ['health-aggregator/src/lib.rs.template', `crates/${crate}/src/lib.rs`], ['tray.rs.template', 'src-tauri/src/tray.rs']]) {
    const output = join(target, destination!);
    if (existsSync(output) && !force) { console.log(`exists, left untouched: ${relative(target, output)} (use --force to overwrite)`); continue; }
    const content = text(join(templates, template!)).replaceAll('@@CRATE_NAME@@', crate).replaceAll('@@TRAY_ID@@', tray);
    mkdirSync(dirname(output), { recursive: true }); writeFileSync(output, content);
    console.log(`✓ ${relative(target, output)}`);
  }
  console.log(`Next steps:\n1. Add crates/${crate} and src-tauri to workspace members.\n2. Add ${crate} = { path = "../crates/${crate}" } to src-tauri dependencies.\n3. Declare mod tray; in Tauri setup call tray::apply_accessory_policy(app), tray::build_tray(app)?, and tray::intercept_dashboard_close(app). apply_accessory_policy requires &mut App.\n4. Define the popover window: frameless, transparent, always-on-top, skipTaskbar; keep the main dashboard window.\n5. Verify: cargo test -p ${crate}\nscaffolded into ${target}`);
});
