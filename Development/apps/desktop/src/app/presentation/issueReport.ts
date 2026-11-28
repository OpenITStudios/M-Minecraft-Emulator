// Selective, desktop-observed issue summary. Not a native audit log or telemetry.
import type {NativeIssueReport, ProfileSlot} from '../bridge/lazyTester.js';
import {userFeedback, type UserMessageKey} from './sessionFeedback.js';

export type IssueCode = ReturnType<typeof userFeedback>['code'] |
  'LT-GAM-05' | 'LT-CAP-01' | 'LT-CAP-02' | 'LT-CAP-03' | 'LT-CAP-04' | 'LT-CAP-05' |
  'LT-AGT-01' | 'LT-AGT-02' | 'LT-AGT-03' | 'LT-AGT-04' | 'LT-AGT-05' | 'LT-SET-07';

export const MAX_ISSUES = 48;
interface IssueEntry { c: IssueCode; s?: ProfileSlot; t: number; n?: number; l?: number; r?: number; e?: number }

// Known preflight/contended close refusals are not unexpected failures.
export function reportableFeedback(key: UserMessageKey): boolean {
  return key !== 'graphicsBlocked' && key !== 'performanceBlocked' &&
    key !== 'closeRejected' && key !== 'closeAll';
}

export class IssueReport {
  private readonly entries = new Map<string, IssueEntry>();
  private truncated = false;
  private checkpointDue = false;

  constructor(private readonly seconds = () => Math.floor(Date.now()/1000)) {}

  // State observations are retained once per code/slot for this app lifetime.
  // Only discrete attempted operations request a repeat count; do not count polls.
  note(code: IssueCode, slot?: ProfileSlot, countAttempt = false): boolean {
    const key = code + ':' + (slot ?? 0);
    const prior = this.entries.get(key);
    if (prior) {
      const recurring = prior.r !== undefined;
      if (!countAttempt && !recurring) return false;
      if (recurring) {
        delete prior.r;
        prior.e = Math.min((prior.e ?? 0) + 1, 0xffffffff);
      }
      if (countAttempt) prior.n = Math.min((prior.n ?? 0) + 1, 0xffffffff);
      prior.l = this.seconds();
      if (recurring || (countAttempt && prior.n !== undefined &&
          (prior.n & (prior.n-1)) === 0)) this.checkpointDue = true;
      return true;
    }
    if (this.entries.size >= MAX_ISSUES) {
      const changed = !this.truncated;
      this.truncated = true;
      if (changed) this.checkpointDue = true;
      return changed;
    }
    this.entries.set(key,{c:code,...(slot === undefined ? {} : {s:slot}),
      t:this.seconds(),...(countAttempt ? {n:1} : {})});
    this.checkpointDue = true;
    return true;
  }

  // Called only by an explicit positive receipt for the same scope.
  // No healthy record is created and repeated success cannot rewrite history.
  resolve(code: IssueCode, slot?: ProfileSlot): void {
    const prior = this.entries.get(code + ':' + (slot ?? 0));
    if (prior && prior.r === undefined) {
      prior.r = this.seconds();
      this.checkpointDue = true;
    }
  }
  takeCheckpointDue(): boolean {
    const due = this.checkpointDue;
    this.checkpointDue = false;
    return due;
  }

  // One export; keep observations of different producers separate unless correlated.
  readCombined(native: NativeIssueReport | null | undefined, storageUnavailable = false): string | null {
    if (!this.entries.size && !native) return null;
    return JSON.stringify({v:2,scope:'desktop-process',issues:[...this.entries.values()],
      ...(this.truncated ? {truncated:true} : {}),
      ...(native === undefined ? {nativeUnavailable:true} : {native}),
      ...(storageUnavailable ? {storageUnavailable:true} : {})});
  }

  // Legacy desktop-only format; serialize once only on explicit user request.
  read(): string | null {
    if (!this.entries.size) return null;
    return JSON.stringify({v:1,scope:'desktop-observed',issues:[...this.entries.values()],
      ...(this.truncated ? {truncated:true} : {})});
  }
}
