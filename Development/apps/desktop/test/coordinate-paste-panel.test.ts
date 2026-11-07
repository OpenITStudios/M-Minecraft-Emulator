import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const panel=readFileSync(new URL('../src/CoordinatePastePanel.svelte',import.meta.url),'utf8');
const play=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/lazy-tester.css',import.meta.url),'utf8');

describe('optional manual XYZ user-paste UI',()=>{
  it('lives only in the existing Play header without changing five launcher ownership',()=>{
    expect(play).toContain("import CoordinatePastePanel from './CoordinatePastePanel.svelte';");
    expect(play).toContain('<CoordinatePastePanel selectedSlot={active} />');
    expect(play).toContain('class="lt-play-tools"');
    expect(play).toContain('class="lt-play-utilities"');
    expect(play).toContain('<LauncherRail {profiles} {labels} {healthIssues}');
    expect(play).toContain('<div class="lt-stage-space" bind:this={stageSpace}>');
    expect((play.match(/setInterval\(/g)??[])).toHaveLength(1);
    expect(css).toContain('.lt-coordinate-popup{position:fixed;');
    expect(play).not.toContain('<div class="lt-coordinate-popup"');
  });

  it('parses only user input, retains drafts on close, and offers readonly select-to-copy',()=>{
    for(const anchor of [
      'formatUserPastedCoordinates(raw, mode)', 'User-pasted · Unverified',
      'bind:value={raw}', 'maxlength="128"', 'on:toggle={resetOnClose}',
      'readonly', 'disabled={!output}', 'outputField.select()',
      'TELEPORT_POSITION_ONLY', 'No game command is executed.'
    ])expect(panel).toContain(anchor);
  });

  it('closes an in-panel Escape, restores summary focus and preserves native toggle reset',()=>{
    expect(panel).toContain("<svelte:window on:keydown={closeOnEscape} on:blur={closeOnWindowBlur} on:resize={repositionPopup} on:scroll|capture={repositionPopup} />");
    expect(panel).toContain('name="lt-play-disclosures"');
    expect(panel).toContain('on:focusout={closeWhenFocusLeaves}');
    expect(panel).toContain('style={popupStyle}');
    expect(panel).toContain("bind:this={detailsElement}");
    expect(panel).toContain("bind:this={summaryElement}");
    expect(panel).toContain("event.key !== 'Escape' || !detailsElement?.open");
    expect(panel).toContain("event.target instanceof Node");
    expect(panel).toContain("!detailsElement.contains(event.target)");
    expect(panel).toContain("detailsElement.open = false;");
    expect(panel).toContain("summaryElement?.focus({preventScroll: true});");
    expect(panel).toContain("on:toggle={resetOnClose}");
    const close = panel.split('function resetOnClose(')[1]?.split('</script>')[0] ?? '';
    expect(close).not.toContain("raw = ''");
    expect(close).not.toContain("bookmarkName = ''");
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
  it('rechecks the latest coordinate input and format after DOM updates before selecting',()=>{
    expect(panel).toContain("import {tick} from 'svelte';");
    expect(panel).toContain('async function selectOutput(): Promise<void>');
    expect(panel).toContain('const source = raw;');
    expect(panel).toContain('const format = mode;');
    expect(panel).toContain('expected = formatUserPastedCoordinates(source, format);');
    expect(panel).toContain('await tick();');
    expect(panel).toContain('!detailsElement?.open || source !== raw || format !== mode');
    expect(panel).toContain('invalid || output !== expected || !outputField?.isConnected');
    expect(panel).toContain('outputField.value !== expected');
    expect(panel).toContain('outputField.focus({preventScroll: true});');
    expect(panel).toContain('outputField.select();');
    expect(panel).not.toContain('if (!output || !outputField) return;');
  });

  it('clears earlier output selection when the user types or changes format',()=>{
    expect(panel).toContain('on:input={clearOutputSelection}');
    expect(panel).toContain('on:change={clearOutputSelection}');
    expect(panel).toContain('outputField.setSelectionRange(0, 0);');
    expect(panel).toContain('bind:value={raw}');
    expect(panel).toContain('bind:value={mode}');
    expect(panel).toContain('disabled={!output}');
  });
  it('uses only explicitly saved manual coordinates and pinned launcher labels',()=>{
    for(const anchor of [
      "export let selectedSlot: ProfileSlot | null = null;",
      'coordinateBookmarks.list()', 'coordinateBookmarks.save(', 'coordinateBookmarks.delete(',
      "selectedSlot === null", 'USER_PASTED_UNVERIFIED',
      'Save XYZ as location', 'World label (required, entered by you)',
      'Confirm delete', 'bookmark.xyz', 'savedStatus',
      'no player/world attribution is verified'
    ]) {
      // The source may spell the disclosure warning with title case.
      if (anchor==='USER_PASTED_UNVERIFIED') {
        expect(panel).toContain('User-pasted · Unverified');
      } else if (anchor==='no player/world attribution is verified') {
        expect(panel).toContain('no player/world attribution is verified');
      } else {
        expect(panel).toContain(anchor);
      }
    }
    expect(panel).toContain('deleteReview !== id');
    expect(panel).toContain('loadedSlot !== selectedSlot');
    expect(panel).toContain('loadTicket');
  });
  it('keeps unsaved XYZ and manual labels isolated per selected launcher',()=>{
    expect(panel).toContain('type ManualDraft = {');
    expect(panel).toContain('new Map<ProfileSlot | null, ManualDraft>()');
    expect(panel).toContain('let draftSlot: ProfileSlot | null = selectedSlot;');
    expect(panel).toContain('if (draftSlot !== selectedSlot)');
    expect(panel).toContain('draftsBySlot.set(draftSlot, {');
    expect(panel).toContain('const draft = draftsBySlot.get(selectedSlot);');
    expect(panel).toContain("raw = draft?.raw ?? '';");
    expect(panel).toContain("bookmarkName = draft?.name ?? '';");
    expect(panel).toContain("worldLabel = draft?.world ?? '';");
    expect(panel).toContain("dimension = draft?.dimension ?? 'Unknown';");
    expect(panel).toContain('Unsaved text and manual labels stay separate for each launcher');
    expect(panel).toContain('coordinateBookmarks.save({slot,name,worldLabel:world,dimension,raw:source})');
  });
  it('does not read/write clipboard, call the game controller or send commands',()=>{
    for(const forbidden of [
      'navigator.clipboard', 'readText(', 'writeText(', 'execCommand(',
      'invokeRuntime(', 'lazyTester.', 'workspaceSurface.', 'runCommand(',
      'setInterval(', 'system.runInterval(', 'localStorage', 'sessionStorage', 'fetch(',
    ])expect(panel).not.toContain(forbidden);
    expect(panel).toContain("Minecraft Education's built-in coordinate-copy shortcut is not verified.");
  });
});
