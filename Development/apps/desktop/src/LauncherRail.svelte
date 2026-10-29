<script lang="ts">
  // Presentation only; native game/session ownership and scheduling stay in LazyTesterApp.
  import {canRequestLaunch, type ProfileSession, type ProfileSlot} from './app/bridge/lazyTester.js';
  import {guidedRecoveryAllowed} from './app/presentation/guidedRecovery.js';
  import type {SessionHealthIssue, SessionHealthAction} from './app/presentation/sessionHealth.js';
  import SessionHealthNotice from './SessionHealthNotice.svelte';

  export let profiles: ProfileSession[];
  export let labels: Record<ProfileSession['state'], string>;
  export let healthIssues: SessionHealthIssue[];
  export let active: ProfileSlot | null;
  export let primary: ProfileSlot;
  export let detached: boolean;
  export let separateSlots: ReadonlySet<ProfileSlot>;
  export let unresolvedRecovery: ReadonlySet<ProfileSlot>;
  export let available: boolean;
  export let bindingConfigured: boolean;
  export let busy: boolean;
  export let closeInProgress: boolean;
  export let nativeExitReview: boolean;
  export let guidedRecoveryBlocked: boolean;
  export let observePreview: (node: HTMLElement, slot: ProfileSlot) => {destroy(): void};
  export let onSelect: (slot: ProfileSlot) => void;
  export let onLaunch: (slot: ProfileSlot) => void;
  export let onRequestClose: (slot: ProfileSlot) => void;
  export let onGuidedRecovery: (slot: ProfileSlot, action: SessionHealthAction) => void;
</script>

<div class="lt-filmstrip" aria-label="Minecraft launchers">
  {#each profiles as profile (profile.slot)}
    {@const healthItem = healthIssues.find(item => item.slot === profile.slot)}
    <div class="lt-launcher-entry">
      <button class="lt-card" class:selected={active===profile.slot&&!detached}
        aria-pressed={active===profile.slot&&!detached}
        aria-label={(profile.state==='RUNNING'?'Open ':'Launch ')+'Minecraft - '+profile.slot}
        disabled={busy||!available||!bindingConfigured||
          (profile.state!=='RUNNING'&&!canRequestLaunch(profile))}
        on:click={()=>profile.state==='RUNNING'?onSelect(profile.slot):canRequestLaunch(profile)&&onLaunch(profile.slot)}>
        <strong>Minecraft - {profile.slot}{active===profile.slot&&!detached?' ✓':''}</strong>
        <span class="lt-card-picture" use:observePreview={profile.slot} aria-hidden="true">{profile.state==='RUNNING'?profile.slot:profile.state==='STARTING'?'Starting…':profile.state==='NOT_RESPONDING'?'Unavailable':profile.state==='ATTENTION'?'Needs attention':'▷ Launch'}</span>
        <small>{active===profile.slot&&!detached?'Playing here':profile.state==='RUNNING'&&separateSlots.has(profile.slot)?'Separate window':profile.state==='RUNNING'?'Running in background':labels[profile.state]}</small>
      </button>
      {#if healthItem}
        <SessionHealthNotice item={healthItem} onAction={onGuidedRecovery}
          disabled={!guidedRecoveryAllowed(healthItem, profile, profile.slot,
            healthItem.action, {
              blocked: guidedRecoveryBlocked,
              unresolvedRecovery,
              separateWindows: separateSlots,
              primarySlot: primary,
            })}/>
      {/if}
      {#if ['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state)}
        <button class="lt-secondary lt-close-launcher"
          aria-label={'Close Minecraft - '+profile.slot+' normally'}
          disabled={busy||closeInProgress||nativeExitReview||!available||!bindingConfigured}
          on:click={()=>onRequestClose(profile.slot)}>Close game</button>
      {/if}
    </div>
  {/each}
</div>
