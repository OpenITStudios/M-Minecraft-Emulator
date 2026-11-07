<script lang="ts">
  import {tick} from 'svelte';
  import type {F3InspectorView} from './app/presentation/f3Inspector.js';
  import {playToolPopupStyle} from './app/presentation/playToolPopup.js';

  // One nonmodal read-only disclosure; never owns game/session state.
  export let view: F3InspectorView;
  let detailsElement: HTMLDetailsElement | undefined;
  let summaryElement: HTMLElement | undefined;
  let popupStyle = 'visibility:hidden;';

  function repositionPopup(event?: Event): void {
    if (!detailsElement?.open || !summaryElement) return;
    if (event?.type === 'scroll' && event.target instanceof Node && detailsElement.contains(event.target)) return;
    const anchor = summaryElement.getBoundingClientRect();
    // Close an offscreen nonmodal tool rather than leaving a floating panel
    // over the five native preview cards or selected Minecraft viewport.
    if (anchor.bottom <= 0 || anchor.top >= window.innerHeight ||
        anchor.right <= 0 || anchor.left >= window.innerWidth) {
      detailsElement.open = false;
      return;
    }
    popupStyle = playToolPopupStyle(anchor,
      {width:window.innerWidth,height:window.innerHeight},420) + 'visibility:visible;';
  }
  function onToggle(): void {
    if (detailsElement?.open) repositionPopup();
    else popupStyle = 'visibility:hidden;';
  }
  function closeWhenFocusLeaves(event: FocusEvent): void {
    if (!detailsElement?.open) return;
    const next = event.relatedTarget;
    // Tab/Shift+Tab must remain native browser navigation; never restore
    // focus on an ordinary move to another control.
    if (next instanceof Node) {
      if (!detailsElement.contains(next)) detailsElement.open = false;
      return;
    }
    // Focus leaving WebView can have relatedTarget=null. Recheck only after
    // browser focus settles, so an internal transition with a null target
    // does not prematurely dismiss the disclosure.
    void tick().then(() => {
      if (!detailsElement?.open) return;
      const focused = document.activeElement;
      if (!document.hasFocus() || !(focused instanceof Node) ||
          !detailsElement.contains(focused)) detailsElement.open = false;
    });
  }
  function closeOnWindowBlur(): void {
    // A native Minecraft focus transfer must leave no floating Play popup.
    // Unlike Escape, it never re-focuses the WebView or game.
    if (detailsElement?.open) detailsElement.open = false;
  }

  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || !detailsElement?.open ||
        !(event.target instanceof Node) || !detailsElement.contains(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    detailsElement.open = false;
    if (document.hasFocus() && summaryElement?.isConnected)
      summaryElement?.focus({preventScroll: true});
  }
</script>
<svelte:window on:keydown={closeOnEscape} on:blur={closeOnWindowBlur} on:resize={repositionPopup} on:scroll|capture={repositionPopup} />

<details class="lt-coordinate-paste lt-f3-inspector" name="lt-play-disclosures" bind:this={detailsElement} on:toggle={onToggle} on:focusout={closeWhenFocusLeaves}>
  <summary bind:this={summaryElement} aria-label="Open read-only F3 Inspector">F3 Inspector</summary>
  <div class="lt-coordinate-popup lt-f3-popup" style={popupStyle} role="group" aria-label="F3 Inspector for selected Minecraft">
    <strong>F3 Inspector</strong>
    <small class="lt-coordinate-status">Read-only · Last native session observation</small>
    <p class="lt-f3-summary" role="status">{view.summary}</p>
    <dl class="lt-f3-data">
      <dt>Selected client</dt><dd>{view.clientLabel}</dd>
      <dt>Native state</dt><dd>{view.nativeState}</dd>
      <dt>Process PID</dt><dd>{view.processId}</dd>
      <dt>Windows session</dt><dd>{view.windowsSession}</dd>
      <dt>Game window</dt><dd>{view.nativeWindow}</dd>
      <dt>FPS targets (last report)</dt><dd>{view.configuredFps}</dd>
      <dt>Measured FPS</dt><dd>{view.measuredFps}</dd>
      <dt>Player XYZ</dt><dd>{view.coordinates}</dd>
      <dt>Rotation</dt><dd>{view.rotation}</dd>
      <dt>Dimension</dt><dd>{view.dimension}</dd>
    </dl>
    <p class="lt-f3-disclaimer">Windows process details do not identify the player or world. Minecraft Education does not include Java's built-in F3 debug screen. Use Show Coordinates in Minecraft's world settings; Paste XYZ is a separate manual text tool.</p>
  </div>
</details>
