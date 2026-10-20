import {createRequire} from 'node:module';
if(!process.argv.includes('--live'))throw Error('Explicit --live required: this may open one native Minecraft instance. No login or world automation.');
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.MIVUBI_PLAYWRIGHT_MODULE||'playwright');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
const four=process.argv.includes('--four');
(async()=>{
 const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
 try{
 const pages=browser.contexts().flatMap(c=>c.pages());
 const page=pages.find(p=>/127\.0\.0\.1:1421|tauri\.localhost/.test(p.url()));
 if(!page)throw Error('Developer manager page not found');
 page.setDefaultTimeout(15000);
 await page.getByRole('tab',{name:'Minecraft 01',exact:true}).waitFor();
 await page.waitForFunction(()=>[...document.querySelectorAll('[role=tab]')].length===4&&[...document.querySelectorAll('[role=tab]')].every(tab=>!tab.disabled),{},{timeout:60000});
 const inspectAll=()=>page.evaluate(async()=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;
   const config=await invoke('desktop_manager_settings',{settings:null});
   if(!config.observation)throw Error('Engine not verified');
   return config.observation;
 });
 const inspect=async()=>(await inspectAll()).nativeProcesses;
 if(four){
   const sessions=observation=>new Map([['01',observation.nativeProcesses.length===1?observation.nativeProcesses[0]:null],...observation.clients.map(slot=>[slot.clientId,slot.processes.length===1?slot.processes[0]:null])]);
   const original=sessions(await inspectAll());
   for(const id of ['01','02','03','04']){const target=original.get(id);if(!target?.responding||target.windowHandle==='0x0')throw Error(`Client ${id}: four responsive attributed windows required; no automatic launch in --four mode`);}
   for(let round=0;round<3;round++)for(const id of ['01','02','03','04']){
     await page.getByRole('tab',{name:`Minecraft ${id}`,exact:true}).click();
     await page.locator('.nw-stage').getByRole('heading',{name:`Minecraft ${id}`,exact:true}).waitFor();
     await page.waitForFunction(()=>[...document.querySelectorAll('[role=tab]')].every(tab=>!tab.disabled));
     const alerts=await page.locator('.um-alert:visible').allTextContents();if(alerts.length)throw Error(alerts.join('\n'));
   }
   const after=sessions(await inspectAll());
   for(const [id,target] of original){const current=after.get(id);if(!current?.responding||current.pid!==target.pid||current.createdFileTime!==target.createdFileTime||current.sessionId!==target.sessionId||current.windowHandle!==target.windowHandle)throw Error(`Client ${id}: identity or responsiveness lost`);}
   console.log('PASS: 12 actual-manager tab switches; four process/birth/session/HWND identities and window responsiveness retained.');
   console.log('NOT_EXECUTED: rendered gameplay/input, visual taskbar count, Alt-Tab, licensed worlds, multiplayer, 60-minute gameplay or displayed FPS.');
   return;
 }
 let native=await inspect();
 if(!native.length)await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('desktop_open_native_minecraft'));
 for(let i=0;i<16;i++){native=await inspect();if(native.length===1&&native[0].windowHandle!=='0x0'&&native[0].responding)break;await pause(1500);}
 if(native.length!==1||native[0].windowHandle==='0x0'||!native[0].responding)throw Error('Native baseline did not yield one responsive window');
 const original=native[0];
 console.log('BASELINE: one attributed responsive native window; no world or login operation');
 for(let i=0;i<4;i++){await pause(5000);native=await inspect();if(native.length!==1||native[0].pid!==original.pid||native[0].createdFileTime!==original.createdFileTime||!native[0].responding)throw Error('Unmodified baseline lost identity/responsiveness');}
 console.log('BASELINE: 20 seconds identity/responsiveness PASS (idle only)');
 await page.getByRole('tab',{name:'Minecraft 01',exact:true}).click();
 await page.locator('.nw-stage').getByRole('heading',{name:'Minecraft 01',exact:true}).waitFor();
 for(let i=0;i<3;i++){
   await page.getByRole('button',{name:'Settings',exact:true}).click();
   await page.getByText('Automatic setup',{exact:true}).waitFor();
   native=await inspect();if(native.length!==1||native[0].pid!==original.pid||!native[0].responding)throw Error('Suspended window lost attribution/responsiveness');
   await page.getByRole('button',{name:'Clients 04',exact:true}).click();
   await page.locator('.nw-stage').getByRole('heading',{name:'Minecraft 01',exact:true}).waitFor();
 }
 await pause(10000);native=await inspect();
 if(native.length!==1||native[0].pid!==original.pid||!native[0].responding)throw Error('Presented window lost identity/responsiveness');
 const alerts=await page.locator('.um-alert:visible').allTextContents();if(alerts.length)throw Error(alerts.join('\n'));
 console.log('MANAGER: actual tab adoption, three Settings suspend/resume cycles, and 10-second response PASS');
 console.log('NOT_EXECUTED: rendered gameplay, login, direct input, visual taskbar count, Alt-Tab, multiplayer or four clients');
 }finally{/* Exit this collector to detach CDP without closing the manager browser. */}
})().then(()=>process.exit(0),e=>{console.error(e.message);process.exit(1);});
