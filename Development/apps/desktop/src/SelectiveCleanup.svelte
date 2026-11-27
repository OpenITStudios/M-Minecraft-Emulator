<script lang="ts">
  import {onMount, onDestroy, tick} from 'svelte';
  import type {ProfileSlot, SelectiveCategory, SelectiveInventory, SelectiveSelection, SelectiveReviewReceipt} from './app/bridge/lazyTester.js';
  import {formatStorageBytes} from './app/presentation/storageSize.js';

  export let slot: ProfileSlot;
  export let inspect: (slot: ProfileSlot, category: SelectiveCategory) => Promise<SelectiveInventory>;
  export let review: (slot: ProfileSlot, selections: SelectiveSelection[]) => Promise<SelectiveReviewReceipt>;
  export let onClose: () => void;

  const categories: {value: SelectiveCategory; label: string}[] = [
    {value:'Worlds',label:'Worlds'}, {value:'ResourcePacks',label:'Resource Packs'}, {value:'BehaviorPacks',label:'Behavior Packs'}
  ];
  let dialog: HTMLDialogElement;
  let closeButton: HTMLButtonElement;
  let reviewHeading: HTMLHeadingElement | undefined;
  let closed = false;
  let category: SelectiveCategory = 'Worlds';
  let inventories: Partial<Record<SelectiveCategory, SelectiveInventory>> = {};
  let selected: SelectiveSelection[] = [];
  let receipt: SelectiveReviewReceipt | null = null;
  let loading = false;
  let issue = '';
  let generation = 0;
  $: items = inventories[category]?.items ?? [];
  $: allSelected = items.length>0 && items.every(item=>selected.some(s=>s.identity===item.identity));

  onMount(()=>{
    dialog.showModal();
    closeButton.focus({preventScroll:true});
    void load('Worlds');
  });
  function close(): void {
    if(closed)return;
    closed=true;
    generation++;
    if(dialog?.open)dialog.close();
    onClose();
  }
  onDestroy(()=>{
    closed=true;
    generation++;
    if(dialog?.open)dialog.close();
  });
  async function load(next: SelectiveCategory, refresh=false): Promise<void> {
    if(closed||loading)return;
    category=next;
    // Review totals belong to the exact selection, not the visible category.
    receipt=null;
    issue='';
    if(inventories[next]&&!refresh)return;
    const request=++generation;
    loading=true;
    try {
      const result=await inspect(slot,next);
      if(request!==generation)return;
      inventories={...inventories,[next]:result};
      // A refresh cannot preserve a selection whose native identity changed.
      selected=selected.filter(s=>s.category!==next||result.items.some(i=>
        i.identity===s.identity&&i.root===s.root&&i.name===s.name));
      receipt=null;
    } catch {
      if(request===generation) {
        inventories={...inventories,[next]:undefined};
        selected=selected.filter(s=>s.category!==next);
        receipt=null;
        issue='Inventory unavailable. Refresh this category to try again.';
      }
    } finally {if(request===generation)loading=false;}
  }
  function toggle(item: SelectiveInventory['items'][number], checked: boolean): void {
    if(closed||loading)return;
    receipt=null;
    issue='';
    if(checked && selected.length<128 && !selected.some(s=>s.identity===item.identity))
      selected=[...selected,{category,...item}];
    else if(!checked)selected=selected.filter(s=>s.identity!==item.identity);
  }
  function selectAll(): void {
    if(closed||loading)return;
    receipt=null;
    issue='';
    const others=selected.filter(s=>s.category!==category);
    if(allSelected){selected=others;return;}
    if(others.length+items.length>128){issue='Select up to 128 items per review.';return;}
    selected=[...others,...items.map(item=>({category,...item}))];
    issue='';
  }
  async function inspectSelection(): Promise<void> {
    if(closed||loading||!selected.length)return;
    const request=++generation;
    loading=true;issue='';receipt=null;
    try {
      const result=await review(slot,selected.slice());
      if(request===generation){
        receipt=result;
        await tick();
        if(request===generation&&reviewHeading?.isConnected)reviewHeading.focus({preventScroll:true});
      }
    } catch {
      if(request===generation){
        // No trustworthy receipt: discard cached selection across every category.
        inventories={};selected=[];
        issue='Review unavailable or data changed. Refresh inventory and select data again.';
      }
    }
    finally {if(request===generation)loading=false;}
  }
</script>

<dialog class="lazytester-selective-dialog" bind:this={dialog} aria-labelledby="lazytester-selective-title" on:cancel|preventDefault={close}>
  <header class="lazytester-selective-heading">
    <div><h2 id="lazytester-selective-title">Selective Cleanup</h2><p>Minecraft {slot}</p></div>
    <button class="lazytester-secondary" bind:this={closeButton} on:click={close}>Close</button>
  </header>
  <div class="lazytester-selective-categories" role="group" aria-label="Data category">
    {#each categories as entry}
      <button class="lazytester-secondary" aria-pressed={category===entry.value} disabled={loading} on:click={()=>load(entry.value)}>{entry.label}</button>
    {/each}
    <button class="lazytester-secondary" disabled title="Download cache location is not verified">Download Cache</button>
  </div>
  <div class="lazytester-selective-toolbar">
    <button class="lazytester-secondary" disabled={loading||!items.length} on:click={selectAll}>{allSelected?'Deselect All':'Select All'}</button>
    <button class="lazytester-secondary" disabled={loading} on:click={()=>load(category,true)}>Refresh</button>
  </div>
  <div class="lazytester-selective-items" aria-busy={loading}>
    {#if loading}<p role="status">Inspecting data…</p>{/if}
    {#if !loading && inventories[category] && !items.length}<p>No items found.</p>{/if}
    {#if !loading && !inventories[category]}<p>Inventory not available. Select Refresh to inspect this category.</p>{/if}
    {#each items as item (item.identity)}
      <label class="lazytester-selective-item"><input type="checkbox" checked={selected.some(s=>s.identity===item.identity)} disabled={loading||(!selected.some(s=>s.identity===item.identity)&&selected.length>=128)} on:change={event=>toggle(item,event.currentTarget.checked)}/><span><strong>{item.name}</strong><small>{item.root}</small></span></label>
    {/each}
  </div>
  {#if issue}<p class="lazytester-field-error" role="status">{issue}</p>{/if}
  {#if receipt}
    <section class="lazytester-cache-confirm" aria-label="Selection review">
      <h3 bind:this={reviewHeading} tabindex="-1">Review selected data</h3>
      <p>{receipt.items.length} items · {receipt.files} files · {formatStorageBytes(receipt.bytes)}</p>
      <details>
        <summary>Reviewed items ({receipt.items.length})</summary>
        <ul aria-label="Reviewed items">
          {#each receipt.items as item (item.identity)}
            <li>{categories.find(entry=>entry.value===item.category)?.label}: {item.name} ({item.root})</li>
          {/each}
        </ul>
      </details>
      {#each receipt.warnings as warning}<p class="lazytester-help">{warning}</p>{/each}
      <p role="status">Deletion is unavailable until native validation is complete. No data has been changed.</p>
    </section>
  {/if}
  <footer class="lazytester-selective-footer">
    <span>{selected.length} selected{receipt?' · '+formatStorageBytes(receipt.bytes):''}</span>
    <button class="lazytester-secondary" disabled={loading||!selected.length} on:click={inspectSelection}>Review Selection</button>
  </footer>
</dialog>
