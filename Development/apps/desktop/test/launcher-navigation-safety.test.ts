import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester startup inventory and review-safe navigation source contract',()=>{
  it('projects five stable keyed cards from fixed slots even before native observation',()=>{
    const rail=source('LauncherRail.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(rail).toContain("import {canRequestLaunch, profileSlots,");
    expect(rail).toContain('profileSlots.map((slot): ProfileSession =>');
    expect(rail).toContain("issue: 'UNOBSERVED_UI_PLACEHOLDER'");
    expect(rail).toContain('state: \'STOPPED\', process: null');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(app).toContain("{#if nativeBackend === 'WindowsNative'}");
    expect(app).toContain('<LauncherRail {profiles} {labels} {healthIssues}');
  });
  it('shows unknown state instead of an invented STOPPED result or enabled Launch',()=>{
    const rail=source('LauncherRail.svelte');
    expect(rail).toContain('observedSlot = profiles.some(item => item.slot === profile.slot)');
    expect(rail).toContain("statusLabel = !observedSlot || !available || !bindingConfigured");
    expect(rail).toContain('disabled={busy||reviewPending||!observedSlot||!available||!bindingConfigured');
    expect(rail).toContain(">{!observedSlot||!available||!bindingConfigured?'Unavailable'");
    expect(rail).toContain("observedSlot && ['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state)");
    expect(rail).toContain('profile, observedSlot && available && bindingConfigured');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(rail).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\s*\.\w+\s*\(/);
  });
  it('does not silently dismiss an active close or storage review when navigating',()=>{
    const app=source('LazyTesterApp.svelte');
    const storage=source('DataStorage.svelte');
    expect(app).toContain('$: navigationReviewOpen = closeReviewPending || clearingData ||');
    expect(app).toContain('clearingData || clearSlot !== null || recovery !== null || fullReview !== null) return;');
    expect(app).toContain('closeReviewActive(reviewState()) ||');
    expect(app).toContain('clearSlot !== null || recovery !== null || fullReview !== null) return;');
    expect(app).toContain('aria-disabled={navigationReviewOpen}');
    expect(app).toContain('disabled={navigationReviewOpen||nativeBackend!==\'WindowsNative\'}');
    expect(app).toContain('disabled={navigationReviewOpen} class:chosen={page===\'storage\'}');
    expect(app).toContain('Cancel before changing pages.');
    expect(storage).toContain('Finish or cancel this review before switching profiles.');
    expect(app).toContain('function cancelCloseOne()');
    expect(storage).toContain('cancelReview(onCancelFullReview)');
  });
  it('shows the date as well as the clock time of recorded Storage checks',()=>{
    const app=source('LazyTesterApp.svelte');
    expect(app).toContain('function storageReportReceivedTime(): string');
    expect(app).toContain("toLocaleString('en-US',{dateStyle:'medium',timeStyle:'short'})");
    expect(app).toContain('storageReportReceivedAt=storageReportReceivedTime()');
    expect(app).not.toContain("new Date().toLocaleTimeString('en-US')");
  });
  it('preserves native session and deletion admission and preview identity',()=>{
    const app=source('LazyTesterApp.svelte');
    const rail=source('LauncherRail.svelte');
    const storage=source('DataStorage.svelte');
    const css=source('lazy-tester.css');
    expect(app).toContain('const nextAction = nextPendingUiAction({');
    expect(app).toContain('await lazyTester.closeOne(slot)');
    expect(app).toContain('await workspaceSurface.suspend()');
    expect(app).toContain('await lazyTester.present(active, viewport.getBoundingClientRect())');
    expect(rail).toContain('canRequestLaunch(profile)&&onLaunch(profile.slot)');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
