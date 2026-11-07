<script lang="ts">
  import {tick} from 'svelte';
  import {formatUserPastedCoordinates, type PastedCoordinateOutput} from './app/coordinates.js';
  import {coordinateBookmarks, type CoordinateBookmark, type BookmarkDimension} from './app/bridge/coordinateBookmarks.js';
  import type {ProfileSlot} from './app/bridge/lazyTester.js';
  import {playToolPopupStyle} from './app/presentation/playToolPopup.js';

  // Selected native launcher is an OPERATOR-ASSIGNED notebook label only.
  // Clipboard, player identity, active world and dimension are never inferred.
  export let selectedSlot: ProfileSlot | null = null;

  // Ephemeral UI-only input. Never identify the selected Minecraft/world.
  let raw = '';
  let mode: PastedCoordinateOutput = 'XYZ';
  let output = '';
  let invalid = false;
  let outputField: HTMLInputElement | undefined;
  let detailsElement: HTMLDetailsElement | undefined;
  let summaryElement: HTMLElement | undefined;
  let popupStyle = 'visibility:hidden;';

  function repositionPopup(event?: Event): void {
    if (!detailsElement?.open || !summaryElement) return;
    if (event?.type === 'scroll' && event.target instanceof Node && detailsElement.contains(event.target)) return;
    const anchor = summaryElement.getBoundingClientRect();
    // A scrolled-away trigger must not leave a floating panel covering the
    // native Play view. This closes only this existing HTML disclosure.
    if (anchor.bottom <= 0 || anchor.top >= window.innerHeight ||
        anchor.right <= 0 || anchor.left >= window.innerWidth) {
      detailsElement.open = false;
      return;
    }
    popupStyle = playToolPopupStyle(anchor,
      {width:window.innerWidth,height:window.innerHeight},390) + 'visibility:visible;';
  }
  function closeWhenFocusLeaves(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (detailsElement?.open && next instanceof Node && !detailsElement.contains(next))
      detailsElement.open = false;
  }
  let bookmarkName = '';
  let worldLabel = '';
  let dimension: BookmarkDimension = 'Unknown';
  let saved: readonly CoordinateBookmark[] = [];
  let savedStatus = '';
  let saving = false;
  let loading = false;
  let deleting = false;
  let deleteReview: string | null = null;
  let loadedSlot: ProfileSlot | null = null;
  let loadTicket = 0;

  // An unverified manual note must not silently follow MC01 -> MC02. Only
  // the selected launcher's ephemeral text draft is displayed and saved.
  // Closing the disclosure preserves that launcher's draft until app exit.
  type ManualDraft = {
    raw: string; mode: PastedCoordinateOutput; name: string;
    world: string; dimension: BookmarkDimension;
  };
  const draftsBySlot = new Map<ProfileSlot | null, ManualDraft>();
  let draftSlot: ProfileSlot | null = selectedSlot;

  $: if (draftSlot !== selectedSlot) {
    draftsBySlot.set(draftSlot, {
      raw, mode, name: bookmarkName, world: worldLabel, dimension,
    });
    const draft = draftsBySlot.get(selectedSlot);
    draftSlot = selectedSlot;
    raw = draft?.raw ?? '';
    mode = draft?.mode ?? 'XYZ';
    bookmarkName = draft?.name ?? '';
    worldLabel = draft?.world ?? '';
    dimension = draft?.dimension ?? 'Unknown';
    clearOutputSelection();
  }

  $: if (detailsElement?.open && loadedSlot !== selectedSlot) {
    loadedSlot = selectedSlot;
    void refreshBookmarks();
  }

  async function refreshBookmarks(): Promise<void> {
    const slot = selectedSlot;
    const ticket = ++loadTicket;
    saved = [];
    deleteReview = null;
    savedStatus = '';
    if (!detailsElement?.open || slot === null) {
      loading = false;
      if (detailsElement?.open) savedStatus = 'Select an active Minecraft launcher to view its saved notes.';
      return;
    }
    loading = true;
    try {
      const report = await coordinateBookmarks.list();
      if (ticket !== loadTicket || !detailsElement?.open || slot !== selectedSlot) return;
      saved = report.items.filter(item => item.slot === slot).slice().reverse();
    } catch {
      if (ticket === loadTicket && detailsElement?.open) {
        savedStatus = 'Saved locations are unavailable. Existing data was not reset.';
      }
    } finally {
      if (ticket === loadTicket) loading = false;
    }
  }

  async function saveBookmark(): Promise<void> {
    const slot = selectedSlot;
    if (saving || loading || !detailsElement?.open || slot === null || !raw.trim() || invalid) return;
    const source = raw;
    const name = bookmarkName.trim();
    const world = worldLabel.trim();
    if (!name || !world) return;
    saving = true;
    savedStatus = '';
    try {
      const report = await coordinateBookmarks.save({slot,name,worldLabel:world,dimension,raw:source});
      if (!detailsElement?.open || slot !== selectedSlot) return;
      saved = report.items.filter(item => item.slot === slot).slice().reverse();
      bookmarkName = '';
      savedStatus = 'Location saved as a manual note. Player/world identity is unverified.';
    } catch {
      if (detailsElement?.open) savedStatus = 'Save failed. Check XYZ, labels or storage; existing notes were preserved.';
    } finally {
      saving = false;
    }
  }

  async function deleteBookmark(id: string): Promise<void> {
    const slot = selectedSlot;
    if (deleting || slot === null || deleteReview !== id || !detailsElement?.open ||
        !saved.some(item => item.id === id)) return;
    deleting = true;
    deleteReview = null;
    savedStatus = '';
    try {
      const report = await coordinateBookmarks.delete(id);
      if (!detailsElement?.open || slot !== selectedSlot) return;
      saved = report.items.filter(item => item.slot === slot).slice().reverse();
      savedStatus = 'Saved location deleted by your explicit request.';
    } catch {
      if (detailsElement?.open) savedStatus = 'Delete failed. The original saved notes were not reset.';
    } finally {
      deleting = false;
    }
  }

  function useBookmark(bookmark: CoordinateBookmark): void {
    if (bookmark.slot !== selectedSlot) return;
    raw = bookmark.xyz;
    mode = 'XYZ';
    worldLabel = bookmark.worldLabel;
    dimension = bookmark.dimension;
    clearOutputSelection();
    savedStatus = 'Loaded saved XYZ into the manual formatter. No Minecraft command was sent.';
  }

  $: {
    if (!raw.trim()) {
      output = '';
      invalid = false;
    } else {
      try {
        output = formatUserPastedCoordinates(raw, mode);
        invalid = false;
      } catch {
        output = '';
        invalid = true;
      }
    }
  }

  // Formatting is revalidated against the latest bound input and mode.
  // A stale DOM value can never be selected during a rapid mode change.
  async function selectOutput(): Promise<void> {
    const source = raw;
    const format = mode;
    if (!source.trim()) return;
    let expected: string;
    try {
      expected = formatUserPastedCoordinates(source, format);
    } catch {
      return;
    }
    await tick(); // Svelte has finished applying the pending output update.
    if (!detailsElement?.open || source !== raw || format !== mode ||
        invalid || output !== expected || !outputField?.isConnected ||
        outputField.value !== expected) return;
    outputField.focus({preventScroll: true});
    outputField.select();
    // Selection is user-driven and never invokes a clipboard API.
  }

  // Invalidate any previous selection before showing a new result.
  function clearOutputSelection(): void {
    if (outputField?.isConnected) outputField.setSelectionRange(0, 0);
  }

  // Escape is handled only for this focused, open disclosure.
  // It cannot steal gameplay keys or the game's separate close review.
  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || !detailsElement?.open ||
        !(event.target instanceof Node) || !detailsElement.contains(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    detailsElement.open = false;
    summaryElement?.focus({preventScroll: true});
  }

  function resetOnClose(event: Event): void {
    if (!(event.currentTarget as HTMLDetailsElement).open) {
      ++loadTicket;
      popupStyle = 'visibility:hidden;';
      // Closing a disclosure is not a discard request. Retain unsaved manual text
      // only in this mounted UI; clear the slot-specific saved list and pending review.
      saved = [];
      savedStatus = '';
      deleteReview = null;
      loadedSlot = null;
      loading = false;
    } else {
      loadedSlot = selectedSlot;
      repositionPopup();
      void refreshBookmarks();
    }
  }
