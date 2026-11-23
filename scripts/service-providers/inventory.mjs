import fs from 'node:fs';
import path from 'node:path';
import {definitions,readJson,run} from './catalog.mjs';

const sha=/^[0-9a-f]{40}$/, digest=/^sha256:[0-9a-f]{64}$/;
export async function inventory(o) {
  const tag='v'+o.version;
  let data;
  const local=o.env.PROMETHEUS_SERVICE_IMAGE_INVENTORY;
  if(local) data=readJson(local);
  else {
    const response=await fetch('https://github.com/'+o.repository+'/releases/download/'+tag+'/service-images.json',{signal:AbortSignal.timeout(15000)});
    if(!response.ok) throw Error('No published service-images.json for '+o.repository+' '+tag+' (HTTP '+response.status+'); choose native, reuse or off');
    data=await response.json();
  }
  if(data.schemaVersion!==1 || data.repository!==o.repository || data.tag!==tag || !sha.test(data.sourceCommit) || !data.images) throw Error('Image inventory does not match this pack and version');
  const tagged=run('git',['rev-parse','--verify','refs/tags/'+tag+'^{commit}'],o);
  if(tagged.ok && tagged.text!==data.sourceCommit) throw Error('Image inventory does not match local tag '+tag);
  return data;
}
export function imageFor(id,o,catalog,platform) {
  if(id==='surrealdb') {
    const db=o.pack==='mini' ? fs.readFileSync(path.join(o.root,'versions.toml'),'utf8').match(/^"surrealdb"\s*=\s*\{\s*image\s*=\s*"surrealdb\/surrealdb:v3\.3\.0",\s*digest\s*=\s*"(sha256:[0-9a-f]{64})"/m)?.[1] : 'sha256:c35893410644e65fa90aa1bef446ac9cb0a0c2733b9d405ce3eb56754b85c330';
    if(!db) throw Error('Missing supported SurrealDB release pin in versions.toml');
    return 'surrealdb/surrealdb@'+db;
  }
  const config=readJson(path.join(o.root,'docker/service-images/catalog.json')).services[id], row=catalog.images[id];
  const expected='ghcr.io/'+o.repository.toLowerCase()+'-'+id;
  if(!row || row.repository!==expected || row.binary!==config.binary || !digest.test(row.digest) || !sha.test(row.source?.commit)) throw Error('Invalid pinned image for '+id);
  for(const arch of ['linux/amd64','linux/arm64']) {
    const descriptor=row.platforms?.[arch];
    if(!digest.test(descriptor?.digest) || !/^[a-f0-9]{64}$/.test(descriptor?.binarySha256)) throw Error('Missing binary or platform digest for '+id+' '+arch);
  }
  if(!definitions[id].imported) {
    const tree=run('git',['rev-parse','--verify','HEAD:'+config.directory],o);
    if(row.source.repository!==o.repository || row.source.path!==config.directory || row.source.commit!==catalog.sourceCommit || !sha.test(row.source.tree) || (tree.ok && (tree.text!==row.source.tree || !run('git',['diff','--quiet','HEAD','--',config.directory],o).ok))) throw Error('Pack source tree differs from released '+id);
  } else {
    const dir=config.directory;
    const pinned=o.pack==='full'? readJson(path.join(o.root,'skill-system.json')).imports.find(x=>x.path===dir)?.commit : fs.readFileSync(path.join(o.root,'versions.toml'),'utf8').split('\n').find(line=>line.startsWith('"'+dir+'"'))?.match(/=\s*"([a-f0-9]{40})"/)?.[1];
    const url=run('git',['config','--file','.gitmodules','--get','submodule.'+dir+'.url'],o);
    if(!pinned || pinned!==row.source.commit || row.source.path!=='.' || !url.ok || url.text.replace('https://github.com/','').replace(/\.git$/,'')!==row.source.repository) throw Error('Submodule source pin differs from released '+id);
  }
  if(id==='forge-mcp') {
    const pk=readJson(path.join(o.root,'skill-system.json')).imports.find(x=>x.path==='tools/prometheus-knowledge');
    for(const descriptor of Object.values(row.platforms)) {
      const companion=descriptor.companions?.pk;
      if(!/^[a-f0-9]{64}$/.test(companion?.binarySha256) || companion?.source?.commit!==pk?.commit || companion.source.path!=='.' ||
        companion.source.repository!==pk.repository.replace('https://github.com/','').replace(/\.git$/,'')) throw Error('Bundled pk source pin differs from released forge-mcp');
    }
  }
  // Pin the selected child as well as validating the published multi-platform index.
  return row.repository+'@'+row.platforms[platform].digest;
}
