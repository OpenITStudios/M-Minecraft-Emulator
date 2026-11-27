// UI confirmation payload only. Native storage admission remains authoritative.
import type {ProfileSlot} from './bridge/lazyTester.js';

export type StorageSettingsAction = 'Reset' | 'Restore';
export type StorageReview =
  | {kind: 'cache'; slot: ProfileSlot}
  | {kind: 'settings'; slot: ProfileSlot; action: StorageSettingsAction}
  | {kind: 'full'; slot: ProfileSlot; token: string; pending: boolean};
export type StorageReviewKind = StorageReview['kind'];