</script>
<svelte:window on:keydown={closeOnEscape} on:resize={repositionPopup} on:scroll|capture={repositionPopup} />

<!-- Manual input and explicit notebook actions only. No game command or clipboard access. -->
<details class="lt-coordinate-paste" name="lt-play-disclosures" bind:this={detailsElement} on:toggle={resetOnClose} on:focusout={closeWhenFocusLeaves}>
  <summary bind:this={summaryElement} aria-label="Open coordinate formatter and saved locations">Coordinates & Saves</summary>
  <div class="lt-coordinate-popup" style={popupStyle} role="group" aria-label="Manual coordinate formatter">
    <strong>Manual coordinates</strong>
    <small class="lt-coordinate-status">User-pasted · Unverified</small>
    <p class="lt-coordinate-help">Unsaved text and manual labels stay separate for each launcher until Lazy Tester closes. Check the selected launcher before saving.</p>
    <p id="lt-coordinate-input-hint">In Minecraft Education, enable Settings > Show Coordinates. If your version provides Creator > Enable Copy Coordinate UI, press Ctrl+Alt+C in Minecraft and Ctrl+V here. Education 26.33 copy-shortcut support remains unverified. Otherwise type XYZ manually. This tool never reads the clipboard or identifies a player/world.</p>
    <label for="lt-coordinate-input">XYZ coordinates</label>
    <textarea
      id="lt-coordinate-input"
      rows="2"
      maxlength="128"
      spellcheck={false}
      autocomplete="off"
      placeholder="123.456 64 -45.125"
      aria-invalid={invalid}
      aria-describedby={invalid ? 'lt-coordinate-input-hint lt-coordinate-error' : 'lt-coordinate-input-hint'}
      bind:value={raw}
      on:input={clearOutputSelection}
    ></textarea>
    {#if invalid}
      <p id="lt-coordinate-error" class="lt-coordinate-error" role="alert">Enter exactly three decimal XYZ numbers separated by spaces or commas. Commands and extra text are not accepted.</p>
    {/if}
    <label for="lt-coordinate-format">Output format</label>
    <select id="lt-coordinate-format" bind:value={mode} on:change={clearOutputSelection}>
      <option value="XYZ">Exact XYZ</option>
      <option value="BLOCK">Block XYZ</option>
      <option value="TELEPORT_POSITION_ONLY">Teleport text (position only)</option>
    </select>
    <label for="lt-coordinate-output">Formatted text</label>
    <div class="lt-coordinate-output">
      <input
        id="lt-coordinate-output"
        type="text"
        readonly
        value={output}
        bind:this={outputField}
        placeholder="Paste valid XYZ above"
        aria-label="Formatted coordinate text"
        aria-describedby="lt-coordinate-copy-help"
      />
      <button type="button" class="lt-secondary" disabled={!output} on:click={selectOutput}>Select text</button>
    </div>
    <small id="lt-coordinate-copy-help" class="lt-coordinate-help">Press Ctrl+C to copy selected text. No game command is executed. Minecraft Education's built-in coordinate-copy shortcut is not verified.</small>
    <section class="lt-coordinate-notes" aria-label="Saved location notes">
      <strong>Saved Locations</strong>
      <p class="lt-coordinate-help">Notes are stored locally in Lazy Tester, not in Minecraft. Launcher, world and dimension are manual labels; no player/world attribution is verified.</p>
      <p class="lt-coordinate-note-owner">Launcher label: {selectedSlot === null ? 'Select a running Minecraft first' : 'MC' + String(selectedSlot).padStart(2, '0')}</p>
      <label for="lt-coordinate-name">Location name</label>
      <input id="lt-coordinate-name" type="text" maxlength="64" bind:value={bookmarkName} placeholder="QA Spawn" autocomplete="off" />
      <label for="lt-coordinate-world">World label (required, entered by you)</label>
      <input id="lt-coordinate-world" type="text" maxlength="64" bind:value={worldLabel} placeholder="QA Test World" autocomplete="off" />
      <label for="lt-coordinate-dimension">Dimension label (manual)</label>
      <select id="lt-coordinate-dimension" bind:value={dimension}>
        <option value="Unknown">Unknown / not confirmed</option>
        <option value="Overworld">Overworld</option>
        <option value="Nether">Nether</option>
        <option value="TheEnd">The End</option>
      </select>
      <button type="button" class="lt-secondary" disabled={saving || deleting || loading || selectedSlot===null || !raw.trim() || invalid || !bookmarkName.trim() || !worldLabel.trim()} on:click={() => void saveBookmark()}>
        {saving ? 'Saving…' : 'Save XYZ as location'}
      </button>
      <p class="lt-coordinate-help" role="status" aria-live="polite">{savedStatus}</p>
      {#if selectedSlot !== null}
        <div class="lt-coordinate-saved-header">
          <strong>MC{String(selectedSlot).padStart(2, '0')} locations ({saved.length})</strong>
          <button class="lt-secondary" type="button" disabled={loading || saving || deleting} on:click={() => void refreshBookmarks()}>Reload</button>
        </div>
        {#if loading}
          <p>Reading saved notes…</p>
        {:else if saved.length === 0}
          <p>No saved locations for this launcher.</p>
        {:else}
          <ul class="lt-coordinate-saved-list">
            {#each saved as bookmark (bookmark.id)}
              <li>
                <div class="lt-coordinate-saved-text">
                  <strong>{bookmark.name}</strong>
                  <small>{bookmark.worldLabel} · {bookmark.dimension}</small>
                  <code>{bookmark.xyz}</code>
                </div>
                <div class="lt-coordinate-saved-buttons">
                  <button type="button" class="lt-secondary" on:click={() => useBookmark(bookmark)}>Use XYZ</button>
                  {#if deleteReview === bookmark.id}
                    <button type="button" class="lt-secondary" disabled={deleting} on:click={() => void deleteBookmark(bookmark.id)}>Confirm delete</button>
                    <button type="button" class="lt-secondary" on:click={() => deleteReview = null}>Cancel</button>
                  {:else}
                    <button type="button" class="lt-secondary" disabled={deleting} on:click={() => deleteReview = bookmark.id}>Delete</button>
                  {/if}
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </section>
  </div>
</details>
