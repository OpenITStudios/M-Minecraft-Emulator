import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const panel=readFileSync(new URL('../src/CoordinatePastePanel.svelte',import.meta.url),'utf8');
const play=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/lazy-tester.css',import.meta.url),'utf8');

describe('optional manual XYZ user-paste UI',()=>{
  it('lives only in the existing Play header without changing five launcher ownership',()=>{
    expect(play).toContain("import CoordinatePastePanel from './CoordinatePastePanel.svelte';");
    expect(play).toContain("{#if page==='workspace'}<CoordinatePastePanel />{/if}");
    expect(play).toContain('<LauncherRail {profiles} {labels} {healthIssues}');
    expect(play).toContain('<div class="lt-stage-space" bind:this={stageSpace}>');
    expect((play.match(/setInterval\(/g)??[])).toHaveLength(1);
    expect(css).toContain('.lt-coordinate-popup{position:absolute;');
    expect(play).not.toContain('<div class="lt-coordinate-popup"');
  });

  it('parses only user input, resets on close, and offers readonly select-to-copy',()=>{
    for(const anchor of [
      'formatUserPastedCoordinates(raw, mode)', 'User-pasted · Unverified',
      'bind:value={raw}', 'maxlength="128"', 'on:toggle={resetOnClose}',
      "raw = '';", 'readonly', 'disabled={!output}', 'outputField.select()',
      'TELEPORT_POSITION_ONLY', 'No game command is executed.'
    ])expect(panel).toContain(anchor);
  });

  it('does not read/write clipboard, persist coordinates, invoke native game or send commands',()=>{
    for(const forbidden of [
      'navigator.clipboard', 'readText(', 'writeText(', 'execCommand(',
      'invokeRuntime(', 'lazyTester.', 'workspaceSurface.', 'runCommand(',
      'setInterval(', 'system.runInterval(', 'localStorage', 'sessionStorage', 'fetch(',
    ])expect(panel).not.toContain(forbidden);
    expect(panel).toContain("Minecraft Education's built-in coordinate-copy shortcut is not verified.");
  });
});
