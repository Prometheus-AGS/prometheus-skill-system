#!/usr/bin/env node
import {applyServices,manageServices,planServices} from './index.mjs';

const usage='Usage: node scripts/service-providers/cli.mjs plan|apply|status|stop|uninstall [--service ID=auto|native|docker|reuse|off] [--endpoint ID=URL] [--port ID=PORT] [--enabled ID,...] [--home DIR] [--non-interactive] [--json]';
try {
  const [action,...args]=process.argv.slice(2);
  if(['--help','-h','help'].includes(action)) console.log(usage);
  else {
    if(!['plan','apply','status','stop','uninstall'].includes(action)) throw Error(usage);
    const o={interactive:!process.env.CI,selections:{},ports:{},endpoints:{}};
    for(let i=0;i<args.length;i++) {
      const flag=args[i];
      if(flag==='--non-interactive') {o.interactive=false;continue;}
      if(flag==='--json') {o.json=true;o.interactive=false;continue;}
      if(flag==='--source-only') {o.sourceOnly=true;continue;}
      const val=args[++i];if(val==null||val.startsWith('--')) throw Error('Missing value for '+flag);
      if(flag==='--home') o.home=val;
      else if(flag==='--enabled'||flag==='--only') o[flag==='--only'?'only':'enabled']=val.split(',').filter(Boolean);
      else if(['--port','--service','--endpoint'].includes(flag)) {
        const at=val.indexOf('=');if(at<1 || at===val.length-1) throw Error('Expected ID=value after '+flag);
        const field={'--port':'ports','--service':'selections','--endpoint':'endpoints'}[flag];
        o[field][val.slice(0,at)]=val.slice(at+1);
      }else throw Error('Unknown option '+flag);
    }
    const lifecycle=!['plan','apply'].includes(action);
    if(lifecycle && !o.enabled && Object.keys(o.selections).length) o.enabled=Object.keys(o.selections);
    const result=action==='apply'?await applyServices(o):action==='plan'?await planServices(o):await manageServices(action,o);
    if(o.json) console.log(JSON.stringify(result,null,2));
    else {
      for(const s of result.services) console.log([s.id,s.provider,s.endpoint,s.status||s.phase||'',s.origin||''].filter(Boolean).join('\t'));
      for(const c of result.conflicts) console.error(c);
    }
    if(!result.applicable || result.healthy===false) process.exitCode=1;
  }
}catch(err){console.error(err.message);process.exitCode=1;}
