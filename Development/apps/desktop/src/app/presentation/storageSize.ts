/** Human-readable byte quantities from verified non-negative native counts.
 * Binary units are labelled honestly (KiB/MiB/GiB), never MB for 2^20 bytes.
 * Unknown/invalid values must never render as zero or imply a live measurement.
 * This is pure display logic; it does not inspect or change stored files.
 */
export function formatStorageBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isSafeInteger(bytes) || bytes < 0) return 'Not measured';
  if (bytes === 0) return '0 B';
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'] as const;
  let number = bytes;
  let unitIndex = 0;
  while (number >= 1024 && unitIndex < units.length - 1) {
    number /= 1024;
    unitIndex += 1;
  }
  const precision = unitIndex === 0 ? 0 : number < 10 ? 1 : 0;
  return number.toFixed(precision) + ' ' + units[unitIndex];
}

/** Partial observations are minimums, not complete size measurements. */
export function formatStorageExtent(bytes: number | null | undefined, partial: boolean): string {
  const formatted = formatStorageBytes(bytes);
  if (formatted === 'Not measured') return formatted;
  return partial ? 'At least ' + formatted : formatted;
}
