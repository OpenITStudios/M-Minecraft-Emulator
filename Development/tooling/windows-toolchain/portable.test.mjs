import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const read = file => readFileSync(resolve(root,file),'utf8');
const launcher = read('../DEV.cmd');
const bootstrap = read('tooling/windows-toolchain/bootstrap.ps1');
const environment = read('tooling/windows-toolchain/local-env.ps1');
const developer = read('tooling/windows-toolchain/dev.ps1');
const config = JSON.parse(read('toolchain.json'));

test('DEV.cmd dispatches only workspace-local PowerShell after opt-in bootstrap', () => {
  assert.match(launcher, /if \/I "%~1"=="bootstrap"/);
  assert.match(launcher, /if "%~1"=="" goto help/);
  assert.match(launcher, /if \/I "%~1"=="help" goto help/);
  assert.match(launcher, /powershell\.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass/);
  assert.match(launcher, /workspace\\toolchains\\powershell\\pwsh\.exe/);
  assert.match(launcher, /Portable developer environment is missing/);
  assert.match(launcher, /"%LOCAL_PWSH%" -NoLogo -NoProfile/);
  assert.doesNotMatch(launcher, /(?:^|\n)\s*pwsh -NoLogo/m);
});
test('download route is pinned, checksum-verified, local and non-global', () => {
  assert.equal(config.node.version,'24.22.0');
  assert.equal(config.npm.version,'12.2.0');
  assert.equal(config.rust.version,'1.99.1');
  assert.equal(config.powershell.version,'7.6.7');
  assert.equal(config.policy.localDeveloperStorage,true);
  for(const asset of [config.portableWindowsX64.node,config.portableWindowsX64.powershell]) {
    assert.match(asset.url,/^https:\/\//);
    assert.match(asset.sha256,/^[a-f0-9]{64}$/);
  }
  assert.match(config.portableWindowsX64.node.url,/^https:\/\/nodejs\.org\//);
  assert.match(config.portableWindowsX64.powershell.url,/^https:\/\/github\.com\/PowerShell\/PowerShell\//);
  assert.match(config.portableWindowsX64.rustup.sha256Url,/^https:\/\/static\.rust-lang\.org\//);
  for(const snippet of [
    'workspace', "'downloads'", "'toolchains'", "'tmp'",
    'Get-VerifiedDownload', 'Get-FileHash', 'Expand-Archive',
  ]) assert.ok(bootstrap.includes(snippet), 'missing portable bootstrap contract: '+snippet);
  assert.match(bootstrap,/--no-modify-path/);
  assert.match(bootstrap,/\$env:CARGO_HOME = Join-Path \$tools 'cargo'/);
  assert.match(bootstrap,/\$env:RUSTUP_HOME = Join-Path \$tools 'rustup'/);
  assert.match(bootstrap,/Downloading .*workspace\/downloads/);
  assert.match(bootstrap,/SHA-256 mismatch/);
  assert.match(bootstrap,/Refuse non-directory or reparse-point/);
  assert.ok(bootstrap.indexOf('$env:TEMP = $temp') < bootstrap.indexOf('function Get-VerifiedDownload'),
    'bootstrap download temporary directory must be local before network IO');
  assert.match(bootstrap,/Rustup checksum sidecar must be a bounded ordinary file/);
  assert.match(bootstrap,/Portable Node\/npm\/PowerShell runtime identity mismatch/);
  assert.doesNotMatch(bootstrap,/\b(?:Start-Service|New-Service|sc\.exe|Start-Process -Verb RunAs)\b/);
});
test('developer commands isolate HOME/cache/temp, require local runtime and preserve app state', () => {
  assert.match(developer,/Set-LocalDeveloperEnvironment -Root \$Root/);
  assert.match(developer,/workspace\/toolchains\/node\/npm\.cmd/);
  assert.match(developer, /resolves outside workspace\/toolchains/);
  assert.match(developer,/& \$LocalNpm @NpmArguments/);
  assert.match(developer,/if \(\$Command -in @\("setup", "check", "check:frontend", "check:rust", "bench:frontend", "bench:rust", "test", "dev"\)\)/);
  assert.match(developer,/if \(\$Command -in @[\s\S]*?Test-DeveloperToolchain[\s\S]*?switch \(\$Command\)/);
  for(const snippet of [
    '$env:CARGO_HOME', '$env:RUSTUP_HOME', '$env:NPM_CONFIG_CACHE',
    '$env:NPM_CONFIG_PREFIX', '$env:NUGET_PACKAGES', '$env:TEMP', '$env:TMP'
  ]) assert.ok(environment.includes(snippet),'missing local environment: '+snippet);
  assert.match(environment,/no global fallback/i);
  assert.match(environment,/Do not override USERPROFILE\/APPDATA\/LOCALAPPDATA/);
  assert.doesNotMatch(environment,/\[Environment\]::SetEnvironmentVariable\([^)]*,\s*'Machine'/);
});
test('developer storage route supports report, explain, and non-destructive retention plan', () => {
  const inventory=read('tooling/storage/inventory.mjs');
  const planner=read('tooling/storage/retention.mjs');
  assert.match(developer,/\[string\]\$StorageMode = "report"/);
  assert.match(developer,/ValidateSet\("report", "explain", "plan", "cargo"\)/);
  assert.match(developer,/if \(\$Command -eq "storage"\)/);
  assert.match(developer,/tooling\/storage\/inventory\.mjs/);
  assert.match(developer,/tooling\/storage\/retention\.mjs/);
  assert.match(developer,/tooling\/storage\/cargo-footprint\.mjs/);
  assert.ok(developer.indexOf('if ($Command -eq "storage")') <
    developer.indexOf('Set-LocalDeveloperEnvironment -Root $Root'),
    'read-only inventory must bypass developer cache initialization');
  assert.match(developer,/if \(\$PSBoundParameters\.ContainsKey\('StorageMode'\)\)/);
  assert.match(inventory,/lstatSync/);
  assert.match(inventory,/opendirSync/);
  assert.doesNotMatch(inventory,/\b(?:rmSync|unlinkSync|writeFileSync|readFileSync|execFileSync|spawnSync)\b/);
  assert.match(planner,/destructiveCapability:false/);
  assert.match(planner,/no --apply mode/);
  assert.doesNotMatch(planner,/\b(?:rmSync|unlinkSync|renameSync|writeFileSync|execFileSync|spawnSync)\b/);
  const cargo=read('tooling/storage/cargo-footprint.mjs');
  assert.match(cargo,/x86_64-pc-windows-msvc/);
  assert.match(cargo,/opendirSync/);
  assert.match(cargo,/lstatSync/);
  assert.doesNotMatch(cargo,/\b(?:rmSync|unlinkSync|renameSync|writeFileSync|readFileSync|execFileSync|spawnSync)\b/);
});

test('developer Cargo target directory is canonical and refuses ambient overrides', () => {
  assert.match(environment,/\$canonicalCargoTarget = \[IO\.Path\]::GetFullPath/);
  // Cargo 1.99 checks three inherited path settings through one shared guard.
  // Pin the complete set and the exact validation/error path rather than
  // expecting the retired single-variable check.
  assert.match(environment,/foreach \(\$overrideName in @\('CARGO_TARGET_DIR','CARGO_BUILD_TARGET_DIR','CARGO_BUILD_BUILD_DIR'\)\)/);
  assert.match(environment,/\[IO\.Path\]::GetFullPath\(\$overrideValue\) -ine \$canonicalCargoTarget/);
  assert.match(environment,/external \$overrideName overrides are refused/);
  assert.match(environment,/\[IO\.FileAttributes\]::ReparsePoint/);
  assert.match(environment,/\$env:CARGO_TARGET_DIR = \$canonicalCargoTarget/);
  assert.match(environment,/\$env:CARGO_BUILD_TARGET_DIR = \$canonicalCargoTarget/);
  assert.match(environment,/\$env:CARGO_BUILD_BUILD_DIR = \$canonicalCargoTarget/);
  assert.match(environment,/GetEnvironmentVariable\(\$overrideName,'Process'\)/);
  assert.match(environment,/CARGO_BUILD_BUILD_DIR/);
  assert.ok(environment.indexOf('$canonicalCargoTarget') <
    environment.indexOf('New-Item -ItemType Directory'),
    'invalid Cargo override should fail before cache directories are created');
  assert.match(launcher,/DEV\.cmd storage cargo/);
  const scripts=JSON.parse(read('package.json')).scripts;
  assert.equal(scripts['storage:cargo'],'node tooling/storage/cargo-footprint.mjs');
  assert.match(scripts['test:localtest'],/tooling\/storage\/cargo-footprint\.test\.mjs/);
});

test('scoped frontend and native source checks avoid accidental full verification', () => {
  const rootManifest=JSON.parse(read('package.json'));
  const desktopManifest=JSON.parse(read('apps/desktop/package.json'));
  const rootScripts=rootManifest.scripts, desktopScripts=desktopManifest.scripts;
  assert.equal(rootScripts['check:frontend'], 'npm --prefix apps/desktop run check:frontend');
  assert.equal(desktopScripts['check:frontend'], 'npm run typecheck && npm run test');
  assert.doesNotMatch(desktopScripts['check:frontend'], /cargo|tauri|build:frontend|verify:source/i);
  assert.equal(rootScripts['check:rust'],
    'npm --prefix apps/desktop run check:rust:target && cargo check --locked --target x86_64-pc-windows-msvc --manifest-path engine/runtime-core/Cargo.toml');
  assert.ok(desktopScripts['check:rust:target'].includes('cargo check --locked --target x86_64-pc-windows-msvc'));
  assert.ok(desktopScripts['check:rust:target'].includes('prepare:icons'));
  assert.ok(!desktopScripts['check:rust:target'].includes('--all-targets'));
  assert.ok(desktopScripts['check:rust'].includes('cargo check --all-targets --locked'));
  assert.ok(rootScripts['verify:source'].includes('npm run check:desktop'));
  assert.ok(desktopScripts['verify:source'].includes('npm run build:frontend'));
  assert.ok(desktopScripts['verify:source'].includes('npm run test:rust'));
  assert.ok(rootScripts['verify:source'].includes('npm run test:runtime'));
});

test('benchmark uses exact Local checkout and two cache-preserving passes', () => {
  const section=developer.slice(developer.indexOf('function Invoke-SourceBenchmark {'),
    developer.indexOf('Push-Location $DevelopmentRoot'));
  assert.ok(section.startsWith('function Invoke-SourceBenchmark {'));
  assert.match(section,/Assert-LocalCheckout/);
  assert.match(section,/Assert-CleanSourceAtHead -ExpectedSha \$sha/);
  assert.match(section,/foreach \(\$pass in 1\.\.2\)/);
  assert.match(section,/Stopwatch\]::StartNew\(\)/);
  assert.match(section,/Invoke-RepositoryNpm -NpmArguments @\("run", \$scriptName\)/);
  assert.match(section,/no PASS recorded/);
  assert.match(section,/No cold\/warm classification/);
  assert.doesNotMatch(section,/\b(?:Remove-Item|Clear-Content|cargo clean|rm -rf|Start-Process)\b/i);
  for (const cmd of ['check:frontend','check:rust','bench:frontend','bench:rust']) {
    assert.ok(developer.includes('"'+cmd+'"'), 'missing developer route: '+cmd);
  }
  assert.match(developer,/Test-DeveloperToolchain/);
  assert.match(launcher,/DEV\.cmd bench:frontend/);
  assert.match(launcher,/DEV\.cmd bench:rust/);
});


test('Rust benchmark compares active-target logical bytes outside timed source passes', () => {
  const inspector=read('tooling/storage/cargo-footprint.mjs');
  const section=developer.slice(developer.indexOf('function Get-CargoTargetSnapshot {'),
    developer.indexOf('Push-Location $DevelopmentRoot'));
  assert.ok(section.startsWith('function Get-CargoTargetSnapshot {'));
  assert.match(section,/--active-json/);
  assert.match(section,/scope -cne 'active-cargo-target'/);
  assert.match(section,/complete -cne \$true/);
  assert.match(section,/ConvertFrom-Json/);
  assert.match(section,/UNKNOWN \(read-only scan failed or was partial\)/);
  assert.match(section,/logical-byte change after pass/);
  assert.match(section,/Observed size change is not attributable savings/);
  assert.match(inspector,/schema:1,scope:'active-cargo-target'/);
  assert.match(inspector,/maxDurationMs:15000,maxEntries:250000/);
  assert.doesNotMatch(section,/\b(?:Remove-Item|Clear-Content|cargo clean|Start-Process)\b/i);
  const start=section.indexOf('$watch = [System.Diagnostics.Stopwatch]::StartNew()');
  const stop=section.indexOf('$watch.Stop()',start);
  assert.ok(start>=0&&stop>start);
  assert.ok(!section.slice(start,stop).includes('Get-CargoTargetSnapshot'),
    'file scans must not be included in compilation timing');
});
test('portable setup scripts parse in Windows PowerShell without running downloads', {
  skip: process.platform !== 'win32',
}, () => {
  for (const name of [
    'tooling/windows-toolchain/bootstrap.ps1',
    'tooling/windows-toolchain/local-env.ps1',
    'tooling/windows-toolchain/dev.ps1',
  ]) {
    const file=resolve(root,name).replace(/'/g,"''");
    const command = "$tokens=$null;$errors=$null;" +
      "[System.Management.Automation.Language.Parser]::ParseFile('" + file +
      "',[ref]$tokens,[ref]$errors)|Out-Null;" +
      "if($errors.Count){$errors|ForEach-Object{Write-Error $_};exit 1}";
    const result=spawnSync('powershell.exe',
      ['-NoProfile','-NonInteractive','-Command',command],
      {encoding:'utf8',windowsHide:true,timeout:30000});
    assert.equal(result.status,0, name+' parser errors: '+(result.stderr||result.error));
  }
});
