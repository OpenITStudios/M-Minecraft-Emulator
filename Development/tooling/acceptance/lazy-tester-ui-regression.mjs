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
    const settings = {activeFps: 144, backgroundFps: 10, renderDistanceChunks: 12, canApplyPreset: true, changedProfiles: null};
    const snapshot = () => ({schema: 1, storage: null, profiles: profiles.map(profile => window.__runningSlot === profile.slot || window.__runningSlots?.includes(profile.slot) ? {
      ...profile, state: 'RUNNING', process: {pid: 100 + profile.slot, createdFileTime: '134000000000000001', sessionId: 1, windowHandle: '0x' + (0x123 + profile.slot).toString(16), responding: true},
    } : profile)});
    let callback = 0;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
    window.__TAURI_INTERNALS__ = {
      transformCallback() { return ++callback; },
      async invoke(command, args) {
        window.__calls.push({command, args});
        if (command === 'plugin:event|listen') return callback;
        if (command === 'plugin:event|unlisten') return null;
        if (command === 'lazy_tester_profile_binding') {
          if (window.__bindingFailure) throw window.__bindingFailure;
          if (!window.__finishBinding) await new Promise(resolve => { window.__finishBinding = resolve; });
          return {schema: 1, required: false, configured: true, issue: null};
        }
        if (command === 'lazy_tester_data_location') {
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
        if (operation === 'InspectSettings') return {...snapshot(), settings: {...settings, canApplyPreset: !window.__runningSlot}};
        if (operation === 'Launch') return {...snapshot(), profiles: profiles.map(profile => profile.slot === args.request.slot ? {...profile, state: 'STARTING'} : profile)};
        if (operation === 'SavePerformance') {
          settings.activeFps = args.request.activeFps; settings.backgroundFps = args.request.backgroundFps;
          return {...snapshot(), settings};
        }
        if (operation === 'InspectStorage') return {...snapshot(), storage: profiles.map(profile => ({
          slot: profile.slot, bytes: 1048576, files: 1, partial: true, disk: null,
          cache: {bytes: 0, files: 0, partial: false, canClear: false, issue: null},
        }))};
        if (operation === 'Inspect') return snapshot();
        throw new Error('Unexpected operation: ' + operation);
      },
    };
  });
  const heading = name => page.getByRole('heading', {name, exact: true});
  const nav = name => page.getByRole('navigation').getByRole('button', {name, exact: true});
  await page.goto('http://lazy-tester.test/');
  await heading('Getting ready…').waitFor();
  assert.equal(await page.getByText('Profile setup required', {exact: true}).count(), 0);
  await page.waitForFunction(() => !!window.__finishBinding);
  await page.evaluate(() => window.__finishBinding());
  await heading('Choose a Minecraft').waitFor();

  // A click during the periodic inspection must be retained until the request completes.
  await page.evaluate(() => { window.__holdInspect = true; });
  await page.waitForFunction(() => !!window.__finishInspect);
  await nav('Storage').click();
  await page.evaluate(() => { window.__holdInspect = false; window.__finishInspect(); });
  await heading('Data location').waitFor();
  await page.getByRole('button', {name: 'Refresh', exact: true}).click();
  await page.getByText(/Last checked at/).waitFor();
  assert.equal(await page.getByText('At least 1.0 MB', {exact: true}).count(), 5);
  assert.equal(await page.getByText('1 file · partial count', {exact: true}).count(), 5);

  await nav('Settings').click();
  const active = page.getByLabel('Active FPS limit', {exact: true});
  const background = page.getByLabel('Background FPS limit', {exact: true});
  await active.fill('60'); await background.fill('5');
  await nav('Launchers').click(); await heading('Minecraft launchers').waitFor();
  assert.equal(await page.getByRole('button', {name: 'Launch', exact: true}).count(), 5);
  await nav('Settings').click(); await heading('Performance').waitFor();
  assert.equal(await active.inputValue(), '60');
  assert.equal(await background.inputValue(), '5');
  await page.getByRole('button', {name: 'Save', exact: true}).click();
  await page.getByText('Saved. New limits apply when each Minecraft profile next starts.').waitFor();

  for (const size of [{width: 920, height: 620}, {width: 1366, height: 900}]) {
    await page.setViewportSize(size);
    for (const name of ['Workspace', 'Launchers', 'Storage', 'Settings']) {
      await nav(name).click();
      await page.waitForFunction(name => document.querySelector('nav [aria-current="page"]')?.textContent === name, name);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, name + ' overflows horizontally');
      assert.equal(await page.evaluate(() => [...document.querySelectorAll('.lt-panel, .lt-profile-list, .lt-viewport, .lt-filmstrip')].filter(node => node.getClientRects().length).some(node => node.getBoundingClientRect().bottom > innerHeight)), false, name + ' leaves the window');
    }
  }
  // A game becoming ready must not pull the player away from another page.
  await nav('Launchers').click();
  await page.getByRole('button', {name: 'Launch', exact: true}).first().click();
  await heading('Starting Minecraft - 1').first().waitFor();
  await nav('Settings').click(); await heading('Performance').waitFor();
  await page.evaluate(() => { window.__runningSlot = 1; });
  await page.getByRole('button', {name: 'Refresh', exact: true}).click();
  await page.waitForFunction(() => [...document.querySelectorAll('button')].find(node => node.textContent === 'Apply to all profiles')?.disabled);
  assert.equal(await nav('Settings').getAttribute('aria-current'), 'page');
  await page.evaluate(() => { window.__offline = true; });
  await page.getByRole('button', {name: 'Refresh', exact: true}).click();
  await page.getByRole('alert').waitFor();
  await page.waitForFunction(() => document.querySelector('.lt-sidebar-bottom')?.textContent.startsWith('Unavailable'));
  await page.evaluate(() => { window.__offline = false; });
  await page.getByRole('button', {name: 'Refresh', exact: true}).click();
  await page.waitForFunction(() => !document.querySelector('[role="alert"]'));
  assert.deepEqual(errors, []);
  assert.equal(await page.evaluate(() => window.__calls.filter(call => call.args?.request?.operation === 'Launch').length), 1);
  assert.equal(await page.evaluate(() => window.__calls.some(call => ['ClearProfileCache', 'ApplyPreset'].includes(call.args?.request?.operation))), false);

  await page.evaluate(() => { window.__runningSlots = [1,2]; });
  await nav('Workspace').click();
  await page.getByRole('button', {name: 'Use desktop windows', exact: true}).click();
  await heading('Desktop mode').waitFor();
  await heading('Games are in separate windows').waitFor();
  assert.equal(await page.getByText('Choose a Minecraft to start', {exact:true}).count(), 0);
  assert.equal(await page.locator('.lt-filmstrip:visible').count(), 0);
  assert.equal(await page.locator('[aria-label="Desktop sessions"]').count(), 1);
  await page.waitForFunction(()=>document.querySelectorAll('.lt-desktop-sessions article').length===2);
  assert.equal(await page.locator('.lt-desktop-sessions article').count(), 2);
  await page.setViewportSize({width:920,height:620});
  assert.equal(await page.evaluate(()=>document.querySelector('.lt-desktop').getBoundingClientRect().bottom>innerHeight),false);
  const captures = fileURLToPath(new URL('../../../workspace/runs/lazy-tester-ui-regression/', import.meta.url));
  await mkdir(captures,{recursive:true});
  await page.screenshot({path:path.join(captures,'desktop-mode.png')});
  await page.getByRole('button', {name:'Back to Workspace',exact:true}).click();
  await page.getByRole('button', {name:'Use desktop windows',exact:true}).waitFor();
  assert.equal(await page.locator('.lt-desktop:visible').count(), 0);

  // Broken/missing saved data must retain an available, non-destructive recovery route.
  await page.setViewportSize({width: 920, height: 620});
  await page.goto('http://lazy-tester.test/?broken=1');
  await page.getByText('Profile setup unavailable', {exact: true}).waitFor();
  await page.getByText('Saved profiles could not be verified.', {exact: true}).waitFor();
  assert.equal(await page.evaluate(() => [...document.querySelectorAll('.lt-viewport, .lt-filmstrip')].some(node => node.getBoundingClientRect().bottom > innerHeight)), false, 'Failed setup leaves the window');
  await page.getByRole('button', {name: 'Manage data', exact: true}).click();
  await heading('Data location').waitFor();
  await page.getByText('The saved data folder is unavailable.', {exact: true}).waitFor();
  const locate = page.getByRole('button', {name: 'Locate data', exact: true});
  assert.equal(await locate.isEnabled(), true);
  assert.equal(await page.getByRole('button', {name: 'Choose data folder', exact: true}).count(), 0);
  assert.equal(await page.getByRole('button', {name: 'Open folder', exact: true}).isDisabled(), true);
  await locate.click(); // Cancel leaves the broken saved pointer and the recovery action intact.
  await page.waitForFunction(() => window.__calls.some(call => call.args?.action === 'Locate'));
  assert.equal(await page.evaluate(() => window.__dataPath), null);
  await page.evaluate(() => { window.__locateOutcome = 'invalid'; });
  await locate.click();
  await page.getByText('The selected folder does not match the saved profile identities.', {exact: true}).waitFor();
  assert.equal(await page.evaluate(() => window.__dataPath), null);
  await page.evaluate(() => { window.__locateOutcome = 'valid'; });
  await locate.click();
  await page.getByText('D:\\Recovered Data', {exact: true}).waitFor();
  await page.waitForFunction(() => !document.querySelector('[aria-label="Profile setup"]'));
  await nav('Launchers').click(); await heading('Minecraft launchers').waitFor();
  assert.equal(await page.getByRole('button', {name: 'Launch', exact: true}).first().isEnabled(), true);
  await nav('Storage').click(); await heading('Data location').waitFor();
  await page.evaluate(() => { window.__locationFailure = 'The saved data folder is unavailable.'; });
  await page.getByRole('button', {name: 'Refresh location', exact: true}).click();
  await page.getByText('Profile setup unavailable', {exact: true}).waitFor();
  await nav('Launchers').click(); await heading('Minecraft launchers').waitFor();
  assert.equal(await page.getByRole('button', {name: 'Launch', exact: true}).first().isDisabled(), true);
  assert.equal(await page.evaluate(() => window.__calls.some(call => call.args?.action === 'Choose' || ['Launch', 'ClearProfileCache'].includes(call.args?.request?.operation))), false);
  assert.deepEqual(errors, []);
  console.log('PASS: app flows and missing-data recovery, cancel, invalid selection, verified reconnection, stale setup refusal (mocked Tauri only).');
} finally {
  await browser.close();
}
