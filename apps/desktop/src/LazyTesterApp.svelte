<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, nextProfile, type ProfileSession, type ProfileSlot, type ProfileStorage, type PreviewStatus} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazy-tester.css';

  let profiles: ProfileSession[] = [], storage: ProfileStorage[] | null = null;
  let clearSlot: ProfileSlot | null = null, storageNotice = '';
  let page: 'workspace' | 'launchers' | 'storage' | 'settings' = 'workspace';
  let active: ProfileSlot | null = null, requested: ProfileSlot | null = null, queued: ProfileSlot | null = null;
  let viewport: HTMLDivElement, observer: ResizeObserver | undefined;
  let busy = false, refreshing = false, resizing = false, available = false;
  let mounted = false, disposed = false, dirty = false, detached = false;
  let issue = '', connectionIssue = '', previewIssue = '', selectedIdentity = '';
  let previewStatuses: PreviewStatus[] = [];
  const images = new Map<ProfileSlot, HTMLElement>();
  const labels = {STOPPED: 'Belum dibuka', STARTING: 'Sedang membuka', RUNNING: 'Berjalan', NOT_RESPONDING: 'Tidak merespons', ATTENTION: 'Perlu perhatian'};
  const identity = (profile?: ProfileSession) => profile?.process ? [profile.process.pid, profile.process.createdFileTime, profile.process.sessionId, profile.process.windowHandle].join('/') : '';
  $: primary = active ?? requested ?? profiles.find(profile => profile.state === 'RUNNING')?.slot ?? 1;
  $: others = profiles.filter(profile => profile.slot !== primary);
  $: runningCount = profiles.filter(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)).length;
  $: playableCount = profiles.filter(profile => profile.state === 'RUNNING').length;
  $: if (mounted && !busy && !refreshing && !resizing && queued !== null) void consumeSelection();
  $: if (mounted && !busy && !refreshing && !resizing && queued === null && dirty) void syncLayout();

  function image(node: HTMLElement, slot: ProfileSlot) {
    images.set(slot, node); observer?.observe(node); dirty = true;
    return {destroy() { observer?.unobserve(node); if (images.get(slot) === node) images.delete(slot); dirty = true; }};
  }
  function select(slot: ProfileSlot) { requested = null; queued = slot; }
  async function consumeSelection() {
    const slot = queued; queued = null;
    if (slot === null || !available || profiles.find(profile => profile.slot === slot)?.state !== 'RUNNING') return;
    busy = true; issue = '';
    try {
      page = 'workspace'; await tick();
      await lazyTester.present(slot, viewport.getBoundingClientRect());
      active = slot; selectedIdentity = identity(profiles.find(profile => profile.slot === slot));
      detached = false; requested = null; dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function refresh() {
    if (busy || refreshing || resizing || disposed) return;
    refreshing = true;
    try {
      const snapshot = await lazyTester.inspect();
      if (disposed) return;
      profiles = snapshot.profiles; available = true; connectionIssue = '';
      if (active !== null && selectedIdentity !== identity(profiles.find(profile => profile.slot === active))) {
        await workspaceSurface.release(); active = null; selectedIdentity = '';
      }
      if (requested !== null) {
        const profile = profiles.find(profile => profile.slot === requested);
        if (profile?.state === 'RUNNING') queued = requested;
        else if (profile?.state === 'ATTENTION' || profile?.state === 'STOPPED') {
          issue = profile.issue ?? ('Minecraft - ' + requested + ' berhenti sebelum ditampilkan.'); requested = null;
        }
      } else if (active === null && !detached && page === 'workspace' && queued === null) {
        queued = profiles.find(profile => profile.state === 'RUNNING')?.slot ?? null;
      }
      dirty = true;
    } catch (error) { if (!disposed) { available = false; connectionIssue = String(error); dirty = true; } }
    finally { refreshing = false; }
  }
  async function launch(slot: ProfileSlot) {
    if (busy || refreshing || resizing) return;
    busy = true; issue = ''; requested = slot;
    try {
      profiles = (await lazyTester.launch(slot)).profiles; storage = null; available = true;
      detached = false; page = 'workspace'; dirty = true;
      if (requested === slot && profiles.find(profile => profile.slot === slot)?.state === 'RUNNING') queued = slot;
      else if (active !== null) {
        await tick();
        await lazyTester.present(active, viewport.getBoundingClientRect());
      }
    } catch (error) { issue = String(error); requested = null; }
    finally { busy = false; }
  }
  async function navigate(next: typeof page) {
    if (busy || refreshing || resizing || page === next) return;
    busy = true; issue = '';
    try {
      if (next !== 'workspace') await workspaceSurface.suspend();
      page = next; await tick();
      if (next === 'workspace' && active !== null) await lazyTester.present(active, viewport.getBoundingClientRect());
      dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function release() {
    if (busy || refreshing || resizing) return;
    busy = true;
    try {
      await workspaceSurface.release(); active = null; selectedIdentity = ''; requested = null;
      detached = true; previewStatuses = []; dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function syncLayout() {
    if (busy || refreshing || resizing || !mounted) return;
    dirty = false; resizing = true;
    try {
      if (page !== 'workspace' || detached || !available || document.hidden) {
        await lazyTester.clearPreviews(); previewStatuses = []; return;
      }
      await tick();
      if (active !== null) await workspaceSurface.update(viewport.getBoundingClientRect());
      const tiles = others.flatMap(profile => {
        const element = images.get(profile.slot); if (!element) return [];
        const rect = element.getBoundingClientRect();
        return rect.x >= 0 && rect.y >= 0 && rect.width >= 32 && rect.height >= 24 &&
          rect.right <= window.innerWidth && rect.bottom <= window.innerHeight ? [{slot: profile.slot, rect}] : [];
      });
      previewStatuses = (await lazyTester.previews(viewport.getBoundingClientRect(), tiles)).tiles;
      previewIssue = '';
    } catch (error) { previewIssue = String(error); }
    finally { resizing = false; }
  }
  function step(direction: 1 | -1) {
    const next = nextProfile(profiles, active, direction); if (next !== null) select(next);
  }
  async function inspectStorage() {
    if (busy || refreshing || resizing) return;
    busy = true; issue = ''; clearSlot = null; storageNotice = '';
    try { const snapshot = await lazyTester.storage(); profiles = snapshot.profiles; storage = snapshot.storage; }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function openFolder(slot: ProfileSlot) {
    if (busy || refreshing || resizing) return;
    busy = true; issue = '';
    try { await lazyTester.openFolder(slot); }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function clearCache(slot: ProfileSlot) {
    if (busy || refreshing || resizing || clearSlot !== slot || runningCount > 0) return;
    busy = true; issue = ''; storageNotice = ''; clearSlot = null;
    try {
      const snapshot = await lazyTester.clearCache(slot);
      profiles = snapshot.profiles; storage = snapshot.storage;
      storageNotice = 'Cache web Minecraft - ' + slot + ' telah dibersihkan. Cache dapat dibuat kembali saat dibutuhkan.';
    } catch (error) {
      issue = String(error); storage = null;
    } finally { busy = false; }
  }
  onMount(() => {
    mounted = true; void refresh();
    const interval = setInterval(() => { if (!document.hidden) void refresh(); }, 2000);
    observer = new ResizeObserver(() => { dirty = true; }); observer.observe(viewport, {box: 'device-pixel-content-box'});
    for (const node of images.values()) observer.observe(node);
    const changed = () => { dirty = true; };
    window.addEventListener('resize', changed); document.addEventListener('visibilitychange', changed);
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => { if (disposed) stop(); else stops.push(stop); };
    void listen<string>('workspace-surface-error', event => { issue = event.payload; }).then(retain).catch(error => { issue = String(error); });
    void listen<number>('lazy-tester-preview-select', event => {
      if ([1,2,3,4,5].includes(event.payload)) select(event.payload as ProfileSlot);
    }).then(retain).catch(error => { previewIssue = String(error); });
    return () => {
      mounted = false; disposed = true; clearInterval(interval); observer?.disconnect();
      window.removeEventListener('resize', changed); document.removeEventListener('visibilitychange', changed);
      stops.forEach(stop => stop()); void workspaceSurface.release().catch(() => {});
    };
  });
</script>

<svelte:head><title>Lazy Tester</title></svelte:head>
<div class="lt-app">
  <aside class="lt-sidebar">
    <a class="lt-brand" href="#workspace" on:click|preventDefault={() => navigate('workspace')}><span>LT</span> Lazy Tester</a>
    <p class="lt-caption">MINECRAFT EDUCATION</p>
    <nav aria-label="Navigasi utama">
      {#each [['workspace','Workspace'],['launchers','Launcher'],['storage','Penyimpanan'],['settings','Pengaturan']] as [id,label]}
        <button class:chosen={page === id} disabled={busy || resizing} on:click={() => navigate(id as typeof page)}>{label}</button>
      {/each}
    </nav>
    <div class="lt-sidebar-bottom"><span class="lt-dot" class:waiting={!available}></span>{available ? 'Runtime terhubung' : 'Memeriksa koneksi'}<p>{available ? runningCount + ' dari 5 Minecraft berjalan' : 'Status sesi belum tersedia'}</p></div>
  </aside>
  <main>
    <header class="lt-page-header"><div><p class="lt-caption">RUANG BERMAIN ANDA</p><h1>{page === 'workspace' ? 'Workspace' : page === 'launchers' ? 'Launcher Minecraft' : page === 'storage' ? 'Penyimpanan' : 'Pengaturan'}</h1></div><button class="lt-secondary" disabled={busy || refreshing || resizing} on:click={refresh}>Perbarui status</button></header>
    {#if issue || connectionIssue}<div class="lt-alert" role="alert"><strong>Operasi membutuhkan perhatian</strong>{#if issue}<p>{issue}</p>{/if}{#if connectionIssue}<p>{connectionIssue}</p>{/if}<span>Game dan data yang sudah ada tetap dipertahankan.</span></div>{/if}
    <section class="lt-workspace" hidden={page !== 'workspace'} aria-label="Workspace Minecraft">
      <header class="lt-stage-header"><div><h2>{active ? 'Minecraft - ' + active : requested ? 'Membuka Minecraft - ' + requested : 'Pilih Minecraft untuk mulai'}</h2><p>{active ? 'Sedang ditampilkan · klik di dalam game untuk mengontrol' : detached ? 'Jendela telah dikembalikan ke Windows.' : 'Buka profil pilihan Anda melalui halaman Launcher.'}</p></div>
        <div class="lt-actions"><button class="lt-secondary" disabled={busy || !available || !playableCount} on:click={() => step(-1)} aria-label="Minecraft sebelumnya">←</button><button class="lt-secondary" disabled={busy || !available || !playableCount} on:click={() => step(1)}>Berikutnya →</button>{#if runningCount}<button class="lt-secondary" disabled={busy || resizing} on:click={release}>Lepas jendela</button>{/if}</div>
      </header>
      <div class="lt-viewport" bind:this={viewport}>{#if !active}<div class="lt-empty"><span>LT</span><h2>{requested ? 'Menyiapkan Minecraft - ' + requested : 'Semua Minecraft, satu tempat'}</h2><p>{requested ? 'Sesi akan tampil setelah jendelanya tersedia.' : 'Setiap profil memiliki login dan penyimpanan sendiri.'}</p>{#if !requested}<button on:click={() => navigate('launchers')}>Pilih launcher</button>{/if}</div>{/if}</div>
      <div class="lt-filmstrip" aria-label="Minecraft lainnya">
        {#each others as profile (profile.slot)}
          <button class="lt-card" aria-label={'Kontrol Minecraft - ' + profile.slot} disabled={busy || !available || profile.state !== 'RUNNING'} on:click={() => select(profile.slot)}>
            <span class="lt-card-picture" use:image={profile.slot} aria-hidden="true">{profile.slot}</span><strong>Minecraft - {profile.slot}</strong><small>{labels[profile.state]}</small>
            {#if profile.state === 'RUNNING' && previewStatuses.find(tile => tile.slot === profile.slot)?.reason}<small class="lt-preview-fallback" title={previewStatuses.find(tile => tile.slot === profile.slot)?.reason ?? ''}>Preview belum tersedia · klik untuk kontrol</small>{/if}
          </button>
        {/each}
      </div>
      <p class="lt-help">{resizing ? 'Menyesuaikan workspace…' : 'Klik preview untuk mengganti Minecraft yang dikontrol. Sesi lain memakai batas 10 FPS di latar.'}</p>
      {#if previewIssue}<details class="lt-preview-details"><summary>Preview membutuhkan perhatian</summary><p>{previewIssue}</p></details>{/if}
    </section>
    {#if page === 'launchers'}
      <p class="lt-description">Buka hanya Minecraft yang ingin Anda gunakan. Tidak ada game yang dibuka otomatis.</p>
      <div class="lt-profile-list">{#each profiles as profile (profile.slot)}<article><span class="lt-profile-number">{profile.slot}</span><div><h2>Minecraft - {profile.slot}</h2><p>{labels[profile.state]}</p>{#if profile.issue}<p class="lt-error">{profile.issue}</p>{/if}</div>{#if profile.state === 'RUNNING'}<button disabled={busy || !available} on:click={() => select(profile.slot)}>Tampilkan</button>{:else}<button disabled={busy || refreshing || resizing || !available || ['STARTING','NOT_RESPONDING'].includes(profile.state)} on:click={() => launch(profile.slot)}>{profile.state === 'STARTING' ? 'Membuka…' : profile.state === 'NOT_RESPONDING' ? 'Tidak merespons' : 'Buka Minecraft'}</button>{/if}</article>{/each}</div>
    {:else if page === 'storage'}
      <section class="lt-panel">
        <h2>Penyimpanan Minecraft</h2>
        <p>Akun, dunia, dan pengaturan tetap berada di profil masing-masing. Ukuran dihitung saat diminta dan dapat berubah selama bermain.</p>
        <p><button disabled={busy || refreshing || resizing} on:click={inspectStorage}>{busy ? 'Memproses…' : 'Hitung penggunaan ruang'}</button></p>
        {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
        {#if storage}
          {#each storage as item}
            <article class="lt-storage-profile">
              <h3>Minecraft - {item.slot}</h3>
              <div class="lt-storage-metrics">
                <div><span>Data profil</span><strong>{(item.bytes / 1048576).toFixed(1)} MB</strong><small>{item.files.toLocaleString('id-ID')} file{item.partial ? ' · perhitungan sebagian' : ''}</small></div>
                <div><span>Ruang disk tersedia</span><strong>{item.disk ? (item.disk.availableBytes / 1073741824).toFixed(1) + ' GB' : 'Tidak tersedia'}</strong><small>{item.disk ? 'dari ' + (item.disk.totalBytes / 1073741824).toFixed(1) + ' GB · dipakai bersama' : 'Windows belum memberikan informasi disk'}</small></div>
                <div><span>Cache web yang dikenali</span><strong>{item.cache.partial ? 'Belum dapat dihitung' : (item.cache.bytes / 1048576).toFixed(1) + ' MB'}</strong><small>Termasuk dalam ukuran profil</small></div>
              </div>
              {#if item.cache.issue}<p>{item.cache.issue}</p>{/if}
              <div class="lt-actions">
                <button class="lt-secondary" disabled={busy || refreshing || resizing} on:click={() => openFolder(item.slot)}>Buka folder</button>
                <button class="lt-secondary" disabled={busy || refreshing || resizing || runningCount > 0 || !item.cache.canClear} on:click={() => clearSlot = item.slot}>Bersihkan cache</button>
              </div>
              {#if clearSlot === item.slot && item.cache.canClear && runningCount === 0}
                <div class="lt-cache-confirm" role="group" aria-label={'Konfirmasi pembersihan cache Minecraft - ' + item.slot}>
                  <p>Hapus {(item.cache.bytes / 1048576).toFixed(1)} MB cache web Minecraft - {item.slot}? Login, dunia, pengaturan, dan cache lain tetap dipertahankan. Pembukaan konten web berikutnya mungkin lebih lambat.</p>
                  <div class="lt-actions"><button disabled={busy || refreshing || resizing} on:click={() => clearCache(item.slot)}>Hapus cache terpilih</button><button class="lt-secondary" disabled={busy} on:click={() => clearSlot = null}>Batal</button></div>
                </div>
              {/if}
            </article>
          {/each}
        {/if}
        <p>Pembersihan hanya mencakup cache HTTP Code Builder yang dikenali. Semua Minecraft harus ditutup normal; berkas terkunci, tautan, dan data yang belum dikenali tidak dibersihkan.</p>
      </section>
    {:else if page === 'settings'}
      <section class="lt-panel"><h2>Performa</h2><div class="lt-setting"><span>FPS saat game aktif</span><strong>144</strong></div><div class="lt-setting"><span>FPS saat game di latar</span><strong>10</strong></div><p>Angka di atas adalah batas FPS. Counter di dalam game menunjukkan hasil aktual.</p><h2>Preset grafis</h2><p>Simple · jarak render 12 chunk · efek visual hemat. Diterapkan sebelum profil dibuka; jarak simulasi dipertahankan.</p><h2>Saat aplikasi ditutup</h2><p>Minecraft tetap berjalan. Jendela yang ditampilkan dikembalikan ke Windows.</p></section>
    {/if}
  </main>
</div>
