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

  it('closes an in-panel Escape, restores summary focus and preserves native toggle reset',()=>{
    expect(panel).toContain("<svelte:window on:keydown={closeOnEscape} />");
    expect(panel).toContain("bind:this={detailsElement}");
    expect(panel).toContain("bind:this={summaryElement}");
    expect(panel).toContain("event.key !== 'Escape' || !detailsElement?.open");
    expect(panel).toContain("event.target instanceof Node");
    expect(panel).toContain("!detailsElement.contains(event.target)");
    expect(panel).toContain("detailsElement.open = false;");
    expect(panel).toContain("summaryElement?.focus({preventScroll: true});");
    expect(panel).toContain("on:toggle={resetOnClose}");
    expect(panel).toContain("raw = '';");
  });

  it('links invalid text and readonly output help for screen readers',()=>{
    expect(panel).toContain('id="lt-coordinate-input-hint"');
    expect(panel).toContain("aria-invalid={invalid}");
    expect(panel).toContain("aria-describedby={invalid ? 'lt-coordinate-input-hint lt-coordinate-error' : 'lt-coordinate-input-hint'}");
    expect(panel).toContain('id="lt-coordinate-error" class="lt-coordinate-error" role="alert"');
    expect(panel).toContain('aria-describedby="lt-coordinate-copy-help"');
    expect(panel).toContain('id="lt-coordinate-copy-help" class="lt-coordinate-help"');
    expect(panel).toContain("outputField.select()");
    expect(panel).not.toContain('role="dialog"');
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
