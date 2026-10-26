// Optional source-only check: build the frontend first and supply MIVUBI_PLAYWRIGHT_MODULE.
// Uses a headless browser and an in-memory Tauri adapter; never opens profiles or games.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFile, mkdir} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const require = createRequire(import.meta.url);
const {chromium} = require(process.env.MIVUBI_PLAYWRIGHT_MODULE || 'playwright');
const dist = fileURLToPath(new URL('../../apps/desktop/dist/', import.meta.url));
const browser = await chromium.launch({headless: true, channel: 'msedge'});
try {
  const page = await browser.newPage({viewport: {width: 920, height: 620}});
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.route('http://lazy-tester.test/**', async route => {
    const pathname = new URL(route.request().url()).pathname;
    const file = path.resolve(dist, '.' + (pathname === '/' ? '/index.html' : pathname));
    assert.ok(file.startsWith(dist));
    await route.fulfill({body: await readFile(file), contentType: file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'text/html'});
  });
  await page.addInitScript(() => {
    window.__calls = [];
    window.__offline = false;
    window.__holdInspect = false;
    const brokenSetup = new URL(location.href).searchParams.has('broken');
    window.__bindingFailure = brokenSetup ? 'Saved profiles could not be verified.' : '';
    window.__locationFailure = brokenSetup ? 'The saved data folder is unavailable.' : '';
    window.__dataPath = null;
    window.__locateOutcome = 'cancel';
    const profiles = [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}));
    const settings = {graphicsPresets: ['Low','Low','Low','Low','Low'], activeFps: 144, backgroundFps: 10, renderDistanceChunks: 12, canApplyPreset: true, changedProfiles: null};
    const snapshot = () => ({schema: 1, storage: null, profiles: profiles.map(profile => window.__runningSlot === profile.slot || window.__runningSlots?.includes(profile.slot) ? {
      ...profile, state: 'RUNNING', process: {pid: 100 + profile.slot, createdFileTime: '134000000000000001', sessionId: 1, windowHandle: '0x' + (0x123 + profile.slot).toString(16), responding: true},
    } : profile)});
    let callback = 0;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return ++callback; },
      async invoke(command, args) {
        window.__calls.push({command, args});
        if(command==='startup_diagnostic_result')return null;
        if (command === 'plugin:event|listen') return callback;
        if (command === 'plugin:event|unlisten') return null;
        if (command === 'lazy_tester_profile_binding') {
          if(new URL(location.href).searchParams.has('startup-retry')&&!window.__bindingCheckedOnce){
            window.__bindingCheckedOnce=true;window.__finishBinding=()=>{};
            return {schema:1,required:true,configured:false,issue:'No saved launcher connection was found. Set up a data folder or connect existing launcher data.'};
          }
          if (window.__bindingFailure) throw window.__bindingFailure;
          if (!window.__finishBinding) await new Promise(resolve => { window.__finishBinding = resolve; });
          return {schema: 1, required: false, configured: true, issue: null};
        }
        if (command === 'lazy_tester_data_location') {
          if(new URL(location.href).searchParams.has('startup-location-retry')&&!window.__locationCheckedOnce){window.__locationCheckedOnce=true;throw 'Saved data is temporarily unavailable.';}
          if (args.action === 'Locate') {
            if (window.__locateOutcome === 'invalid') throw 'The selected folder does not match the saved profile identities.';
            if (window.__locateOutcome === 'valid') {
              window.__bindingFailure = ''; window.__locationFailure = '';
              window.__dataPath = 'D:\\Recovered Data';
              window.__finishBinding = () => {};
            }
          }
          if (window.__locationFailure) throw window.__locationFailure;
          return {schema: 1, defaultPath: 'D:\\Lazy Tester\\Data', path: window.__dataPath,
            profiles: window.__dataPath ? profiles.map(profile => ({slot: profile.slot, path: window.__dataPath + '\\Profiles\\Minecraft-' + profile.slot})) : [], canLocate: true};
        }
        if (command === 'lazy_tester_present' || command === 'lazy_tester_clear_previews' || command.startsWith('desktop_')) return null;
        if (command === 'lazy_tester_previews') return {schema: 1, tiles: args.tiles.map(tile => ({slot: tile.slot, registered: false, reason: null}))};
        if (command !== 'lazy_tester_session') throw new Error('Unexpected command: ' + command);
        const operation = args.request.operation;
        if (operation === 'Inspect' && window.__holdInspect) await new Promise(resolve => { window.__finishInspect = resolve; });
        if (window.__offline) throw 'Session service unavailable. Use Refresh to reconnect.';
        if (operation === 'InspectSettings') return {...snapshot(), settings: {...settings, canApplyPreset: !(window.__runningSlot || window.__runningSlots?.length)}};
        if (operation === 'Launch') return {...snapshot(), profiles: profiles.map(profile => profile.slot === args.request.slot ? {...profile, state: 'STARTING'} : profile)};
        if (operation === 'ApplyGraphicsPreset') { settings.graphicsPresets[args.request.slot-1] = args.request.preset; return {...snapshot(), settings:{...settings,changedProfiles:1}}; }
        if (operation === 'SavePerformance') {
          settings.activeFps = args.request.activeFps; settings.backgroundFps = args.request.backgroundFps;
          return {...snapshot(), settings};
        }
        if (operation === 'InspectStorage') return {...snapshot(), storage: profiles.map(profile => ({
          slot: profile.slot, bytes: 1048576, files: 1, partial: true, disk: null,
          cache: {bytes: 0, files: 0, partial: false, canClear: false, issue: null},
        }))};
        if (operation === 'Inspect') { await new Promise(resolve=>setTimeout(resolve,window.__inspectDelay||0));return snapshot(); }
        throw new Error('Unexpected operation: ' + operation);
      },
    };
  });
  const heading = name => page.getByRole('heading', {name, exact:true});
  const nav = name => page.getByRole('navigation', {name:'Main navigation'}).getByRole('button',{name,exact:true});
  const section = name => page.getByRole('navigation',{name:'Settings navigation'}).getByRole('button',{name,exact:true});
  for(const query of ['startup-retry','startup-retry','startup-location-retry']){
    await page.goto('http://lazy-tester.test/?'+query+'=1');
    if(query==='startup-location-retry'){
      await page.waitForFunction(()=>!!window.__finishBinding);await page.evaluate(()=>window.__finishBinding());
    }
    await heading('Ready when you are').waitFor({timeout:7000});
    assert.equal(await page.getByRole('button',{name:'Launch Minecraft - 1',exact:true}).isEnabled(),true,'startup must recover an available existing connection without setup clicks');
    assert.equal(await page.getByRole('button',{name:'Set up launchers',exact:true}).count(),0);
    assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.import===true||['Choose','Locate'].includes(c.args?.action)||c.args?.request?.operation==='Launch')),false);
  }
  await page.goto('http://lazy-tester.test/');
  await page.waitForFunction(()=>!!window.__finishBinding);await page.evaluate(()=>window.__finishBinding());
  await heading('Ready when you are').waitFor();
  assert.equal(await page.locator('.lt-card').count(),5);
  assert.equal(await nav('Play').getAttribute('aria-current'),'page');
  // Unchanged background checks must not flash controls, lock cards or move native surfaces.
  await page.waitForTimeout(200);
  await page.evaluate(()=>{
    window.__inspectDelay=150;window.__visualChanges=[];
    window.__previewBaseline=window.__calls.filter(c=>c.command==='lazy_tester_previews').length;
    const refresh=document.querySelector('.lt-page-header button'),help=document.querySelector('.lt-help');
    const cards=[...document.querySelectorAll('.lt-card')];
    window.__stableObserver=new MutationObserver(records=>{
      for(const record of records){const node=record.target.nodeType===3?record.target.parentElement:record.target;
        if(refresh.contains(node)||refresh===node)window.__visualChanges.push('refresh');
        if(help.contains(node)||help===node)window.__visualChanges.push('help');
        if(cards.includes(node))window.__visualChanges.push('card');
      }
    });window.__stableObserver.observe(document.querySelector('.lt-app'),{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:['disabled']});
  });
  await page.waitForTimeout(30500);
  assert.deepEqual(await page.evaluate(()=>window.__visualChanges),[],'unchanged polling must leave controls stable');
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.command==='lazy_tester_previews').length===window.__previewBaseline),true,'unchanged polling must not reapply native geometry');
  await page.evaluate(()=>window.__stableObserver.disconnect());
  await nav('Settings').click();await heading('General').waitFor();
  await page.getByLabel('While playing',{exact:true}).fill('60');await page.getByLabel('In the background',{exact:true}).fill('5');
  await section('Data Storage').click();await heading('Data Storage').waitFor();
  await page.getByText('At least 1.0 MB',{exact:true}).waitFor();
  await section('General').click();assert.equal(await page.getByLabel('While playing',{exact:true}).inputValue(),'60');
  await page.getByRole('button',{name:'Save changes',exact:true}).click();
  await page.getByText('Saved. New limits apply when each Minecraft profile next starts.').waitFor();
  await page.getByLabel('Graphics preset',{exact:true}).selectOption('High');
  await page.getByRole('button',{name:'Apply preset',exact:true}).click();
  await page.getByText(/High saved and applied/).waitFor();
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.args?.request?.operation==='ApplyGraphicsPreset').length),1);
  await section('Data Storage').click();
  assert.equal(await page.getByLabel('Graphics preset',{exact:true}).count(),0);
  const advanced=page.locator('.lt-advanced');await advanced.locator('summary').click();
  assert.equal(await page.getByRole('button',{name:'Clear all data…',exact:true}).isDisabled(),true);
  assert.equal(await page.getByRole('button',{name:'Restore…',exact:true}).isDisabled(),true);
  assert.match(await advanced.getAttribute('open')??'',/^$/);
  for(const size of [{width:920,height:620},{width:1280,height:900},{width:1366,height:728},{width:1920,height:1080},{width:2560,height:1378},{width:3440,height:1440},{width:1000,height:1200}]){
    await page.setViewportSize(size);
    await nav('Play').click();await heading('Ready when you are').waitFor();
    assert.equal(await page.locator('.lt-card').count(),5);
    assert.equal(await page.evaluate(()=>document.querySelector('.lt-filmstrip').getBoundingClientRect().bottom>innerHeight),false);
    const geometry=await page.evaluate(()=>{
      const rect=selector=>{const r=document.querySelector(selector).getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom,right:r.right};};
      return {stage:rect('.lt-stage-space'),game:rect('.lt-viewport'),dock:rect('.lt-filmstrip'),previews:[...document.querySelectorAll('.lt-card-picture')].map(e=>({width:e.clientWidth,height:e.clientHeight}))};
    });
    const {stage,game,dock,previews}=geometry;
    assert.ok(Math.abs(game.width-game.height*16/9)<1,'game area must stay 16:9');
    assert.ok(Math.abs(game.width-Math.min(stage.width,stage.height*16/9))<1,'game area must use the largest fit');
    assert.ok(Math.abs(game.x+game.width/2-stage.x-stage.width/2)<1,'game must be centered horizontally');
    assert.ok(Math.abs(game.y+game.height/2-stage.y-stage.height/2)<1,'game must be centered vertically');
    assert.ok(game.x>=stage.x&&game.y>=stage.y&&game.right<=stage.right+1&&game.bottom<=stage.bottom+1,'game must fit without cropping');
    assert.ok(dock.width<=1320,'launcher cards must not stretch across ultrawide screens');
    assert.ok(previews.every(p=>Math.abs(p.width-p.height*16/9)<2),'previews must stay 16:9');
    await nav('Settings').click();await section('Data Storage').click();
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  }
  await page.setViewportSize({width:1280,height:900});
  // Preserve navigation clicks while an inspect is in flight.
  await page.evaluate(()=>{window.__holdInspect=true;});await page.waitForFunction(()=>!!window.__finishInspect);
  await nav('Settings').click();await section('General').click();
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});await heading('General').waitFor();
  await nav('Play').click();
  await page.evaluate(()=>{window.__holdInspect=true;window.__finishInspect=null;});
  await page.waitForFunction(()=>!!window.__finishInspect);
  await page.getByRole('button',{name:'Launch Minecraft - 1',exact:true}).click();
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.args?.request?.operation==='Launch').length),0,'launch must wait for the in-flight inspection');
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});
  await heading('Starting Minecraft - 1').first().waitFor();await nav('Settings').click();
  await page.evaluate(()=>{window.__runningSlots=[1,2];});await page.getByRole('button',{name:'Refresh',exact:true}).click();
  assert.equal(await page.getByRole('button',{name:'Apply preset',exact:true}).isDisabled(),true);
  await nav('Play').click();await page.getByRole('button',{name:'Open in separate windows',exact:true}).click();
  await heading('Games are in separate windows').waitFor();assert.equal(await page.locator('.lt-card').count(),5);
  await page.getByRole('button',{name:'Back to Play',exact:true}).click();await page.getByRole('button',{name:'Open in separate windows',exact:true}).waitFor();
  await page.waitForTimeout(200);
  const placement=await page.evaluate(()=>{
    const rect=document.querySelector('.lt-viewport').getBoundingClientRect();
    const call=window.__calls.filter(c=>c.command==='lazy_tester_present').at(-1);
    return {rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},viewport:call.args.viewport};
  });
  for(const key of ['x','y','width','height'])assert.equal(placement.viewport[key],placement.rect[key],'native placement must use the visible game rectangle');
  assert.equal(placement.viewport.pixelRatio,1);
  const cdp=await page.context().newCDPSession(page);
  for(const scale of [1.25,1.5]){
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:scale,mobile:false});
    await page.evaluate(()=>window.dispatchEvent(new Event('resize')));
    await page.waitForFunction(scale=>window.__calls.filter(c=>c.command==='desktop_update_workspace').at(-1)?.args.viewport.pixelRatio===scale,scale);
    const scaled=await page.evaluate(()=>{
      const r=document.querySelector('.lt-viewport').getBoundingClientRect();
      return {rect:{x:r.x,y:r.y,width:r.width,height:r.height},viewport:window.__calls.filter(c=>c.command==='desktop_update_workspace').at(-1).args.viewport};
    });
    for(const key of ['x','y','width','height'])assert.equal(scaled.viewport[key],scaled.rect[key],'DPI updates must preserve the exact input/placement rectangle');
  }
  await cdp.send('Emulation.clearDeviceMetricsOverride');await cdp.detach();
  const captures=fileURLToPath(new URL('../../../workspace/runs/lazy-tester-ui-regression/',import.meta.url));await mkdir(captures,{recursive:true});
  await page.screenshot({path:path.join(captures,'play.png')});
  await nav('Settings').click();await section('Data Storage').click();await page.locator('.lt-advanced summary').click();
  await page.screenshot({path:path.join(captures,'data-storage.png')});
  await page.evaluate(()=>{window.__offline=true;});await page.getByRole('button',{name:'Refresh',exact:true}).click();await page.getByRole('alert').waitFor();
  await page.evaluate(()=>{window.__offline=false;});await nav('Play').click();await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('.lt-connection').textContent==='Connected');
  // Existing-data recovery is still available without launching or deleting profiles.
  await page.goto('http://lazy-tester.test/?broken=1');await page.getByText('Saved profiles could not be verified.',{exact:true}).waitFor();
  assert.equal(await heading('Ready when you are').count(),0);
  await heading('Connection needs attention').waitFor();
  assert.equal(await page.getByRole('button',{name:'Launch Minecraft - 1',exact:true}).isDisabled(),true);
  // A transient first check must recover through a deliberate Refresh, without import/reset.
  await page.evaluate(()=>{window.__bindingFailure='';window.__locationFailure='';window.__finishBinding=()=>{};});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await heading('Ready when you are').waitFor();
  assert.equal(await page.getByRole('button',{name:'Launch Minecraft - 1',exact:true}).isEnabled(),true);
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.import||c.args?.action==='Choose'||c.args?.request?.operation==='Launch')),false);
  await page.goto('http://lazy-tester.test/?broken=1');await heading('Connection needs attention').waitFor();
  await page.getByRole('region',{name:'Launcher recovery'}).getByRole('button',{name:'Data Storage',exact:true}).click();await heading('Data Storage').waitFor();
  await page.getByText('Connect or locate launcher data',{exact:true}).click();
  const locate=page.locator('.lt-location-options').getByRole('button',{name:'Locate existing data',exact:true});await locate.click();
  await page.evaluate(()=>{window.__locateOutcome='invalid';});await locate.click();await page.getByText('The selected folder does not match the saved profile identities.',{exact:true}).waitFor();
  await page.evaluate(()=>{window.__locateOutcome='valid';});await locate.click();await page.getByText('D:\\Recovered Data',{exact:true}).waitFor();
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.action==='Choose'||['Launch','ClearProfileCache'].includes(c.args?.request?.operation))),false);
  assert.deepEqual(errors,[]);
  console.log('PASS: automatic startup reconnect across reloads and transient binding/location failures without setup/import/launch; largest-fit 16:9 at seven window sizes; matching native IPC at 100/125/150% browser DPI; 30 seconds stable polling; queued launch/navigation; High/Low/FPS; desktop return; offline/data recovery. Mocked Tauri, no real game/data mutations.');
} finally { await browser.close(); }
