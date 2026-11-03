<script lang="ts">
  import {formatUserPastedCoordinates, type PastedCoordinateOutput} from './app/coordinates.js';

  // Ephemeral UI-only input. Never identify the selected Minecraft/world.
  let raw = '';
  let mode: PastedCoordinateOutput = 'XYZ';
  let output = '';
  let invalid = false;
  let outputField: HTMLInputElement | undefined;

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

  function selectOutput(): void {
    if (!output || !outputField) return;
    outputField.focus();
    outputField.select();
    // Selecting text does NOT write to the operating-system clipboard.
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

<!-- User-initiated text only. No live game, native IPC or clipboard access. -->
<details class="lt-coordinate-paste" on:toggle={resetOnClose}>
  <summary aria-label="Open manual coordinate formatter">Paste XYZ</summary>
  <div class="lt-coordinate-popup" role="group" aria-label="Manual coordinate formatter">
    <strong>Manual coordinates</strong>
    <small class="lt-coordinate-status">User-pasted · Unverified</small>
    <p>Paste three XYZ numbers. This tool does not read the clipboard or connect to a Minecraft player or world.</p>
    <label for="lt-coordinate-input">XYZ coordinates</label>
    <textarea
      id="lt-coordinate-input"
      rows="2"
      maxlength="128"
      spellcheck={false}
      autocomplete="off"
      placeholder="123.456 64 -45.125"
      bind:value={raw}
    ></textarea>
    {#if invalid}
      <p class="lt-coordinate-error" role="alert">Enter exactly three decimal XYZ numbers separated by spaces or commas. Commands and extra text are not accepted.</p>
    {/if}
    <label for="lt-coordinate-format">Output format</label>
    <select id="lt-coordinate-format" bind:value={mode}>
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
      />
      <button type="button" class="lt-secondary" disabled={!output} on:click={selectOutput}>Select text</button>
    </div>
    <small class="lt-coordinate-help">Press Ctrl+C to copy selected text. No game command is executed. Minecraft Education's built-in coordinate-copy shortcut is not verified.</small>
  </div>
</details>
