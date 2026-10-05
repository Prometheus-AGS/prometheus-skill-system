#!/usr/bin/env node
// Final-boundary caller observes the production volume and actual frozen output.
import fs from 'node:fs';
import path from 'node:path';
import { probeFilesystemCapabilities } from '../lib/capabilities.js';
import { captureMaterialization, compareMaterializations } from '../generated-materialization-inventory.mjs';

let code = 0;
const results = [];
const args = new Map();
try {
  for (let i=2;i<process.argv.length;i+=2) {
    const key=process.argv[i], value=process.argv[i+1];
    if (!['--manifest','--scratch','--evidence'].includes(key) || args.has(key) || !value || !path.isAbsolute(value))
      throw new Error('unique explicit absolute manifest, scratch and evidence paths required');
    args.set(key,value);
  }
  for (const key of ['--manifest','--scratch','--evidence']) if (!args.has(key)) throw new Error(`missing ${key}`);
  const receipts=JSON.parse(fs.readFileSync(args.get('--manifest'),'utf8'));
  if (!Array.isArray(receipts) || !receipts.length) throw new Error('no production materializations supplied');
  const probe=fs.mkdtempSync(path.join(fs.realpathSync(args.get('--scratch')),'materialization-volume-'));
  try {
    const capability=probeFilesystemCapabilities(probe);
    for (const input of receipts) {
      const first=JSON.parse(fs.readFileSync(input.first,'utf8'));
      const second=JSON.parse(fs.readFileSync(input.second,'utf8'));
      const current=captureMaterialization(input.root,second.ownership,capability);
      // Never attach a different volume's empirical capability to old snapshots.
      if (fs.statSync(input.root).dev!==fs.statSync(probe).dev) throw new Error('materialization probe is on a different volume');
      first.posixModes=capability.posixModes; second.posixModes=capability.posixModes;
      // Older mini receipts observed hashes and modes without byte lengths.
      // Compare precisely that observed subset, without inventing past sizes.
      if (input.format==='hash-mode-entries') {
        const normalize=inventory=>({schemaVersion:1,ownership:[...inventory.ownership].sort(),posixModes:inventory.posixModes,
          entries:inventory.entries.map(row=>({path:row.path,kind:row.kind,
            mode:row.mode.replace(/^0o/,''),sha256:row.sha256??null})).sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0)});
        for (const inventory of [first,second,current]) Object.assign(inventory,normalize(inventory));
      }
      for (const [name,before,after] of [['two-production-materializations',first,second],['frozen-current-materialization',second,current]]) {
        const comparison=compareMaterializations(before,after);
        results.push({root:input.root,name,...comparison,capability,currentDigest:current.materializationSha256});
        code= comparison.exitCode===1 ? 1 : code===1 ? 1 : Math.max(code,comparison.exitCode);
      }
    }
  } finally { fs.rmSync(probe,{recursive:true,force:true}); }
} catch(error) { results.push({exitCode:2,diagnosis:error.message}); if (code!==1) code=2; }
if (args.has('--evidence')) {
  fs.mkdirSync(args.get('--evidence'),{recursive:true});
  fs.writeFileSync(path.join(args.get('--evidence'),'generated-byte-mode-certification.json'),JSON.stringify({exitCode:code,results},null,2)+'\n');
}
console.log(JSON.stringify({exitCode:code,results}));
process.exitCode=code;
