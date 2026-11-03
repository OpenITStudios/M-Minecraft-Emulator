<script lang="ts">
  import type {F3InspectorView} from './app/presentation/f3Inspector.js';

  // One nonmodal read-only disclosure; never owns game/session state.
  export let view: F3InspectorView;
  let detailsElement: HTMLDetailsElement | undefined;
  let summaryElement: HTMLElement | undefined;

  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || !detailsElement?.open ||
        !(event.target instanceof Node) || !detailsElement.contains(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    detailsElement.open = false;
    summaryElement?.focus({preventScroll: true});
  }
</script>
<svelte:window on:keydown={closeOnEscape} />

<details class="lt-coordinate-paste lt-f3-inspector" bind:this={detailsElement}>
  <summary bind:this={summaryElement} aria-label="Open read-only F3 Inspector">F3 Inspector</summary>
  <div class="lt-coordinate-popup lt-f3-popup" role="group" aria-label="F3 Inspector for selected Minecraft">
    <strong>F3 Inspector</strong>
    <small class="lt-coordinate-status">Read-only · Native session observations</small>
    <p class="lt-f3-summary" role="status">{view.summary}</p>
    <dl class="lt-f3-data">
      <dt>Selected client</dt><dd>{view.clientLabel}</dd>
      <dt>Native state</dt><dd>{view.nativeState}</dd>
      <dt>Process PID</dt><dd>{view.processId}</dd>
      <dt>Windows session</dt><dd>{view.windowsSession}</dd>
      <dt>Game window</dt><dd>{view.nativeWindow}</dd>
      <dt>FPS limit</dt><dd>{view.configuredFps}</dd>
      <dt>Measured FPS</dt><dd>{view.measuredFps}</dd>
      <dt>Player XYZ</dt><dd>{view.coordinates}</dd>
      <dt>Rotation</dt><dd>{view.rotation}</dd>
      <dt>Dimension</dt><dd>{view.dimension}</dd>
    </dl>
    <p class="lt-f3-disclaimer">Windows process details do not identify the player or world. Minecraft Education does not include Java's built-in F3 debug screen. Use Show Coordinates in Minecraft's world settings; Paste XYZ is a separate manual text tool.</p>
  </div>
</details>
