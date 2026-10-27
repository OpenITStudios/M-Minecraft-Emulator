// Build-only MUI resources. Every runtime role still opens the same executable file.
import {execFileSync,spawnSync} from 'node:child_process';
import {existsSync,lstatSync,mkdirSync,mkdtempSync,readFileSync,readdirSync,renameSync,writeFileSync} from 'node:fs';
import {dirname,isAbsolute,relative,resolve,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
const desktop=resolve(fileURLToPath(new URL('..',import.meta.url)));
export const processRoles=JSON.parse(readFileSync(resolve(desktop,'src-tauri/windows/process-roles.json'),'utf8'));
export const roleMuiNames=['lazy-tester.exe',...processRoles.map(r=>r.file)].map(name=>'en-US/'+name+'.mui');
for(const role of processRoles)if(!['sessions','recovery'].includes(role.role)||!/^Lazy Tester - [A-Za-z ]+\.exe$/.test(role.file)||!/^[\x20-\x7e]{11}$/.test(role.description))throw new Error('Invalid process role metadata');
if(processRoles.length!==2||new Set(processRoles.map(r=>r.role)).size!==2||new Set(processRoles.map(r=>r.file)).size!==2)throw new Error('Both distinct helper roles are required');
export function roleResource(bytes,description) {
  const result=Buffer.from(bytes),key=Buffer.from('FileDescription\0','utf16le'),original=Buffer.from('Lazy Tester\0','utf16le');
  const at=result.indexOf(key),value=at<0?-1:result.indexOf(original,at+key.length);
  if(value<0||value>at+key.length+4||description.length!==11)throw new Error('Unexpected fixed-length Windows description resource');
  Buffer.from(description+'\0','utf16le').copy(result,value);return result;
}
function ordinary(path) {
  for(let dir=resolve(path);;dir=dirname(dir)) {
    if(existsSync(dir)){const info=lstatSync(dir);if(info.isSymbolicLink()||!info.isDirectory())throw new Error('Role output ancestors must be ordinary directories');}
    if(dirname(dir)===dir)break;
  }
}
function main() {
  if(process.platform!=='win32'||process.argv.length!==2)throw new Error('Windows role packaging accepts no arguments');
  const root=resolve(desktop,'../../..'),target=process.env.CARGO_TARGET_DIR;
  const local=target?relative(resolve(root,'workspace/builds'),target):'';
  if(!target||!isAbsolute(target)||!local||local==='..'||isAbsolute(local)||local.startsWith('..'+sep))throw new Error('Use the project-local Windows package command');
  const release=resolve(target,'x86_64-pc-windows-msvc/release');ordinary(release);
  const executable=resolve(release,'lazy-tester.exe');
  const info=lstatSync(executable);if(!info.isFile()||info.isSymbolicLink())throw new Error('Ordinary compiled executable required');
  const sdk='C:/Program Files (x86)/Windows Kits/10/bin';
  const tool=readdirSync(sdk).sort((a,b)=>b.localeCompare(a,undefined,{numeric:true})).map(v=>resolve(sdk,v,'x64/muirct.exe')).find(existsSync);
  if(!tool)throw new Error('Windows SDK MUI resource tool is required');
  const output=resolve(desktop,'src-tauri/package-resources/lazy-tester');ordinary(output);mkdirSync(output,{recursive:true});
  const locale=resolve(output,'en-US');ordinary(locale);mkdirSync(locale,{recursive:true});
  const expected=roleMuiNames.map(n=>n.slice('en-US/'.length));
  for(const name of readdirSync(locale))if(!expected.includes(name))throw new Error('Unexpected generated locale file; preserve it for review');
  function safeWrite(path,bytes) {
    if(existsSync(path)){const info=lstatSync(path);if(!info.isFile()||info.isSymbolicLink()||info.nlink!==1)throw new Error('Generated role resource must be an independent ordinary file');}
    writeFileSync(path,bytes);
  }
  const base=resolve(locale,'lazy-tester.exe.mui');
  if(existsSync(base)){const info=lstatSync(base);if(!info.isFile()||info.isSymbolicLink()||info.nlink!==1)throw new Error('Invalid prior MUI resource');}
  const dump=spawnSync(tool,['-d',executable],{encoding:'utf8',windowsHide:true});
  const report=(dump.stdout??'')+(dump.stderr??'');
  if(dump.status===1&&report.includes('Not a valid MUI file')) {
    const scratch=mkdtempSync(resolve(release,'role-build-'));
    const config=resolve(scratch,'roles.xml'),neutral=resolve(scratch,'neutral.exe');
    writeFileSync(config,'<?xml version="1.0" encoding="utf-8"?><localization><resources><win32Resources fileType="Application" language="en-US" ultimateFallbackLanguage="en-US" ultimateFallbackLocation="external"><neutralResources><resourceType typeNameId="#16"/></neutralResources><localizedResources><resourceType typeNameId="#16"/></localizedResources></win32Resources></resources></localization>');
    execFileSync(tool,['-q',config,'-f','-g','0x409','-x','0x409',executable,neutral,base],{stdio:'inherit',windowsHide:true});
    // Preserve Cargo's original artifact links and replace only the packaging output.
    renameSync(executable,resolve(scratch,'original.exe'));renameSync(neutral,executable);
  } else if(dump.status!==0||!report.includes('UltimateFallbackLanguage'))throw new Error('Unrecognized executable resource layout');
  const original=readFileSync(base);
  for(const role of processRoles)safeWrite(resolve(locale,role.file+'.mui'),roleResource(original,role.description));
  ordinary(resolve(release,'en-US'));
  mkdirSync(resolve(release,'en-US'),{recursive:true});
  for(const name of roleMuiNames)safeWrite(resolve(release,name),readFileSync(resolve(output,name)));
  const macro='; Generated from windows/process-roles.json. Application links only.\n!macro LAZY_TESTER_ROLE_FILES FUNCTION\n'+processRoles.map(r=>'  Push "'+r.file+'"\n  Call ${FUNCTION}\n').join('')+'!macroend\n';
  safeWrite(resolve(output,'process-role-links.nsh'),macro);
  console.log('Process role MUI resources prepared; runtime code and data ownership are unchanged.');
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))main();
