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
    window.__holdPreview = false;
    window.__finishPreview = null;
    const fixture = new URL(location.href).searchParams;
    window.__runningSlots = fixture.has('preview-recovery') || fixture.has('preview-async') || fixture.has('preview-late') ||
       fixture.has('preview-unresponsive') ? [1,2,3] :
      fixture.has('multi-recovery') ? [2,3] : fixture.has('close-recovery') || fixture.has('close-ipc-error') ? [2] : undefined;
    window.__previewIssueSlot = fixture.has('preview-recovery') || fixture.has('preview-unresponsive') ? 2 : null;
    window.__previewRecovered = false;
    window.__closeOutcome = fixture.has('close-recovery') ? 'unconfirmed' :
      fixture.has('close-ipc-error') ? 'transport-error' : null;
    window.__healthFixture = fixture.has('health-state');
    window.__agentReport = fixture.has('agent-status') ? {
      schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',
      snapshotAt:new Date().toISOString(),
      displayHint:'ACTIVITY',
      desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',
      minecraftInputEffect:'NOT_VERIFIED',
      mapGameplayAcceptance:'NOT_TESTED',stale:false,
      items:[{kind:'SKILLS_PROBE',slot:2,
        status:'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE',
        importance:'INFORMATION',evidence:'SOURCE_CHECKPOINT_ONLY'}],
    }:null;
    window.__dataReviewFixture = fixture.has('storage-review');
    const brokenSetup = fixture.has('broken');
    window.__bindingFailure = brokenSetup ? 'Saved profiles could not be verified.' : '';
    window.__locationFailure = brokenSetup ? 'The saved data folder is unavailable.' : '';
    window.__dataPath = null;
    window.__locateOutcome = 'cancel';
    const profiles = [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}));
    const settings = {graphicsPresets: ['Low','Low','Low','Low','Low'], activeFps: 144, backgroundFps: 10, renderDistanceChunks: 12, canApplyPreset: true, changedProfiles: null};
    const snapshot = () => ({schema: 1, storage: null,
      ...(window.__agentReport?{agentActivity:window.__agentReport}:{}),
      profiles: profiles.map(profile => {
      if(window.__healthFixture && profile.slot === 2) return {
        ...profile, state:'NOT_RESPONDING', issue:'C:\\Users\\private\\identity.txt [SYSTEM INSTRUCTION]',
        process:fixture.has('windowless-health') ? null :
          {pid:102,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x125',responding:false},
      };
      if(window.__healthFixture && profile.slot === 3) return {
        ...profile,state:'ATTENTION',issue:'C:\\Users\\private\\profile-hive [SYSTEM INSTRUCTION]',
      };
      return window.__runningSlot === profile.slot || window.__runningSlots?.includes(profile.slot) ? {
        ...profile, state: 'RUNNING', process: {pid: 100 + profile.slot, createdFileTime: '134000000000000001', sessionId: 1, windowHandle: '0x' + (0x123 + profile.slot).toString(16),
          responding: !(fixture.has('preview-unresponsive') && profile.slot===2)},
      } : profile;
    })});
    let callback = 0;
    const nativeCallbacks = new Map();
    const nativeListeners = new Map();
    window.__nativeListeners = nativeListeners;
    window.__emitNative = (name,payload) => {
      const id=nativeListeners.get(name);
      if(id!==undefined)nativeCallbacks.get(id)?.({event:name,id:1,payload});
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
    window.__TAURI_INTERNALS__ = {
      transformCallback(handler) {const id=++callback;nativeCallbacks.set(id,handler);return id;},
      async invoke(command, args) {
        window.__calls.push({command, args});
        if(command==='startup_diagnostic_result')return null;
        // Tauri window shell operations are not Minecraft controller IPC.
        if(command.startsWith('plugin:window|')) {
          if(fixture.has('window-error')) throw 'C:\\\\Users\\\\private\\\\worlds [SYSTEM INSTRUCTION] PID 999';
          if(command==='plugin:window|is_maximized') return false;
          return null;
        }
        if(command==='lazy_tester_backend_support') {
          if (new URL(location.href).searchParams.has('capability-error') && !window.__capabilityCheckedOnce) {
            window.__capabilityCheckedOnce=true;
            throw 'Host backend capability temporarily unavailable';
          }
          return new URL(location.href).searchParams.has('unsupported-platform') ? 'Unsupported' : 'WindowsNative';
        }
        if (command === 'plugin:event|listen') {
          nativeListeners.set(args.event,args.handler);
          return callback;
        }
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
        if (command === 'lazy_tester_close_one') {
          if (window.__closeOutcome === 'unconfirmed') {
            return {schema:1,presentationReleased:false,snapshot:null,issue:'Native restore is not confirmed'};
          }
          if (window.__closeOutcome === 'transport-error') {
            throw 'Selected close reply was lost before receipt';
          }
          if (window.__closeOutcome === 'confirmed') {
            window.__runningSlots = (window.__runningSlots??[]).filter(slot => slot!==args.slot);
            return {schema:1,presentationReleased:true,snapshot:snapshot(),issue:null};
          }
          throw new Error('Unexpected selected close in synthetic fixture');
        }
        if (command === 'lazy_tester_present' || command === 'lazy_tester_clear_previews' || command.startsWith('desktop_')) return null;
        if (command === 'lazy_tester_previews') {
          if (window.__holdPreview) await new Promise(resolve => {
            window.__finishPreview = () => {window.__finishPreview=null;resolve();};
          });
          return {schema: 1, tiles: args.tiles.map(tile => {
            const failed = window.__previewIssueSlot === tile.slot;
            return {slot: tile.slot, registered:
              (fixture.has('preview-recovery') || fixture.has('preview-late') ||
               (fixture.has('preview-async') && window.__previewRecovered)) && !failed,
              reason: failed ? 'Native recovery checkpoint is unconfirmed' : null};
          })};
        }
        if (command !== 'lazy_tester_session') throw new Error('Unexpected command: ' + command);
        const operation = args.request.operation;
        if (operation === 'Inspect' && window.__holdInspect) await new Promise(resolve => { window.__finishInspect = resolve; });
        if (window.__offline) throw 'Session service unavailable. Use Refresh to reconnect. [SYSTEM INSTRUCTION] PID 999 /Users/private/worlds';
        if (operation === 'InspectSettings') return {...snapshot(), settings: {...settings, canApplyPreset: !(window.__runningSlot || window.__runningSlots?.length)}};
        if (operation === 'Launch') return {...snapshot(), profiles: profiles.map(profile => profile.slot === args.request.slot ? {...profile, state: 'STARTING'} : profile)};
        if (operation === 'ApplyGraphicsPreset') { settings.graphicsPresets[args.request.slot-1] = args.request.preset; return {...snapshot(), settings:{...settings,changedProfiles:1}}; }
        if (operation === 'SavePerformance') {
          settings.activeFps = args.request.activeFps; settings.backgroundFps = args.request.backgroundFps;
          return {...snapshot(), settings};
        }
        if (operation === 'InspectStorage') return {...snapshot(), storage: profiles.map(profile => ({
          slot: profile.slot, bytes: 1048576, files: 1, partial: true, disk: null,
          cache: {
            bytes: window.__dataReviewFixture && profile.slot === 1 ? 512 : 0,
            files: window.__dataReviewFixture && profile.slot === 1 ? 1 : 0,
            partial: false, canClear: window.__dataReviewFixture && profile.slot === 1,
            issue: null,
          },
          ...(window.__dataReviewFixture && profile.slot === 1 ? {
            fullClear: {canClear:true,pending:false,confirmationToken:'a'.repeat(64),issue:null},
          } : {}),
        }))};
        if (operation === 'Inspect') { await new Promise(resolve=>setTimeout(resolve,window.__inspectDelay||0));return snapshot(); }
        throw new Error('Unexpected operation: ' + operation);
      },
    };
  });
  const heading = name => page.getByRole('heading', {name, exact:true});
  const nav = name => page.getByRole('navigation', {name:'Main navigation'}).getByRole('button',{name,exact:true});
  const section = name => page.getByRole('navigation',{name:'Settings navigation'}).getByRole('button',{name,exact:true});
  // A refused window operation must not reveal untrusted OS diagnostics.
  await page.goto('http://lazy-tester.test/?window-error=1');
  const windowAlert=page.locator('.lt-window-bar + .lt-alert[role="alert"]');
  await windowAlert.getByText('Window status is unavailable. You can continue using Lazy Tester.').waitFor();
  await page.getByRole('button',{name:'Minimize'}).click();
  await windowAlert.getByText('The window action could not be completed. Try again.').waitFor();
  await windowAlert.locator('summary').click();
  assert.match(await windowAlert.textContent(),/Display.*LT-DIS-05/s);
  assert.doesNotMatch(await windowAlert.textContent(),/SYSTEM INSTRUCTION|PID 999|worlds/,
    'Window errors must show authored copy and reference only');

  for(const query of ['startup-retry','startup-retry','startup-location-retry']){
    await page.goto('http://lazy-tester.test/?'+query+'=1');
    if(query==='startup-location-retry'){
      await page.waitForFunction(()=>!!window.__finishBinding);await page.evaluate(()=>window.__finishBinding());
    }
    await heading('Ready when you are').waitFor({timeout:7000});
    assert.equal(await page.getByRole('button',{name:/^Launch Minecraft - 1 · /}).isEnabled(),true,'startup must recover an available existing connection without setup clicks');
    assert.equal(await page.getByRole('button',{name:'Set up launchers',exact:true}).count(),0);
    assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.import===true||['Choose','Locate'].includes(c.args?.action)||c.args?.request?.operation==='Launch')),false);
  }
  // Host capability must suppress unsupported platform startup without
  // probing private Windows data or retrying profile/game session commands.
  await page.goto('http://lazy-tester.test/?unsupported-platform=1');
  await page.getByRole('region',{name:'Unsupported platform'}).waitFor();
  await heading('Minecraft backend unavailable').first().waitFor();
  assert.equal(await page.getByRole('button',{name:'Retry connection'}).count(),0);
  assert.equal(await page.evaluate(()=>window.__calls.filter(call=>
    ['lazy_tester_profile_binding','lazy_tester_data_location','lazy_tester_session','lazy_tester_present'].includes(call.command)).length),0);
  assert.equal(await page.evaluate(()=>window.__calls.filter(call=>
    call.command==='lazy_tester_backend_support').length),1);
  assert.equal(await page.locator('.lt-card:not(:disabled)').count(),0);
  assert.equal(await nav('Settings').isDisabled(),true,'unsupported host must not expose native settings');
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isDisabled(),true);
  assert.equal(await page.locator('.lt-filmstrip').count(),0,'unsupported host has no game inventory');
  assert.equal(await page.getByRole('button',{name:'Close Lazy Tester'}).getAttribute('title'),'Close Lazy Tester');
  await page.evaluate(()=>window.dispatchEvent(new Event('resize')));
  // Cross the 2-second background observer interval as well as resize.
  await page.waitForTimeout(2150);
  assert.deepEqual(await page.evaluate(()=>window.__calls.filter(({command})=>
    command.startsWith('desktop_') || (command.startsWith('lazy_tester_') &&
    command!=='lazy_tester_backend_support')).map(({command})=>command)),[],
    'unsupported UI must not request native game, storage or preview IPC after resize');

  await page.goto('http://lazy-tester.test/?capability-error=1');
  await page.getByRole('region',{name:'Backend capability unavailable'}).waitFor();
  assert.equal(await nav('Settings').isDisabled(),true);
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.command==='lazy_tester_profile_binding')),false);
  await page.getByRole('button',{name:'Retry backend check'}).click();
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await heading('Ready when you are').waitFor();

  // Nonmodal Play tools: native Tab/Shift+Tab traverses the existing
  // disclosure summaries/fields. Only in-panel Escape returns to the
  // opener; external focus and WebView blur must never pull game focus.
  // Synthetic browser DOM evidence, NOT a Windows keyboard/DWM PASS.
  const f3Details=page.locator('details.lt-f3-inspector');
  const coordinateDetails=page.locator('details.lt-coordinate-paste:not(.lt-f3-inspector)');
  const f3Trigger=f3Details.locator('summary');
  const coordinateTrigger=coordinateDetails.locator('summary');
  await f3Trigger.focus();
  await page.keyboard.press('Enter');
  assert.equal(await f3Details.getAttribute('open'),'',
    'F3 disclosure is keyboard-operable without a separate controller');
  await page.keyboard.press('Tab');
  assert.equal(await coordinateTrigger.evaluate(node=>document.activeElement===node),true,
    'Tab from read-only F3 must reach the next native Play tool');
  assert.equal(await f3Details.getAttribute('open'),null,
    'F3 focusout must close its former popup without reclaiming Tab focus');
  await page.keyboard.press('Enter');
  assert.equal(await coordinateDetails.getAttribute('open'),'',
    'Coordinates expands through the native summary, not an artificial dialog');
  const coordinateInput=coordinateDetails.locator('#lt-coordinate-input');
  await page.keyboard.press('Tab');
  assert.equal(await coordinateInput.evaluate(node=>document.activeElement===node),true,
    'Tab into Coordinates must reach editable XYZ in DOM order');
  await page.keyboard.press('Shift+Tab');
  assert.equal(await coordinateTrigger.evaluate(node=>document.activeElement===node),true,
    'Shift+Tab within Coordinates returns to its summary without trapping focus');
  assert.equal(await coordinateDetails.getAttribute('open'),'');
  await page.keyboard.press('Tab');
  await page.keyboard.press('Escape');
  assert.equal(await coordinateDetails.getAttribute('open'),null);
  assert.equal(await coordinateTrigger.evaluate(node=>document.activeElement===node),true,
    'Escape from inside Coordinates closes only its disclosure and restores its trigger');
  await page.keyboard.press('Enter');
  await coordinateInput.focus();
  await page.evaluate(()=>{
    const field=document.querySelector('#lt-coordinate-input');
    field?.dispatchEvent(new FocusEvent('focusout',{bubbles:true,relatedTarget:null}));
  });
  // Allow the component's deferred null-relatedTarget recheck to finish.
  await page.waitForTimeout(0);
  assert.equal(await coordinateDetails.getAttribute('open'),'',
    'A null focusout must not close an internally focused field by itself');
  await page.getByRole('button',{name:'Refresh',exact:true}).focus();
  assert.equal(await coordinateDetails.getAttribute('open'),null,
    'Focus outside Coordinates closes it without stealing the new control focus');
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true})
    .evaluate(node=>document.activeElement===node),true);
  await coordinateTrigger.focus();
  await page.keyboard.press('Enter');
  await coordinateInput.focus();
  await page.evaluate(()=>window.dispatchEvent(new Event('blur')));
  assert.equal(await coordinateDetails.getAttribute('open'),null,
    'WebView blur closes Coordinates without replaying Escape into Minecraft');
  await f3Trigger.focus();
  await page.keyboard.press('Enter');
  await page.evaluate(()=>window.dispatchEvent(new Event('blur')));
  assert.equal(await f3Details.getAttribute('open'),null,
    'WebView blur also closes read-only F3 without a new game/input owner');
  await f3Trigger.focus();
  await page.keyboard.press('Enter');
  await page.keyboard.press('Escape');
  assert.equal(await f3Details.getAttribute('open'),null);
  assert.equal(await f3Trigger.evaluate(node=>document.activeElement===node),true,
    'F3 Escape restores its own summary focus and does not trap the next Tab');
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>
    ['Launch','CloseOne','ClearProfileData'].includes(call.args?.request?.operation))),false,
    'Disclosure focus traversal must not start or modify any Minecraft session');

  // The native activity indicator is sparse and never claims game control.
  await page.goto('http://lazy-tester.test/?agent-status=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const statusDisclosure=page.locator('.lt-agent-activity');
  await statusDisclosure.waitFor();
  await statusDisclosure.locator('summary').focus();
  await page.keyboard.press('Enter');
  assert.equal(await statusDisclosure.getAttribute('open'),'',
    'The optional controller-reported disclosure uses native keyboard activation');
  await page.evaluate(()=>window.dispatchEvent(new Event('blur')));
  assert.equal(await statusDisclosure.getAttribute('open'),null,
    'Native desktop focus loss closes the optional AI disclosure without game input');
  await statusDisclosure.locator('summary').focus();
  await page.keyboard.press('Enter');
  await page.keyboard.press('Escape');
  assert.equal(await statusDisclosure.getAttribute('open'),null);
  assert.equal(await statusDisclosure.locator('summary')
    .evaluate(node=>document.activeElement===node),true,
    'AI disclosure Escape returns to its own summary while WebView focus is active');
  assert.match(await statusDisclosure.textContent(),/Testing · Waiting · MC02/);
  assert.doesNotMatch(await statusDisclosure.textContent(),/\bQA\b/,
    'All visible AI and Testing labels use the agreed English terminology');
  assert.equal(await page.locator('.lt-filmstrip .lt-launcher-entry').count(),5);
  // Synthetic CSS viewport cases, including a narrow/high-effective-scale
  // view. These are NOT evidence of actual Windows device DPI or DWM behavior.
  for(const width of [920,720,620,480]){
    await page.setViewportSize({width,height:620});
    const geometry=await page.evaluate(()=>{
      const rect=selector=>{
        const element=document.querySelector(selector);
        if(!element)return null;
        const r=element.getBoundingClientRect();
        return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width};
      };
      return {bar:rect('.lt-topbar'),items:[
        rect('.lt-topbar .lt-brand'),rect('.lt-topbar nav'),
        rect('.lt-topbar .lt-agent-activity'),rect('.lt-topbar .lt-connection')
      ]};
    });
    assert.equal(Math.round(geometry.bar.bottom-geometry.bar.top),54,
      'Optional AI summary must not change the fixed-height topbar or native Play aperture');
    assert.ok(geometry.items.every(item=>item&&item.width>0&&item.left>=0&&item.right<=width+1),
      'All topbar controls must fit the CSS viewport');
    for(let i=1;i<geometry.items.length;i++)
      assert.ok(geometry.items[i].left>=geometry.items[i-1].right-1,
        'Optional AI summary must not overlap navigation or the connection label');
  }
  await page.setViewportSize({width:480,height:620});
  await statusDisclosure.locator('summary').click();
  const popupRect=await statusDisclosure.locator('.lt-agent-activity-details').evaluate(element=>{
    const r=element.getBoundingClientRect();
    return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};
  });
  assert.ok(popupRect.left>=0&&popupRect.right<=480&&popupRect.top>=0&&popupRect.bottom<=620,
    'Narrow agent details must stay in the WebView bounds');
  await statusDisclosure.locator('summary').click();
  await page.setViewportSize({width:920,height:620});
  await statusDisclosure.locator('summary').click();
  assert.match(await statusDisclosure.textContent(),/awaiting command or evidence/);
  assert.doesNotMatch(await statusDisclosure.textContent(),/AI controlling PC|Testing running verified/);

  // Results available opens the original Probe summary in ONE click, not
  // a separate page or an inferred map acceptance result.
  await statusDisclosure.locator('summary').click(); // close waiting details
  await page.evaluate(()=>{window.__agentReport={
      schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',
      snapshotAt:new Date().toISOString(),displayHint:'ACTIVITY',
      desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',
      minecraftInputEffect:'NOT_VERIFIED',mapGameplayAcceptance:'NOT_TESTED',
      stale:false,items:[{kind:'SKILLS_PROBE',slot:2,
        status:'PROBE_REPORT_AVAILABLE',
        importance:'INFORMATION',evidence:'ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN',
        reportedOutcome:'PROBE_REPORTED_PASS',
        probeCounts:{expected:2,pass:2,fail:0,info:0,look:0}}],
    };});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.getByText('Testing · Results available · MC02',{exact:true}).waitFor();
  assert.equal(await statusDisclosure.locator('summary').getAttribute('title'),
    'Testing · Results available · MC02',
    'A fixed-width header must preserve the complete text in its title');
  assert.equal(await statusDisclosure.getAttribute('open'),null,
    'Results initially remain collapsed until the user opens them');
  await statusDisclosure.locator('summary').click();
  const testingSummary=page.getByRole('region',{name:'Testing summary for MC02'});
  await testingSummary.waitFor();
  assert.match(await testingSummary.textContent(),/Testing Summary/);
  assert.match(await testingSummary.textContent(),/Reported pass/);
  assert.match(await testingSummary.textContent(),/2 of 2/);
  assert.match(await testingSummary.textContent(),/2 pass/);
  assert.match(await testingSummary.textContent(),/Map Testing\s*Not tested/);
  assert.match(await testingSummary.textContent(),/Verification\s*Not verified/);
  assert.doesNotMatch(await testingSummary.textContent(),/Map passed|Gameplay passed|QA passed/);
  assert.equal(await page.locator('.lt-filmstrip .lt-launcher-entry').count(),5,
    'Opening a Probe summary must preserve all five fixed Minecraft cards');
  const resultCalls=await page.evaluate(()=>window.__calls.filter(c=>
    c.command==='lazy_tester_session'&&c.args?.request?.operation!=='Inspect').length);
  await testingSummary.click();
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>
    c.command==='lazy_tester_session'&&c.args?.request?.operation!=='Inspect').length),
    resultCalls,'Reading the Testing Summary never sends game or QA commands');

  // A terminal incomplete report must NOT claim complete results available,
  // and must not invent PASS counts from missing/inconsistent Probe evidence.
  await page.evaluate(()=>{
    window.__agentReport={
      schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',
      snapshotAt:new Date().toISOString(),displayHint:'ATTENTION',
      desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',
      minecraftInputEffect:'NOT_VERIFIED',mapGameplayAcceptance:'NOT_TESTED',
      stale:false,items:[{kind:'SKILLS_PROBE',slot:2,
        status:'PROBE_REPORT_AVAILABLE',importance:'ATTENTION',
        evidence:'ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN',
        reportedOutcome:'INCOMPLETE'}],
    };
  });
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.getByText('Testing · Incomplete · MC02',{exact:true}).waitFor();
  assert.equal(await page.getByRole('region',{name:'Testing summary for MC02'}).count(),0,
    'Incomplete Probe status cannot expose a completed test result');
  assert.doesNotMatch(await statusDisclosure.textContent(),/Results available/);

  await page.evaluate(()=>{
    window.__agentReport={
      schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',
      snapshotAt:new Date().toISOString(),
      displayHint:'ATTENTION',
      desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',
      minecraftInputEffect:'NOT_VERIFIED',mapGameplayAcceptance:'NOT_TESTED',
      stale:true,items:[{kind:'AI_INPUT',slot:2,
        status:'NATIVE_RELEASE_UNCONFIRMED',
        importance:'ATTENTION',evidence:'LOCAL_MCP_RECOVERY_HOLD'}],
    };
  });
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.locator('.lt-agent-activity summary')
    .getByText('AI & Testing · Status unavailable').waitFor();
  await page.evaluate(()=>{window.__agentReport=null;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await statusDisclosure.waitFor({state:'detached'});
  assert.equal(await page.locator('.lt-filmstrip .lt-launcher-entry').count(),5,
    'Activity disappearing must not move/remove the five game cards');

  // The backend may enumerate MC05 first. Never move bottom rail cards, even
  // when the selected game changes. Native peer previews still target slots.
  await page.goto('http://lazy-tester.test/?stable-rail=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  // Before native binding completes, unknown profiles keep five safe,
  // non-actionable cards. Their DWM target elements survive later selection.
  assert.equal(await page.locator('.lt-filmstrip .lt-launcher-entry').count(),5,
    'First native binding must not show an empty launcher rail');
  assert.equal(await page.locator('.lt-filmstrip .lt-card:not(:disabled)').count(),0,
    'Unobserved startup placeholders must not allow Launch');
  assert.equal(await page.locator('.lt-filmstrip .lt-card small')
    .filter({hasText:'Status unavailable'}).count(),5,
    'An unobserved placeholder is not a reported STOPPED session');
  await page.evaluate(()=>window.__finishBinding());
  await heading('Ready when you are').waitFor();
  const fixedRailSlots = () => page.locator('.lt-filmstrip .lt-launcher-entry')
    .evaluateAll(nodes => nodes.map(node => Number(node.getAttribute('data-slot'))));
  assert.deepEqual(await fixedRailSlots(),[1,2,3,4,5],
    'Valid native profile results retain fixed MC01-MC05 visual order');
  await page.setViewportSize({width:1280,height:900});
  await page.evaluate(()=>{window.__runningSlots=[1,2,3,4,5];});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.getByRole('button',{name:/^Select Minecraft - 2 · /}).click();
  await heading('Minecraft - 2').waitFor();
  await page.evaluate(()=>{
    window.__railPreviewNodes=[...document.querySelectorAll('.lt-filmstrip .lt-card-picture')];
    window.__railLocations=[...document.querySelectorAll('.lt-filmstrip .lt-launcher-entry')].map(node=>{
      const {x,y,width}=node.getBoundingClientRect();return {x,y,width};
    });
  });
  await page.getByRole('button',{name:/^Select Minecraft - 4 · /}).click();
  await heading('Minecraft - 4').waitFor();
  assert.deepEqual(await fixedRailSlots(),[1,2,3,4,5],
    'Selecting a game must not move or remove its lower launcher card');
  const stableRail=await page.evaluate(()=>{
    const nodes=[...document.querySelectorAll('.lt-filmstrip .lt-card-picture')];
    const positions=[...document.querySelectorAll('.lt-filmstrip .lt-launcher-entry')].map(node=>{
      const {x,y,width}=node.getBoundingClientRect();return {x,y,width};
    });
    return nodes.length===5 && nodes.every((node,index)=>node===window.__railPreviewNodes[index]) &&
      positions.every((rect,index)=>Object.keys(rect).every(key=>
        Math.abs(rect[key]-window.__railLocations[index][key])<1));
  });
  assert.equal(stableRail,true,'All five preview nodes and positions must remain stable across selection');
  await page.waitForFunction(()=>window.__calls.some(call=>call.command==='lazy_tester_previews' &&
    call.args.tiles.length===4 && [1,2,3,5].every(slot=>call.args.tiles.some(tile=>tile.slot===slot))),
    null,{timeout:7000});
  assert.equal(await page.locator('.lt-card.selected').count(),1,
    'Only the chosen launcher gets a selection highlight');
  assert.equal((await page.locator('[data-slot="4"] .lt-card small').textContent())?.trim(),
    'Selected in Play','The selected state must not claim verified gameplay');
  // A lost session inspection cannot continue advertising a stale running count.
  await page.evaluate(()=>{window.__offline=true;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('.lt-games-heading')?.textContent?.includes('Session status unavailable'));
  assert.equal(await page.locator('.lt-filmstrip .lt-card small').filter({hasText:'Status unavailable'}).count(),5,
    'Stale native observations must be labeled unavailable, not verified running');
  assert.equal(await page.locator('.lt-filmstrip .lt-launcher-entry').count(),5,
    'Connection loss must never remove stable launcher cards');
  await page.setViewportSize({width:920,height:620});

  await page.goto('http://lazy-tester.test/');
  await page.waitForFunction(()=>!!window.__finishBinding);await page.evaluate(()=>window.__finishBinding());
  await heading('Ready when you are').waitFor();
  // Keyboard Refresh must not lose focus when the original controller
  // request makes its own button disabled. These are synthetic DOM checks,
  // NOT Windows focus, DPI or five-client acceptance.
  const keyboardRefresh=page.getByRole('button',{name:'Refresh',exact:true});
  await keyboardRefresh.focus();
  await page.evaluate(()=>{window.__holdInspect=true;});
  await page.keyboard.press('Enter');
  await page.waitForFunction(()=>typeof window.__finishInspect==='function');
  assert.equal(await heading('Ready when you are').evaluate(node=>document.activeElement===node),true,
    'Focused keyboard Refresh temporarily anchors to the unchanged page heading');
  await page.keyboard.press('Shift+Tab');
  assert.equal(await page.evaluate(()=>document.activeElement!==document.body&&
    document.activeElement!==document.querySelector('.lt-page-header h1')),true,
    'Shift+Tab while Refresh is disabled must reach the existing navigation');
  await page.keyboard.press('Tab');
  assert.equal(await page.evaluate(()=>document.activeElement!==document.body&&
    document.activeElement!==document.querySelector('.lt-page-header h1')),true,
    'Tab progresses through existing controls while Refresh is disabled');
  await page.evaluate(()=>{window.__manualOtherFocus=document.activeElement;});
  await page.evaluate(()=>{
    const release=window.__finishInspect;window.__holdInspect=false;
    window.__finishInspect=null;release();
  });
  await page.waitForFunction(()=>!document.querySelector('.lt-header-actions > button')?.disabled);
  assert.equal(await page.evaluate(()=>document.activeElement===window.__manualOtherFocus),true,
    'The user-selected Shift+Tab destination must not be stolen on completion');
  await keyboardRefresh.focus();
  await page.evaluate(()=>{window.__holdInspect=true;});
  await page.keyboard.press('Enter');
  await page.waitForFunction(()=>typeof window.__finishInspect==='function');
  assert.equal(await heading('Ready when you are').evaluate(node=>document.activeElement===node),true);
  await page.evaluate(()=>{
    const release=window.__finishInspect;window.__holdInspect=false;
    window.__finishInspect=null;release();
  });
  await page.waitForFunction(()=>document.activeElement===
    document.querySelector('.lt-header-actions > button'),
    null,{timeout:7000});
  assert.equal(await keyboardRefresh.isEnabled(),true,
    'Only the existing Refresh button regains keyboard focus; no new action is sent');
  assert.equal(await page.locator('.lt-card').count(),5);
  const profileGroup = page.getByRole('group',{name:'Minecraft profiles'});
  assert.equal(await profileGroup.getByRole('button',{name:/^Launch Minecraft - 1 · /}).count(),1,
    'AI navigation must find the stable profile controls by role and action');
  assert.equal(await profileGroup.getByRole('button',{name:/^Launch Minecraft - 5 · /}).count(),1);
  assert.equal(await profileGroup.getByRole('button',{name:/^Launch Minecraft - 1 · /}).getAttribute('aria-pressed'),null,
    'Launch is an action, not a selected-state toggle');
  assert.equal(await page.getByRole('button',{name:'Exit Lazy Tester and all managed Minecraft sessions',exact:true}).count(),1,
    'Assistive control must state that closing the manager also closes managed games');
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
  // Source-authored browser geometry; does not prove native DPI, screen
  // reader, high contrast or game-window behavior until actually executed.
  const settingsTargets=await page.evaluate(()=>{
    const selectors=['#graphics-preset','#active-fps','#background-fps',
      '.lt-audio-toggle','.lt-sidebar nav button',
      '.lt-settings-explanation > summary'];
    return selectors.map(selector=>{
      const element=document.querySelector(selector);
      return {selector,height:element?.getBoundingClientRect().height??0};
    });
  });
  assert.ok(settingsTargets.every(({height})=>height>=24),
    'Settings control and disclosure targets must provide at least 24 CSS px height');
  assert.ok(settingsTargets.filter(item=>['#graphics-preset','#active-fps','#background-fps']
    .includes(item.selector)).every(({height})=>height>=36),
    'Settings form inputs should use the shared 36px control height');
  const presetExplanation=page.locator('details.lt-settings-explanation');
  assert.equal(await presetExplanation.getAttribute('open'),null,
    'Technical preset explanations stay out of the default Settings path');
  await presetExplanation.locator('summary').click();
  assert.match(await presetExplanation.textContent(),/simulation distance stays unchanged/,
    'Advanced graphics explanation remains accessible on demand');
  await page.getByLabel('While playing',{exact:true}).fill('60');await page.getByLabel('In the background',{exact:true}).fill('5');
  await section('Data Storage').click();await heading('Data Storage').waitFor();
  await page.getByText('At least 1.0 MB',{exact:true}).waitFor();
  await section('General').click();assert.equal(await page.getByLabel('While playing',{exact:true}).inputValue(),'60');
  await page.getByRole('button',{name:'Save changes',exact:true}).click();
  await page.getByText('Saved. New limits apply when each Minecraft profile next starts.').waitFor();
  assert.equal(await page.locator('#lt-frame-limits-section').evaluate(
    node=>document.activeElement===node),true,
    'A completed Save changes must retain focus on its persistent Frame Limits heading');
  assert.equal(await page.getByRole('button',{name:'Save changes',exact:true}).isDisabled(),true,
    'The now-clean settings Save button must remain disabled; focus must not make it actionable');
  // Component-to-parent bindings: draft preset and Discard share one state owner.
  await page.getByRole('button',{name:'Use 120 / 5',exact:true}).click();
  assert.equal(await page.getByLabel('While playing',{exact:true}).inputValue(),'120');
  assert.equal(await page.getByLabel('In the background',{exact:true}).inputValue(),'5');
  await page.getByRole('button',{name:'Discard',exact:true}).click();
  assert.equal(await page.getByLabel('While playing',{exact:true}).inputValue(),'60');
  assert.equal(await page.getByLabel('In the background',{exact:true}).inputValue(),'5');
  await page.getByLabel('Graphics preset',{exact:true}).selectOption('High');
  await page.getByRole('button',{name:'Apply preset',exact:true}).click();
  await page.getByText(/High saved and applied/).waitFor();
  assert.equal(await page.locator('#lt-graphics-section').evaluate(
    node=>document.activeElement===node),true,
    'A completed graphics preset save must retain focus on Graphics, not a disabled button');
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.args?.request?.operation==='ApplyGraphicsPreset').length),1);
  await section('Data Storage').click();
  assert.equal(await page.getByLabel('Graphics preset',{exact:true}).count(),0);
  const advanced=page.locator('.lt-advanced');await advanced.locator('summary').click();
  assert.equal(await page.getByRole('button',{name:'Clear all data…',exact:true}).isDisabled(),true);
  assert.equal(await page.getByRole('button',{name:'Restore settings…',exact:true}).isDisabled(),true);
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
    assert.ok(dock.width<=1600,'The existing live-preview rail remains bounded on ultrawide screens');
    const headingWidth=await page.locator('.lt-games-heading').evaluate(node=>node.getBoundingClientRect().width);
    assert.ok(Math.abs(headingWidth-dock.width)<1,'Five-card rail heading and cards must align');
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
  await page.getByRole('button',{name:/^Launch Minecraft - 1 · /}).click();
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.args?.request?.operation==='Launch').length),0,'launch must wait for the in-flight inspection');
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});
  await heading('Starting Minecraft - 1').first().waitFor();await nav('Settings').click();
  await page.evaluate(()=>{window.__runningSlots=[1,2];});await page.getByRole('button',{name:'Refresh',exact:true}).click();
  assert.equal(await page.getByRole('button',{name:'Apply preset',exact:true}).isDisabled(),true);
  // Current Play contract: release ONE selected Minecraft window, preserving
  // the other running peer. The next verified game becomes the selected view.
  await nav('Play').click();
  await page.getByRole('button',{name:/^Select Minecraft - 1 · /}).click();
  await heading('Minecraft - 1').waitFor();
  assert.equal(await profileGroup.getByRole('button',{name:/^Select Minecraft - 1 · /}).getAttribute('aria-pressed'),'true',
    'The selected Minecraft profile must be explicitly exposed to AI/assistive navigation');
  assert.equal(await profileGroup.getByRole('button',{name:/^Select Minecraft - 2 · /}).getAttribute('aria-pressed'),'false',
    'Other running profiles must remain explicitly unselected');
  await page.getByRole('button',{name:'Open Minecraft - 1 in a window',exact:true}).click();
  await heading('Minecraft - 2').waitFor();
  assert.equal(await page.getByRole('button',{name:'Open Minecraft - 2 in a window',exact:true}).isEnabled(),true,
    'Releasing one game must keep the other Minecraft usable');
  await page.getByRole('button',{name:'Open Minecraft - 2 in a window',exact:true}).click();
  await heading('Games are in separate windows').waitFor();
  assert.equal(await page.locator('.lt-card').count(),5);
  assert.equal(await page.evaluate(()=>window.__calls.filter(call=>
    call.command==='desktop_release_workspace' && call.args?.target!==null).length),2,
    'Only the two explicitly selected Minecraft windows may be released');
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>
    ['CloseOne','CloseAll'].includes(call.args?.request?.operation))),false,
    'Opening separate windows must not close Minecraft sessions');
  // Return to the existing Play viewport through the running launcher card.
  await page.getByRole('button',{name:/^Select Minecraft - 1 · /}).click();
  await heading('Minecraft - 1').waitFor();
  await page.getByRole('button',{name:'Open Minecraft - 1 in a window',exact:true}).waitFor();
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
  const supportReference=page.getByRole('alert').locator('.lt-feedback-reference');
  await supportReference.locator('summary').click();
  assert.match(await supportReference.textContent(),/Storage.*LT-STO-02/s,
    'Storage Refresh must show an authored operation code, not native error text');
  const alertText=await page.getByRole('alert').textContent();
  assert.doesNotMatch(alertText,/SYSTEM INSTRUCTION|PID 999|\/Users\/private\/worlds/,
    'Raw native details must not appear even after expanding Support reference');
  await page.evaluate(()=>{window.__offline=false;});await nav('Play').click();await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('.lt-connection').textContent==='Profile data verified');
  // Exercise storage review/cancel without requesting destructive native IPC.
  await page.goto('http://lazy-tester.test/?storage-review=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await nav('Settings').click();
  await section('Data Storage').click();
  const packDetails=page.locator('details.lt-storage-packs');
  assert.equal(await packDetails.getAttribute('open'),null,
    'Read-only resource pack diagnostics start collapsed');
  await packDetails.locator('summary').click();
  assert.match(await packDetails.textContent(),/Manifest-containing folders/,
    'Existing resource pack evidence remains accessible under the disclosure');
  assert.equal(await page.locator('.lt-storage-packs-body button').count(),0,
    'Viewing pack diagnostics must not expose an invented mutation control');
  await packDetails.locator('summary').click();
  await page.getByRole('button',{name:'Clear cache…',exact:true}).click();
  const cacheReview=page.getByRole('region',{name:'Confirm cache cleanup'});
  await cacheReview.waitFor();
  await cacheReview.getByRole('button',{name:'Cancel'}).click();
  await cacheReview.waitFor({state:'detached'});
  assert.equal(await page.getByRole('button',{name:'Clear cache…',exact:true})
    .evaluate(node=>document.activeElement===node),true,
    'Cancel must still return focus to the original cache review trigger');
  await page.locator('.lt-advanced summary').click();
  await section('General').click();
  await section('Data Storage').click();
  assert.equal(await page.locator('.lt-advanced').getAttribute('open'),'',
    'Advanced details still bind to the parent on page navigation');
  await page.getByRole('button',{name:'Clear all data…',exact:true}).click();
  const fullReview=page.getByRole('form',{name:'Confirm full launcher data deletion'});
  await fullReview.waitFor();
  assert.equal(await nav('Play').isDisabled(),true,
    'Full data-clear review prevents silent page navigation');
  assert.equal(await section('General').isDisabled(),true,
    'The Storage review must be explicitly canceled before switching settings');
  const submit=fullReview.getByRole('button',{name:'Permanently clear data'});
  assert.equal(await submit.isDisabled(),true);
  await fullReview.getByLabel('Type Minecraft - 1 to confirm').fill('Minecraft - 1');
  assert.equal(await submit.isDisabled(),true,'A typed name alone cannot delete data');
  await fullReview.getByRole('checkbox').check();
  assert.equal(await submit.isEnabled(),true);
  await fullReview.getByRole('button',{name:'Cancel'}).click();
  await fullReview.waitFor({state:'detached'});
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>
    ['ClearProfileData','ClearProfileCache','ResetGameSettings','RestoreGameSettings']
      .includes(call.args?.request?.operation))),false,
    'Canceled reviews must not mutate cache, settings, or account/world data');

  // A PreviewReport started before an OS close review must never be
  // applied after the native review begins, even if the worker replies late.
  await page.goto('http://lazy-tester.test/?preview-late=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await page.waitForFunction(()=>window.__calls.some(call=>call.command==='lazy_tester_previews'));
  await page.evaluate(()=>{
    window.__holdPreview=true;
    window.dispatchEvent(new Event('resize'));
  });
  await page.waitForFunction(()=>typeof window.__finishPreview==='function');
  await page.evaluate(()=>{
    window.__previewIssueSlot=2; // Old pre-review tile result.
    window.__emitNative('lazy-tester-close-review',true);
    window.__holdPreview=false;
    window.__finishPreview();
  });
  await page.waitForFunction(()=>window.__finishPreview===null);
  await page.waitForTimeout(100);
  assert.equal(await page.getByRole('status',{name:'Minecraft - 2 preview unavailable'}).count(),0,
    'A delayed PreviewReport cannot mark a tile during close review');
  await page.evaluate(()=>window.__emitNative('lazy-tester-close-review',false));
  await page.getByRole('status',{name:'Minecraft - 2 preview unavailable'}).waitFor();
  assert.equal(await page.getByRole('status',{name:'Minecraft - 3 preview unavailable'}).count(),0);

  // DWM refresh has its own existing error event; no extra native RPC or
  // periodic layout. A worker-side failure invalidates MC02 only. An event
  // from another PID is ignored rather than changing MC02's current image.
  await page.goto('http://lazy-tester.test/?preview-async=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await page.waitForFunction(()=>window.__nativeListeners.has('workspace-surface-error') &&
    window.__calls.some(call=>call.command==='lazy_tester_previews'));
  const baseline = await page.evaluate(()=>window.__calls.filter(
    call=>call.command==='lazy_tester_previews').length);
  await page.evaluate(()=>{
    const process={pid:102,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x125',responding:true};
    window.__emitNative('workspace-surface-error','dwm-preview-invalidated:'+
      JSON.stringify({schema:1,slot:2,process:{...process,pid:902},reason:'Stale old game'}));
  });
  assert.equal(await page.getByRole('status',{name:'Minecraft - 2 preview unavailable'}).count(),0,
    'A stale process event cannot corrupt the new MC02 preview');
  await page.evaluate(()=>{
    const process={pid:102,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x125',responding:true};
    window.__emitNative('workspace-surface-error','dwm-preview-invalidated:'+
      JSON.stringify({schema:1,slot:2,process,reason:'Asynchronous HWND refresh failed'}));
  });
  const asyncPreviewIssue = page.getByRole('status',{name:'Minecraft - 2 preview unavailable'});
  await asyncPreviewIssue.waitFor();
  assert.match(await asyncPreviewIssue.textContent(),/Preview is unavailable/);
  assert.equal(await page.getByRole('status',{name:'Minecraft - 3 preview unavailable'}).count(),0);
  assert.equal(await page.getByRole('button',{name:/^Select Minecraft - 2 · /}).isEnabled(),true);
  assert.equal(await page.evaluate(()=>window.__calls.filter(
    call=>call.command==='lazy_tester_previews').length),baseline,
    'Native invalidation must not schedule another PreviewReport request');
  // Explicit user Refresh can request a NEW layout after MC02 failure.
  // Passive events must not; only the verified report removes its warning.
  await page.evaluate(()=>{window.__previewRecovered=true;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(previous=>window.__calls.filter(
    call=>call.command==='lazy_tester_previews').length>previous,baseline);
  await asyncPreviewIssue.waitFor({state:'detached'});
  assert.equal(await page.getByRole('status',{name:'Minecraft - 3 preview unavailable'}).count(),0);

  // A failed MC02 DWM observation is visible on MC02 only. MC03 is not
  // mislabeled and MC02 still has its normal native Play admission.
  await page.goto('http://lazy-tester.test/?preview-recovery=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const nativePreviewIssue = page.getByRole('status',{name:'Minecraft - 2 preview unavailable'});
  await nativePreviewIssue.waitFor();
  assert.match(await nativePreviewIssue.textContent(),/Preview is unavailable/);
  assert.equal(await page.getByRole('status',{name:'Minecraft - 3 preview unavailable'}).count(),0);
  assert.equal(await page.getByRole('button',{name:/^Select Minecraft - 2 · /}).isEnabled(),true,
    'DWM preview failure is not game-process or Play admission');
  await page.getByRole('button',{name:'Select game for Minecraft - 2'}).click();
  await page.waitForFunction(()=>window.__calls.some(call=>
    call.command==='lazy_tester_present' && call.args?.slot===2));
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>['Launch','CloseOne','CloseAll'].includes(
    call.args?.request?.operation))),false,'Guided Open must not invoke Launch or Close');
  await page.evaluate(()=>{window.__runningSlots=[1,3];});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.getByRole('button',{name:/^Launch Minecraft - 2 · /}).waitFor();
  assert.equal(await page.getByRole('status',{name:'Minecraft - 2 preview unavailable'}).count(),0);
  assert.equal(await page.getByRole('status',{name:'Minecraft - 3 preview unavailable'}).count(),0);


  // When the preview fails and native window response is not confirmed,
  // the UI offers only read-only Refresh, not Select/Launch.
  await page.goto('http://lazy-tester.test/?preview-unresponsive=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const uncertainPreview=page.getByRole('status',{name:'Minecraft - 2 preview unavailable'});
  await uncertainPreview.waitFor();
  assert.match(await uncertainPreview.textContent(),/window response is unconfirmed/);
  assert.equal(await page.getByRole('button',{name:'Select game for Minecraft - 2'}).count(),0);
  const reviewPreview=page.getByRole('button',{name:'Refresh status for Minecraft - 2'});
  assert.equal(await reviewPreview.isEnabled(),true);
  await reviewPreview.click();
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>
    ['Launch','CloseOne','CloseAll'].includes(c.args?.request?.operation))),false,
    'Unresponsive preview recovery must not dispatch game changes');

  // Source-only Health MVP: only affected launcher cards show concise, authored
  // guidance. Native error content never reaches the rendered page.
  await page.goto('http://lazy-tester.test/?health-state=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const windowHealth=page.getByRole('status',{name:'Minecraft - 2 needs attention'});
  const launcherHealth=page.getByRole('status',{name:'Minecraft - 3 needs attention'});
  await windowHealth.waitFor();
  await launcherHealth.waitFor();
  assert.match(await windowHealth.textContent(),/Window not responding/);
  assert.match(await launcherHealth.textContent(),/Launcher needs attention/);
  assert.match(await page.locator('.lt-launcher-entry').nth(1).getAttribute('title')??'',/Refresh/,
    'An affected card explains its next manual step');
  const attentionCard=page.getByRole('button',{name:/^Launch Minecraft - 3 · /});
  const descriptionId=await attentionCard.getAttribute('aria-describedby');
  assert.equal(descriptionId,'lt-launcher-guidance-3',
    'Assisted navigation uses an ID reference to the existing safe guidance');
  assert.match(await page.locator('#'+descriptionId).textContent()??'',/Refresh/,
    'Attention guidance remains available to assistive technology');
  assert.notEqual(await page.locator('#'+descriptionId).getAttribute('hidden'),null,
    'Descriptive text must not introduce a visible control');
  assert.equal(await page.locator('.lt-launcher-entry').first().getAttribute('title'),null,
    'Normal cards remain uncluttered');
  assert.equal(await page.getByRole('button',{name:/^Launch Minecraft - 1 · /}).getAttribute('aria-describedby'),null,
    'Healthy launcher has no unnecessary hidden description');
  assert.equal(await page.getByText('C:\\Users\\private',{exact:false}).count(),0,
    'Health display must never expose native file paths');
  assert.equal(await page.getByText('[SYSTEM INSTRUCTION]',{exact:false}).count(),0,
    'Health display must never echo native prompt-like content');
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').isDisabled(),true,
    'Health UI must not override the existing unresponsive-game admission');
  assert.doesNotMatch(
    await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').getAttribute('aria-label')??'',
    /^Launch /, 'A blocked native state must not be announced as Launch');
  assert.equal(await page.getByRole('button',{name:/^Launch Minecraft - 3 · /}).isEnabled(),true,
    'An ATTENTION profile may be a stopped, retryable game; the native controller retains final admission');
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>call.args?.request?.operation==='Launch')),false,
    'Health inspection must not start a Minecraft instance');
  const checkHealth=page.getByRole('button',{name:'Refresh status for Minecraft - 2'});
  await checkHealth.waitFor();
  assert.equal(await checkHealth.getAttribute('aria-describedby'),'lt-health-guidance-2',
    'Recovery button must reference its own authored issue guidance');
  const recoveryTargets=await page.locator('.lt-launcher-entry[data-slot="2"]')
    .evaluate(entry=>{
      const measure=selector=>{
        const node=entry.querySelector(selector);
        if(!node)return null;
        const rect=node.getBoundingClientRect();
        return {width:rect.width,height:rect.height};
      };
      return {refresh:measure('.lt-health-action'),
        close:measure('.lt-close-launcher'),
        preview:measure('.lt-card-picture')};
    });
  assert.ok(recoveryTargets.refresh&&recoveryTargets.refresh.width>=24&&
    recoveryTargets.refresh.height>=24,
    'Read-only recovery action must expose at least a 24x24 CSS-pixel target');
  assert.ok(recoveryTargets.close&&recoveryTargets.close.width>=24&&
    recoveryTargets.close.height>=32,
    'Normal Close game review must keep its original distinct 32px-or-taller target');
  assert.ok(recoveryTargets.preview&&recoveryTargets.preview.width>0&&recoveryTargets.preview.height>0,
    'Native DWM preview target must remain allocated independently');
  await page.evaluate(()=>window.__emitNative('lazy-tester-close-review',true));
  assert.equal(await checkHealth.isDisabled(),true,
    'Guided action must be disabled during a native close review');
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isDisabled(),true,
    'Header Refresh must not queue a misleading deferred action during native close review');
  await page.evaluate(()=>window.__emitNative('lazy-tester-close-review',false));
  await page.waitForFunction(()=>!document.querySelector('[aria-label="Refresh status for Minecraft - 2"]')?.disabled);
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isEnabled(),true,
    'Header Refresh should return after native close review ends');
  const oldInspectCount=await page.evaluate(()=>window.__calls.filter(call=>
    call.command==='lazy_tester_session' && call.args?.request?.operation==='Inspect').length);
  await checkHealth.click();
  await page.waitForFunction(before=>window.__calls.filter(call=>
    call.command==='lazy_tester_session' && call.args?.request?.operation==='Inspect').length>before,oldInspectCount);
  // Finish the first requested refresh before selecting a second action.
  const checkOther=page.getByRole('button',{name:'Refresh status for Minecraft - 3'});
  await page.waitForFunction(()=>document.querySelector('[aria-label="Refresh status for Minecraft - 3"]')?.disabled===false);
  const secondInspectCount=await page.evaluate(()=>window.__calls.filter(call=>
    call.command==='lazy_tester_session' && call.args?.request?.operation==='Inspect').length);
  await checkOther.click();
  await page.waitForFunction(before=>window.__calls.filter(call=>
    call.command==='lazy_tester_session' && call.args?.request?.operation==='Inspect').length>before,secondInspectCount);
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>
    ['Launch','CloseOne','CloseAll','ClearProfileData'].includes(call.args?.request?.operation))),false,
    'Guided Refresh must issue Inspect only');
  // Reset the synthetic issue AFTER both actions complete. Clicking the
  // disappearing warning would race the first successful Refresh.
  await page.waitForFunction(()=>document.querySelector('[aria-label="Refresh status for Minecraft - 3"]')?.disabled===false);
  await page.evaluate(()=>{window.__healthFixture=false;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await windowHealth.waitFor({state:'detached'});
  await launcherHealth.waitFor({state:'detached'});
  assert.equal(await page.getByRole('button',{name:/^Launch Minecraft - 3 · /}).isEnabled(),true,
    'Only a subsequent clean STOPPED observation may enable Launch');

  // A keyboard-issued Refresh disables its own button while native Inspect
  // is pending. The focused authored notice persists until the old warning
  // disappears; focus then returns to the same stable native slot's button.
  // These synthetic checks do not prove Windows/WebView2 or DWM focus.
  await page.goto('http://lazy-tester.test/?health-state=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const focusedHealth=page.getByRole('status',{name:'Minecraft - 2 needs attention'});
  const focusedRefresh=page.getByRole('button',{name:'Refresh status for Minecraft - 2'});
  await focusedRefresh.waitFor();
  await focusedRefresh.focus();
  await page.evaluate(()=>{window.__holdInspect=true;window.__healthFixture=false;});
  await page.keyboard.press('Enter');
  await page.waitForFunction(()=>typeof window.__finishInspect==='function');
  assert.equal(await focusedHealth.evaluate(node=>document.activeElement===node),true,
    'Keyboard Refresh should keep focus in its read-only notice while the button is disabled');
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});
  await focusedHealth.waitFor({state:'detached'});
  await page.waitForFunction(()=>document.activeElement?.matches(
    '.lt-launcher-entry[data-slot="2"] > .lt-card')===true);
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').isEnabled(),true,
    'A verified STOPPED observation returns focus to the same newly enabled launcher card');
  assert.equal(await page.evaluate(()=>window.__calls.some(call=>
    ['Launch','CloseOne','CloseAll'].includes(call.args?.request?.operation))),false,
    'The focus handoff must not trigger native game operations');

  // If native Inspect fails and the card is disabled, return focus to a
  // non-Tabbable named group for the same slot instead of document.body.
  await page.goto('http://lazy-tester.test/?health-state=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const unobservedHealth=page.getByRole('status',{name:'Minecraft - 2 needs attention'});
  await unobservedHealth.waitFor();
  await page.evaluate(()=>{window.__holdInspect=true;window.__offline=true;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(()=>typeof window.__finishInspect==='function');
  await unobservedHealth.focus();
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});
  await unobservedHealth.waitFor({state:'detached'});
  await page.waitForFunction(()=>document.activeElement?.matches(
    '.lt-launcher-entry[data-slot="2"][role="group"]')===true);
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').isDisabled(),true,
    'Connection uncertainty must not create a new Launch admission');

  // A later explicit focus change takes precedence over any stale notice.
  await page.goto('http://lazy-tester.test/?health-state=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const supersededHealth=page.getByRole('status',{name:'Minecraft - 2 needs attention'});
  await supersededHealth.waitFor();
  await page.evaluate(()=>{window.__holdInspect=true;window.__healthFixture=false;});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await page.waitForFunction(()=>typeof window.__finishInspect==='function');
  await supersededHealth.focus();
  await nav('Play').focus();
  await page.evaluate(()=>{window.__holdInspect=false;window.__finishInspect();});
  await supersededHealth.waitFor({state:'detached'});
  assert.equal(await nav('Play').evaluate(node=>document.activeElement===node),true,
    'Removing an old notice cannot steal focus after the operator chose another control');

  // A windowless NOT_RESPONDING observation must not tell users to inspect
  // a non-existent game window or allow a second Launch.
  await page.goto('http://lazy-tester.test/?health-state=1&windowless-health=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const missingWindowNotice=page.getByRole('status',{name:'Minecraft - 2 needs attention'});
  await missingWindowNotice.waitFor();
  assert.match(await missingWindowNotice.textContent(),/Game window unavailable/);
  const missingWindowHint=await page.locator('.lt-launcher-entry').nth(1).getAttribute('title')??'';
  assert.match(missingWindowHint,/window is unavailable/);
  assert.doesNotMatch(missingWindowHint,/Check its window/);
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').isDisabled(),true,
    'A missing native process/window observation must not enable Launch');

  // An unconfirmed native window restoration owns its Play warning. The
  // per-card Refresh control is intentionally absent, so the card
  // cannot continue recommending that unavailable action.
  await page.goto('http://lazy-tester.test/?health-state=1&close-recovery=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  const affectedCard=page.locator('.lt-launcher-entry').nth(1);
  assert.match(await affectedCard.getAttribute('title')??'',/Refresh/);
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  const closeReview=page.getByRole('region',{name:'Confirm one Minecraft close'});
  await page.waitForFunction(()=>document.activeElement?.textContent?.trim()==='Cancel');
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isDisabled(),true,
    'Manual Refresh is unavailable while the selected-game confirmation is open');
  // Review navigation is disabled; Escape on an unrelated still-focusable
  // window control must NOT dismiss the pending game-close review.
  assert.equal(await nav('Play').isDisabled(),true);
  assert.equal(await nav('Settings').isDisabled(),true);
  // Tauri drag region is pointer-only, not a bogus keyboard button.
  // Native Maximize remains the real, independently focusable window action.
  assert.equal(await page.locator('.lt-window-drag[data-tauri-drag-region]').count(),1,
    'The branded titlebar keeps its original Tauri-native drag region');
  assert.equal(await page.getByRole('button',{name:'Move Lazy Tester window'}).count(),0,
    'Pointer-only titlebar drag cannot pollute the keyboard button list');
  await page.getByRole('button',{name:'Maximize',exact:true}).focus();
  await page.keyboard.press('Escape');
  assert.equal(await closeReview.count(),1,
    'Escape outside the focused CloseOne review must not intercept other UI');
  await closeReview.getByRole('button',{name:'Cancel',exact:true}).focus();
  await page.keyboard.press('Escape');
  await closeReview.waitFor({state:'detached'});
  assert.equal(await page.getByRole('button',{name:'Close Minecraft - 2 normally'})
    .evaluate(node=>document.activeElement===node),true,
    'Escape cancels the review and returns focus to its original launcher');
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isEnabled(),true,
    'Cancel restores Refresh without sending a game-close request');
  assert.equal(await page.evaluate(()=>window.__calls.filter(call=>
    call.command==='lazy_tester_close_one').length),0,
    'Canceling CloseOne must not close any Minecraft session');
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  await closeReview.getByRole('button',{name:'Cancel',exact:true}).click();
  await closeReview.waitFor({state:'detached'});
  assert.equal(await page.getByRole('button',{name:'Close Minecraft - 2 normally'})
    .evaluate(node=>document.activeElement===node),true,
    'Cancel button returns keyboard focus without modifying Minecraft');
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  assert.equal(await page.getByRole('button',{name:'Refresh',exact:true}).isDisabled(),true);
  await page.getByRole('region',{name:'Confirm one Minecraft close'})
    .getByRole('button',{name:'Close Minecraft - 2',exact:true}).click();
  await page.waitForFunction(()=>document.activeElement?.matches(
    '.lt-launcher-entry[data-slot="2"][role="group"]')===true);
  assert.equal(await affectedCard.locator('.lt-card').count(),1,
    'Confirmed CloseOne review keeps its original stable launcher focus group');
  await page.getByRole('status',{name:'Window recovery unconfirmed'}).waitFor();
  await page.waitForFunction(()=>
    document.querySelectorAll('[aria-label="Refresh status for Minecraft - 2"]').length===0);
  assert.equal(await affectedCard.getAttribute('title'),null,
    'Unconfirmed restoration must suppress advice for a missing action');
  assert.equal(await affectedCard.locator('.lt-card').getAttribute('aria-describedby'),null,
    'An unresolved recovery must not retain a stale accessibility description');
  assert.equal(await affectedCard.locator('#lt-launcher-guidance-2').count(),0,
    'Guidance is absent when the action is intentionally unavailable');
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="2"] > .lt-card').isDisabled(),true,
    'Recovery guidance must not enable another game launch');
  assert.equal(await page.evaluate(()=>window.__calls.filter(call=>
    call.args?.request?.operation==='Launch').length),0,
    'Explaining an unresolved recovery must never start Minecraft');

  // Synthetic CloseOne: a failed native recovery receipt remains unconfirmed
  // even when a later read-only session observation reports STOPPED.
  await page.goto('http://lazy-tester.test/?close-recovery=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await heading('Minecraft - 2').waitFor();
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  await page.getByRole('region',{name:'Confirm one Minecraft close'}).getByRole('button',{name:'Close Minecraft - 2',exact:true}).click();
  await page.getByRole('status',{name:'Window recovery unconfirmed'}).waitFor();
  assert.equal(await page.getByRole('heading',{name:'Window restoration not verified'}).count(),1);
  await page.evaluate(()=>{window.__runningSlots=[];});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  // Wait specifically for MC02 to change from Open to Launch, proving
  // STOPPED reached Svelte before checking whether recovery was retained.
  await page.getByRole('button',{name:/^Launch Minecraft - 2 · /}).waitFor();
  assert.equal(await page.getByRole('status',{name:'Window recovery unconfirmed'}).count(),1,
    'STOPPED observations cannot clear an unacknowledged native checkpoint');
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.command==='lazy_tester_close_one').length),1,
    'a read-only refresh must not retry CloseOne');

  // A lost IPC receipt is likewise ambiguous: the UI must not keep claiming
  // the old embedded HWND; an explicit confirmed receipt can later clear it.
  await page.goto('http://lazy-tester.test/?close-ipc-error=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await heading('Minecraft - 2').waitFor();
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  await page.getByRole('region',{name:'Confirm one Minecraft close'}).getByRole('button',{name:'Close Minecraft - 2',exact:true}).click();
  await page.getByRole('status',{name:'Window recovery unconfirmed'}).waitFor();
  assert.equal(await page.getByRole('heading',{name:'Window restoration not verified'}).count(),1,
    'missing IPC receipt cannot retain an assumed embedded Play surface');
  await page.evaluate(()=>{window.__closeOutcome='confirmed';});
  await page.getByRole('button',{name:'Close Minecraft - 2 normally'}).click();
  await page.getByRole('region',{name:'Confirm one Minecraft close'}).getByRole('button',{name:'Close Minecraft - 2',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('[aria-label="Window recovery unconfirmed"]')===null);
  assert.equal(await page.evaluate(()=>window.__calls.filter(c=>c.command==='lazy_tester_close_one').length),2,
    'only an explicit operator retry may request a second selected close');


  // Independent failed recoveries must remain visible until each selected
  // native close returns its own positive window-restoration receipt.
  await page.goto('http://lazy-tester.test/?close-recovery=1&multi-recovery=1');
  await page.waitForFunction(()=>!!window.__finishBinding);
  await page.evaluate(()=>window.__finishBinding());
  await heading('Minecraft - 2').waitFor();
  const closeGame = async slot => {
    await page.getByRole('button',{name:'Close Minecraft - '+slot+' normally'}).click();
    await page.getByRole('region',{name:'Confirm one Minecraft close'})
      .getByRole('button',{name:'Close Minecraft - '+slot,exact:true}).click();
  };
  await closeGame(2);
  await page.getByRole('status',{name:'Window recovery unconfirmed'}).waitFor();
  await closeGame(3);
  const recoveryStatus=page.getByRole('status',{name:'Window recovery unconfirmed'});
  await page.waitForFunction(()=>{
    const notice=document.querySelector('[aria-label="Window recovery unconfirmed"]')?.textContent??'';
    return notice.includes('Minecraft - 2, Minecraft - 3');
  });
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  assert.match(await recoveryStatus.textContent(),/Minecraft - 2, Minecraft - 3/,
    'Refresh cannot erase an unconfirmed window restoration');
  await page.evaluate(()=>{window.__closeOutcome='confirmed';});
  await closeGame(3);
  await page.waitForFunction(()=>{
    const notice=document.querySelector('[aria-label="Window recovery unconfirmed"]')?.textContent??'';
    return notice.includes('Minecraft - 2:') && !notice.includes('Minecraft - 3');
  });
  assert.equal(await recoveryStatus.count(),1,
    'Confirming one slot must preserve the other unresolved recovery');

  // Existing-data recovery is still available without launching or deleting profiles.
  await page.goto('http://lazy-tester.test/?broken=1');await page.getByText('Saved launcher data could not be verified. No game was launched.',{exact:true}).waitFor();
  assert.equal(await page.getByText('Saved profiles could not be verified.',{exact:true}).count(),0,'Raw backend binding errors must not appear in UI');
  assert.equal(await heading('Ready when you are').count(),0);
  await heading('Connection needs attention').waitFor();
  assert.equal(await page.locator('.lt-launcher-entry[data-slot="1"] > .lt-card').isDisabled(),true);
  assert.equal((await page.locator('[data-slot="1"] .lt-card-picture').textContent())?.trim(),'Unavailable',
    'Disconnected profiles cannot advertise a misleading Launch action');
  assert.equal((await page.locator('[data-slot="1"] .lt-card small').textContent())?.trim(),'Status unavailable',
    'Disconnected cards must not claim the last known native state');
  // A transient first check must recover through a deliberate Refresh, without import/reset.
  await page.evaluate(()=>{window.__bindingFailure='';window.__locationFailure='';window.__finishBinding=()=>{};});
  await page.getByRole('button',{name:'Refresh',exact:true}).click();
  await heading('Ready when you are').waitFor();
  assert.equal(await page.getByRole('button',{name:/^Launch Minecraft - 1 · /}).isEnabled(),true);
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.import||c.args?.action==='Choose'||c.args?.request?.operation==='Launch')),false);
  await page.goto('http://lazy-tester.test/?broken=1');await heading('Connection needs attention').waitFor();
  await page.getByRole('region',{name:'Launcher recovery'}).getByRole('button',{name:'Data Storage',exact:true}).click();await heading('Data Storage').waitFor();
  await page.getByText('Connect or locate launcher data',{exact:true}).click();
  const locate=page.locator('.lt-location-options').getByRole('button',{name:'Locate existing data',exact:true});await locate.click();
  await page.evaluate(()=>{window.__locateOutcome='invalid';});await locate.click();await page.getByText('The selected folder does not match the saved profile identities.',{exact:true}).waitFor();
  await page.evaluate(()=>{window.__locateOutcome='valid';});await locate.click();await page.getByText('D:\\Recovered Data',{exact:true}).waitFor();
  assert.equal(await page.evaluate(()=>window.__calls.some(c=>c.args?.action==='Choose'||['Launch','ClearProfileCache'].includes(c.args?.request?.operation))),false);
  assert.deepEqual(errors,[]);
  console.log('PASS: automatic startup reconnect; native capability refusal; layout/DPI; stable polling; settings and offline recovery; synthetic CloseOne recovery uncertainty persists through STOPPED and transport failure until a confirmed native receipt. Mocked Tauri only; no real game/data mutations.');
} finally { await browser.close(); }
