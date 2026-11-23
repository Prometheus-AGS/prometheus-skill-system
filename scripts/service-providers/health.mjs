import {manageServices} from './index.mjs';
import native from './native.mjs';
try {
  const status = await manageServices('status',{home:process.env.HOME},{native});
  const excluded = new Set((process.argv[3] || '').trim().split(/\s+/));
  const names = {'pk-cherry':'prometheus-knowledge','forge-mcp':'forge-rs'};
  let failed = false;
  for (const s of status.services) {
    if (excluded.has(s.id) || excluded.has(names[s.id])) continue;
    const row = {name:names[s.id] || s.id,label:s.id,service:s.provider,owner:s.owner,url:s.endpoint,status:s.status,healthy:s.healthy};
    if (process.argv[2] === 'true') console.log(JSON.stringify(row));
    else console.log([row.name.padEnd(21), row.service.padEnd(8), row.status, row.url || ''].join('  '));
    if (!s.healthy) failed = true;
  }
  if (failed) process.exitCode=1;
} catch (error) {
  console.error('health: '+error.message);
  process.exitCode=1;
}
