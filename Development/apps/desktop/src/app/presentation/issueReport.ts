// Selective, desktop-observed issue summary. Not a native audit log or telemetry.
import type {NativeIssueReport, ProfileSlot} from '../bridge/lazyTester.js';
import {userFeedback, type UserMessageKey} from './sessionFeedback.js';

export type IssueCode = ReturnType<typeof userFeedback>['code'] |
  'LT-GAM-05' | 'LT-CAP-01' | 'LT-CAP-02' | 'LT-CAP-03' | 'LT-CAP-04' | 'LT-CAP-05' |
  'LT-AGT-01' | 'LT-AGT-02' | 'LT-AGT-03' | 'LT-AGT-04' | 'LT-AGT-05' | 'LT-SET-07';

export const MAX_ISSUES = 48;
interface IssueEntry { c: IssueCode; s?: ProfileSlot; t: number; n?: number; l?: number }

// Known preflight/contended close refusals are not unexpected failures.
export function reportableFeedback(key: UserMessageKey): boolean {
  return key !== 'graphicsBlocked' && key !== 'performanceBlocked' &&
    key !== 'closeRejected' && key !== 'closeAll';
}

export class IssueReport {
  private readonly entries = new Map<string, IssueEntry>();
  private truncated = false;

  constructor(private readonly seconds = () => Math.floor(Date.now()/1000)) {}

  // State observations are retained once per code/slot for this app lifetime.
  // Only discrete attempted operations request a repeat count; do not count polls.
  note(code: IssueCode, slot?: ProfileSlot, countAttempt = false): boolean {
    const key = code + ':' + (slot ?? 0);
    const prior = this.entries.get(key);
    if (prior) {
      if (!countAttempt) return false;
      prior.n = Math.min((prior.n ?? 0) + 1, 0xffffffff);
      prior.l = this.seconds();
      return true;
    }
    if (this.entries.size >= MAX_ISSUES) {
      const changed = !this.truncated;
      this.truncated = true;
      return changed;
    }
    this.entries.set(key,{c:code,...(slot === undefined ? {} : {s:slot}),
      t:this.seconds(),...(countAttempt ? {n:1} : {})});
    return true;
  }

  // One export; keep observations of different producers separate unless correlated.
  readCombined(native: NativeIssueReport | null | undefined): string | null {
    if (!this.entries.size && !native) return null;
    return JSON.stringify({v:2,scope:'desktop-process',issues:[...this.entries.values()],
      ...(this.truncated ? {truncated:true} : {}),
      ...(native === undefined ? {nativeUnavailable:true} : {native})});
  }

  // Legacy desktop-only format; serialize once only on explicit user request.
  read(): string | null {
    if (!this.entries.size) return null;
    return JSON.stringify({v:1,scope:'desktop-observed',issues:[...this.entries.values()],
      ...(this.truncated ? {truncated:true} : {})});
  }
}
