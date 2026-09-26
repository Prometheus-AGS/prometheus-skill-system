import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, executable, main, run, text } from './common.mjs';
await main(() => {
  const values: Record<string, string> = {}, args: string[] = [], env: string[] = [], input = process.argv.slice(2);
  for (let i = 0; i < input.length; i++) {
    const flag = input[i]!;
    if (flag === '--help' || flag === '-h') { console.log('node scripts/render-supervisor-plist.mjs --label NAME --program PATH [--arg ARG] [--env KEY=VALUE] [--format launchd|systemd] [--throttle 15] [--description TEXT] [--log-dir DIR] [--working-dir DIR] [--out FILE]'); return; }
    assert(['--label', '--program', '--arg', '--env', '--format', '--throttle', '--description', '--log-dir', '--working-dir', '--out'].includes(flag) && input[i + 1] !== undefined, `unknown option or missing value: ${flag}`);
    const value = input[++i]!; assert(!/[\r\n\0]/.test(value), `${flag} must be a single line`);
    if (flag === '--arg') args.push(value); else if (flag === '--env') { assert(/^[A-Za-z_][A-Za-z0-9_]*=/.test(value), '--env requires KEY=VALUE'); env.push(value); } else values[flag.slice(2)] = value;
  }
  const label = values.label, program = values.program, throttle = values.throttle ?? '15';
  assert(label && /^[A-Za-z0-9._-]+$/.test(label), 'a valid --label is required'); assert(program, '--program is required');
  assert(/^\d+$/.test(throttle) && Number.isSafeInteger(Number(throttle)) && Number(throttle) >= 10, '--throttle must be an integer at or above the 10s floor (R1.1)');
  assert(process.platform !== 'win32' || values.format, 'Windows has no launchd/systemd default; choose --format explicitly for a deployment target');
  const format = values.format ?? (process.platform === 'darwin' ? 'launchd' : 'systemd');
  assert(['launchd', 'systemd'].includes(format), 'unknown --format (want launchd or systemd)');
  const log = values['log-dir'] ?? '$HOME/.prometheus/logs', work = values['working-dir'] ?? '$HOME';
  const xml = (value: string) => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  const sd = (value: string) => value.replaceAll('%', '%%').replace('$HOME', '%h');
  const quote = (value: string, command = true) => '"' + value.replaceAll('\\', '\\\\').replaceAll('"', '\\"').replaceAll('$', command ? '$$' : '$').replaceAll('%', '%%') + '"';
  const substitutions: Record<string, string> = { LABEL: label, THROTTLE: throttle, DESCRIPTION: sd(values.description ?? label), STDOUT_PATH: `${log}/${label}.log`, STDERR_PATH: `${log}/${label}.err`, WORKING_DIRECTORY: work };
  if (format === 'launchd') {
    for (const key of ['STDOUT_PATH', 'STDERR_PATH', 'WORKING_DIRECTORY']) substitutions[key] = xml(substitutions[key]!);
    substitutions.PROGRAM_ARGUMENTS = [program, ...args].map(arg => `    <string>${xml(arg)}</string>`).join('\n');
    substitutions.ENVIRONMENT = ['PATH=/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin', ...env].map(pair => { const index = pair.indexOf('='); return `    <key>${xml(pair.slice(0, index))}</key>\n    <string>${xml(pair.slice(index + 1))}</string>`; }).join('\n');
  } else {
    for (const key of ['STDOUT_PATH', 'STDERR_PATH', 'WORKING_DIRECTORY']) substitutions[key] = sd(substitutions[key]!);
    substitutions.EXEC_START = [program, ...args].map(value => quote(value)).join(' ');
    substitutions.ENVIRONMENT = ['PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin', ...env].map(pair => `Environment=${quote(pair, false)}`).join('\n');
  }
  const template = resolve(dirname(fileURLToPath(import.meta.url)), '../assets/templates/launchagent-supervisor', format === 'launchd' ? 'supervisor.plist.template' : 'supervisor.service.template');
  let rendered = text(template); for (const [key, value] of Object.entries(substitutions)) rendered = rendered.replaceAll(`@@${key}@@`, () => value);
  rendered = rendered.trimEnd() + '\n';
  if (values.out) { mkdirSync(dirname(resolve(values.out)), { recursive: true }); writeFileSync(values.out, rendered); if (format === 'launchd' && executable('plutil')) run('plutil', ['-lint', values.out], { capture: true }); console.log(`render-supervisor-plist: wrote ${values.out} (${format})`); } else process.stdout.write(rendered);
});
