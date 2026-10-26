import {execFileSync} from 'node:child_process';
import {existsSync, mkdirSync, readdirSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';

export function buildRootLauncher(root, release) {
  const sdk = 'C:/Program Files (x86)/Windows Kits/10/bin';
  const compiler = readdirSync(sdk).sort((a,b)=>b.localeCompare(a,undefined,{numeric:true}))
    .map(version=>resolve(sdk,version,'x64/rc.exe')).find(existsSync);
  if (!compiler) throw new Error('Windows SDK resource compiler is required');
  const output = resolve(release,'root-launcher');
  mkdirSync(output,{recursive:true});
  const icon = resolve(root,'Development/apps/desktop/src-tauri/icons/icon.ico');
  const resource = resolve(output,'launcher.rc'), object = resolve(output,'launcher.res');
  writeFileSync(resource,`1 ICON "${icon.replaceAll('\\','\\\\')}"\n`);
  execFileSync(compiler,['/nologo','/fo',object,resource],{stdio:'inherit',windowsHide:true});
  const executable = resolve(release,'Lazy Tester.exe');
  execFileSync('rustc',['--edition=2021','--target','x86_64-pc-windows-msvc',
    '-C','opt-level=s','-C','panic=abort','-C','strip=symbols','-C',`link-arg=${object}`,
    resolve(root,'Development/apps/desktop/scripts/root-launcher.rs'),'-o',executable],
    {stdio:'inherit',windowsHide:true});
  return executable;
}
