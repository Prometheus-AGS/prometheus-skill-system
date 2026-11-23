import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';

const moduleRoot = fileURLToPath(new URL('../../', import.meta.url));
export const definitions = {
  surrealdb: {ports:[28000,8000], internal:8000, binary:'surreal'},
  'liter-llm': {ports:[4000], internal:4000, binary:'liter-llm', imported:'tools/liter-llm',deps:['openai-proxy']},
  'openai-proxy': {ports:[8181], internal:8181, binary:'openai-proxy', imported:'tools/openai-proxy'},
  'surreal-memory': {ports:[23001,3001], internal:3001, binary:'surreal-memory-server', imported:'tools/surreal-memory-server', deps:['surrealdb']},
  'pk-cherry': {ports:[8942], internal:8942, binary:'pk-cherry', imported:'tools/prometheus-knowledge'},
  'forge-mcp': {ports:[8943], internal:8943, binary:'forge', deps:['pk-cherry']},
  'surface-bridge': {ports:[7890], internal:17890, binary:'surface-bridge'},
  'prometheus-research': {ports:[7891], internal:17891, binary:'prometheus-research', deps:['liter-llm']},
  'prometheus-exec': {binary:'prometheus-exec'},
  'learning-worker': {binary:'prometheus-learning-worker', imported:'tools/prometheus-knowledge', deps:['surreal-memory']},
};

