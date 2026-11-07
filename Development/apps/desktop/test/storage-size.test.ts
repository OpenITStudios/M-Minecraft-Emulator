import {describe,expect,it} from 'vitest';
import {formatStorageBytes,formatStorageExtent} from '../src/app/presentation/storageSize.js';

describe('Storage measurement presentation (source fixture; no file operations)',()=>{
  it('does not confuse unknown, zero, and byte-size profile data',()=>{
    expect(formatStorageBytes(null)).toBe('Not measured');
    expect(formatStorageBytes(undefined)).toBe('Not measured');
    expect(formatStorageBytes(NaN)).toBe('Not measured');
    expect(formatStorageBytes(Infinity)).toBe('Not measured');
    expect(formatStorageBytes(-1)).toBe('Not measured');
    expect(formatStorageBytes(0)).toBe('0 B');
    expect(formatStorageBytes(1)).toBe('1 B');
    expect(formatStorageBytes(1023)).toBe('1023 B');
    expect(formatStorageBytes(1024)).toBe('1.0 KiB');
    expect(formatStorageBytes(1536)).toBe('1.5 KiB');
    expect(formatStorageBytes(1024*1024)).toBe('1.0 MiB');
    expect(formatStorageBytes(1024*1024*1024)).toBe('1.0 GiB');
  });
  it('preserves incomplete native counts as minimums, never a complete result',()=>{
    expect(formatStorageExtent(0,false)).toBe('0 B');
    expect(formatStorageExtent(0,true)).toBe('At least 0 B');
    expect(formatStorageExtent(1024*1024,true)).toBe('At least 1.0 MiB');
    expect(formatStorageExtent(null,true)).toBe('Not measured');
  });
});
