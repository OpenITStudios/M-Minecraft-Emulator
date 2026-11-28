import {test} from 'node:test';
import assert from 'node:assert/strict';
import {bundleMetadata,architectureList,entitlementSummary,inspectBundle,machOSummary} from './education-target.mjs';

const plist={CFBundlePackageType:'APPL',CFBundleIdentifier:'com.microsoft.minecraft-edu',
  CFBundleExecutable:'minecraft-edu',CFBundleVersion:'1.26.33.0',CFBundleShortVersionString:'1.26.33'};
test('absent singleton metadata remains unknown; explicit refusal stays visible',()=>{
  assert.equal(bundleMetadata(plist).multipleInstancesProhibited,null);
  assert.equal(bundleMetadata({...plist,LSMultipleInstancesProhibited:true}).multipleInstancesProhibited,true);
  assert.equal(bundleMetadata({...plist,LSMultipleInstancesProhibited:false}).multipleInstancesProhibited,false);
  for (const value of ['../game','/tmp/game','..','a/b','','another-executable']) {
    assert.throws(()=>bundleMetadata({...plist,CFBundleExecutable:value}));
  }
  assert.throws(()=>bundleMetadata({...plist,LSMultipleInstancesProhibited:'false'}));
  assert.throws(()=>bundleMetadata({...plist,CFBundlePackageType:'BNDL'}));
  assert.throws(()=>bundleMetadata({...plist,CFBundleIdentifier:'com.microsoft.other'}));
  assert.throws(()=>bundleMetadata({...plist,CFBundleVersion:undefined}));
  assert.throws(()=>bundleMetadata({...plist,CFBundleVersion:'9'.repeat(65)}));
});
test('architecture list requires recognized distinct slices',()=>{
  assert.deepEqual(architectureList('x86_64 arm64\n'),['arm64','x86_64']);
  for (const value of ['', 'arm64 arm64','ppc','arm64 unknown']) assert.throws(()=>architectureList(value));
});
test('entitlements reveal presence only, never group values; missing output is unknown',()=>{
  assert.deepEqual(entitlementSummary(null),{appSandbox:null,appGroupsPresent:null,keychainGroupsPresent:null});
  assert.deepEqual(entitlementSummary({'com.apple.security.app-sandbox':true,
    'keychain-access-groups':['private.example']}),{appSandbox:true,appGroupsPresent:false,keychainGroupsPresent:true});
  assert.throws(()=>entitlementSummary({'keychain-access-groups':'private.example'}));
  assert.throws(()=>entitlementSummary({'com.apple.security.app-sandbox':1}));
});
test('inspector refuses non-Mac host or non-absolute bundle without native tool invocation',async()=>{
  await assert.rejects(inspectBundle('example.app'),/MACOS_REQUIRED|EXPLICIT_APP_BUNDLE_REQUIRED/);
});

test('offline Mach-O reader bounds thin headers and rejects mismatched fat identity',()=>{
  const h=Buffer.alloc(32);
  h.writeUInt32LE(0xfeedfacf,0);
  h.writeUInt32LE(0x0100000c,4);
  h.writeUInt32LE(2,12);
  const read=(o,n)=>h.subarray(o,o+n);
  assert.deepEqual(machOSummary(read,h.length),[
    {arch:'arm64',linkedLibraryIndicators:[],embeddedSignature:null}]);
  h.writeUInt32LE(1,16); // Command count without command bytes.
  assert.throws(()=>machOSummary(read,h.length));
  h.writeUInt32LE(0,16);
  const fat=Buffer.alloc(60);
  fat.writeUInt32BE(0xcafebabe,0);fat.writeUInt32BE(1,4);
  fat.writeUInt32BE(0x01000007,8);fat.writeUInt32BE(3,12);
  fat.writeUInt32BE(28,16);fat.writeUInt32BE(32,20);
  h.copy(fat,28);
  assert.throws(()=>machOSummary((o,n)=>fat.subarray(o,o+n),fat.length));
});
test('offline code-signature structure is an indication, not trust',()=>{
  const id=Buffer.from('com.microsoft.minecraft-edu\0'),team=Buffer.from('UBF8T346G9\0');
  const d=Buffer.alloc(52+id.length+team.length);
  d.writeUInt32BE(0xfade0c02,0);d.writeUInt32BE(d.length,4);
  d.writeUInt32BE(0x20200,8);d.writeUInt32BE(0x10000,12);
  d.writeUInt32BE(52,20);d.writeUInt32BE(52+id.length,48);
  id.copy(d,52);team.copy(d,52+id.length);
  const superblob=Buffer.alloc(20+d.length);
  superblob.writeUInt32BE(0xfade0cc0,0);
  superblob.writeUInt32BE(superblob.length,4);superblob.writeUInt32BE(1,8);
  superblob.writeUInt32BE(20,16);d.copy(superblob,20);
  const h=Buffer.alloc(48);
  h.writeUInt32LE(0xfeedfacf,0);h.writeUInt32LE(0x0100000c,4);
  h.writeUInt32LE(2,12);h.writeUInt32LE(1,16);h.writeUInt32LE(16,20);
  h.writeUInt32LE(0x1d,32);h.writeUInt32LE(16,36);
  h.writeUInt32LE(48,40);h.writeUInt32LE(superblob.length,44);
  const b=Buffer.concat([h,superblob]);
  const result=machOSummary((o,n)=>b.subarray(o,o+n),b.length);
  assert.deepEqual(result[0].embeddedSignature.codeDirectories,[{
    slot:0,expectedIdentifier:true,expectedTeam:true,hardenedRuntimeFlag:true,adHocFlag:false}]);
  assert.equal(result[0].embeddedSignature.cmsBlobPresent,false);
  assert.equal(result[0].embeddedSignature.xmlEntitlementKeyIndicators,null);
  b.writeUInt32BE(0xffffffff,64); // superblob slot offset
  assert.throws(()=>machOSummary((o,n)=>b.subarray(o,o+n),b.length));
});
