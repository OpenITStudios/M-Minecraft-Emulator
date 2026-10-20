// Exact current-owned delta; never applied to the pristine or legacy fork.
export const CREDENTIAL_FILE='Sandboxie/core/dll/cred.c';
export const STORAGE_FILE='Sandboxie/core/dll/pst.cpp';
export function applyOwnedProtectedStorageIsolation(input){
  let source=input.replaceAll('\r\n','\n');
  const anchor='#include "dll.h"';
  if(source.split(anchor).length!==2)throw Error('Owned protected storage include drift');
  source=source.replace(anchor,()=>anchor+'\n#include "common/my_version.h"\n#if !defined(MIVUBI_PRIVATE_ISOLATION_BUILD) || MIVUBI_PRIVATE_ISOLATION_BUILD != 0x4D495655\n#error Owned_storage_policy_requires_private_build\n#endif');
  const fn=source.indexOf('_FX BOOLEAN Pst_Init(');
  const start=source.indexOf('    //\n    // if OpenProtectedStorage',fn);
  const end=source.indexOf('    PStoreCreateInstance = (P_PStoreCreateInstance)',start);
  if(fn<0||start<0||end<0)throw Error('Owned protected storage routine drift');
  return source.slice(0,start)+'    // Owned profiles retain virtual storage even in App Compartment mode.\n\n'+source.slice(end);
}
export function applyOwnedCredentialIsolation(input){
  let source=input.replaceAll('\r\n','\n');
  function replace(before,after){
    if(source.split(before).length!==2)throw Error('Owned credential policy anchor drift');
    source=source.replace(before,()=>after);
  }
  function routine(name,edit){
    const start=source.indexOf('_FX '+(name==='Cred_Init'?'BOOLEAN':'BOOL')+' '+name+'(');
    if(start<0)throw Error('Missing owned credential routine '+name);
    const end=source.indexOf('\n//---',start);
    const original=source.slice(start,end<0?undefined:end);
    const updated=edit(original);
    if(original===updated)throw Error('Unchanged owned credential routine '+name);
    source=source.slice(0,start)+updated+source.slice(end<0?source.length:end);
  }
  function once(text,before,after){
    if(text.split(before).length!==2)throw Error('Owned credential routine anchor drift');
    return text.replace(before,()=>after);
  }
  replace('#include "dll.h"','#include "dll.h"\n#include "common/my_version.h"\n#if !defined(MIVUBI_PRIVATE_ISOLATION_BUILD) || MIVUBI_PRIVATE_ISOLATION_BUILD != 0x4D495655\n#error Owned_credential_policy_requires_private_build\n#endif');
  routine('Cred_Init',text=>{
    const start=text.indexOf('    //\n    // if OpenProtectedStorage');
    const end=text.indexOf('    //Cred_Trace',start);
    if(start<0||end<0)throw Error('Owned credential init anchor drift');
    return text.slice(0,start)+'    // Owned game profiles never bypass credential virtualization.\n\n'+text.slice(end);
  });
  for(const name of ['Cred_CredReadW','Cred_CredReadDomainCredentialsW','Cred_CredEnumerateW']){
    routine(name,text=>{
      text=once(text,'    if (! Cred_PreparePStore())',name==='Cred_CredReadW'?
        '    *ppCredential = NULL;\n\n    if (! Cred_PreparePStore())':
        '    *pCount = 0;\n    *ppCredentials = NULL;\n\n    if (! Cred_PreparePStore())');
      if(name==='Cred_CredReadW')return once(text,
        '        ok = __sys_CredReadW(TargetName, Type, Flags, ppCredential);',
        '        SetLastError(ERROR_NOT_FOUND);\n        ok = FALSE;');
      if(name==='Cred_CredReadDomainCredentialsW')return once(text,
        '        return __sys_CredReadDomainCredentialsW(\n                                pTargetInfo, Flags, pCount, ppCredentials);',
        '        SetLastError(ERROR_NOT_FOUND);\n        return FALSE;');
      text=once(text,'    ok = __sys_CredEnumerateW(pFilter, Flags, pCount, ppCredentials);\n    err = GetLastError();',
        '    ok = FALSE;\n    err = ERROR_NOT_FOUND;');
      const start=text.indexOf('        if (ok) {');
      const end=text.indexOf('        mrshcreds[num_mrshcreds] = NULL;',start);
      if(start<0||end<0)throw Error('Owned credential merge anchor drift');
      return text.slice(0,start)+text.slice(end);
    });
  }
  for(const name of ['Cred_CredReadDomainCredentialsA','Cred_CredEnumerateA']){
    routine(name,text=>{
      const start=text.indexOf('{');
      return text.slice(0,start+1)+'\n    // Upstream ANSI implementation delegates to the host; fail closed instead.\n    *pCount = 0;\n    *ppCredentials = NULL;\n    SetLastError(ERROR_NOT_SUPPORTED);\n    return FALSE;\n}\n\n';
    });
  }
  routine('Cred_CredReadA',text=>once(text,'    void* pCredentialW = NULL;',
    '    *ppCredential = NULL;\n    void* pCredentialW = NULL;'));
  return source.replace(/\n+$/,'\n');
}
