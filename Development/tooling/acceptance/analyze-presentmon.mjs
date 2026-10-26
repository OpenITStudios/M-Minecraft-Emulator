import { lstatSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const LIMIT_BYTES = 32 * 1024 * 1024;
const LIMIT_ROWS = 250000;
const LIMIT_COLUMNS = 128;
const ALLOWED_APPLICATIONS = new Set([
  'minecraft.windows.exe',
  'minecrafteducation.exe',
  'minecrafteducationedition.exe',
]);

function fail(reason) {
  throw new TypeError('PresentMon evidence rejected: ' + reason);
}

/** Parse RFC4180 quoted fields without retaining every CSV row in memory. */
function forEachCsvRow(csv, callback) {
  if (typeof csv !== 'string' || !csv || Buffer.byteLength(csv, 'utf8') > LIMIT_BYTES) {
    fail('CSV missing or too large');
  }
  let fields = [], cell = '', quoted = false, closedQuote = false;
  let rows = 0;
  const addCell = () => {
    if (cell.length > 8192) fail('oversized CSV field');
    fields.push(cell);
    if (fields.length > LIMIT_COLUMNS) fail('too many CSV columns');
    cell = '';
    closedQuote = false;
  };
  const addRow = () => {
    addCell();
    if (++rows > LIMIT_ROWS) fail('too many CSV rows');
    callback(fields, rows);
    fields = [];
  };
  for (let i = 0; i < csv.length; i++) {
    const ch = csv[i];
    if (quoted) {
      if (ch === '"') {
        if (csv[i + 1] === '"') { cell += '"'; i++; }
        else { quoted = false; closedQuote = true; }
      } else cell += ch;
      continue;
    }
    if (ch === '"') {
      if (cell || closedQuote) fail('invalid quote placement');
      quoted = true;
    } else if (ch === ',') addCell();
    else if (ch === '\n' || ch === '\r') {
      addRow();
      if (ch === '\r' && csv[i + 1] === '\n') i++;
    } else {
      if (closedQuote) fail('text after a closed quote');
      cell += ch;
    }
    if (cell.length > 8192) fail('oversized CSV field');
  }
  if (quoted) fail('unterminated quoted field');
  if (cell || fields.length || closedQuote) addRow();
}

function requirePid(pid) {
  if (typeof pid !== 'string' || !/^[1-9][0-9]{0,9}$/.test(pid) ||
    Number(pid) > 0xffffffff) fail('supply one canonical decimal target PID');
  return pid;
}

function percentile(sorted, fraction) {
  return Number(sorted[Math.ceil(fraction * sorted.length) - 1].toFixed(3));
}

/** Analyze only PresentMon 1.x/2.3.1-compatible MsBetweenPresents.
 * Never interpret v2 FrameTime as the same metric. No FPS, GPU or login proof.
 * Does not emit a PID, process name, swap-chain address or raw CSV content.
 */
export function analyzePresentMonCsv(csv, pid) {
  const selectedPid = requirePid(pid);
  let header = null, required = null, selected = 0, unavailable = 0;
  const chainSet = new Set(), frameIntervals = [];
  forEachCsvRow(csv.replace(/^\uFEFF/, ''), (fields, rowNumber) => {
    if (!header) {
      header = fields;
      if (new Set(header).size !== header.length) fail('duplicate CSV header');
      const names = ['Application', 'ProcessID', 'SwapChainAddress'];
      const intervalHeader = header.includes('MsBetweenPresents')
        ? 'MsBetweenPresents' : header.includes('msBetweenPresents')
          ? 'msBetweenPresents' : null;
      if (!intervalHeader || names.some(name => !header.includes(name))) {
        fail('requires Application, ProcessID, SwapChainAddress, MsBetweenPresents');
      }
      required = [...names, intervalHeader].map(name => header.indexOf(name));
      return;
    }
    if (fields.length !== header.length) fail('inconsistent row width');
    const [applicationIndex, pidIndex, chainIndex, intervalIndex] = required;
    if (fields[pidIndex] !== selectedPid) return;
    if (!ALLOWED_APPLICATIONS.has(fields[applicationIndex].toLowerCase())) {
      fail('target PID contains an unexpected application');
    }
    const chain = fields[chainIndex];
    if (!/^0x[0-9a-f]{1,16}$/i.test(chain)) fail('invalid swap-chain identity');
    chainSet.add(chain.toLowerCase());
    if (chainSet.size > 1) fail('ambiguous multiple swap chains for selected PID');
    selected++;
    const interval = fields[intervalIndex];
    if (interval === '' || interval === 'NA') { unavailable++; return; }
    if (!/^(?:[0-9]+)(?:\.[0-9]+)?$/.test(interval)) fail('invalid present interval');
    const n = Number(interval);
    if (!Number.isFinite(n) || n <= 0 || n > 60000) fail('impossible present interval');
    frameIntervals.push(n);
  });
  if (!header || selected < 30 || frameIntervals.length < 30) fail('insufficient selected frame intervals');
  // Do not quietly declare full coverage when most frames have no interval.
  if (unavailable > selected * 0.05) fail('too many missing present intervals');
  frameIntervals.sort((a, b) => a - b);
  const mean = frameIntervals.reduce((sum, x) => sum + x, 0) / frameIntervals.length;
  const n = frameIntervals.length;
  return {
    schema: 1,
    evidence: 'OPERATOR_SUPPLIED_PRESENTMON_CSV_NOT_AUTHENTICATED',
    metric: 'MS_BETWEEN_PRESENTS_NOT_DISPLAYED_FRAMETIME',
    observedIntervals: n,
    unavailableIntervals: unavailable,
    presentationCoveragePercent: Number((100 * n / selected).toFixed(2)),
    meanPresentIntervalMs: Number(mean.toFixed(3)),
    p50PresentIntervalMs: percentile(frameIntervals, 0.50),
    p95PresentIntervalMs: percentile(frameIntervals, 0.95),
    p99PresentIntervalMs: percentile(frameIntervals, 0.99),
    over33msIntervals: frameIntervals.filter(x => x > 33.333).length,
    over50msIntervals: frameIntervals.filter(x => x > 50).length,
    gameplayVerified: false,
    fpsVerified: false,
    frameDisplayTimingVerified: false,
    accountIsolationVerified: false,
    optimizationVerified: false,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 4 || args[0] !== '--csv' || args[2] !== '--process-id') {
      fail('usage: analyze-presentmon.mjs --csv <existing.csv> --process-id <PID>');
    }
    const path = resolve(args[1]);
    const metadata = lstatSync(path);
    if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size > LIMIT_BYTES) {
      fail('CSV must be an ordinary file no larger than 32 MiB');
    }
    const result = analyzePresentMonCsv(readFileSync(path, 'utf8'), args[3]);
    process.stdout.write(JSON.stringify(result, null, 2) + '\n');
  } catch (error) {
    // Do not echo supplied CSV contents, raw path or sensitive process details.
    process.stderr.write(error instanceof TypeError ? error.message + '\n' :
      'PresentMon evidence could not be read.\n');
    process.exitCode = 1;
  }
}
