// TJ-ARCH-MOB-001 compliant
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { main, packageRoot, json, writeJson, run, ProcessFailure } from './portable/platform.mjs';
await main(async () => {
  const args = process.argv.slice(2), values: Record<string, string> = { source: 'user' }; let action = '', ticketId = '';
  for (let index = 0; index < args.length; index++) {
    const key = args[index].replace(/^--/, '');
    if (key === 'list') { if (action) throw new Error('Choose one action'); action = key; }
    else if (['show', 'approve', 'reject', 'verify', 'ship'].includes(key)) { if (action || !args[index + 1]) throw new Error('Choose one action and ticket ID'); action = key; ticketId = args[++index]; }
    else if (['skill', 'evidence', 'source', 'reason', 'replay-argv'].includes(key) && args[index + 1]) values[key] = args[++index];
    else if (key === 'replay') throw new Error('Shell replay is not portable. Use --replay-argv with a JSON array [executable, ...arguments].');
    else throw new Error(`Unknown or incomplete option: ${args[index]}`);
  }
  const directory = join(packageRoot, '.prometheus/refiner');
  if (action === 'list') {
    const names = existsSync(directory) ? readdirSync(directory).filter(name => name.endsWith('.json')).sort() : [];
    for (const name of names) { const ticket = json(join(directory, name)); process.stdout.write(`${ticket.id}\t${ticket.status}\t${ticket.skill}\t${ticket.source}\n`); }
    if (!names.length) process.stdout.write('No tickets.\n'); return;
  }
  if (action) {
    if (!/^[A-Za-z0-9-]+$/.test(ticketId)) throw new Error('Unsafe ticket ID');
    const file = join(directory, `${ticketId}.json`), ticket = json(file);
    if (action === 'show') { process.stdout.write(`${JSON.stringify(ticket, null, 2)}\n`); return; }
    if (action === 'approve') { if (ticket.status !== 'triaged') throw new Error('Only triaged tickets can be approved'); ticket.status = 'approved'; ticket.approvedAt = new Date().toISOString(); }
    if (action === 'reject') { if (!values.reason) throw new Error('--reject requires --reason'); ticket.status = 'rejected'; ticket.rejectedAt = new Date().toISOString(); ticket.rejectionReason = values.reason; }
    if (action === 'ship') { if (ticket.status !== 'verified' || ticket.replayed !== true || !ticket.verifiedAt || !Array.isArray(ticket.replayArgv) || !ticket.replayArgv.length) throw new Error('Ship requires successful verification with a recorded typed replay'); ticket.status = 'shipped'; ticket.shippedAt = new Date().toISOString(); }
    if (action === 'verify') {
      if (!['approved', 'refining', 'verified'].includes(String(ticket.status))) throw new Error('Verification requires approval');
      ticket.replayed = false; ticket.verifiedAt = ''; ticket.status = 'approved'; await writeJson(file, ticket);
      if (typeof ticket.replay === 'string' && ticket.replay) throw new Error('Legacy ticket contains a shell replay; replace it through a reviewed ticket migration to replayArgv before verification');
      const replay = ticket.replayArgv;
      if (!Array.isArray(replay) || !replay.length || replay.some(value => typeof value !== 'string') || !replay[0]) throw new Error('Verification requires a recorded typed replayArgv [executable, ...arguments]; the reported failure must be re-executed');
      for (const name of ['generate-skill-metadata', 'generate-skill-evals', 'check-skill-contracts', 'sync-harness-skills']) run(process.execPath, [join(packageRoot, 'scripts', `${name}.mjs`), ...(name === 'sync-harness-skills' ? ['--check'] : [])], { cwd: packageRoot });
      run(replay[0], replay.slice(1), { cwd: packageRoot }); ticket.replayed = true;
      ticket.status = 'verified'; ticket.verifiedAt = new Date().toISOString();
    }
    await writeJson(file, ticket); process.stdout.write(`Ticket ${ticketId}: ${ticket.status}\n`); return;
  }
  if (!values.skill || !/^[a-z0-9-]+$/.test(values.skill) || !existsSync(join(packageRoot, 'skills', values.skill, 'SKILL.md'))) throw new Error('Valid --skill required');
  if (!values.evidence || !['log', 'sycophancy', 'user'].includes(values.source)) throw new Error('--evidence and source log|sycophancy|user required');
  const replay: unknown = values['replay-argv'] ? JSON.parse(values['replay-argv']) : undefined;
  if (replay !== undefined && (!Array.isArray(replay) || !replay.length || replay.some(value => typeof value !== 'string'))) throw new Error('--replay-argv must be a nonempty string array');
  const description = (name: string) => readFileSync(join(packageRoot, 'skills', name, 'SKILL.md'), 'utf8').match(/^---\r?\n([\s\S]*?)\r?\n---/)?.[1].match(/^description:\s*(.+)$/m)?.[1].toLowerCase() ?? '';
  const stop = new Set('triggers invoke always before after when which these there their would should every using'.split(' '));
  const words = new Set((description(values.skill).match(/[a-z][a-z-]{4,}/g) ?? []).filter(word => !stop.has(word)));
  const neighbours = readdirSync(join(packageRoot, 'skills')).filter(name => name !== values.skill && existsSync(join(packageRoot, 'skills', name, 'SKILL.md'))).map(name => ({ name, shared: [...new Set(description(name).match(/[a-z][a-z-]{4,}/g) ?? [])].filter(word => words.has(word)).sort() })).filter(row => row.shared.length >= 3).sort((a, b) => b.shared.length - a.shared.length || b.name.localeCompare(a.name)).slice(0, 5);
  const now = new Date().toISOString(), id = `${now.replace(/[-:]/g, '').slice(0, 15).replace('T', '-')}-${values.skill}-${randomUUID().slice(0, 8)}`;
  await writeJson(join(directory, `${id}.json`), { id, skill: values.skill, source: values.source, evidence: values.evidence, replayArgv: replay, replayed: false, detectedAt: now, status: 'triaged', triagedAt: now, neighbours });
  process.stdout.write(`Ticket ${id} triaged. Human approval required before refinement.\n${neighbours.map(row => `${row.name}: ${row.shared.slice(0, 6).join(', ')}`).join('\n')}\nApprove: node scripts/refiner-loop.mjs --approve ${id}\n`);
});
