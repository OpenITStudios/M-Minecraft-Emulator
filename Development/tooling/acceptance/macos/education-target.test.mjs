import {test} from 'node:test';
import assert from 'node:assert/strict';
import {bundleMetadata,architectureList,entitlementSummary,inspectBundle} from './education-target.mjs';

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
