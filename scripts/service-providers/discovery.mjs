import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import net from 'node:net';
import http from 'node:http';
import https from 'node:https';
import {defaultEndpoint,definitions,embedding,endpoint,localhost,profile,readState,run} from './catalog.mjs';

const memoryKeys=['SURREAL_NAMESPACE','SURREAL_DATABASE','EMBEDDING_PROVIDER','LOCAL_EMBEDDING_MODEL','LOCAL_EMBEDDING_DIMENSIONS','OPENAI_EMBEDDING_MODEL','COHERE_EMBEDDING_MODEL'];
function memoryProfile(env) {
  if(!env.SURREAL_NAMESPACE || !env.SURREAL_DATABASE || !env.EMBEDDING_PROVIDER) return undefined;
  return {namespace:env.SURREAL_NAMESPACE,database:env.SURREAL_DATABASE,...embedding(env)};
}
function sameProfile(a,b) { return a && Object.keys(b).every(k=>String(a[k]??'')===b[k]); }
function text(file) { try {return fs.readFileSync(file,'utf8');} catch {return '';} }
export function dockerEngine(o) {
  if(o.engine) return o.engine;
  const host=o.env.DOCKER_HOST || run('docker',['context','inspect','--format','{{.Endpoints.docker.Host}}',...(o.env.DOCKER_CONTEXT?[o.env.DOCKER_CONTEXT]:[])],o).text;
  if(!/^(unix:|npipe:|tcp:\/\/(localhost|127\.0\.0\.1|\[::1\]):)/.test(host)) return {error:'Select a running local Docker context (Unix socket, named pipe, or loopback)'};
  // DOCKER_CONTEXT overrides DOCKER_HOST in Docker itself.
  if(o.env.DOCKER_HOST && o.env.DOCKER_CONTEXT) return {error:'Set either DOCKER_CONTEXT or DOCKER_HOST so the local engine is unambiguous'};
  const info=run('docker',['info','--format','{{.OSType}}|{{.Architecture}}|{{.OperatingSystem}}'],o);
  if(!info.ok || !info.text.startsWith('linux|')) return {error:'Start local Docker with Linux containers'};
  if(!run('docker',['compose','version','--short'],o).ok) return {error:'Install the Docker Compose v2 plugin'};
  const [,arch,os]=info.text.split('|');
  const platform=['x86_64','amd64'].includes(arch)?'linux/amd64':['aarch64','arm64'].includes(arch)?'linux/arm64':'';
  return platform?{platform,desktop:process.platform!=='linux' || /desktop/i.test(os),host}:{error:'Unsupported Docker Engine architecture '+arch};
}
function inspectContainers(o,engine) {
  if(engine.error) return [];
  const ps=run('docker',['ps','--format','{{.ID}} {{.Label "com.docker.compose.service"}} {{.Label "io.prometheus.service"}} {{.Label "org.prometheus.service"}}'],o);
  if(!ps.ok || !ps.text) return [];
  const ids=ps.text.split('\n').map(line=>line.split(/\s+/)).filter(([, ...names])=>names.some(n=>[...o.ids,'liter','memory','surreal','surreal-memory-server','prometheus-learning-worker'].includes(n))).map(([id])=>id);
  if(!ids.length) return [];
  const inspected=run('docker',['inspect',...ids],o);
  if(!inspected.ok) return [];
  return JSON.parse(inspected.text).flatMap(c=>{
    const labels=c.Config.Labels||{}, service=labels['io.prometheus.service'] || labels['com.docker.compose.service'] || labels['org.prometheus.service'];
    const id=({'memory':'surreal-memory','surreal-memory-server':'surreal-memory',liter:'liter-llm',surreal:'surrealdb','prometheus-learning-worker':'learning-worker'})[service]||service;
    if(!o.ids.includes(id)) return [];
    const entry={id,origin:labels['com.docker.compose.project']||'Docker',token:labels['io.prometheus.owner']};
    const vars=Object.fromEntries((c.Config.Env||[]).filter(s=>Object.values(memoryKeys).some(k=>s.startsWith(k+'='))).map(s=>[s.slice(0,s.indexOf('=')),s.slice(s.indexOf('=')+1)]));
    if(id==='surreal-memory') entry.profile=memoryProfile(vars);
    if(id==='prometheus-exec') {
      const mount=c.Mounts.find(m=>m.Destination==='/run/prometheus');
      const args=c.Config.Cmd || [], pos=args.indexOf('--socket');
      return mount && pos>=0 ? [{...entry,endpoint:'unix://'+path.join(mount.Source,path.basename(args[pos+1]))}]:[];
    }
    if(id==='learning-worker') {
      const mount=c.Mounts.find(m=>m.Destination==='/home/prometheus/.prometheus/learning-queue');
      return mount?[{...entry,endpoint:mount.Source,scheduled:/run-once/.test(JSON.stringify(c.Config.Cmd)) && (c.State.Health?.Status==='healthy' || (!c.State.Health && c.State.Running)),container:true}]:[];
    }
    const ports=c.NetworkSettings.Ports||{};
    return (ports[definitions[id].internal+'/tcp']||[]).filter(p=>['0.0.0.0','127.0.0.1','::','::1'].includes(p.HostIp)).map(p=>({...entry,endpoint:defaultEndpoint(id,o,p.HostPort)}));
  });
}
function bossCandidates(o) {
  const dir=process.platform==='darwin'?path.join(o.home,'Library/Application Support'):process.platform==='win32'?(o.env.APPDATA||o.home):(o.env.XDG_CONFIG_HOME||path.join(o.home,'.config'));
  const candidates=[];
  for(const app of ['TheBoss','The Boss','CherryStudio','Cherry Studio']) {
    const data=path.join(dir,app,'Data'), db=path.join(data,'cherrystudio.sqlite');
    const services=path.join(data,'Prometheus','services');
    if(fs.existsSync(path.join(services,'compose.yaml'))) {
      const vars=text(path.join(services,'.env'));
      for(const [id,key] of [['surrealdb','SURREAL_PORT'],['surreal-memory','MEMORY_PORT'],['liter-llm','LITER_PORT']]) {
        const port=vars.match(new RegExp('^'+key+'=[\"\']?(\\d+)', 'm'))?.[1];
        if(port && Number(port)<65536 && Number(port)>0) candidates.push({id,endpoint:defaultEndpoint(id,o,port),origin:'The Boss managed Compose'});
      }
    }
    if(!fs.existsSync(db)) continue;
    const query=run('sqlite3',['-readonly',db,"SELECT value FROM preference WHERE scope='default' AND key='app.prometheus.integrations' LIMIT 1"],o);
    if(!query.ok || !query.text) continue;
    try {
      let pref=JSON.parse(query.text);
      if(typeof pref==='string') pref=JSON.parse(pref);
      const s=(pref.config||pref).services || {};
      for(const [id,key,port] of [['surrealdb','surrealdb','surrealPort'],['surreal-memory','memory','memoryPort'],['liter-llm','liter','literPort']]) {
        const addr=s[key]?.endpoint || (s[port] && defaultEndpoint(id,o,s[port]));
        if(addr && localhost(new URL(addr).hostname)) candidates.push({id,endpoint:endpoint(id,addr),origin:'The Boss ('+app+')'});
      }
    } catch { /* Ignore unsupported preference versions. Live probes remain authoritative. */ }
  }
  return candidates;
}
function listenerOwned(o,pid,port) {
  if(!/^[1-9]\d*$/.test(String(pid)) || !/^\d+$/.test(String(port))) return false;
  if(process.platform==='linux') {
    const tables=['tcp','tcp6'].map(t=>text('/proc/net/'+t)).join('\n');
    const inodes=new Set(tables.split('\n').map(line=>line.trim().split(/\s+/)).filter(row=>row[3]==='0A' && parseInt(row[1]?.split(':')[1],16)===Number(port)).map(row=>row[9]));
    try{return fs.readdirSync('/proc/'+pid+'/fd').some(fd=>{try{return inodes.has(fs.readlinkSync('/proc/'+pid+'/fd/'+fd).match(/^socket:\[(\d+)\]$/)?.[1]);}catch{return false;}});}catch{return false;}
  }
  return run('lsof',['-nP','-a','-p',String(pid),'-iTCP:'+port,'-sTCP:LISTEN','-Fp'],o).ok;
}
function familyOwnsListener(o,pid,port) {
  if(listenerOwned(o,pid,port)) return true;
  if(!/^[1-9]\d*$/.test(String(pid))) return false;
  const children=run('pgrep',['-P',String(pid)],o);
  return children.ok && children.text.split(/\s+/).some(child=>listenerOwned(o,child,port));
}
function registeredProcess(o,label,unit) {
  if(process.platform==='darwin') {
    const loaded=run('launchctl',['print','gui/'+process.getuid()+'/'+label],o);
    if(loaded.ok && loaded.text.split('\n').some(line=>line.trim()==='path = '+unit)) return loaded.text.match(/^\s*pid = (\d+)/m)?.[1];
  } else if(process.platform==='linux') {
    const svc=label+'.service';
    const loaded=run('systemctl',['--user','show','--property=FragmentPath','--value',svc],o);
    if(loaded.ok && loaded.text===unit) return run('systemctl',['--user','show','--property=MainPID','--value',svc],o).text;
  }
}
function nativeCandidates(o) {
  const results=[],dir=path.join(o.home,'Library/LaunchAgents');
  const plist=path.join(dir,'ai.prometheus.surreal-memory-native.plist');
  if(fs.existsSync(plist)) {
    const parsed=run('plutil',['-convert','json','-o','-',plist],o);
    if(parsed.ok) {
      const env=JSON.parse(parsed.text).EnvironmentVariables||{};
      if(familyOwnsListener(o,registeredProcess(o,'ai.prometheus.surreal-memory-native',plist),env.API_PORT)) results.push({id:'surreal-memory',endpoint:defaultEndpoint('surreal-memory',o,env.API_PORT),profile:memoryProfile(env),origin:'launchd memory registration'});
    }
  }
  const systemd=path.join(o.env.XDG_CONFIG_HOME||path.join(o.home,'.config'),'systemd/user');
  for(const file of ['ai.prometheus.surreal-memory-native.service','surreal-memory-native.service','surreal-memory.service']) {
    const unit=text(path.join(systemd,file)), env={};
    for(const [,key,value] of unit.matchAll(/^Environment="?([A-Z_]+)=([^"\r\n]*)"?$/gm)) env[key]=value;
    if(familyOwnsListener(o,registeredProcess(o,file.slice(0,-8),path.join(systemd,file)),env.API_PORT)) results.push({id:'surreal-memory',endpoint:defaultEndpoint('surreal-memory',o,env.API_PORT),profile:memoryProfile(env),origin:'systemd memory registration'});
  }
  const worker='learning-worker',w=defaultEndpoint(worker,o);
  const name='ai.prometheus.learning-worker';
  const unit=text(path.join(dir,name+'.plist'));
  if(unit.includes('prometheus-learning-worker') && unit.includes('run-once') && /StartInterval|StartCalendarInterval/.test(unit)) {
    const loaded=run('launchctl',['print','gui/'+process.getuid?.()+'/'+name],o);
    if(loaded.ok && loaded.text.includes(path.join(dir,name+'.plist'))) results.push({id:worker,endpoint:w,scheduled:true,origin:'launchd learning-worker'});
  }
  for(const label of [name,'prometheus-learning-worker']) {
    const service=text(path.join(systemd,label+'.service'));
    if(service.includes('prometheus-learning-worker') && service.includes('run-once') && fs.existsSync(path.join(systemd,label+'.timer')) && run('systemctl',['--user','is-active','--quiet',label+'.timer'],o).ok) results.push({id:worker,endpoint:w,scheduled:true,origin:'systemd learning-worker'});
  }
  return results;
}
export function engineForDiscovery(o,services) {
  const managed=services.some(s=>s.provider==='docker' || s.requested==='docker' || s.previous?.provider==='docker');
  const discovering=services.some(s=>['auto','reuse'].includes(s.requested) || s.provider==='reuse');
  if(managed || o.env.PROMETHEUS_DISCOVER_DOCKER==='1' || (discovering && o.env.PROMETHEUS_DISCOVER_DOCKER!=='0')) return dockerEngine(o);
  return {error:'Docker inspection was not selected'};
}
export function evidence(o,engine) {
  const records=[];
  for(const pack of [o.pack,o.pack==='full'?'mini':'full']) {
    for(const r of Object.values(readState(o,pack).services)) {
      if(r.provider==='off' || r.phase==='stopped') continue;
      records.push({id:r.id,endpoint:r.endpoint,receiptPack:pack,origin:pack+' pack receipt'});
      // A stored selection alone cannot prove the live memory server's profile.
      if(r.id==='surreal-memory' && r.provider==='native' && pack==='mini') {
        const file=path.join(o.dir,'mini-native',r.id,'launch.json');
        try {const launch=JSON.parse(text(file)),env=launch.environment||{};
          const label='ai.prometheus.mini.'+r.id+'.'+createHash('sha256').update(o.home).digest('hex').slice(0,12);
          const unit=process.platform==='darwin'?path.join(o.home,'Library/LaunchAgents',label+'.plist'):path.join(o.home,'.config/systemd/user',label+'.service');
          if(defaultEndpoint(r.id,o,env.API_PORT)===r.endpoint && familyOwnsListener(o,registeredProcess(o,label,unit),env.API_PORT)) records.push({id:r.id,endpoint:r.endpoint,profile:memoryProfile(env),origin:'Mini memory registration'});
        }catch{}
      }
    }
  }
  return [...records,...bossCandidates(o),...nativeCandidates(o),...inspectContainers(o,engine)];
}
function transport(addr) {
  const url=new URL(addr);
  return url.protocol==='unix:'?{socketPath:decodeURIComponent(url.pathname)}:{hostname:url.hostname.replace(/^\[|\]$/g,''),port:Number(url.port)||(url.protocol==='https:'?443:80)};
}
function open(addr) {
  return new Promise(resolve=>{
    const t=transport(addr), sock=net.createConnection(t.socketPath?{path:t.socketPath}: {host:t.hostname,port:t.port});
    const finish=v=>{sock.destroy();resolve(v);};
    sock.once('connect',()=>finish(true));sock.once('error',e=>finish(!['ECONNREFUSED','ENOENT'].includes(e.code)));
    sock.setTimeout(650,()=>finish(true));
  });
}
async function request(addr,route,body) {
  return new Promise(resolve=>{
    const target=new URL(addr);
    const client=target.protocol==='https:'?https:http;
    const bytes=body ? JSON.stringify(body):undefined;
    const req=client.request({...transport(addr),path:route,method:body?'POST':'GET',headers:{Accept:'application/json, text/event-stream',...(body?{'Content-Type':'application/json'}:{})}},res=>{
      let size=0, chunks=[];
      res.on('error',()=>resolve({}));res.on('data',chunk=>{size+=chunk.length;if(size>1048576){req.destroy();resolve({});}else chunks.push(chunk);});
      res.on('end',()=>{const raw=Buffer.concat(chunks).toString();let data;try{data=JSON.parse(raw.startsWith('event:')?raw.split('\n').find(s=>s.startsWith('data:')).slice(5):raw);}catch{}
        resolve({ok:res.statusCode>=200 && res.statusCode<300,raw,data});});
    });
    const timer=setTimeout(()=>{req.destroy();resolve({});},1600);req.on('close',()=>clearTimeout(timer));
    req.on('error',()=>resolve({}));req.end(bytes);
  });
}
export async function probe(id,addr,records,o) {
  const matching=records.filter(r=>r.id===id && r.endpoint===addr);
  const result=(healthy,status,occupied=true)=>({healthy:Boolean(healthy),occupied,status,origin:matching.map(x=>x.origin).filter(Boolean).join(', ') || 'endpoint'});
  if(id==='learning-worker') {
    const running=matching.some(r=>r.scheduled===true);
    return result(running,running?'scheduled':'no verified schedule',running);
  }
  if(!await open(addr)) {
    const staleSocket=id==='prometheus-exec' && fs.existsSync(decodeURIComponent(new URL(addr).pathname));
    return result(false,staleSocket?'occupied socket path without a responding peer':'unused',staleSocket);
  }
  const get=async(route,body)=>(await request(addr,route,body)).data||{};
  let ok=false;
  switch(id) {
    case 'surrealdb': {
      const v=(await request(addr,'/version')).raw||'';
      ok=new RegExp('^surrealdb-'+(o.pack==='mini'?'3\\.3\\.0':'4\\.0\\.0')+'(?:$|[+\\s-])').test(v.trim()) && (await request(addr,'/health')).ok;
      break;
    }
    case 'liter-llm': {
      const schema=await get('/openapi.json'),ready=await get('/readyz');
      ok=/liter-llm/i.test(schema.info?.title||'') && ready.status==='ready';
      if(ok) {
        const h=await get('/health'),required=(o.env.PROMETHEUS_LITER_REQUIRED_MODELS || 'kbd-judge,kbd-critic').split(',').filter(Boolean);
        if(!Array.isArray(h.models) || required.some(model=>!h.models.includes(model))) return result(false,'gateway is missing required models '+required.join(', '));
      }
      break;
    }
    case 'openai-proxy': ok=(await get('/v1/models')).data?.some(m=>m.owned_by==='openai-proxy') && (await get('/health')).status==='ok';break;
    case 'surreal-memory': {
      const health=await get('/health'),ready=await get('/ready');
      if(health.service!=='surreal-memory-server' || health.status!=='ok' || ready.capabilities?.ledger!==true || ready.search_ready!==true) break;
      const p=profile(o);
      if(!sameProfile(ready.profile,p) && !matching.some(r=>sameProfile(r.profile,p))) return result(false,'memory namespace, database, embedding model or dimensions differ or cannot be verified');
      ok=true;break;
    }
    case 'pk-cherry': ok=(await get('/ready')).status==='ready' && (await get('/mcp',{jsonrpc:'2.0',id:1,method:'initialize',params:{protocolVersion:'2024-11-05',capabilities:{},clientInfo:{name:'prometheus-installer',version:'1'}}})).result?.serverInfo?.name==='prometheus-knowledge';break;
    case 'forge-mcp': case 'prometheus-research': case 'prometheus-exec': {
      const h=await get('/health'); ok=h.status==='ok' && h.service===id;
      if(id==='prometheus-exec') ok=ok && (await get('/ready')).ready===true;
      if(id==='prometheus-research') ok=ok && h.execution?.harness?.status==='ok' && h.execution?.driver?.status==='ok';
      break;
    }
    case 'surface-bridge': {const h=await get('/health'),s=await get('/mcp/detect-surface-tier',{});ok=h.status==='ok' && Boolean(h.version) && typeof s.harness==='string' && typeof s.tier==='string';break;}
  }
  return result(ok,ok?'ready':'occupied; identity, version or readiness could not be verified');
}
