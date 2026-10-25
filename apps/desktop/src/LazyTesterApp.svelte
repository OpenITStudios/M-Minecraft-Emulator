<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, nextProfile, type ProfileSession, type ProfileSlot, type ProfileStorage} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazy-tester.css';

  let profiles: ProfileSession[] = [];
  let storage: ProfileStorage[] | null = null;
  let page: 'workspace' | 'launchers' | 'storage' | 'settings' = 'workspace';
  let active: ProfileSlot | null = null;
  let busy = false, issue = '', available = false;
  let viewport: HTMLDivElement;
  let disposed = false, refreshing = false, resizing = false;
  let selectedIdentity = '';
  const labels = {STOPPED: 'Belum dibuka', STARTING: 'Sedang membuka', RUNNING: 'Berjalan', NOT_RESPONDING: 'Tidak merespons', ATTENTION: 'Perlu perhatian'};
  $: selected = profiles.find(profile => profile.slot === active);
  $: others = profiles.filter(profile => profile.slot !== active);
  $: runningCount = profiles.filter(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)).length;
  const identity = (profile?: ProfileSession) => profile?.process ?
    `${profile.process.pid}/${profile.process.createdFileTime}/${profile.process.sessionId}/${profile.process.windowHandle}` : '';

  async function refresh() {
    if (busy || refreshing || resizing || disposed) return;
    refreshing = true;
    try {
      const snapshot = await lazyTester.inspect();
      if (disposed) return;
      profiles = snapshot.profiles; available = true;
      if (active !== null && selectedIdentity !== identity(profiles.find(profile => profile.slot === active))) {
        await workspaceSurface.release(); active = null; selectedIdentity = '';
      }
    } catch (error) { if (!disposed) { available = false; issue = String(error); } }
    finally { refreshing = false; }
  }
  async function launch(slot: ProfileSlot) {
    if (busy || refreshing || resizing) return;
    busy = true; issue = '';
    try { profiles = (await lazyTester.launch(slot)).profiles; storage = null; available = true; }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function select(slot: ProfileSlot) {
    if (busy || refreshing || !available) return;
    const profile = profiles.find(profile => profile.slot === slot);
    if (profile?.state !== 'RUNNING') return;
    busy = true; issue = '';
    try {
      page = 'workspace'; await tick();
      await lazyTester.present(slot, viewport.getBoundingClientRect());
      active = slot; selectedIdentity = identity(profile);
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function navigate(next: typeof page) {
    if (busy || refreshing || page === next) return;
    busy = true; issue = '';
    try {
      if (next !== 'workspace') await workspaceSurface.suspend();
      page = next; await tick();
      if (next === 'workspace' && active !== null) await lazyTester.present(active, viewport.getBoundingClientRect());
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function release() {
    if (busy || refreshing) return;
    busy = true;
    try { await workspaceSurface.release(); active = null; selectedIdentity = ''; }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function resize() {
    if (busy || refreshing || resizing || active === null || page !== 'workspace') return;
    resizing = true;
    try { await workspaceSurface.update(viewport.getBoundingClientRect()); }
    catch (error) { issue = String(error); }
    finally { resizing = false; }
  }
  function step(direction: 1 | -1) {
    const next = nextProfile(profiles, active, direction);
    if (next !== null) void select(next);
  }
  async function inspectStorage() {
    if (busy || refreshing || resizing) return;
    busy = true; issue = '';
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
  onMount(() => {
    void refresh();
    const interval = setInterval(() => { if (!document.hidden) void refresh(); }, 2000);
    const observer = new ResizeObserver(() => { void resize(); }); observer.observe(viewport);
    let unlisten: (() => void) | undefined;
    void listen<string>('workspace-surface-error', event => { issue = event.payload; }).then(stop => {
      if (disposed) stop(); else unlisten = stop;
    }).catch(error => { issue = String(error); });
    return () => { disposed = true; clearInterval(interval); observer.disconnect(); unlisten?.(); void workspaceSurface.release().catch(() => {}); };
  });
</script>

<svelte:head><title>Lazy Tester</title></svelte:head>
<div class="lt-app">
  <aside class="lt-sidebar">
    <a class="lt-brand" href="#workspace" on:click|preventDefault={() => navigate('workspace')}><span>LT</span> Lazy Tester</a>
    <p class="lt-caption">MINECRAFT EDUCATION</p>
    <nav aria-label="Navigasi utama">
      {#each [['workspace','Workspace'],['launchers','Launcher'],['storage','Penyimpanan'],['settings','Pengaturan']] as [id,label]}
        <button class:chosen={page === id} disabled={busy} on:click={() => navigate(id as typeof page)}>{label}</button>
      {/each}
    </nav>
    <div class="lt-sidebar-bottom"><span class="lt-dot"></span> Runtime mandiri<p>{runningCount} dari 5 Minecraft berjalan</p></div>
  </aside>
  <main>
    <header class="lt-page-header"><div><p class="lt-caption">RUANG BERMAIN ANDA</p><h1>{page === 'workspace' ? 'Workspace' : page === 'launchers' ? 'Launcher Minecraft' : page === 'storage' ? 'Penyimpanan' : 'Pengaturan'}</h1></div>
      <button class="lt-secondary" disabled={busy || refreshing} on:click={refresh}>Perbarui status</button>
    </header>
    {#if issue}<div class="lt-alert" role="alert"><strong>Operasi membutuhkan perhatian</strong><p>{issue}</p><span>Game dan data yang sudah ada tetap dipertahankan.</span></div>{/if}
    <section class="lt-workspace" hidden={page !== 'workspace'} aria-label="Workspace Minecraft">
      <header class="lt-stage-header"><div><h2>{active ? `Minecraft - ${active}` : 'Pilih Minecraft untuk mulai'}</h2><p>{active ? 'Sedang ditampilkan · klik di dalam game untuk mengontrol' : 'Buka profil pilihan Anda melalui halaman Launcher.'}</p></div>
        <div class="lt-actions"><button class="lt-secondary" disabled={busy || !available || !runningCount} on:click={() => step(-1)} aria-label="Minecraft sebelumnya">←</button><button class="lt-secondary" disabled={busy || !available || !runningCount} on:click={() => step(1)}>Berikutnya →</button>{#if active}<button class="lt-secondary" disabled={busy} on:click={release}>Lepas jendela</button>{/if}</div>
      </header>
      <div class="lt-viewport" bind:this={viewport}>{#if !active}<div class="lt-empty"><span>LT</span><h2>Semua Minecraft, satu tempat</h2><p>Setiap profil memiliki login dan penyimpanan sendiri.</p><button on:click={() => navigate('launchers')}>Pilih launcher</button></div>{/if}</div>
      <div class="lt-filmstrip" aria-label="Minecraft lainnya">
        {#each others as profile (profile.slot)}
          <button class="lt-card" disabled={busy || !available || profile.state !== 'RUNNING'} on:click={() => select(profile.slot)}>
            <span class="lt-card-picture" aria-hidden="true">{profile.slot}</span><strong>Minecraft - {profile.slot}</strong><small>{labels[profile.state]}</small>
          </button>
        {/each}
      </div>
      <p class="lt-help">Memilih Minecraft mengganti area bermain. Sesi lain tetap berjalan dengan batas 10 FPS di latar.</p>
    </section>
    {#if page === 'launchers'}
      <p class="lt-description">Buka hanya Minecraft yang ingin Anda gunakan. Tidak ada game yang dibuka otomatis.</p>
      <div class="lt-profile-list">{#each profiles as profile (profile.slot)}
        <article><span class="lt-profile-number">{profile.slot}</span><div><h2>Minecraft - {profile.slot}</h2><p>{labels[profile.state]}</p>{#if profile.issue}<p class="lt-error">{profile.issue}</p>{/if}</div>
          {#if profile.state === 'RUNNING'}<button disabled={busy || !available} on:click={() => select(profile.slot)}>Tampilkan</button>
          {:else}<button disabled={busy || refreshing || !available || ['STARTING','NOT_RESPONDING'].includes(profile.state)} on:click={() => launch(profile.slot)}>{profile.state === 'STARTING' ? 'Membuka…' : profile.state === 'NOT_RESPONDING' ? 'Tidak merespons' : 'Buka Minecraft'}</button>{/if}
        </article>
      {/each}</div>
    {:else if page === 'storage'}
      <section class="lt-panel"><h2>Data terpisah untuk setiap Minecraft</h2><p>Akun, dunia, dan pengaturan memakai lima profil yang sama dengan sesi sebelumnya. Ukuran dihitung hanya saat diminta, tanpa membaca isi file.</p><p><button disabled={busy || refreshing} on:click={inspectStorage}>{busy ? 'Menghitung…' : 'Hitung penggunaan ruang'}</button></p>
        {#if storage}{#each storage as item}<div class="lt-setting"><div><strong>Minecraft - {item.slot}</strong><p>{(item.bytes / 1048576).toFixed(1)} MB · {item.files.toLocaleString('id-ID')} file{item.partial ? ' · sebagian belum terhitung' : ''}</p></div><button class="lt-secondary" disabled={busy || refreshing} on:click={() => openFolder(item.slot)}>Buka folder</button></div>{/each}<p>Ukuran merupakan hasil pengukuran terakhir dan dapat berubah selama bermain.</p>{/if}
        <p>Pembersihan cache belum diaktifkan: folder yang aman dihapus harus dipisahkan terlebih dahulu dari data login dan dunia.</p></section>
    {:else if page === 'settings'}
      <section class="lt-panel"><h2>Performa</h2><div class="lt-setting"><span>FPS saat game aktif</span><strong>144</strong></div><div class="lt-setting"><span>FPS saat game di latar</span><strong>10</strong></div><p>Angka di atas adalah batas FPS. Counter di dalam game menunjukkan hasil aktual.</p><h2>Preset grafis</h2><p>Simple · jarak render 12 chunk · efek visual hemat. Diterapkan sebelum profil dibuka; jarak simulasi dipertahankan.</p><h2>Saat aplikasi ditutup</h2><p>Minecraft tetap berjalan. Jendela yang ditampilkan dikembalikan ke Windows.</p></section>
    {/if}
  </main>
</div>
