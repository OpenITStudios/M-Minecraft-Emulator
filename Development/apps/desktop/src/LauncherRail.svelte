<script lang="ts">
  // Presentation only; native game/session ownership and scheduling stay in LazyTesterApp.
  import {canRequestLaunch, type ProfileSession, type ProfileSlot} from './app/bridge/lazyTester.js';
  import {guidedRecoveryAllowed} from './app/presentation/guidedRecovery.js';
  import {launcherCardGuidance} from './app/presentation/sessionHealth.js';
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
  export let onRequestClose: (slot: ProfileSlot, trigger: HTMLButtonElement) => void;
  export let onGuidedRecovery: (slot: ProfileSlot, action: SessionHealthAction) => void;

  // A changing native enumeration never changes a launcher's visual slot.
  // Existing keyed elements retain their original native DWM preview targets.
  $: orderedProfiles = [...profiles].sort((left, right) => left.slot - right.slot);

  // Accessible action names describe what is currently possible. A blocked or
  // unobserved launcher is never announced as a ready-to-launch button.
  function cardActionName(profile: ProfileSession, statusLabel: string): string {
    const name = 'Minecraft - ' + profile.slot;
    if (!available || !bindingConfigured) return name + ' · ' + statusLabel;
    if (profile.state === 'RUNNING') return 'Select ' + name + ' · ' + statusLabel;
    if (canRequestLaunch(profile)) return 'Launch ' + name + ' · ' + statusLabel;
    return name + ' · ' + statusLabel;
  }
</script>

<div class="lt-filmstrip" role="group" aria-label="Minecraft profiles">
  {#each orderedProfiles as profile (profile.slot)}
    {@const healthItem = healthIssues.find(item => item.slot === profile.slot)}
    {@const guidance = launcherCardGuidance(
      profile, available && bindingConfigured && !unresolvedRecovery.has(profile.slot)
    )}
    {@const statusLabel = !available || !bindingConfigured
      ? 'Status unavailable'
      : active === profile.slot && !detached && profile.state === 'RUNNING'
        ? 'Selected in Play'
        : profile.state === 'RUNNING' && separateSlots.has(profile.slot)
          ? 'Separate window'
          : profile.state === 'RUNNING'
            ? 'Running in background'
            : labels[profile.state]}
    <div class="lt-launcher-entry" data-slot={profile.slot} title={guidance??undefined}>
      <button class="lt-card" class:selected={active===profile.slot&&!detached}
        aria-pressed={profile.state==='RUNNING' ? active===profile.slot&&!detached : undefined}
        aria-label={cardActionName(profile, statusLabel)}
        aria-describedby={guidance ? 'lt-launcher-guidance-'+profile.slot : undefined}
        disabled={busy||!available||!bindingConfigured||
          (profile.state!=='RUNNING'&&!canRequestLaunch(profile))}
        on:click={()=>profile.state==='RUNNING'?onSelect(profile.slot):canRequestLaunch(profile)&&onLaunch(profile.slot)}>
        <strong title={'Minecraft - '+profile.slot}>Minecraft - {profile.slot}{active===profile.slot&&!detached?' ✓':''}</strong>
        <span class="lt-card-picture" use:observePreview={profile.slot} aria-hidden="true">{!available||!bindingConfigured?'Unavailable':profile.state==='RUNNING'?profile.slot:profile.state==='STARTING'?'Starting…':profile.state==='NOT_RESPONDING'?'Unavailable':profile.state==='ATTENTION'?'Needs attention':'▷ Launch'}</span>
        <small class={available&&bindingConfigured
          ? (profile.state==='ATTENTION'||profile.state==='NOT_RESPONDING'
            ? 'lt-status-attention'
            : profile.state==='STARTING'
              ? 'lt-status-pending'
              : active===profile.slot&&!detached&&profile.state==='RUNNING'
                ? 'lt-status-selected' : '')
          : ''} title={statusLabel}>{statusLabel}</small>
      </button>
      {#if guidance}<span id={'lt-launcher-guidance-'+profile.slot} hidden>{guidance}</span>{/if}
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
          on:click={event=>onRequestClose(profile.slot,event.currentTarget)}>Close game</button>
      {/if}
    </div>
  {/each}
</div>
