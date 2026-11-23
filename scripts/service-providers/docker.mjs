import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {atomic,definitions,envFor,localhost,profile,run} from './catalog.mjs';

const configPath=o=>o.env.LITER_LLM_CONFIG || path.join(o.home,'.config/liter-llm/liter-llm-proxy.toml');
const authPath=o=>o.env.OPENAI_PROXY_AUTH_PATH || o.env.CODEX_AUTH_PATH || path.join(o.home,'.codex/auth.json');
const bind=(source,target,read_only=false)=>({type:'bind',source,target,read_only,bind:{create_host_path:false}});
export function dockerValidate(s,plan,o) {
  const e=o.env;
  if(s.id==='liter-llm' && !fs.existsSync(configPath(o))) return 'Set LITER_LLM_CONFIG to a readable gateway configuration';
  if(s.id==='surrealdb' && (!(e.SURREAL_ROOT_USERNAME||e.SURREAL_USERNAME) || !(e.SURREAL_ROOT_PASSWORD||e.SURREAL_PASSWORD))) return 'Set SURREAL_ROOT_USERNAME and SURREAL_ROOT_PASSWORD for the database';
  if(s.id==='surreal-memory') {
    if(o.pack==='full' && (!e.AUTH_JWT_ISSUER || !e.AUTH_JWT_AUDIENCE || !e.AUTH_JWT_ALGORITHM || !(e.AUTH_JWT_KEY || (e.AUTH_JWT_KEY_FILE && fs.existsSync(e.AUTH_JWT_KEY_FILE))))) return 'Set AUTH_JWT_ISSUER, AUTH_JWT_AUDIENCE, AUTH_JWT_ALGORITHM and AUTH_JWT_KEY or AUTH_JWT_KEY_FILE to enable Gate authentication';
    if(!['local','openai','cohere'].includes(profile(o).provider)) return 'Select a verifiable local, openai, or cohere embedding profile';
    if(!plan.env.SURREAL_ENDPOINT && !e.SURREAL_ENDPOINT) return 'Select surrealdb or set SURREAL_ENDPOINT';
    if(!(e.MEMORY_USERNAME||e.SURREAL_USERNAME) || !(e.MEMORY_PASSWORD||e.SURREAL_PASSWORD)) return 'Set MEMORY_USERNAME and MEMORY_PASSWORD for the selected SurrealDB user';
  }
  if(s.id==='openai-proxy' && !fs.existsSync(authPath(o)) && !e.OPENAI_API_KEY) return 'Set OPENAI_API_KEY or OPENAI_PROXY_AUTH_PATH to a Codex auth.json';
  if(s.id==='forge-mcp' && !e.FORGE_MCP_TOKEN) return 'Set FORGE_MCP_TOKEN for the MCP client and server';
  if(s.id==='prometheus-research') {
    const d=e.PROMETHEUS_RESEARCH_HARNESS_DIR;
    if(!d || !['claude','codex'].some(x=>fs.existsSync(path.join(d,x)))) return 'Set PROMETHEUS_RESEARCH_HARNESS_DIR to a directory containing an executable Linux claude or codex';
  }
  if(s.id==='learning-worker' && !e.PROMETHEUS_LEARNING_PROJECT_ROOT) return 'Set PROMETHEUS_LEARNING_PROJECT_ROOT to the absolute parent of projects referenced by queued learning jobs';
  if(s.id==='learning-worker' && (!path.isAbsolute(e.PROMETHEUS_LEARNING_PROJECT_ROOT) || !fs.statSync(e.PROMETHEUS_LEARNING_PROJECT_ROOT,{throwIfNoEntry:false})?.isDirectory())) return 'Worker project root must be an existing absolute directory';
}
const project=o=>'prometheus-'+o.pack+'-'+createHash('sha256').update(o.home).digest('hex').slice(0,12);
const filename=(id,o)=>path.join(o.dir,o.pack,'compose',id+'.json');
const runtimeDir=(o,id)=>path.join(o.dir,o.pack,'runtime',id);
function command(o,args,opts={}) {
  const v=run('docker',args,o,{timeout:300000,...opts});
  if(!v.ok) throw Error('Docker '+args[0]+' failed: '+(v.error || 'exit '+v.code).slice(0,600));
  return v.text;
}
function compose(o,s,args) {
  return command(o,['compose','--project-name',project(o),'--file',filename(s.id,o),...args]);
}
function checkOwner(s,o,token) {
  const p=project(o);
  const records=command(o,['ps','-a','--filter','label=com.docker.compose.project='+p,'--format','{{.ID}}']);
  if(records) {
    for(const label of command(o,['inspect','--format','{{index .Config.Labels "io.prometheus.owner"}}',...records.split(/\s+/)]).split('\n')) {
      if(label!==token) throw Error('Preserving unrelated container in Compose project '+p);
    }
  }
  for(const [kind,label] of [['network','Network'],['volume','Volume']]) {
    const owned=command(o,[kind,'ls','-q','--filter','label=com.docker.compose.project='+p]);
    // Compose warns about pre-existing conventional names and may attach to them.
    const named=command(o,[kind,'ls','--format','{{.Name}}']).split('\n').filter(name=>name.startsWith(p+'_'));
    const resources=[...new Set([...owned.split(/\s+/),...named].filter(Boolean))].join(' ');
    if(resources) for(const value of command(o,[kind,'inspect','--format','{{index .Labels "io.prometheus.owner"}}',...resources.split(/\s+/)]).split('\n')) {
      if(value!==token) throw Error('Preserving unrelated '+label+' in Compose project '+p);
    }
  }
}
function literal(value) {
  // Compose interpolates dollars even in JSON. Preserve exact credentials and paths.
  if(typeof value==='string') return value.replaceAll('$','$$$$');
  if(Array.isArray(value)) return value.map(literal);
  if(value && typeof value==='object') return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,literal(v)]));
  return value;
}
function manifest(s,plan,o,token) {
  const id=s.id, labels={'io.prometheus.owner':token,'io.prometheus.pack':o.pack,'io.prometheus.service':id};
  const manifest={services:{},networks:{default:{labels:{'io.prometheus.owner':token}}},volumes:{}};
  const base=manifest.services[id]={image:s.image,restart:'unless-stopped',labels,environment:{},volumes:[]};
  const env=base.environment, e=o.env;
  const pairs={};
  // Give Linux bind mounts the installing user's identity. Do not apply it to the upstream DB.
  if(id!=='surrealdb' && process.getuid) base.user=process.getuid()+':'+process.getgid();
  if(s.port) base.ports=[{target:definitions[id].internal,published:String(s.port),host_ip:'127.0.0.1',protocol:'tcp'}];
  function mkdir(dir) {fs.mkdirSync(dir,{mode:0o700,recursive:true});return dir;}
  function volume(name,target) {manifest.volumes[name]={labels:{'io.prometheus.owner':token}};base.volumes.push({type:'volume',source:name,target});}
  function mount(dir,target,ro=false) {base.volumes.push(bind(path.resolve(dir),target,ro));}
  if(id!=='surrealdb') mount(mkdir(path.join(runtimeDir(o,id),'home')),'/home/prometheus');
  function pass(keys) {for(const key of keys) if(e[key]) env[key]=e[key];}
  function upstream(dep,addr) {
    const service=plan.services.find(x=>x.id===dep && x.provider!=='off');
    if(service?.provider==='docker') return 'http://'+dep+':'+definitions[dep].internal;
    if(!addr) return;
    const url=new URL(addr);
    if(!localhost(url.hostname)) return url.origin;
    if(o.engine.desktop) {url.hostname='host.docker.internal';return url.origin;}
    // Host networking is limited to a UNIX-to-loopback relay. The worker's bridge stays isolated.
    const relay=id+'-to-'+dep, socket=relay+'-socket', port=Number(url.port)||80;
    manifest.volumes[socket]={labels:{'io.prometheus.owner':token}};
    const relayConfig={image:s.image,labels:{...labels,'io.prometheus.service':relay},restart:'unless-stopped',user:'0:0',entrypoint:['/usr/bin/socat'],volumes:[socket+':/relay']};
    manifest.services[relay+'-host']={...relayConfig,network_mode:'host',command:['UNIX-LISTEN:/relay/upstream.sock,mode=0666,fork,unlink-early','TCP:127.0.0.1:'+port]};
    manifest.services[relay]={...relayConfig,command:['TCP-LISTEN:'+port+',fork,reuseaddr','UNIX-CONNECT:/relay/upstream.sock'],depends_on:[relay+'-host']};
    (base.depends_on ||= []).push(relay);
    url.hostname=relay;return url.origin;
  }
  const supplied={'openai-proxy':e.OPENAI_PROXY_URL,'surrealdb':e.SURREAL_ENDPOINT,'surreal-memory':e.SURREAL_MEMORY_URL,'pk-cherry':e.PK_MCP_URL,'liter-llm':e.LITER_LLM_BASE_URL};
  for(const dep of definitions[id].deps||[]) {
    const addr=plan.services.find(x=>x.id===dep && x.provider!=='off')?.endpoint||supplied[dep];
    if(id==='learning-worker' && dep==='surreal-memory' && e.PROMETHEUS_WORKER_MEMORY_GATEWAY) pairs[dep]=upstream(null,e.PROMETHEUS_WORKER_MEMORY_GATEWAY);
    else if(addr) pairs[dep]=upstream(dep,addr);
  }
  Object.assign(env,envFor(Object.entries(pairs).map(([id,endpoint])=>({id,endpoint}))));
  if(id==='surrealdb') {
    env.SURREAL_USER=e.SURREAL_ROOT_USERNAME||e.SURREAL_USERNAME;env.SURREAL_PASS=e.SURREAL_ROOT_PASSWORD||e.SURREAL_PASSWORD;
    base.command=['start','--bind','0.0.0.0:8000','surrealkv:/data/database'];
    volume('surreal-data','/data');
  } else if(id==='surreal-memory') {
    const p=profile(o);
    Object.assign(env,{SURREAL_USERNAME:e.MEMORY_USERNAME||e.SURREAL_USERNAME,SURREAL_PASSWORD:e.MEMORY_PASSWORD||e.SURREAL_PASSWORD,
      SURREAL_AUTH_LEVEL:e.SURREAL_AUTH_LEVEL || (o.pack==='mini'?'namespace':'root'),SURREAL_NAMESPACE:p.namespace,SURREAL_DATABASE:p.database,
      EMBEDDING_PROVIDER:p.provider,LOCAL_EMBEDDING_MODEL:e.LOCAL_EMBEDDING_MODEL || 'BAAI/bge-small-en-v1.5',MODEL_CACHE_DIR:'/home/prometheus/.cache/huggingface',SURREAL_REMOTE_WIRE_ALLOW_EXTERNAL:'1'});
    if(o.pack==='full') {
      env.AUTH_MODE='jwt';pass(['AUTH_JWT_ISSUER','AUTH_JWT_AUDIENCE','AUTH_JWT_ALGORITHM','AUTH_JWT_KEY','AUTH_JWT_MAX_AGE_SECS']);
      if(!env.AUTH_JWT_KEY && e.AUTH_JWT_KEY_FILE) {mount(e.AUTH_JWT_KEY_FILE,'/etc/prometheus/gate.key',true);env.AUTH_JWT_KEY_FILE='/etc/prometheus/gate.key';}
    }
    pass(['LOCAL_EMBEDDING_MODEL_REVISION','LOCAL_EMBEDDING_DIMENSIONS','OPENAI_EMBEDDING_MODEL','COHERE_EMBEDDING_MODEL','COHERE_API_KEY','OPENAI_API_KEY','OPENAI_BASE_URL','SURREAL_MEMORY_API_KEY']);
    mount(mkdir(e.MODEL_CACHE_DIR||path.join(runtimeDir(o,id),'model-cache')),env.MODEL_CACHE_DIR);
  } else if(id==='liter-llm') {
    let config=fs.readFileSync(configPath(o),'utf8');
    if(/https?:\/\/(?:localhost|127\.0\.0\.1):8181\b/.test(config)) {
      const translated=pairs['openai-proxy'] || upstream('openai-proxy',e.OPENAI_PROXY_URL || 'http://127.0.0.1:8181');
      config=config.replace(/https?:\/\/(?:localhost|127\.0\.0\.1):8181\b/g,translated);
    }
    const generated=path.join(runtimeDir(o,id),'liter.toml');
    atomic(generated,config);mount(generated,'/etc/prometheus/liter.toml',true);
    for(const match of config.matchAll(/\$\{([A-Za-z_][A-Za-z0-9_]*)\}/g)) pass([match[1]]);
    pass(['LITER_LLM_MASTER_KEY']);
  } else if(id==='openai-proxy') {
    if(fs.existsSync(authPath(o))) mount(authPath(o),'/home/prometheus/.codex/auth.json',true);
    pass(['OPENAI_API_KEY','CODEX_DEFAULT_MODEL']);
    pass(['CODEX_BACKEND_URL','CODEX_WIRE_API']);
  } else if(id==='pk-cherry') {
    mount(mkdir(e.PK_KB_DIR || path.join(o.home,'.prometheus/knowledge')),'/home/prometheus/.prometheus/knowledge');
  } else if(id==='forge-mcp') {
    mount(o.root,'/pack',true);mount(e.PROMETHEUS_PROJECT_ROOT || o.root,'/work',true);pass(['FORGE_MCP_TOKEN']);
  } else if(id==='prometheus-research') {
    mount(o.root,'/pack',true);mount(e.PROMETHEUS_RESEARCH_HARNESS_DIR,'/opt/harness',true);
    mount(mkdir(e.PROMETHEUS_RESEARCH_WORK_DIR||path.join(runtimeDir(o,id),'work')),'/work');
    env.PATH='/opt/harness:/usr/local/bin:/usr/bin:/bin';env.CLAUDE_PLUGIN_ROOT='/pack';
    if(e.PROMETHEUS_RESEARCH_AUTH_DIR) mount(e.PROMETHEUS_RESEARCH_AUTH_DIR,'/home/prometheus/.codex',true);
    pass(['ANTHROPIC_API_KEY','OPENAI_API_KEY','RESEARCH_HARNESS_ARGS']);
  } else if(id==='prometheus-exec') {
    const socket=decodeURIComponent(new URL(s.endpoint).pathname),target='/run/prometheus/'+path.basename(socket);
    mount(mkdir(path.dirname(socket)),'/run/prometheus');mount(mkdir(path.join(o.home,'.prometheus/exec')),'/home/prometheus/.prometheus/exec');mount(o.root,'/pack',true);
    base.command=['daemon','--socket',target,'--state-dir','/home/prometheus/.prometheus/exec','--identity','/home/prometheus/.prometheus/exec/identity.json','--plugin-root','/pack'];
  } else if(id==='learning-worker') {
    mount(mkdir(s.endpoint),'/home/prometheus/.prometheus/learning-queue');
    for(const name of ['learning-index','learning-log','knowledge','promotion-candidates','skill-candidates','skill-updates']) mount(mkdir(path.join(o.home,'.prometheus',name)),'/home/prometheus/.prometheus/'+name);
    mount(e.PROMETHEUS_LEARNING_PROJECT_ROOT,e.PROMETHEUS_LEARNING_PROJECT_ROOT);
    base.entrypoint=['/usr/bin/tini','--','/bin/sh','-ec'];
    base.command=['while :; do /usr/local/bin/prometheus-learning-worker run-once || echo worker-exited-with-error >&2; sleep 300 & wait $!; done'];
    base.healthcheck={test:['CMD','/usr/local/bin/prometheus-learning-worker','status','--json'],interval:'30s',timeout:'10s',retries:3};
  }
  return literal(manifest);
}
export function dockerAction(action,s,plan,o,token) {
  checkOwner(s,o,token);
  if(action==='apply') {
    const file=filename(s.id,o), current=manifest(s,plan,o,token);
    if(fs.existsSync(file)) {
      const old=JSON.parse(fs.readFileSync(file,'utf8'));
      const retired=Object.keys(old.services).filter(name=>!Object.hasOwn(current.services,name));
      if(retired.length) compose(o,s,['rm','--stop','--force',...retired]);
    }
    atomic(file,JSON.stringify(current,null,2)+'\n');
    if(s.id==='prometheus-exec' && !fs.existsSync(path.join(o.home,'.prometheus/exec/identity.json'))) compose(o,s,['run','--rm','--no-deps',s.id,'init','--identity','/home/prometheus/.prometheus/exec/identity.json']);
    compose(o,s,['up','--detach','--no-build',s.id]);return;
  }
  if(!fs.existsSync(filename(s.id,o))) throw Error('Owned Compose manifest is missing for '+s.id);
  // Manifests are per service; retain the shared project network and persistent volumes.
  const names=Object.keys(JSON.parse(fs.readFileSync(filename(s.id,o),'utf8')).services);
  if(action==='stop') compose(o,s,['stop',...names]);
  if(action==='uninstall') {compose(o,s,['rm','--stop','--force',...names]);fs.rmSync(filename(s.id,o));}
}

