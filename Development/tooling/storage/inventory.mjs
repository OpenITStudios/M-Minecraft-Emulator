// Read-only metadata inventory. Never read file contents, scan protected data,
// follow links/junctions, or infer that measured bytes are reclaimable.
import {lstatSync, opendirSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {fileURLToPath} from 'node:url';

export const CATEGORIES = Object.freeze([
  {name:'Active Cargo target', path:'workspace/builds/desktop/target'},
  {name:'Other build outputs', path:'workspace/builds', exclude:['desktop/target']},
  {name:'Archived compiler cache', path:'workspace/archive/compiler-cache-20261026'},
  {name:'Other retained archives', path:'workspace/archive', exclude:['compiler-cache-20261026']},
  {name:'Package candidates', path:'workspace/packages'},
  {name:'Cargo registry downloads', path:'workspace/toolchains/cargo/registry'},
  {name:'Cargo git dependencies', path:'workspace/toolchains/cargo/git'},
  {name:'Rustup toolchains', path:'workspace/toolchains/rustup'},
  {name:'Other portable toolchains', path:'workspace/toolchains', exclude:['cargo/registry','cargo/git','rustup']},
  {name:'npm cache', path:'workspace/cache/npm'},
  {name:'NuGet cache', path:'workspace/cache/nuget'},
  {name:'Other developer caches', path:'workspace/cache', exclude:['npm','nuget']},
  {name:'Dependency downloads', path:'workspace/downloads'},
  {name:'Temporary developer files', path:'workspace/tmp'},
  {name:'Development scratch', path:'workspace/scratch'},
  {name:'Desktop Node modules', path:'Development/apps/desktop/node_modules'},
  {name:'Desktop legacy Cargo target', path:'Development/apps/desktop/src-tauri/target'},
  {name:'Core legacy Cargo target', path:'Development/engine/runtime-core/target'},
  {name:'Frontend build output', path:'Development/apps/desktop/dist'},
  {name:'Svelte checker cache', path:'Development/apps/desktop/.svelte-check'},
  {name:'Desktop package staging', path:'Development/apps/desktop/src-tauri/package-resources'},
  {name:'Prepared local App', path:'App'},
]);

const isMissing = error => error?.code === 'ENOENT';
const maxDurationMs = 120_000;
const maxVisitedEntries = 1_200_000;

function inspectRoot(root, path) {
  let cursor = root;
  for (const segment of path.split('/')) {
    cursor = join(cursor, segment);
    let info;
    try { info = lstatSync(cursor); }
    catch (error) { return {state:isMissing(error)?'missing':'partial'}; }
    if (info.isSymbolicLink() || !info.isDirectory()) return {state:'partial'};
  }
  return {state:'complete', cursor};
}

function measureCategory(root, category, budget) {
  const row = {name:category.name, path:category.path, bytes:0n,
    files:0, directories:0, links:0, errors:0, state:'complete'};
  if (budget.stopped) return {...row, state:'partial'};
  const admission = inspectRoot(root, category.path);
  if (admission.state !== 'complete') return {...row, state:admission.state, errors:admission.state==='partial'?1:0};
  const excluded = new Set(category.exclude ?? []);
  // Stream one directory entry at a time instead of accumulating a large
  // readdir result or a recursive call stack for old compiler caches.
  const stack = [{absolute:admission.cursor, relative:'', opened:false, handle:null}];
  try {
    while (stack.length) {
      if (budget.visited >= budget.maxEntries || Date.now() >= budget.deadline) {
        row.state = 'partial';
        budget.stopped = true;
        break;
      }
      const entry = stack[stack.length-1];
      if (!entry.opened) {
        entry.opened = true;
        budget.visited++;
        let info;
        try { info = lstatSync(entry.absolute, {bigint:true}); }
        catch { row.errors++; row.state='partial'; stack.pop(); continue; }
        if (info.isSymbolicLink()) { row.links++; row.state='partial'; stack.pop(); continue; }
        if (info.isFile()) { row.files++; row.bytes += info.size; stack.pop(); continue; }
        if (!info.isDirectory()) { row.errors++; row.state='partial'; stack.pop(); continue; }
        row.directories++;
        try { entry.handle = opendirSync(entry.absolute); }
        catch { row.errors++; row.state='partial'; stack.pop(); continue; }
      }
      let child;
      try { child = entry.handle.readSync(); }
      catch { row.errors++; row.state='partial'; if(entry.handle){try{entry.handle.closeSync();}catch{}} stack.pop(); continue; }
      if (child===null) {
        try { entry.handle.closeSync(); }
        catch { row.errors++; row.state='partial'; }
        entry.handle=null;
        stack.pop();
        continue;
      }
      const rel = entry.relative ? entry.relative+'/'+child.name : child.name;
      // Other-build and other-archive partitions must not recount dedicated rows.
      if (excluded.has(rel)) continue;
      if (stack.length >= 64) { row.errors++; row.state='partial'; continue; }
      stack.push({absolute:join(entry.absolute,child.name),relative:rel,opened:false,handle:null});
    }
  } finally {
    for (const entry of stack) {
      if (entry.handle) try { entry.handle.closeSync(); } catch { row.errors++; row.state='partial'; }
    }
  }
  return row;
}

export function collectInventory(root, options={}) {
  const location = resolve(root);
  const check = lstatSync(location);
  if (!check.isDirectory() || check.isSymbolicLink()) throw new Error('Checkout root must be an ordinary directory');
  const duration = options.maxDurationMs ?? maxDurationMs;
  const entries = options.maxVisitedEntries ?? maxVisitedEntries;
  if (!Number.isSafeInteger(duration) || duration < 1 || !Number.isSafeInteger(entries) || entries < 1) {
    throw new Error('Inventory limits must be positive integers');
  }
  const budget = {maxEntries:entries, deadline:Date.now()+duration, visited:0, stopped:false};
  const categories = options.categories ?? CATEGORIES;
  const rows = categories.map(category=>measureCategory(location,category,budget));
  const total = rows.reduce((sum,row)=>sum+row.bytes,0n);
  return {rows, total, visited:budget.visited,
    complete:rows.every(row=>row.state!=='partial'), limited:budget.stopped};
}

const displaySize = bytes => {
  const value=Number(bytes);
  if (value<1024) return `${value} B`;
  if (value<1024**2) return `${(value/1024).toFixed(1)} KiB`;
  if (value<1024**3) return `${(value/1024**2).toFixed(1)} MiB`;
  return `${(value/1024**3).toFixed(2)} GiB`;
};

export function renderInventory(inventory, mode='report') {
  const lines=['Lazy Tester developer storage — READ ONLY (logical file bytes)',
    'Category                          Size        Files    Status',
    '----------------------------------------------------------------'];
  const rows=[...inventory.rows].sort((a,b)=>a.bytes===b.bytes?0:a.bytes>b.bytes?-1:1);
  for(const r of rows) {
    const status=r.state==='missing'?'ABSENT':r.state==='partial'?'PARTIAL':'OK';
    lines.push(`${r.name.padEnd(33)} ${displaySize(r.bytes).padStart(10)} ${String(r.files).padStart(8)}  ${status}`);
  }
  lines.push(`Measured total: ${displaySize(inventory.total)}${inventory.complete?'':' (LOWER BOUND; partial scan)'}`);
  lines.push(`Visited entries: ${inventory.visited}; ${inventory.limited?'scan limit reached':'within scan limits'}.`);
  lines.push('Excluded: Data/, workspace/private/, workspace/runs/, runtime/VM data, .git/, user/system stores.');
  lines.push('Totals are logical file bytes, not physical allocated space or safely reclaimable bytes.');
  lines.push('Nothing was deleted, relocated, hashed, opened for content, or modified.');
  if(mode==='explain') {
    lines.push('');
    lines.push('Largest measured categories (not a deletion recommendation):');
    for(const r of rows.filter(r=>r.bytes>0n).slice(0,5)) lines.push(`- ${r.name}: ${r.path} (${displaySize(r.bytes)})`);
    lines.push('Archived build outputs and package candidates require independent retention review.');
    lines.push('PARTIAL means a link, access failure or scan limit prevents an exact total.');
  }
  return lines.join('\n');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2] ?? 'report';
  if (!['report','explain'].includes(mode) || process.argv.length>3) {
    console.error('Usage: node tooling/storage/inventory.mjs [report|explain]');
    process.exitCode=2;
  } else {
    try {
      const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));
      const inventory=collectInventory(root);
      console.log(renderInventory(inventory,mode));
      if(!inventory.complete) process.exitCode=3;
    } catch(error) {
      console.error('Developer storage inventory failed: '+(error?.code??error?.message??'unknown error'));
      process.exitCode=1;
    }
  }
}
