<script lang="ts">
  import type { ClientId, DisplayInfo, WindowLayout } from "../../contracts.js";
  import type { WindowLayoutPreference } from "../windowLayoutPreference.js";

  export let displays: DisplayInfo[];
  export let preference: WindowLayoutPreference;
  export let onCancel: () => void;
  export let onApply: (preference: WindowLayoutPreference) => void;
  export let onIdentify: (preference: WindowLayoutPreference) => void;
  export let onReset: () => WindowLayoutPreference;

  let draft: WindowLayoutPreference = structuredClone(preference);
  const clients: { id: ClientId; label: string }[] = [
    { id: "Native", label: "This PC" },
    { id: "Virtual-01", label: "Virtual 1" },
    { id: "Virtual-02", label: "Virtual 2" },
    { id: "Virtual-03", label: "Virtual 3" },
  ];
  const layouts: { id: WindowLayout; label: string }[] = [
    { id: "GRID", label: "Grid" },
    { id: "FOCUS", label: "Focus" },
    { id: "COLUMNS", label: "Columns" },
  ];
</script>

<div class="dialog-backdrop" role="presentation">
  <div class="layout-dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="window-layout-title">
    <header>
      <div><span class="eyebrow">WINDOW LAYOUT</span><h2 id="window-layout-title">Arrange Minecraft windows</h2></div>
      <button class="icon-close" aria-label="Close" on:click={onCancel}>×</button>
    </header>

    <div class="layout-section">
      <strong>Layout</strong>
      <div class="layout-options">
        {#each layouts as layout}
          <button class:active={draft.layout === layout.id} on:click={() => (draft.layout = layout.id)}>
            <span class="layout-preview {layout.id.toLowerCase()}"></span>
            {layout.label}
          </button>
        {/each}
      </div>
    </div>

    <label class="layout-field">Display
      <select bind:value={draft.displayIndex}>
        {#each displays as display}
          <option value={display.index}>Display {display.index + 1} · {display.width}×{display.height}{display.primary ? " · Primary" : ""}</option>
        {/each}
      </select>
    </label>

    {#if draft.layout === "FOCUS"}
      <label class="layout-field">Main window
        <select bind:value={draft.mainWindow}>
          {#each clients as client}<option value={client.id}>{client.label}</option>{/each}
        </select>
      </label>
    {/if}

    <div class="overlay-summary">
      <div><strong>Screen Overlay</strong><small>Show screen number and label on each arranged window.</small></div>
      <input type="checkbox" bind:checked={draft.overlay.enabled} aria-label="Show Screen Overlay" />
    </div>

    {#if draft.overlay.enabled}
      <details class="overlay-options">
        <summary>Overlay options</summary>
        <div class="overlay-controls">
          <label><input type="checkbox" bind:checked={draft.overlay.showScreenNumber} /> Screen number</label>
          <label><input type="checkbox" bind:checked={draft.overlay.showLabel} /> Label</label>
          <label>Position
            <select bind:value={draft.overlay.position}><option value="TOP_LEFT">Top left</option><option value="TOP_RIGHT">Top right</option></select>
          </label>
          {#each clients as client}
            <label>{client.label}<input maxlength="32" bind:value={draft.overlay.labels[client.id]} /></label>
          {/each}
        </div>
      </details>
    {/if}

    <footer><button on:click={() => (draft = onReset())}>Reset to defaults</button>{#if draft.overlay.enabled}<button class="identify-button" on:click={() => onIdentify(draft)}>Identify screens</button>{/if}<span class="dialog-spacer"></span><button on:click={onCancel}>Cancel</button><button class="primary" on:click={() => onApply(draft)}>Apply</button></footer>
  </div>
</div>
