// Build from one clean Local revision; export only an installer and its receipts.
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {copyFileSync, constants, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, renameSync, writeFileSync} from 'node:fs';
import {dirname, isAbsolute, relative, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyLazyTesterPackage} from './check-lazy-tester-package.mjs';

const developmentRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const root = resolve(developmentRoot, '..');
const git = (cwd, ...args) => execFileSync('git', args, {cwd, encoding:'utf8'}).trim();
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

export function sourceRevision(cwd) {
  const branch = git(cwd, 'rev-parse', '--abbrev-ref', 'HEAD');
  const sha = git(cwd, 'rev-parse', 'HEAD');
  if (!['Local','HEAD'].includes(branch) || sha !== git(cwd, 'rev-parse', 'refs/heads/Local')) {
    throw new Error('Package only Local, or a detached worktree at the exact Local revision');
  }
  if (git(cwd, 'status', '--porcelain', '--untracked-files=normal')) {
    throw new Error('Packaging requires clean source. Preserve or commit changes before building.');
  }
  return sha;
}

function ordinaryDirectories(path) {
  for (let current = resolve(path);;) {
    if (existsSync(current)) {
      const info = lstatSync(current);
      if (!info.isDirectory() || info.isSymbolicLink()) throw new Error('Package paths must not traverse links');
    }
    const parent = dirname(current);
    if (parent === current) break;
    current = parent;
  }
}

export function assembleCandidate(cwd, installer, sha, version, stamp) {
  if (!/^[a-f0-9]{40}$/.test(sha) || !/^\d+\.\d+\.\d+$/.test(version) ||
      !/^\d{8}T\d{9}Z$/.test(stamp)) throw new Error('Invalid candidate identity');
  ordinaryDirectories(dirname(installer));
  const info = lstatSync(installer);
  if (!info.isFile() || info.isSymbolicLink() || info.nlink !== 1 || info.size < 64 || info.size > 512 * 1024 * 1024) {
    throw new Error('Expected an ordinary built installer');
  }
  const bytes = readFileSync(installer);
  if (bytes[0] !== 0x4d || bytes[1] !== 0x5a) throw new Error('Expected a Windows executable');
  const name = `Lazy-Tester-${version}-windows-x86_64-setup.exe`;
  const output = resolve(cwd, 'workspace/packages/lazy-tester', `${version}-${sha.slice(0,12)}-${stamp}`);
  ordinaryDirectories(dirname(output));
  mkdirSync(dirname(output), {recursive:true});
  mkdirSync(output); // Never overwrite or merge an earlier candidate.
  const target = resolve(output, name), hash = digest(bytes);
  copyFileSync(installer, target, constants.COPYFILE_EXCL);
  if (digest(readFileSync(target)) !== hash) throw new Error('Installer changed while exporting; candidate is incomplete');
  writeFileSync(resolve(output,'SHA256SUMS.txt'), `${hash}  ${name}\n`, {flag:'wx'});
  // Publish the completion receipt last. A partial directory is not a candidate.
  const manifest = {
    schema:1, product:'Lazy Tester', version, sourceCommit:sha, sourceBranch:'Local',
    classification:'DEVELOPMENT_CANDIDATE', releaseReady:false,
    buildCommand:'npm --prefix Development/apps/desktop run build:app',
    installer:{name, sha256:hash, bytes:bytes.length},
    signing:'NOT_VERIFIED', installedAcceptance:'NOT_EXECUTED',
    minecraftAcceptance:'NOT_EXECUTED', publication:'DISABLED',
  };
  writeFileSync(resolve(output,'package-manifest.json'), JSON.stringify(manifest,null,2)+'\n', {flag:'wx'});
  return {output, manifest};
}

export function stageLocalApp(cwd, release, sha) {
  if (!/^[a-f0-9]{40}$/.test(sha)) throw new Error('Invalid source revision');
  const app = resolve(cwd,'App'), data = resolve(cwd,'Data');
  ordinaryDirectories(app); ordinaryDirectories(data); ordinaryDirectories(release);
  // Never replace a running local candidate or modify its account/world data.
  if (process.platform === 'win32' && existsSync(app)) {
    const paths = execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',
      "$ErrorActionPreference='Stop'; @(Get-Process -Name lazy-tester -ErrorAction SilentlyContinue) | ForEach-Object { if (-not $_.Path) { throw 'Process path unavailable' }; $_.Path }"],
      {encoding:'utf8',windowsHide:true}).split(/\r?\n/).filter(Boolean);
    if (paths.some(path => resolve(path).toLowerCase() === resolve(app,'lazy-tester.exe').toLowerCase())) {
      throw new Error('Close the local Lazy Tester before refreshing App');
    }
  }
  const names = ['lazy-tester.exe','independent-bootstrap/bootstrap-compat.dll',
    'independent-bootstrap/bootstrap-enroll.dll','distribution/lazy-tester-package.json'];
  const files = names.map(name => {
    const source = resolve(release,name); ordinaryDirectories(dirname(source));
    const info = lstatSync(source);
    // Cargo hardlinks its executable into deps; read bytes and write an independent copy.
    if (!info.isFile() || info.isSymbolicLink() || info.size > 64 * 1024 * 1024) throw new Error('Invalid local app payload');
    const bytes = readFileSync(source);
    if (name.endsWith('.json')) JSON.parse(bytes.toString('utf8'));
    else if (bytes[0] !== 0x4d || bytes[1] !== 0x5a) throw new Error('Expected a Windows executable');
    return {name,bytes,sha256:digest(bytes)};
  });
  const parent = resolve(cwd,'workspace/private/local-app-backups');
  ordinaryDirectories(parent); mkdirSync(parent,{recursive:true});
  const attempt = mkdtempSync(resolve(parent,'refresh-')), staged = resolve(attempt,'new-App');
  mkdirSync(staged);
  for (const {name,bytes,sha256} of files) {
    const target = resolve(staged,name); mkdirSync(dirname(target),{recursive:true});
    writeFileSync(target,bytes,{flag:'wx'});
    if (digest(readFileSync(target)) !== sha256) throw new Error('Local app copy failed verification');
  }
  for (const name of ['data-location.json','performance.json','graphics.json']) {
    const locator = resolve(app,name);
    if (!existsSync(locator)) continue;
    const info = lstatSync(locator);
    if (!info.isFile() || info.isSymbolicLink() || info.size > 16384) throw new Error('Invalid existing data locator');
    copyFileSync(locator,resolve(staged,name),constants.COPYFILE_EXCL);
  }
  writeFileSync(resolve(staged,'build-info.json'),JSON.stringify({sourceCommit:sha,
    files:files.map(({name,sha256})=>({name,sha256})),runtimeAcceptance:'NOT_EXECUTED'},null,2)+'\n',{flag:'wx'});
  const backup = existsSync(app) ? resolve(attempt,'previous-App') : null;
  if (backup) renameSync(app,backup);
  try { renameSync(staged,app); }
  catch (error) { if (backup) renameSync(backup,app); throw error; }
  if (!existsSync(data)) mkdirSync(data); // Keep existing profiles and all contents untouched.
  return {app,data,backup};
}

