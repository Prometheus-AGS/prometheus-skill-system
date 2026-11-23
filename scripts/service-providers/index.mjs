import fs from 'node:fs';
import path from 'node:path';
import {randomUUID} from 'node:crypto';
import {createInterface} from 'node:readline/promises';
import {setTimeout as delay} from 'node:timers/promises';
import {atomic,context,defaultEndpoint,definitions,endpoint,envFor,localhost,prefix,profile,readState} from './catalog.mjs';
import {dockerEngine,engineForDiscovery,evidence,probe} from './discovery.mjs';
import {dockerAction,dockerValidate,initializeNamespace} from './docker.mjs';
import {inventory,imageFor} from './inventory.mjs';
import nativeDefault from './native.mjs';

export function readServiceState(options={}) {return readState(context(options));}
const modes=['auto','native','docker','reuse','off'];
function value(o,field,id,suffix) {return o[field]?.[id] ?? o.env[prefix(id)+'_'+suffix];}
function positivePort(val,id) {
  if(!/^\d+$/.test(String(val)) || Number(val)<1 || Number(val)>65535 || !definitions[id].ports) throw Error('Invalid port for '+id);
  return Number(val);
}
async function choose(o,native,state) {
  const enabled=new Set(o.enabled??await native.defaults(o));
  for(const id of [...enabled,...(o.only||[]),...Object.keys(o.selections||{}),...Object.keys(o.ports||{}),...Object.keys(o.endpoints||{})]) if(!o.ids.includes(id)) throw Error('Unknown service '+id+' for '+o.pack);
  const prompt=o.interactive && process.stdin.isTTY && process.stderr.isTTY ?createInterface({input:process.stdin,output:process.stderr}):null;
  const rows=[];
  try {
    for(const id of o.ids) {
      const prev=state.services[id], explicit=value(o,'selections',id,'MODE'), port=value(o,'ports',id,'PORT'), addr=value(o,'endpoints',id,'ENDPOINT');
      const active=enabled.has(id)||prev||explicit||addr||port;
      let mode=explicit ?? prev?.requested ?? (active?(o.env.PROMETHEUS_SERVICE_MODE || 'auto'):'off');
      if(prompt && !explicit && (!o.only || o.only.includes(id))) mode=(await prompt.question(id+' ['+modes.join('/')+'] ('+mode+'): ')).trim()||mode;
      if(!modes.includes(mode)) throw Error('Invalid service mode for '+id+': '+mode);
      const selected=port!=null?positivePort(port,id):undefined;
      if(addr && port!=null && Number(new URL(addr).port)!==selected) throw Error('Endpoint and port disagree for '+id);
      rows.push({id,requested:mode,explicit:explicit!==undefined,selected:addr?endpoint(id,addr):selected?defaultEndpoint(id,o,selected):undefined,previous:prev});
    }
  } finally {prompt?.close();}
  return rows;
}
function conflict(plan,id,reason) {plan.conflicts.push(id+': '+reason);plan.applicable=false;}
async function planWith(o,native) {
  const state=readState(o), rows=await choose(o,native,state);
  const plan={pack:o.pack,version:o.version,applicable:true,conflicts:[],services:[],env:{}};
  const enabled=rows.some(x=>x.requested!=='off');
  o.engine=enabled?engineForDiscovery(o,rows):{error:'No Docker services selected'};
  const records=enabled?evidence(o,o.engine):[];
  let release;
  for(const row of rows) {
    const {id,requested,selected,previous}=row;
    if(o.only && !o.only.includes(id)) {
      plan.services.push(previous || {id,requested:'off',provider:'off',owner:'external',endpoint:defaultEndpoint(id,o)});
      continue;
    }
    let address=selected || previous?.endpoint || defaultEndpoint(id,o);
    const s={id,requested,provider:requested,endpoint:address,owner:'pack'};
    plan.services.push(s);
    if(requested==='off') {s.owner=previous?.owner||'external';continue;}
    const candidates=selected?[selected]:[address,...records.filter(r=>r.id===id && r.endpoint).map(r=>r.endpoint),...(definitions[id].ports||[]).map(port=>defaultEndpoint(id,o,port))];
    const inspected=await Promise.all([...new Set(candidates)].map(async addr=>({addr,...await probe(id,addr,records,o)})));
    const live=inspected.filter(v=>v.healthy), occupied=inspected.filter(v=>v.occupied && !v.healthy);
    const own=previous?.owner==='pack' && ['docker','native'].includes(previous.provider);
    const ownedLive=own && live.find(v=>v.addr===previous.endpoint);
    // Never adopt a new process at a previous endpoint solely on the strength of a stale receipt.
    let ownsNative=false;
    if(ownedLive && previous.provider==='native') {
      try {ownsNative=(await native.status?.(previous,plan,o))?.healthy===true;} catch { /* A changed registration belongs to the external owner. */ }
    }
    const ownsListener=ownedLive && (previous.provider==='docker'
      ? records.some(r=>r.id===id && r.endpoint===ownedLive.addr && r.token===state.ownerToken && r.token)
      : ownsNative);
    if(live.length>1 && !selected) conflict(plan,id,'multiple compatible services; select one with --endpoint '+id+'=URL');
    const picked=ownedLive || live[0];
    if(picked && (requested==='auto'||requested==='reuse')) {
      s.endpoint=picked.addr;s.provider=ownsListener?previous.provider:'reuse';s.owner=ownsListener?'pack':'external';s.origin=picked.origin;
    } else if(requested==='reuse') conflict(plan,id,'no compatible service found; '+(occupied[0]?.status||'check the endpoint'));
    else {
      s.provider=requested==='auto'?(own?previous.provider:'native'):requested;
      if(picked && !ownsListener) conflict(plan,id,'an external service is ready at '+picked.addr+'; choose reuse or select another port');
      // A managed replacement may use its current port, but cannot displace an unrelated listener.
      const collision=occupied.find(v=>v.addr===address) || (!selected && !picked && occupied[0]);
      if(collision) conflict(plan,id,collision.addr+' '+collision.status+'; select another port or off');
      if(s.provider==='native' && !await native.supports(id,o)) conflict(plan,id,'native registration unavailable; choose docker, reuse or off');
      if(definitions[id].ports && !localhost(new URL(s.endpoint).hostname)) conflict(plan,id,'native and Docker providers require a loopback endpoint');
    }
    if(definitions[id].ports) s.port=Number(new URL(s.endpoint).port)||(new URL(s.endpoint).protocol==='https:'?443:80);
    if(id==='surreal-memory') s.profile=profile(o);
  }
  plan.env=envFor(plan.services);
  const worker=plan.services.find(s=>s.id==='learning-worker' && ['native','docker'].includes(s.provider));
  if(worker && plan.services.some(s=>s.id==='surreal-memory' && s.provider==='docker') && !o.env.PROMETHEUS_WORKER_MEMORY_GATEWAY)
    conflict(plan,'learning-worker','Full Docker memory requires Gate JWTs. Configure PROMETHEUS_WORKER_MEMORY_GATEWAY to an authenticating local gateway or disable the worker');
  const occupiedPorts=new Map();
  for(const s of plan.services) {
    if(s.provider==='off' || !s.port || !localhost(new URL(s.endpoint).hostname)) continue;
    if(occupiedPorts.has(s.port)) conflict(plan,s.id,'port '+s.port+' also selected for '+occupiedPorts.get(s.port));
    occupiedPorts.set(s.port,s.id);
  }
  for(const s of plan.services) {
    if(o.only && !o.only.includes(s.id)) continue;
    if(s.id==='prometheus-exec' && (s.provider==='docker' || (s.provider==='native' && process.platform!=='darwin'))) {
      conflict(plan,s.id,'This release requires the macOS Seatbelt Tier P backend to pass /ready. Use native on macOS, reuse a ready local Unix socket, or select off.');
      continue;
    }
    if(s.provider==='native' && !s.origin) {
      try {const error=await native.validate?.(s,plan,{...o,env:{...o.env,...plan.env}});if(error) conflict(plan,s.id,error);}catch(e){conflict(plan,s.id,e.message);}
    }
    if(s.provider!=='docker') continue;
    if(o.engine.error) {conflict(plan,s.id,o.engine.error);continue;}
    try {
      if(s.id!=='surrealdb') {release??=inventory(o);release=await release;}
      s.image=imageFor(s.id,o,release,o.engine.platform);s.platform=o.engine.platform;
      const error=dockerValidate(s,plan,o);if(error) conflict(plan,s.id,error);
    }catch(e){conflict(plan,s.id,e.message);}
  }
  return plan;
}
export async function planServices(options={}, {native=nativeDefault}={}) {return planWith(context(options),native);}
async function locked(o,action) {
  fs.mkdirSync(o.dir,{mode:0o700,recursive:true});
  const lock=path.join(o.dir,'transaction.lock');
  let fd;
  try {fd=fs.openSync(lock,'wx',0o600);}catch(err){if(err.code==='EEXIST') throw Error('Service transaction locked at '+lock+'; check the owning PID before removing a stale lock');throw err;}
  try {fs.writeFileSync(fd,String(process.pid));return await action();}finally{fs.closeSync(fd);fs.rmSync(lock,{force:true});}
}
function save(o,state) {atomic(path.join(o.dir,o.pack+'.json'),JSON.stringify(state,null,2)+'\n');}
async function change(action,s,plan,o,state,native) {
  if(s.owner!=='pack') return;
  if(s.provider==='docker') {
    o.engine=dockerEngine({...o,engine:undefined});
    if(o.engine.error) throw Error(o.engine.error);
    return dockerAction(action,s,plan,o,state.ownerToken);
  }
  if(s.provider==='native') {
    const env={...o.env,...plan.env};
    if(s.id==='learning-worker' && o.env.PROMETHEUS_WORKER_MEMORY_GATEWAY) env.SURREAL_MEMORY_URL=o.env.PROMETHEUS_WORKER_MEMORY_GATEWAY;
    return native[action](s,plan,{...o,env});
  }
}
async function syncMcp(o) {
  if(o.pack==='full') await (await import('./mcp-config.mjs')).syncInstalledMcp(o);
}
export async function applyServices(options={}, {native=nativeDefault}={}) {
  const o=context(options);
  return locked(o,async()=>{
    const plan=await planWith(o,native);
    if(!plan.applicable) return plan;
    const state=readState(o);state.ownerToken ||= randomUUID();
    for(const s of plan.services) {
      if(o.only && !o.only.includes(s.id)) continue;
      const previous=state.services[s.id];
      if(previous?.owner==='pack' && previous.provider!=='off' && (s.provider!==previous.provider || s.endpoint!==previous.endpoint)) {
        if(s.provider==='reuse' && s.endpoint===previous.endpoint) {
          // The previous owner's process is gone. Retain the external registration untouched.
        } else await change('uninstall',previous,plan,o,state,native);
      }
      if(s.provider==='off' || s.owner==='external') {state.services[s.id]={...s,phase:s.provider==='off'?'off':'ready'};save(o,state);continue;}
      state.services[s.id]={...s,phase:'applying'};save(o,state);
      try {
        await change('apply',s,plan,o,state,native);
        const deadline=Date.now()+(s.id==='surreal-memory'?180000:30000);
        let health;
        do {
          if(s.id==='learning-worker' && s.provider==='native') health=await native.status(s,plan,o);
          else {
            const records=['learning-worker','surreal-memory'].includes(s.id)?evidence(o,o.engine):[];
            if(s.id==='surreal-memory' && s.provider==='native' && (await native.status?.(s,plan,o))?.healthy && !records.some(r=>r.id===s.id && r.profile && r.endpoint===s.endpoint)) {
              // Native adapters just wrote the active unit and verify its live registration.
              records.push({...s,origin:o.pack+' native registration'});
            }
            health=await probe(s.id,s.endpoint,records,o);
          }
          if(health.healthy) break;
          if(Date.now()<deadline) await delay(500);
        }while(Date.now()<deadline);
        if(health?.healthy && s.id==='surrealdb' && (!previous || previous.phase==='failed' || previous.provider==='off') && plan.services.some(v=>v.id==='surreal-memory' && ['docker','native'].includes(v.provider))) await initializeNamespace(s,o);
        if(!health?.healthy) throw Error(s.id+' failed its readiness check: '+health?.status);
        state.services[s.id]={...s,phase:'ready'};save(o,state);
      }catch(err){state.services[s.id].phase='failed';save(o,state);throw err;}
    }
    await syncMcp(o);
    return plan;
  });
}
export async function manageServices(action,options={}, {native=nativeDefault}={}) {
  if(!['status','stop','uninstall'].includes(action)) throw Error('Unsupported service action '+action);
  const o=context(options);
  const operation=async()=>{
    const state=readState(o),selected=options.only||options.enabled||Object.keys(state.services);
    const services=selected.map(id=>{if(!o.ids.includes(id)) throw Error('Unknown service '+id);return state.services[id];}).filter(Boolean);
    const plan={pack:o.pack,applicable:true,healthy:true,conflicts:[],services,env:envFor(Object.values(state.services))};
    if(action==='status') {
      const records=services.some(s=>s.provider!=='off')?evidence(o,engineForDiscovery(o,services)):[];
      plan.services=await Promise.all(services.map(async s=>{
        let health;
        if(s.provider==='off') health={healthy:true,status:'off'};
        else if(s.phase==='stopped') health={healthy:false,status:'stopped'};
        else if(s.id==='learning-worker' && s.owner==='pack' && s.provider==='native') health=await native.status(s,plan,o);
        else health=await probe(s.id,s.endpoint,records,o);
        return {...s,...health};
      }));
      plan.healthy=plan.services.every(s=>s.healthy);return plan;
    }
    for(const s of services.reverse()) {
      if(s.owner==='pack' && s.provider!=='off') await change(action,s,plan,o,state,native);
      if(action==='uninstall') delete state.services[s.id];else if(s.owner==='pack' && s.provider!=='off') state.services[s.id].phase='stopped';
      save(o,state);
    }
    if(action==='uninstall') await syncMcp(o);
    return plan;
  };
  return action==='status'?operation():locked(o,operation);
}
