<script lang="ts">
  import {onMount, onDestroy, tick} from 'svelte';
  import type {ProfileStorage, ProfileSlot, SelectiveCategory, SelectiveInventory, SelectiveSelection, SelectiveReviewReceipt} from './app/bridge/lazyTester.js';
  import {formatStorageBytes} from './app/presentation/storageSize.js';

  export let slot: ProfileSlot;
  export let inspect: (slot: ProfileSlot, category: SelectiveCategory) => Promise<SelectiveInventory>;
  export let review: (slot: ProfileSlot, selections: SelectiveSelection[]) => Promise<SelectiveReviewReceipt>;
  export let onClose: () => void;
  export let downloadCache: ProfileStorage['cache'] | undefined = undefined;
  export let confirmDownloadCache: (slot: ProfileSlot) => Promise<void>;
  let downloadView=false;
  let downloadAck=false;
  $: downloadEligible=!!downloadCache?.canClear&&!downloadCache.partial&&downloadCache.issue===null&&downloadCache.files>0;
  $: if(!downloadEligible)downloadAck=false;
  export let confirmWorlds: (slot: ProfileSlot, receipt: SelectiveReviewReceipt) => Promise<void>;
  // Source-only until native disposition acceptance; never imply deletion is available.
  export let worldDeletionAvailable = false;
  export let packDeletionAvailable = false;
  let acknowledged = false;
  let deleting = false;
  $: worldsOnly=!!receipt?.items.length&&receipt.items.every(item=>item.category==='Worlds');
  $: packsOnly=!!receipt?.items.length&&receipt.items.every(item=>item.category!=='Worlds');
  $: eligible = !!receipt?.confirmationToken && receipt.warnings.length===0 && (worldsOnly||packsOnly);
  $: deletionAvailable=worldsOnly?worldDeletionAvailable:packsOnly&&packDeletionAvailable;
  $: if (!receipt) acknowledged=false;

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
    if(closed||deleting)return;
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
    downloadView=false;downloadAck=false;
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
  function showDownloadCache():void {
    if(closed||loading||deleting)return;
    generation++;
    downloadView=true;downloadAck=false;receipt=null;issue='';selected=[];
  }
  async function deleteDownloadCache():Promise<void> {
    if(closed||loading||deleting||!downloadView||!downloadEligible||!downloadAck)return;
    const request=++generation;
    deleting=true;loading=true;downloadAck=false;issue='';
    try {
      await confirmDownloadCache(slot);
      if(request===generation)issue='Download cache cleanup completed.';
    } catch {
      if(request===generation)issue='Download cache cleanup was refused or its result is uncertain. Refresh session and storage before retrying.';
    } finally {
      if(request===generation){deleting=false;loading=false;}
    }
  }
  async function deleteWorlds(): Promise<void> {
    if(closed||loading||deleting||!deletionAvailable||!eligible||!acknowledged||!receipt)return;
    const confirmed=receipt;
    const request=++generation;
    deleting=true;loading=true;receipt=null;acknowledged=false;issue='';
    try {
      await confirmWorlds(slot,confirmed);
      if(request!==generation)return;
      inventories={};selected=[];
      loading=false;
      await load(category,true);
      if(!closed && !issue)issue='Selected data deleted.';
    } catch {
      if(request===generation){
        inventories={};selected=[];
        issue='Cleanup was refused or its result is uncertain. Refresh session and inventory before reviewing again. Do not retry automatically.';
      }
    } finally {if(!closed){deleting=false;loading=false;}}
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
    <button class="lazytester-secondary" bind:this={closeButton} disabled={deleting} on:click={close}>Close</button>
  </header>
  <div class="lazytester-selective-categories" role="group" aria-label="Data category">
    {#each categories as entry}
      <button class="lazytester-secondary" aria-pressed={!downloadView&&category===entry.value} disabled={loading} on:click={()=>load(entry.value)}>{entry.label}</button>
    {/each}
    <button class="lazytester-secondary" aria-pressed={downloadView} disabled={loading} on:click={showDownloadCache}>Download Cache</button>
  </div>
  {#if downloadView}
    <section class="lazytester-cache-confirm" aria-label="Download cache review">
      <h3>Download Cache — Minecraft {slot}</h3>
      {#if downloadCache&&!downloadCache.partial}
        <p>{downloadCache.files} files · {formatStorageBytes(downloadCache.bytes)}</p>
      {:else}
        <p>Size unavailable. Unknown does not mean empty.</p>
      {/if}
      {#if !downloadEligible}
        <p role="status">{downloadCache?.issue??'Download cache cleanup is unavailable. Refresh storage to check again.'}</p>
      {:else}
        <p>Only the reviewed download cache for this launcher will be cleared. Worlds, packs, accounts and web cache are not targeted.</p>
        <label><input type="checkbox" bind:checked={downloadAck} disabled={loading}/> I understand this will remove downloaded cache files.</label>
        <button class="lazytester-danger" disabled={loading||!downloadAck||!downloadEligible} on:click={deleteDownloadCache}>Clear Download Cache</button>
      {/if}
      {#if deleting}<p role="status">Clearing download cache…</p>{/if}
      {#if issue}<p role="status">{issue}</p>{/if}
    </section>
  {:else}
  <div class="lazytester-selective-toolbar">
    <button class="lazytester-secondary" disabled={loading||!items.length} on:click={selectAll}>{allSelected?'Deselect All':'Select All'}</button>
    <button class="lazytester-secondary" disabled={loading} on:click={()=>load(category,true)}>Refresh</button>
  </div>
  <div class="lazytester-selective-items" aria-busy={loading}>
    {#if loading}<p role="status">{deleting?'Deleting selected data…':'Inspecting data…'}</p>{/if}
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
      {#if eligible}
        <p>Selected {worldsOnly?'worlds':'packs'} will be permanently deleted. Other profile data is not targeted.</p>
        <label><input type="checkbox" bind:checked={acknowledged} disabled={!deletionAvailable||loading}/> I understand this action cannot be undone.</label>
        <button class="lazytester-danger" disabled={!deletionAvailable||!acknowledged||loading} on:click={deleteWorlds}>Delete Selected {worldsOnly?'Worlds':'Packs'}</button>
      {/if}
      {#if receipt.items.some(item=>item.category!=='Worlds') && !deletionAvailable}
        <p role="status">Pack cleanup is review-only until native dependency and global activation coverage are verified. No data has been changed by this review.</p>
      {:else if !deletionAvailable || !eligible}
        <p role="status">Deletion is unavailable until native validation is complete. No data has been changed by this review.</p>
      {/if}
    </section>
  {/if}
  <footer class="lazytester-selective-footer">
    <span>{selected.length} selected{receipt?' · '+formatStorageBytes(receipt.bytes):''}</span>
    <button class="lazytester-secondary" disabled={loading||!selected.length} on:click={inspectSelection}>Review Selection</button>
  </footer>
  {/if}
</dialog>
