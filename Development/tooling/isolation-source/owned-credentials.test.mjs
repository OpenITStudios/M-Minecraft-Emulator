import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {spawnSync} from 'node:child_process';
const source=readFileSync(new URL('../../engine/isolation-owned/Sandboxie/core/dll/cred.c',import.meta.url),'utf8').replaceAll('\r\n','\n');
function nativeEnvironment(){
  const vswhere=join(process.env['ProgramFiles(x86)'],'Microsoft Visual Studio/Installer/vswhere.exe');
  const found=spawnSync(vswhere,['-latest','-products','*','-property','installationPath'],{encoding:'utf8',windowsHide:true});
  assert.equal(found.status,0,found.stderr);
  const dev=join(found.stdout.trim(),'Common7/Tools/VsDevCmd.bat');
  assert.ok(!/["\r\n]/.test(dev));
  const setup=spawnSync('cmd.exe',['/d','/s','/c',`""${dev}" -no_logo -arch=x64 >nul && set"`],{encoding:'utf8',windowsHide:true,windowsVerbatimArguments:true});
  assert.equal(setup.status,0,setup.stderr);
  const env={...process.env};for(const row of setup.stdout.split(/\r?\n/)){const at=row.indexOf('=');if(at>0)env[row.slice(0,at)]=row.slice(at+1);}
  return env;
}
test('actual owned credential routines never consult a host store',{skip:process.platform!=='win32'},()=>{
  const env=nativeEnvironment();
  const directory=mkdtempSync(resolve('../workspace/credential-policy-'));
  try{
    const names=['Cred_Init','Cred_CredReadW','Cred_CredReadDomainCredentialsW','Cred_CredEnumerateW','Cred_CredReadDomainCredentialsA','Cred_CredEnumerateA'];
    const functions=names.map(name=>{const start=source.indexOf('_FX '+(name==='Cred_Init'?'BOOLEAN':'BOOL')+' '+name+'(');assert.ok(start>=0,name);const end=source.indexOf('\n//---',start);return source.slice(start,end<0?undefined:end);}).join('\n');
    const storage=readFileSync(new URL('../../engine/isolation-owned/Sandboxie/core/dll/pst.cpp',import.meta.url),'utf8').replaceAll('\r\n','\n');
    const start=storage.indexOf('_FX BOOLEAN Pst_Init('),end=storage.indexOf('\n//---',start);
    assert.ok(start>=0);
    writeFileSync(join(directory,'functions.inc'),functions+'\n'+storage.slice(start,end));
    writeFileSync(join(directory,'test.c'),readFileSync(new URL('./fixtures/owned-credentials.c',import.meta.url)));
    const built=spawnSync('cl.exe',['/nologo','/TC','/W3','test.c','/Fe:test.exe'],{cwd:directory,env,encoding:'utf8',windowsHide:true,timeout:30000});
    assert.equal(built.status,0,built.stdout+built.stderr);
    const run=spawnSync(join(directory,'test.exe'),[],{encoding:'utf8',windowsHide:true,timeout:5000});
    assert.equal(run.status,0,run.stdout+run.stderr);
  }finally{rmSync(directory,{recursive:true});}
});
test('actual protected storage rejects host handles before reads or truncation',{skip:process.platform!=='win32'},()=>{
  const env=nativeEnvironment();
  const directory=mkdtempSync(resolve('../workspace/storage-stream-'));
  try{
    const storage=readFileSync(new URL('../../engine/isolation-owned/Sandboxie/core/dll/ipstore_impl.cpp',import.meta.url),'utf8').replaceAll('\r\n','\n');
    const start=storage.indexOf('void *IPStoreImpl::open_stream(bool forWrite)'),end=storage.indexOf('\nbool IPStoreImpl::read()',start);
    assert.ok(start>=0&&end>start);
    writeFileSync(join(directory,'stream.inc'),storage.slice(start,end));
    const service=readFileSync(new URL('../../engine/isolation-owned/Sandboxie/core/svc/pstoreserver.cpp',import.meta.url),'utf8').replaceAll('\r\n','\n');
    const handler=service.indexOf('MSG_HEADER *PStoreServer::Handler('),handlerEnd=service.indexOf('\n//---',handler);
    assert.ok(handler>=0&&handlerEnd>handler);
    writeFileSync(join(directory,'service.inc'),service.slice(handler,handlerEnd));
    writeFileSync(join(directory,'test.cpp'),readFileSync(new URL('./fixtures/owned-storage-stream.cpp',import.meta.url)));
    const built=spawnSync('cl.exe',['/nologo','/TP','/std:c++17','/W3','test.cpp','/Fe:test.exe'],{cwd:directory,env,encoding:'utf8',windowsHide:true,timeout:30000});
    assert.equal(built.status,0,built.stdout+built.stderr);
    const run=spawnSync(join(directory,'test.exe'),[],{encoding:'utf8',windowsHide:true,timeout:5000});
    assert.equal(run.status,0,run.stdout+run.stderr);
  }finally{rmSync(directory,{recursive:true});}
});
