<script lang="ts">
  import {tick} from 'svelte';
  import {formatUserPastedCoordinates, type PastedCoordinateOutput} from './app/coordinates.js';

  // Ephemeral UI-only input. Never identify the selected Minecraft/world.
  let raw = '';
  let mode: PastedCoordinateOutput = 'XYZ';
  let output = '';
  let invalid = false;
  let outputField: HTMLInputElement | undefined;
  let detailsElement: HTMLDetailsElement | undefined;
  let summaryElement: HTMLElement | undefined;

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
      raw = '';
      mode = 'XYZ';
      output = '';
      invalid = false;
    }
  }
</script>
<svelte:window on:keydown={closeOnEscape} />

<!-- User-initiated text only. No live game, native IPC or clipboard access. -->
<details class="lt-coordinate-paste" bind:this={detailsElement} on:toggle={resetOnClose}>
  <summary bind:this={summaryElement} aria-label="Open manual coordinate formatter">Paste XYZ</summary>
  <div class="lt-coordinate-popup" role="group" aria-label="Manual coordinate formatter">
    <strong>Manual coordinates</strong>
    <small class="lt-coordinate-status">User-pasted · Unverified</small>
    <p id="lt-coordinate-input-hint">Paste three XYZ numbers. This tool does not read the clipboard or connect to a Minecraft player or world.</p>
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
  </div>
</details>
