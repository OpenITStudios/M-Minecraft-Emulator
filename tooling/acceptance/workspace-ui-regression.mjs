import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.MIVUBI_PLAYWRIGHT_MODULE||'playwright');
import fs from 'node:fs';
// Optional browser acceptance harness. Mocked Tauri only; never controls a game.
fs.mkdirSync('workspace',{recursive:true});
let browser;
(async()=>{
 browser=await chromium.launch({headless:true,channel:'msedge'});
 const page=await browser.newPage({viewport:{width:1366,height:900}});
 const catalog=JSON.parse(fs.readFileSync('engine/isolation-native/owned-runtime/minecraft-targets.json','utf8'));
 const process={pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true};
 const snapshot={schema:1,provider:'OWNED_V1186_LOCAL_TEST',operation:'Inspect',launcherExitCode:null,focusError:null,nativeProcesses:[],clients:catalog.targets.slice(1).map((t,i)=>({clientId:t.id,box:t.profile,profileExists:i<2,settingsVerified:i<2,boxedProcessCount:i<2?1:0,launchAllowed:false,status:i<2?'RUNNING':'ABSENT',processes:i<2?[{...process,pid:77+i,windowHandle:`0x${(0x123+i).toString(16)}`}]:[]}))};
 const errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(({catalog,snapshot})=>{
  window.__calls=[];window.__listeners={};
  window.__startupFailure=true;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={transformCallback:callback=>{window.__listeners[1]=callback;return 1;},invoke:async(command,args)=>{
   window.__calls.push({command,args});
   if(command==='plugin:event|listen')return 1;
   if(command==='plugin:event|unlisten')return null;
   if(command==='desktop_suspend_workspace')return true;
   if(command==='desktop_manager_settings')return {observation:snapshot,engineError:null,detectedMinecraft:'C:\\Game\\Minecraft.Windows.exe',targetError:null,settings:{sessionDirectory:'D:\\Developer engine',minecraftExecutable:'C:\\Game\\Minecraft.Windows.exe'},catalog};
   if(command==='desktop_present_workspace'&&window.__startupFailure&&args.clientId==='02'){window.__startupFailure=false;throw 'Client exited during startup admission';}
   if(command==='desktop_present_workspace'&&window.__failPresentation)throw 'Workspace could not prepare the replacement';
   if(command==='desktop_update_workspace'&&window.__failUpdate)throw 'Workspace geometry changed during resize';
   if(['desktop_present_workspace','desktop_update_workspace','desktop_release_workspace','desktop_suspend_workspace'].includes(command))return null;
   if(command==='desktop_owned_test_operation')return {...snapshot,operation:args.request.operation};
   throw Error(command);
  }};
 },{catalog,snapshot});
 await page.goto('http://127.0.0.1:1421');
 await page.waitForFunction(()=>document.querySelector('[role=tab][aria-selected=true]')?.textContent==='Minecraft 03',{},{timeout:5000});
 if(!await page.evaluate(()=>window.__calls.some(c=>c.command==='desktop_present_workspace'&&c.args.clientId==='02')))throw Error('Failed startup slot was not exercised');
 const card=page.locator('.um-card').filter({has:page.getByRole('heading',{name:'Minecraft 02',exact:true})});
 await card.getByRole('button',{name:'Play',exact:true}).click();
 await page.getByRole('heading',{name:'Minecraft 02',exact:true}).first().waitFor();
 if(await page.locator('.um-card:visible').count()!==3)throw Error('Active client duplicated in rail');
 await page.getByRole('tab',{name:'Minecraft 03',exact:true}).click();
 if(await page.getByRole('tab',{name:'Minecraft 03',exact:true}).getAttribute('aria-selected')!=='true')throw Error('Tab selection did not commit');
 await page.getByRole('tab',{name:'Minecraft 02',exact:true}).click();
 for(const [width,height] of [[1366,900],[1180,760],[920,620]]){
  await page.setViewportSize({width,height});await page.waitForTimeout(150);
  const layout=await page.evaluate(()=>({horizontal:document.documentElement.scrollWidth>innerWidth,vertical:document.documentElement.scrollHeight>innerHeight,rect:(()=>{const r=document.querySelector('.nw-native-slot').getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height,bottom:r.bottom}})()}));
  console.log(width,height,JSON.stringify(layout));
  if(layout.horizontal||layout.vertical||layout.rect.bottom>height)throw Error('Workspace leaves the visible page');
  await page.screenshot({path:`workspace/native-workspace-${width}.png`});
 }
 await page.getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByText('Automatic setup',{exact:true}).waitFor();
 if(!await page.evaluate(()=>window.__calls.some(c=>c.command==='desktop_suspend_workspace')))throw Error('Settings did not suspend native surface');
 if(await page.getByRole('button',{name:/Choose/}).count())throw Error('Manual chooser returned');
 await page.getByRole('button',{name:'Clients 04',exact:true}).click();
 await page.getByRole('tab',{name:'Minecraft 02',exact:true}).click();
 await page.getByRole('button',{name:'Restore window',exact:true}).click();
 await page.getByRole('heading',{name:'Your Minecraft workspace',exact:true}).waitFor();
 await page.evaluate(()=>window.__failPresentation=true);await card.getByRole('button',{name:'Play',exact:true}).click();
 if(await card.getByRole('button',{name:'Play',exact:true}).isHidden())throw Error('Failed presentation committed active identity');
 await page.evaluate(()=>{window.__failPresentation=false;window.__failUpdate=false;});
 await card.getByRole('button',{name:'Play',exact:true}).click();
 await page.evaluate(()=>window.__failUpdate=true);
 await page.setViewportSize({width:1000,height:720});await page.waitForTimeout(300);
 if(!await page.locator('.nw-stage').getByRole('heading',{name:'Minecraft 02',exact:true}).count())throw Error('Resize error lost the selected client while backend lease still exists');
 await page.getByRole('button',{name:'Restore window',exact:true}).click();
 await page.evaluate(()=>window.__failUpdate=false);
 await card.getByRole('button',{name:'Play',exact:true}).click();
 await page.evaluate(()=>window.__listeners[1]({event:'workspace-surface-error',id:1,payload:'Workspace helper recovery test'}));
 await page.getByText('The workspace needs attention.',{exact:true}).waitFor();
 if(!await page.locator('.nw-stage').getByRole('heading',{name:'Minecraft 02',exact:true}).count())throw Error('Native event error lost recovery controls');
 await page.getByRole('button',{name:'Restore window',exact:true}).click();
 if(errors.length)throw Error(errors.join('\n'));
 console.log('Mocked bridge: selection, settings release, normal restore, failed presentation and narrow/normal layout PASS. No game captured or controlled.');
 await browser.close();
})().catch(async error=>{console.error(error);await browser?.close();process.exitCode=1});