// Only a database just provisioned by this transaction may be initialized.
// A named external instance never receives admin credentials or schema mutations.
export async function initializeNamespace(service,o) {
  const p=profile(o),e=o.env,level=e.SURREAL_AUTH_LEVEL || (o.pack==='mini'?'namespace':'root');
  if(level!=='namespace') return;
  const user=e.MEMORY_USERNAME || e.SURREAL_USERNAME, pass=e.MEMORY_PASSWORD || e.SURREAL_PASSWORD;
  const root=e.SURREAL_ROOT_USERNAME || e.SURREAL_USERNAME, password=e.SURREAL_ROOT_PASSWORD || e.SURREAL_PASSWORD;
  if(!root || !password || !user || !pass) throw Error('Set both database administrator and memory namespace credentials');
  if(![user,p.namespace,p.database].every(x=>/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(x))) throw Error('Automatic namespace provisioning requires SQL identifiers containing only letters, digits and underscore');
  const sql='DEFINE NAMESPACE IF NOT EXISTS '+p.namespace+'; USE NS '+p.namespace+'; DEFINE DATABASE IF NOT EXISTS '+p.database+'; DEFINE USER IF NOT EXISTS '+user+' ON NAMESPACE PASSWORD '+JSON.stringify(pass)+' ROLES EDITOR;';
  const response=await fetch(service.endpoint+'/sql',{method:'POST',redirect:'error',signal:AbortSignal.timeout(8000),headers:{Accept:'application/json','Content-Type':'text/plain','Authorization':'Basic '+Buffer.from(root+':'+password).toString('base64')},body:sql});
  const results=response.ok ? await response.json():null;
  if(!Array.isArray(results)||results.length!==4||results.some(r=>r.status!=='OK')) throw Error('Namespace provisioning failed for the new pack-owned database; verify the administrator and namespace credentials');
}
