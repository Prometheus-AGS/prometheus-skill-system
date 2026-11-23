import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath} from 'node:url';
import {atomic} from './catalog.mjs';

const root = fileURLToPath(new URL('../../',import.meta.url));
const ids = {'surreal-memory':'surreal-memory', 'prometheus-knowledge':'pk-cherry', 'forge-rs':'forge-mcp'};
const read = file => JSON.parse(fs.readFileSync(file,'utf8'));

function scoped(home) {
  const dir = path.join(home,'.prometheus/service-providers');
  const receipt = path.join(dir,'full.json');
  const services = fs.existsSync(receipt) ? read(receipt).services : null;
  const table = read(path.join(root,'scripts/mcp-port-table.json')).services;
  const previousFile = path.join(dir,'full-mcp-endpoints.json');
  return {home,services,table,previousFile,previous:fs.existsSync(previousFile) ? read(previousFile) : {}};
}

const tokenNames = {'forge-rs':'FORGE_MCP_TOKEN','surreal-memory':'PROMETHEUS_MEMORY_BEARER_TOKEN'};
function dockerToken(ctx,name) {
  return ctx.services?.[ids[name]]?.provider === 'docker' ? tokenNames[name] : null;
}

function urlFor(ctx,name) {
  const svc = ctx.services?.[ids[name]];
  if (!svc) return ctx.services && Object.values(ctx.previous).some(row => Object.hasOwn(row,name)) ? '' : ctx.table[name].url;
  return svc.provider === 'off' || !svc.endpoint ? '' : svc.endpoint + ctx.table[name].path;
}

function tableFor(ctx) {
  const services = structuredClone(ctx.table);
  for (const name of Object.keys(ids)) {
    const url = urlFor(ctx,name);
    if (url) services[name].url = url;
    else delete services[name];
  }
  if (ctx.services?.['liter-llm'] && ctx.services['liter-llm'].provider !== 'native' &&
      !fs.existsSync(path.join(ctx.home,'.local/bin/liter-llm'))) delete services['liter-llm'];
  return services;
}

function ownedUrl(ctx,file,name,url) {
  const initial = ctx.table[name].url;
  return url === initial || url === initial.replace('localhost','127.0.0.1') ||
    (Boolean(url) && url === ctx.previous[file]?.[name]);
}

function updateJson(ctx,file,key) {
  const config = read(file);
  const entries = config[key] || {};
  let changed = false;
  for (const name of Object.keys(ids)) {
    let entry = entries[name];
    const url = urlFor(ctx,name);
    if (!entry && ctx.previous[file]?.[name] === '' && url) {
      config[key] ??= entries;
      entry = entries[name] = key === 'mcp' ? {type:'remote',url,enabled:true} : {type:ctx.table[name].type,url};
      changed = true;
    }
    if (!entry || (!ownedUrl(ctx,file,name,entry.url) && entry.url !== url)) continue;
    const canReference = tokenNames[name] && (key === 'mcp' || file === path.join(ctx.home,'.claude/mcp.json'));
    const ref = !canReference ? null : 'Bearer '+(key === 'mcp' ? '{env:'+tokenNames[name]+'}' : '${'+tokenNames[name]+'}');
    const customHeaders = Object.entries(entry.headers || {}).some(([k,v]) => k !== 'Authorization' || v !== ref);
    if (entry.url !== url && customHeaders) continue;
    ctx.previous[file] ??= {};
    ctx.previous[file][name] = url;
    if (ref && entry.headers?.Authorization === ref && !dockerToken(ctx,name)) {
      delete entry.headers.Authorization;
      if (!Object.keys(entry.headers).length) delete entry.headers;
      changed = true;
    } else if (ref && dockerToken(ctx,name) && !entry.headers?.Authorization) {
      (entry.headers ??= {}).Authorization = ref;
      changed = true;
    }
    if (url === entry.url) continue;
    if (url) entry.url = url;
    else if (Object.keys(entry).every(k => ['type','url','enabled'].includes(k))) delete entries[name];
    else continue;
    changed = true;
  }
  if (changed) atomic(file,JSON.stringify(config,null,2)+'\n');
}

