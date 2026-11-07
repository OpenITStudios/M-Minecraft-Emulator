// One approved vector drives the WebView and all generated Tauri icons.
// Use content identity instead of mtime to avoid stale taskbar/installer art.
import {existsSync,mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';

const root=resolve(import.meta.dirname,'..');
const source=resolve(root,'src-tauri','app-icon.svg');
const iconDir=resolve(root,'src-tauri','icons');
const outputs=['32x32.png','128x128.png','128x128@2x.png','icon.ico']
  .map(name=>resolve(iconDir,name));
const stamp=resolve(iconDir,'.app-icon-source.sha256');
const digest=createHash('sha256').update(readFileSync(source)).digest('hex');
if(outputs.every(existsSync)&&existsSync(stamp)&&readFileSync(stamp,'utf8').trim()===digest)
  process.exit(0);

const cli=createRequire(import.meta.url).resolve('@tauri-apps/cli/tauri.js');
const result=spawnSync(process.execPath,[cli,'icon',source],{cwd:root,stdio:'inherit'});
if(result.error)throw result.error;
if(result.status!==0)process.exit(result.status??1);
if(!outputs.every(existsSync))throw new Error('Tauri did not produce all required app icons');
mkdirSync(iconDir,{recursive:true});
writeFileSync(stamp,digest+'\n','utf8');
