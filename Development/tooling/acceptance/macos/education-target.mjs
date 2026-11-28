// Explicit, offline artifact metadata only. Never a game launch or admission route.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createReadStream, lstatSync, realpathSync} from 'node:fs';
import {isAbsolute, join, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';

const dictionary = value => value !== null && typeof value === 'object' && !Array.isArray(value);
// Observed in the user-supplied 1.26.33.0 bundle metadata; not a signer pin.
const expectedBundleId = 'com.microsoft.minecraft-edu';
const expectedExecutable = 'minecraft-edu';
const version = /^\d+(?:\.\d+){0,4}$/;
const validVersion = value => typeof value==='string' && value.length<=64 && version.test(value);

export function bundleMetadata(plist) {
  if (!dictionary(plist) || plist.CFBundlePackageType !== 'APPL' ||
      plist.CFBundleIdentifier !== expectedBundleId ||
      plist.CFBundleExecutable !== expectedExecutable ||
      !validVersion(plist.CFBundleVersion) || !validVersion(plist.CFBundleShortVersionString))
    throw Error('INVALID_BUNDLE_METADATA');
  for (const key of ['LSMultipleInstancesProhibited','LSRequiresNativeExecution'])
    if (Object.hasOwn(plist,key) && typeof plist[key] !== 'boolean') throw Error('INVALID_BUNDLE_METADATA');
  if (Object.hasOwn(plist,'LSMinimumSystemVersion') &&
      (typeof plist.LSMinimumSystemVersion !== 'string' || !version.test(plist.LSMinimumSystemVersion)))
    throw Error('INVALID_BUNDLE_METADATA');
  return {bundleId:plist.CFBundleIdentifier, executable:plist.CFBundleExecutable,
    version:plist.CFBundleShortVersionString, build:plist.CFBundleVersion,
    minimumMacOS:plist.LSMinimumSystemVersion ?? null,
    multipleInstancesProhibited:plist.LSMultipleInstancesProhibited ?? null,
    requiresNativeExecution:plist.LSRequiresNativeExecution ?? null};
}

export function architectureList(output) {
  const items=output.trim().split(/\s+/);
  if (!items.length || items.length>4 || new Set(items).size!==items.length ||
      items.some(item=>!['arm64','arm64e','x86_64','x86_64h'].includes(item)))
    throw Error('UNREVIEWED_ARCHITECTURE');
  return items.sort();
}

export function entitlementSummary(plist) {
  if (plist === null) return {appSandbox:null,appGroupsPresent:null,keychainGroupsPresent:null};
  if (!dictionary(plist)) throw Error('INVALID_ENTITLEMENTS');
  const sandbox=plist['com.apple.security.app-sandbox'];
  if (sandbox!==undefined && typeof sandbox!=='boolean') throw Error('INVALID_ENTITLEMENTS');
  const group = key => {
    if (!Object.hasOwn(plist,key)) return false;
    if (!Array.isArray(plist[key]) || plist[key].some(value=>typeof value!=='string'))
      throw Error('INVALID_ENTITLEMENTS');
    return plist[key].length>0;
  };
  return {appSandbox:sandbox??false,
    appGroupsPresent:group('com.apple.security.application-groups'),
    keychainGroupsPresent:group('keychain-access-groups')};
}

function run(tool,args,input) {
  const result=spawnSync('/usr/bin/'+tool,args,{input,encoding:'utf8',timeout:90_000,
    maxBuffer:256*1024,env:{PATH:'/usr/bin:/bin',LC_ALL:'C'},shell:false});
  if (result.error || result.status!==0) throw Error('ARTIFACT_TOOL_FAILED:'+tool);
  return {stdout:result.stdout,stderr:result.stderr};
}
function fingerprint(path,kind) {
  const stat=lstatSync(path,{bigint:true});
  if (realpathSync(path)!==path || (kind==='directory'?!stat.isDirectory():!stat.isFile()) ||
      (kind==='file' && (stat.size===0n || stat.size>2_147_483_648n)))
    throw Error('INVALID_ARTIFACT_PATH');
  return [stat.dev,stat.ino,stat.size,stat.mtimeNs,stat.ctimeNs].join(':');
}
async function fileHash(path) {
  const hash=createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}
function signingMetadata(output,expected) {
  const id=output.match(/^Identifier=([A-Za-z0-9.-]+)$/m)?.[1];
  const team=output.match(/^TeamIdentifier=([A-Z0-9]{10})$/m)?.[1];
  const cdhash=output.match(/^CDHash=([0-9a-f]{40})$/m)?.[1];
  if (id!==expected || !team || !cdhash) throw Error('UNREVIEWED_SIGNING_METADATA');
  return {identifier:id,teamIdentifier:team,cdHash:cdhash};
}

export async function inspectBundle(path) {
  if (process.platform!=='darwin') throw Error('MACOS_REQUIRED');
  if (typeof path!=='string' || !isAbsolute(path) || !path.endsWith('.app'))
    throw Error('EXPLICIT_APP_BUNDLE_REQUIRED');
  const root=resolve(path);
  const plist=join(root,'Contents','Info.plist');
  const fixed=[[root,'directory'],[plist,'file']];
  const before=fixed.map(([name,kind])=>fingerprint(name,kind));
  const metadata=bundleMetadata(JSON.parse(run('plutil',['-convert','json','-o','-',plist]).stdout));
  const exe=join(root,'Contents','MacOS',metadata.executable);
  fixed.push([exe,'file']);
  before.push(fingerprint(exe,'file'));
  const architectures=architectureList(run('lipo',['-archs',exe]).stdout);
  const executableSha256=await fileHash(exe);
  run('codesign',['--verify','--deep','--strict','--all-architectures',root]);
  const signatures=architectures.map(arch=>{
    const display=run('codesign',['--display','--arch',arch,'--verbose=4',root]);
    const requirement=run('codesign',['--display','--arch',arch,'-r','-',root]);
    const designated=requirement.stderr.match(/^designated => (.+)$/m)?.[1];
    if (!designated) throw Error('DESIGNATED_REQUIREMENT_UNAVAILABLE');
    const ent=run('codesign',['--display','--arch',arch,'--entitlements','-','--xml',root]).stdout.trim();
    const values=ent?JSON.parse(run('plutil',['-convert','json','-o','-','-'],ent).stdout):null;
    return {arch,...signingMetadata(display.stderr,metadata.bundleId),
      designatedRequirementSha256:createHash('sha256').update(designated).digest('hex'),
      ...entitlementSummary(values)};
  });
  if (fixed.some(([name,kind],index)=>fingerprint(name,kind)!==before[index]))
    throw Error('ARTIFACT_CHANGED');
  return {schema:1,status:'STATIC_METADATA_ONLY',...metadata,architectures,executableSha256,
    signatureIntegrity:'CODESIGN_VERIFIED',signatures,
    officialPublisher:'NOT_PINNED',distributionPackage:'NOT_VERIFIED',
    profileIsolation:'NOT_VERIFIED',runtimeAdmission:false};
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href===import.meta.url) {
  try {
    if (process.argv.length!==3) throw Error('EXPLICIT_APP_BUNDLE_REQUIRED');
    console.log(JSON.stringify(await inspectBundle(process.argv[2]),null,2));
  } catch {
    // Native tool diagnostics may contain private host paths. Do not print them.
    console.error(JSON.stringify({schema:1,status:'INSPECTION_BLOCKED',runtimeAdmission:false}));
    process.exitCode=1;
  }
}