function updateToml(ctx,file) {
  const original = fs.readFileSync(file,'utf8');
  const seen = new Set();
  let updated = original.replace(/^\[mcp_servers\.([\w-]+)\][^\n]*\n[\s\S]*?(?=^[ \t]*\[|(?![\s\S]))/gm,(body,name) => {
    if (!ids[name]) return body;
    seen.add(name);
    const match = /^[ \t]*url[ \t]*=[ \t]*"([^"\n]+)"[ \t]*$/m.exec(body);
    const url = urlFor(ctx,name);
    if (!match || (!ownedUrl(ctx,file,name,match[1]) && match[1] !== url)) return body;
    const expectedSetting = 'bearer_token_env_var = "' + tokenNames[name] + '"';
    const customHeader = /^\s*(?:http_headers|env_http_headers|headers|bearer_token_env_var)\s*=/m.test(body.replace(expectedSetting,''));
    if (url !== match[1] && customHeader) return body;
    ctx.previous[file] ??= {};
    ctx.previous[file][name] = url;
    if (file === ctx.codexConfig && tokenNames[name]) {
      const setting = 'bearer_token_env_var = "' + tokenNames[name] + '"';
      if (dockerToken(ctx,name) && !/^\s*(?:bearer_token_env_var|http_headers|env_http_headers)\s*=/m.test(body)) {
        body = body.trimEnd() + '\n' + setting + '\n\n';
      } else if (!dockerToken(ctx,name) && body.includes(setting)) body = body.replace(setting + '\n','');
    }
    if (url && url !== match[1] && /^\s*(?:http_headers|env_http_headers|headers)\s*=/m.test(body)) return body;
    if (url) return body.replace(match[1],() => url);
    const simple = body.split('\n').slice(1).every(line => /^\s*(?:#|$|(?:type|url)\s*=)/.test(line));
    return simple ? '' : body;
  });
  for (const name of Object.keys(ids)) {
    const url = urlFor(ctx,name);
    if (seen.has(name) || !url || ctx.previous[file]?.[name] !== '') continue;
    updated += '\n[mcp_servers.' + name + ']\ntype = "' + ctx.table[name].type + '"\nurl = "' + url + '"\n';
    if (file === ctx.codexConfig && dockerToken(ctx,name)) updated += 'bearer_token_env_var = "' + tokenNames[name] + '"\n';
    ctx.previous[file][name] = url;
  }
  if (updated !== original) atomic(file,updated);
}

export function syncInstalledMcp({home = os.homedir(),env = process.env} = {}) {
  const ctx = scoped(home);
  if (!ctx.services) return;
  ctx.codexConfig = path.join(env.CODEX_HOME || path.join(home,'.codex'),'config.toml');
  const files = [
    [path.join(home,'.claude/mcp.json'),'mcpServers'], [path.join(home,'.opencode/opencode.json'),'mcp'],
    [path.join(home,'.minimax/mcp/mcp.json'),'mcpServers'], [path.join(home,'.cursor/mcp.json'),'mcpServers'],
    [path.join(home,'.codeium/windsurf/mcp/mcp.json'),'mcpServers'],
    [ctx.codexConfig], [path.join(home,'.kimi-code/config.toml')],
  ];
  for (const [file,key] of files) {
    if (!fs.existsSync(file) || !fs.lstatSync(file).isFile()) continue;
    if (key) updateJson(ctx,file,key);
    else updateToml(ctx,file);
  }
  atomic(ctx.previousFile,JSON.stringify(ctx.previous,null,2)+'\n');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const ctx = scoped(process.env.HOME || os.homedir());
  if (process.argv[2] === 'table') console.log(JSON.stringify({services:tableFor(ctx)}));
  else if (process.argv[2] === 'url') process.stdout.write(urlFor(ctx,process.argv[3]));
  else if (process.argv[2] === 'liter') process.exitCode = tableFor(ctx)['liter-llm'] ? 0 : 1;
  else syncInstalledMcp({home:ctx.home});
}
