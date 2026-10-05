const fs=require('fs');
const path=require('path');
const {chromium}=require('/Users/gqadonis/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const root=JSON.parse(fs.readFileSync('/tmp/prometheus-docs-visual-current.json')).root;
(async()=>{
const browser=await chromium.launch({headless:true,executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',env:{...process.env,HOME:path.join(root,'home'),CODEX_HOME:path.join(root,'codex'),CORTEX_DATA_DIR:path.join(root,'cortex')}});
const report=[];
for(const [name,base] of [['full','http://127.0.0.1:4317/prometheus-skill-system/'],['mini','http://127.0.0.1:4318/prometheus-skills-mini/']]){
 const context=await browser.newContext({viewport:{width:1440,height:1000},colorScheme:'light',reducedMotion:'reduce'});
 const page=await context.newPage();const errors=[];
 page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});page.on('response',r=>{if(r.status()>=400)errors.push(r.status()+' '+r.url())});
 await page.goto(base,{waitUntil:'networkidle'});
 await page.screenshot({path:path.join(root,`${name}-home-desktop.png`)});
 console.log(name,JSON.stringify(await page.locator('nav').first().getByRole('link').evaluateAll(xs=>xs.map(x=>({text:x.textContent,href:x.getAttribute('href')})))));
 if(name==='full')await page.getByRole('button',{name:'Core',exact:true}).hover().catch(async()=>page.getByText('Core',{exact:true}).first().hover());
 await page.locator('nav').first().getByRole('link',{name:'Agent Teams',exact:true}).click();
 await page.waitForURL(base+'docs/agent-teams/overview');
 await page.locator('main h1').waitFor();
 const links=await page.locator('main a, aside a').evaluateAll(xs=>xs.map(x=>({text:x.textContent,href:x.getAttribute('href')})));
 console.log(name,'team links',JSON.stringify(links));
 const routes=name==='full'?['docs/agent-teams/overview','docs/guide/agent-teams','docs/guide/service-operations']:['docs/agent-teams/overview','docs/services/docker-services'];
 for(const route of routes){
  await page.goto(base+route,{waitUntil:'networkidle'});await page.locator('main h1').waitFor();
  const slug=route.split('/').pop();
  const headings=await page.locator('main h1,main h2,main h3').allTextContents();
  await page.screenshot({path:path.join(root,`${name}-${slug}-desktop.png`),fullPage:false});
  const internalLinks=await page.locator('main a').evaluateAll(xs=>xs.filter(x=>x.hash&&x.pathname===location.pathname).map(x=>({href:x.href,exists:!!document.getElementById(decodeURIComponent(x.hash.slice(1)))})));
  await page.setViewportSize({width:390,height:844});await page.screenshot({path:path.join(root,`${name}-${slug}-mobile.png`),fullPage:false});
  const layout=await page.evaluate(()=>({width:innerWidth,scrollWidth:document.documentElement.scrollWidth,missingImages:[...document.images].filter(i=>!i.complete||i.naturalWidth===0).map(i=>i.src)}));
  report.push({site:name,route,url:page.url(),headings,internalLinks,layout});
  if(route.includes('agent-teams')){
   const modelHeading=page.getByRole('heading').filter({hasText:/model|reasoning/i}).first();
   if(await modelHeading.count()){await modelHeading.scrollIntoViewIfNeeded();await page.screenshot({path:path.join(root,`${name}-${slug}-models-mobile.png`)});}
  }
  await page.setViewportSize({width:1440,height:1000});
 }
 await page.goto(base+'docs/agent-teams/overview',{waitUntil:'networkidle'});
 const theme=page.getByRole('button',{name:/switch between dark and light mode/i});
 if(await theme.count()){await theme.click();await page.screenshot({path:path.join(root,`${name}-teams-dark.png`)});}
 await page.setViewportSize({width:390,height:844});
 await page.getByRole('button',{name:/toggle navigation bar/i}).click();
 await page.screenshot({path:path.join(root,`${name}-mobile-menu.png`)});
 report.push({site:name,errors,mobileMenuVisible:await page.locator('.navbar-sidebar').isVisible()});
 await context.close();
}
await browser.close();fs.writeFileSync(path.join(root,'playwright-report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({report:path.join(root,'playwright-report.json'),screenshots:fs.readdirSync(root).filter(f=>f.endsWith('.png'))}));
})().catch(e=>{console.error(e);process.exitCode=1});