export function context(options={}) {
  const root=path.resolve(options.root || moduleRoot), home=path.resolve(options.home || os.homedir());
  const manifest=readJson(path.join(root,'skill-system.json'));
  let pack, repository, ids;
  if(manifest.name==='prometheus-skill-pack' && manifest.schemaVersion==='prometheus-skill-system-v1') {
    pack='full';
    repository='Prometheus-AGS/prometheus-skill-system';
    ids=Object.keys(definitions);
  } else if(manifest.name==='prometheus-skills-mini' && manifest.schemaVersion==='prometheus-mini-skill-system-v1') {
    pack='mini';
    repository='Prometheus-AGS/prometheus-skills-mini';
    ids=['surrealdb','liter-llm','surreal-memory'];
  } else {
    throw Error('Unrecognized skill pack manifest name or schema at '+root);
  }
  if(typeof manifest.releaseVersion!=='string' || !/^\d+\.\d+\.\d+$/.test(manifest.releaseVersion)) {
    throw Error('Invalid releaseVersion in '+path.join(root,'skill-system.json'));
  }
  const env={...process.env,...options.env,HOME:home};
  if(home!==os.homedir() && !env.DOCKER_CONFIG) env.DOCKER_CONFIG=path.join(os.homedir(),'.docker');
  return {...options,root,home,pack,env,repository,ids,
    dir:path.join(home,'.prometheus','service-providers'),version:manifest.releaseVersion};
}
export function readJson(file) { return JSON.parse(fs.readFileSync(file,'utf8')); }
export function readState(o, pack=o.pack) {
  const file=path.join(o.dir,pack+'.json');
  if (!fs.existsSync(file)) return {schemaVersion:1,pack,services:{}};
  const state=readJson(file);
  if(state.schemaVersion!==1 || state.pack!==pack || !state.services || typeof state.services!=='object') throw Error('Invalid service receipt: '+file);
  return state;
}
export function atomic(file,data) {
  fs.mkdirSync(path.dirname(file),{recursive:true,mode:0o700});
  const temp=file+'.'+process.pid+'.tmp';
  fs.writeFileSync(temp,data,{mode:0o600,flag:'wx'});
  try { fs.renameSync(temp,file); } finally { fs.rmSync(temp,{force:true}); }
}
export function run(cmd,args,o,opts={}) {
  const result=spawnSync(cmd,args,{cwd:o.root, env:o.env, encoding:'utf8',timeout:8000,maxBuffer:2**22,windowsHide:true,...opts});
  return {ok:result.status===0, text:(result.stdout||'').trim(),error:result.error?.message || (result.stderr||'').trim(),code:result.status};
}
export const prefix=id=>'PROMETHEUS_SERVICE_'+id.toUpperCase().replaceAll('-','_');
export const localhost=host=>['localhost','127.0.0.1','[::1]','::1'].includes(host);
export function endpoint(id,value) {
  if(id==='learning-worker') return value;
  const url=new URL(value);
  if(id==='prometheus-exec' && url.protocol==='unix:' && !url.host && path.isAbsolute(fileURLToPath('file://'+url.pathname))) return 'unix://'+url.pathname;
  if(!['http:','https:','ws:','wss:'].includes(url.protocol)||url.username||url.password||url.search||url.hash||id==='prometheus-exec') throw Error('Invalid endpoint for '+id);
  return url.origin.replace(/^ws/,'http');
}
export function defaultEndpoint(id,o,port) {
  if(id==='prometheus-exec') return 'unix://'+(o.env.PROMETHEUS_EXEC_SOCKET || path.join(o.home,'.prometheus/run/prometheus-exec.sock'));
  if(id==='learning-worker') return path.resolve(o.env.PROMETHEUS_LEARNING_QUEUE || path.join(o.home,'.prometheus/learning-queue'));
  return 'http://127.0.0.1:'+(port||definitions[id].ports[0]);
}
export function embedding(env) {
  const provider=env.EMBEDDING_PROVIDER || 'local';
  if(provider==='openai') {
    const model=env.OPENAI_EMBEDDING_MODEL || 'text-embedding-3-small';
    return {provider,model,dimensions:model==='text-embedding-3-large'?'3072':'1536'};
  }
  if(provider==='cohere') {
    const model=env.COHERE_EMBEDDING_MODEL || 'embed-english-v3.0';
    return {provider,model,dimensions:['embed-english-light-v3.0','embed-multilingual-light-v3.0'].includes(model)?'384':'1024'};
  }
  return {provider,model:env.LOCAL_EMBEDDING_MODEL||'BAAI/bge-small-en-v1.5',dimensions:String(env.LOCAL_EMBEDDING_DIMENSIONS||384)};
}
export function profile(o) {
  return {namespace:o.env.SURREAL_NAMESPACE||'memory',database:o.env.SURREAL_DATABASE||(o.pack==='mini'?'main_local_384':'mcp'),...embedding(o.env)};
}
export function envFor(services) {
  const env={};
  for(const s of services.filter(s=>s.provider!=='off' && s.endpoint)) {
    env[prefix(s.id)+'_ENDPOINT']=s.endpoint;
    switch(s.id) {
      case 'surrealdb': env.SURREAL_ENDPOINT=s.endpoint.replace(/^http/,'ws'); break;
      case 'surreal-memory': env.SURREAL_MEMORY_URL=s.endpoint; env.SURREAL_MEMORY_MCP_URL=s.endpoint+'/mcp/sse'; break;
      case 'liter-llm': env.LITER_LLM_BASE_URL=s.endpoint+'/v1'; env.LITER_LLM_URL=s.endpoint; break;
      case 'openai-proxy': env.OPENAI_PROXY_URL=s.endpoint; break;
      case 'pk-cherry': env.PK_MCP_URL=s.endpoint+'/mcp'; break;
      case 'forge-mcp': env.FORGE_MCP_URL=s.endpoint+'/mcp'; break;
      case 'surface-bridge': env.SURFACE_BRIDGE_URL=s.endpoint; break;
      case 'prometheus-research': env.PROMETHEUS_RESEARCH_URL=s.endpoint; break;
      case 'prometheus-exec': env.PROMETHEUS_EXEC_SOCKET=decodeURIComponent(new URL(s.endpoint).pathname); break;
      case 'learning-worker': env.PROMETHEUS_LEARNING_QUEUE=s.endpoint; break;
    }
  }
  return env;
}
