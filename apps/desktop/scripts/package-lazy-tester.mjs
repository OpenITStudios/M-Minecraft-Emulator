// Build from one clean Local revision; export only an installer and its receipts.
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {copyFileSync, constants, existsSync, lstatSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, isAbsolute, relative, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyLazyTesterPackage} from './check-lazy-tester-package.mjs';

const root = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
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
    buildCommand:'npm --prefix apps/desktop run build:app',
    installer:{name, sha256:hash, bytes:bytes.length},
    signing:'NOT_VERIFIED', installedAcceptance:'NOT_EXECUTED',
    minecraftAcceptance:'NOT_EXECUTED', publication:'DISABLED',
  };
  writeFileSync(resolve(output,'package-manifest.json'), JSON.stringify(manifest,null,2)+'\n', {flag:'wx'});
  return {output, manifest};
}

function main() {
  if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('Windows x64 packaging is required');
  const sha = sourceRevision(root);
  const {policy} = verifyLazyTesterPackage();
  if (policy.release.publicationEnabled !== false) throw new Error('This command builds candidates only; publication must remain disabled');
  const toolchain = JSON.parse(readFileSync(resolve(root,'toolchain.json'),'utf8'));
  if (process.versions.node !== toolchain.node.version) throw new Error('Use the declared workspace Node version');
  const npm = process.env.npm_execpath;
  if (!npm || !isAbsolute(npm) || !existsSync(npm)) throw new Error('Run npm run package:windows from the project root');
  const npmVersion = execFileSync(process.execPath,[npm,'--version'],{encoding:'utf8'}).trim();
  if (npmVersion !== toolchain.npm.version) throw new Error('Use the declared workspace npm version');
  const configuredTarget = process.env.CARGO_TARGET_DIR;
  if (configuredTarget && !isAbsolute(configuredTarget)) throw new Error('CARGO_TARGET_DIR must be absolute when provided');
  const target = configuredTarget ?? resolve(root,'apps/desktop/src-tauri/target');
  const localTarget = relative(root,target);
  if (!localTarget || localTarget === '..' || localTarget.startsWith('..'+sep) || isAbsolute(localTarget)) {
    throw new Error('Build output must stay inside this checkout');
  }
  ordinaryDirectories(target);
  const version = readFileSync(resolve(root,'VERSION'),'utf8').trim();
  const stamp = new Date().toISOString().replace(/[-:.]/g,'');
  execFileSync(process.execPath,[npm,'--prefix',resolve(root,'apps/desktop'),'run','build:app'], {cwd:root, stdio:'inherit'});
  if (sourceRevision(root) !== sha) throw new Error('Source changed during the build; no candidate exported');
  const installer = resolve(target,'x86_64-pc-windows-msvc/release/bundle/nsis',`Lazy Tester_${version}_x64-setup.exe`);
  const result = assembleCandidate(root,installer,sha,version,stamp);
  console.log(`Development candidate: ${result.output}\nSigning is not verified; installation and gameplay are not executed; publication is disabled.`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
