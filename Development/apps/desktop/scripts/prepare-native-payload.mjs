// Stage only reviewed native DLL bytes; never profiles, credentials or worlds.
import {createHash} from 'node:crypto';
import {lstatSync, readFileSync, mkdirSync, writeFileSync, existsSync} from 'node:fs';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
export function reviewedDigests(source) {
  const block = source.match(/const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = Some\(PayloadDigests \{([\s\S]*?)\}\);/);
  if (!block) throw new Error('Reviewed native payload policy is unavailable');
  const result = {};
  for (const [field, name] of [['compatibility','bootstrap-compat.dll'],['enroller','bootstrap-enroll.dll']]) {
    const bytes = block[1].match(new RegExp(field + ': \\[([^\\]]*)\\]'))?.[1].match(/0x[0-9a-fA-F]{2}/g);
    if (bytes?.length !== 32) throw new Error('Invalid reviewed payload digest');
    result[name] = bytes.map(byte => byte.slice(2).toLowerCase()).join('');
  }
  return result;
}
function ordinaryAncestors(path) {
  for (let current = resolve(path);;) {
    if (existsSync(current)) {
      const info = lstatSync(current);
      if (info.isSymbolicLink() || !info.isDirectory()) throw new Error('Payload directory must not traverse a link');
    }
    const parent = dirname(current);
    if (parent === current) break;
    current = parent;
  }
}
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function stageNativePayload(source, destination, expected) {
  ordinaryAncestors(source); ordinaryAncestors(destination);
  const files = Object.entries(expected).map(([name, hash]) => {
    if (!['bootstrap-compat.dll','bootstrap-enroll.dll'].includes(name) || !/^[0-9a-f]{64}$/.test(hash)) throw new Error('Unexpected payload policy');
    const path = resolve(source, name), info = lstatSync(path);
    if (!info.isFile() || info.isSymbolicLink() || info.size < 64 || info.size > 32 * 1024 * 1024) throw new Error('Unsupported native DLL file');
    const bytes = readFileSync(path);
    if (digest(bytes) !== hash) throw new Error('Native DLL does not match reviewed runtime bytes: ' + name);
    return {name, hash, bytes};
  });
  if (files.length !== 2) throw new Error('Both reviewed DLLs are required');
  // Complete all byte checks before creating output. Existing different outputs
  // are refused rather than overwritten or recursively removed.
  for (const file of files) {
    const output = resolve(destination, file.name);
    if (existsSync(output)) {
      const info = lstatSync(output);
      if (!info.isFile() || info.isSymbolicLink() || info.size !== file.bytes.length || digest(readFileSync(output)) !== file.hash) throw new Error('Stale generated payload; preserve/review it before rebuilding');
    }
  }
  mkdirSync(destination, {recursive:true});
  for (const file of files) {
    const output = resolve(destination, file.name);
    if (!existsSync(output)) writeFileSync(output, file.bytes, {flag:'wx'});
    if (digest(readFileSync(output)) !== file.hash) throw new Error('Staged payload verification failed');
  }
  return files.map(({name,hash,bytes}) => ({name,sha256:hash,bytes:bytes.length}));
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const source = process.argv[2] ?? process.env.LAZY_TESTER_NATIVE_PAYLOAD_DIR;
  if (!source) throw new Error('Set LAZY_TESTER_NATIVE_PAYLOAD_DIR to the reviewed native DLL directory');
  const policy = reviewedDigests(readFileSync(resolve(root,'src-tauri/src/engine/independent_bootstrap.rs'),'utf8'));
  const result = stageNativePayload(resolve(source),resolve(root,'src-tauri/package-resources/lazy-tester'),policy);
  process.stdout.write(JSON.stringify({staged:result,profileCapabilitiesIncluded:false}) + '\n');
}

