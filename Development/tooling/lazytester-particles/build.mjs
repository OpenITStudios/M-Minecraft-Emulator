import {readFile, lstat, writeFile, mkdir} from 'node:fs/promises';
import {basename, dirname, resolve, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {PARTICLES, SOURCE_URL} from './catalog.mjs';
import {compileFiles, gitBlobSha} from './tune.mjs';
import {createMcpack} from './archive.mjs';

function argumentsOf(args) {
  if (args.includes('--help') && args.length === 1) return {help: true};
  if (args.length !== 4 || !['--source', '--official'].includes(args[0]) || args[2] !== '--output') {
    throw new Error('Usage: node build.mjs --source <official resource_pack directory> --output <file.mcpack> OR --official yes --output <file.mcpack>');
  }
  if (args[0] === '--official' && args[1] !== 'yes') throw new Error('Official source download requires explicit --official yes');
  if (!args[1] || !args[3] || !args[3].endsWith('.mcpack')) throw new Error('Specify an explicit .mcpack output');
  return {origin: args[0], source: args[1], output: resolve(args[3])};
}
async function vanillaFile(origin, directory, file) {
  if (origin === '--source') {
    // Refuse linked sources and oversized files before loading a source into memory.
    const candidate = join(resolve(directory), 'particles', file);
    for (const path of [resolve(directory), join(resolve(directory), 'particles'), candidate]) {
      const state = await lstat(path);
      if (state.isSymbolicLink()) throw new Error(`Vanilla source must not be linked: ${file}`);
      if (path === candidate && (!state.isFile() || state.size > 128 * 1024)) {
        throw new Error(`Unsupported source size/type: ${file}`);
      }
    }
    return readFile(candidate);
  }
  const response = await fetch(SOURCE_URL + file, {redirect: 'error', signal: AbortSignal.timeout(15000)});
  if (!response.ok) throw new Error(`Official vanilla source unavailable (${response.status}): ${file}`);
  const advertised = Number(response.headers.get('content-length'));
  if (advertised > 128 * 1024) throw new Error(`Oversized vanilla source: ${file}`);
  // Missing Content-Length must not permit an unbounded arrayBuffer.
  if (!response.body) throw new Error(`Official vanilla source has no body: ${file}`);
  const reader = response.body.getReader();
  const parts = [];
  let total = 0;
  try {
    for (;;) {
      const {done, value} = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > 128 * 1024) throw new Error(`Oversized vanilla source: ${file}`);
      parts.push(value);
    }
  } catch (error) {
    await reader.cancel().catch(() => {});
    throw error;
  } finally {reader.releaseLock();}
  return Buffer.concat(parts, total);
}
export async function build(args = process.argv.slice(2)) {
  const options = argumentsOf(args);
  if (options.help) return {help: true};
  const sources = new Map();
  for (const spec of PARTICLES) {
    const raw = await vanillaFile(options.origin, options.source, spec.file);
    // Reject the first stale source before fetching/reading remaining files.
    if (gitBlobSha(raw) !== spec.blob) throw new Error(`Vanilla source fingerprint mismatch: ${spec.file}`);
    sources.set(spec.file, raw);
  }
  // Compile and verify every exact source BEFORE writing anything to output.
  const content = createMcpack(compileFiles(sources));
  if (basename(options.output).length > 150) throw new Error('Output basename is too long');
  await mkdir(dirname(options.output), {recursive: true});
  await writeFile(options.output, content, {flag: 'wx'});
  return {output: options.output, bytes: content.length, particleCount: PARTICLES.length};
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = await build();
    if (result.help) console.log('LazyTester Particle Tuning: --official yes --output <file.mcpack> | --source <resource_pack> --output <file.mcpack>');
    else console.log(`LazyTester Particle Tuning: ${result.particleCount} ambient effects -> ${result.output} (${result.bytes} bytes); NOT installed`);
  } catch (error) {
    console.error(`LazyTester Particle Tuning: ${error.message}`);
    process.exitCode = 1;
  }
}
