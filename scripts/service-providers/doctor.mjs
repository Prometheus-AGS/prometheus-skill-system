#!/usr/bin/env node
// The native doctor predates per-service providers. Preserve its independent
// checks and substitute checks over the actual receipt for native-only scopes.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {envFor} from './catalog.mjs';

const root = fileURLToPath(new URL('../../',import.meta.url));
const home = process.env.HOME || os.homedir();
const state = path.join(home,'.prometheus/service-providers/full.json');
const version = JSON.parse(fs.readFileSync(path.join(root,'skill-system.json'))).releaseVersion;
const binDir = process.env.PROMETHEUS_COMMAND_DIRECTORY || path.join(home,'.local/bin');
const args = ['doctor', ...process.argv.slice(2)];
const excluded = ['control.kbd-runtime','state.kbd-orchestrator','control.kbd-rollout','remote-queue'];
const sha = data => createHash('sha256').update(data).digest('hex');

async function hashFile(file) {
  const hash = createHash('sha256');
  for await (const chunk of fs.createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}
function run(cmd, argv, env) {
  const result = spawnSync(cmd,argv,{cwd:root,env,stdio:'inherit'});
  if (result.error) throw result.error;
  return result.status ?? 1;
}
async function selectedBinaries(services) {
  const managed = [['prometheus'],['pk'], ['pk-cherry','pk-cherry'], ['prometheus-learning-worker','learning-worker'],
    ['surreal-memory-server','surreal-memory'], ['prometheus-exec','prometheus-exec']];
  const platform = process.platform + '-' + (process.arch === 'x64' ? 'amd64' : process.arch);
  const tag = 'v' + version;
  const release = path.join(home,'.prometheus/install/releases',tag,platform);
  for (const [name,id] of managed) {
    if (id && services[id]?.provider !== 'native') continue;
    const dest = path.join(binDir,name);
    fs.accessSync(dest,fs.constants.X_OK);
    const digest = await hashFile(dest);
    const receipt = path.join(release,'installed',name,sha(dest)+'.json');
    if (fs.existsSync(receipt)) {
      const installed = JSON.parse(fs.readFileSync(receipt));
      const manifest = fs.readFileSync(path.join(release,'manifest.json'));
      const hash = sha(manifest);
      const source = JSON.parse(manifest).files.find(file => file.name === name);
      const manifestName = 'prometheus-binaries-'+tag+'-'+platform+'.manifest.json';
      const sums = fs.readFileSync(path.join(release,'SHA256SUMS'),'utf8').split(/\r?\n/);
      if (!sums.some(line => line.split(/ [ *]/)[0] === hash && line.endsWith(' '+manifestName)) ||
          hash !== installed.manifestSha256 || installed.destination !== dest || installed.tag !== tag ||
          installed.platform !== platform || installed.installedSha256 !== digest || installed.buildSha256 !== source?.sha256) {
        throw new Error('Invalid installed release evidence for '+name);
      }
    }
    if (process.platform === 'darwin' && run('codesign',['--verify','--strict',dest],process.env)) {
      throw new Error('Invalid native signature for '+name);
    }
    console.log('Native binary '+name+' SHA-256 '+digest);
  }
}
async function main() {
  let env = {...process.env};
  let health = 0;
  if (fs.existsSync(state)) {
    const {services} = JSON.parse(fs.readFileSync(state));
    env = {...env,...envFor(Object.values(services))};
    health = run(process.execPath,['scripts/service-providers/cli.mjs','status','--home',home],env);
    // These legacy checks assume every service has the default host supervisor.
    excluded.push('services.launch-agents');
    if (services['learning-worker']?.provider !== 'native') excluded.push('learning.worker');
    if (services['prometheus-exec']?.provider !== 'native') excluded.push('execution.runtime');
    if (services['surreal-memory']?.provider === 'off' || services['surreal-memory']?.port !== 23001) excluded.push('learning.surreal-memory');
    if (services['liter-llm']?.provider === 'off' && services['openai-proxy']?.provider === 'off') excluded.push('review.judge-gateway');
    if (['pk-cherry','learning-worker','surreal-memory','prometheus-exec'].some(id => services[id]?.provider !== 'native')) {
      await selectedBinaries(services);
      excluded.push('binaries.manifest');
    }
  }
  args.push(...excluded.flatMap(id => ['--exclude',id]));
  const cmd = path.join(binDir,'prometheus');
  const status = fs.existsSync(cmd) ? run(cmd,args,env) : run('cargo',['run','--manifest-path','tools/prometheus-cli/Cargo.toml','--',...args],{...env,RUSTUP_TOOLCHAIN:'stable'});
  process.exitCode = status || health;
}
main().catch(error => { console.error('doctor: '+error.message); process.exitCode=1; });