function main() {
  if (process.argv.slice(2).some(arg => arg !== '--export-only') || process.argv.slice(2).length > 1) throw new Error('Only --export-only is supported');
  if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('Windows x64 packaging is required');
  const sha = sourceRevision(root);
  const {policy} = verifyLazyTesterPackage();
  if (policy.release.publicationEnabled !== false) throw new Error('This command builds candidates only; publication must remain disabled');
  const toolchain = JSON.parse(readFileSync(resolve(developmentRoot,'toolchain.json'),'utf8'));
  if (process.versions.node !== toolchain.node.version) throw new Error('Use the declared workspace Node version');
  const npm = process.env.npm_execpath;
  if (!npm || !isAbsolute(npm) || !existsSync(npm)) throw new Error('Run npm --prefix Development run package:windows from the checkout root');
  const npmVersion = execFileSync(process.execPath,[npm,'--version'],{encoding:'utf8'}).trim();
  if (npmVersion !== toolchain.npm.version) throw new Error('Use the declared workspace npm version');
  const configuredTarget = process.env.CARGO_TARGET_DIR;
  if (configuredTarget && !isAbsolute(configuredTarget)) throw new Error('CARGO_TARGET_DIR must be absolute when provided');
  const target = configuredTarget ?? resolve(root,'workspace/builds/desktop/target');
  const localTarget = relative(root,target);
  if (!localTarget || localTarget === '..' || localTarget.startsWith('..'+sep) || isAbsolute(localTarget)) {
    throw new Error('Build output must stay inside this checkout');
  }
  ordinaryDirectories(target);
  const version = readFileSync(resolve(developmentRoot,'VERSION'),'utf8').trim();
  const stamp = new Date().toISOString().replace(/[-:.]/g,'');
  execFileSync(process.execPath,[npm,'--prefix',resolve(developmentRoot,'apps/desktop'),'run','build:app'], {cwd:developmentRoot, stdio:'inherit', env:{...process.env,CARGO_TARGET_DIR:target}});
  if (sourceRevision(root) !== sha) throw new Error('Source changed during the build; no candidate exported');
  const installer = resolve(target,'x86_64-pc-windows-msvc/release/bundle/nsis',`Lazy Tester_${version}_x64-setup.exe`);
  const result = assembleCandidate(root,installer,sha,version,stamp);
  if (!process.argv.includes('--export-only')) {
    const local = stageLocalApp(root,resolve(target,'x86_64-pc-windows-msvc/release'),sha);
    const shortcuts=execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-File',resolve(developmentRoot,'tooling/windows-toolchain/create-app-shortcuts.ps1'),'-Root',root],{encoding:'utf8',windowsHide:true});
    const startup=execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-File',resolve(developmentRoot,'tooling/windows-toolchain/verify-desktop-startup.ps1'),'-Root',root],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32768});
    verifyDesktopStartup(startup,sha);
    console.log('Actual Windows Explorer/Desktop startup verified; five launcher controls enabled. No game launched.');
    console.log(`Local app: ${local.app}\nOpen Lazy Tester from Desktop or Start Menu. Existing data is preserved.\n${shortcuts.trim()}`);
  }
  console.log(`Development candidate: ${result.output}\nSigning is not verified; installation and gameplay are not executed; publication is disabled.`);
}

export function verifyDesktopStartup(output,expectedSource) {
  const report=JSON.parse(output);
  if(report.event!=='desktop-startup-observation'||report.explorerParent!==true||report.connected!==true||report.issue!==''||report.cards!==5||report.enabled!==5||!Number.isInteger(report.processId)||report.processId<1||!Number.isInteger(report.parentProcessId)||report.parentProcessId<1||!/^[a-f0-9]{40}$/.test(report.sourceCommit))throw new Error('Actual Desktop startup was not confirmed');
  if(report.sourceCommit!==expectedSource)throw new Error('Desktop startup belongs to a different source revision');
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
