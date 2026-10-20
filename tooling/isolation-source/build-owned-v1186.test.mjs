import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const source = readFileSync(new URL('./build-owned-v1186.ps1', import.meta.url), 'utf8').replace(/\r\n/g, '\n');
const profile = JSON.parse(readFileSync(
  new URL('../../engine/isolation-native/owned-runtime/build-profile-v1186.json', import.meta.url),
  'utf8',
));

test('v1.18.6 builder checks exact owned C/C++ source before any MSBuild call', () => {
  assert.match(source, /verify-owned-v1186\.mjs/);
  assert.match(source, /MIVUBI_V1186_NATIVE_SOURCE_PATCHED_NOT_BUILT/);
  assert.match(source, /nativeModifiedFiles -ne 11/);
  assert.match(source, /sourceFiles -ne 1627/);
  assert.match(source, /privateDriverIdentity -cne '5\.73\.6-MIVU'/);
  assert.match(source, /Working tree must be clean/);
  assert.ok(source.indexOf('verify-owned-v1186.mjs') < source.indexOf('foreach ($stage in $stages)'));
});

test('builder only compiles and captures unsigned artifacts, never installs a driver', () => {
  for (const stage of ['SandboxDll.sln','Sandbox.sln','SandboxDrv.sln']) {
    assert.ok(source.includes(stage), 'missing build stage '+stage);
  }
  for (const path of ['SbieDrv.sys','SbieSvc.exe','SbieDll.dll','Start.exe',
    'SbieIni.exe','kmdutil.exe']) {
    assert.ok(profile.requiredArtifacts.some(x => x.endsWith('/'+path)), 'missing '+path);
  }
  assert.match(source, /\$required = @\(\$spec\.requiredArtifacts\)/);
  assert.match(source, /\$stages = @\(\$spec\.buildStages\)/);
  assert.equal(profile.requiredArtifacts.length, 16);
  assert.equal(profile.buildStages.length, 3);
  assert.equal(profile.sourceResources.length, 5);
  assert.match(source, /Preexisting native binary would contaminate build evidence/);
  assert.match(source, /ls-files --others --ignored --exclude-standard -- engine\/isolation-owned/);
  assert.match(source, /Ignored native build outputs exist; use a fresh clean source checkout/);
  assert.match(source, /NATIVE_BUILD_ARTIFACTS_CAPTURED_NOT_INSTALLABLE/);
  assert.match(source, /driverSigned = \$false/);
  assert.match(source, /'\/p:SignMode=Off'/);
  assert.match(source, /'\/p:EnableTestSign=false'/);
  assert.match(source, /driverLoaded = \$false/);
  assert.match(source, /installPermitted = \$false/);
  assert.match(source, /activeProviderChanged = \$false/);
  assert.doesNotMatch(source, /\b(Start-Service|New-Service|sc\.exe|pnputil|signtool|bcdedit)\b/i);
  assert.match(source, /Report destination exists; use a new directory/);
});

test('builder resolves VS2022 deterministically and captures native MSBuild failure evidence', () => {
  assert.match(source, /Visual Studio\/Installer\/vswhere\.exe/);
  assert.match(source, /-version '\[17\.0,18\.0\)'/);
  assert.match(source, /MSBuild\/Current\/Bin\/MSBuild\.exe/);
  assert.match(source, /\$msbuildDigest = \(Get-FileHash -LiteralPath \$msbuild -Algorithm SHA256\)/);
  assert.match(source, /compiler = \[ordered\]@\{/);
  assert.match(source, /fileVersion = \[string\]\$msbuildFile\.VersionInfo\.FileVersion/);
  assert.match(source, /sha256 = \$msbuildDigest/);
  assert.doesNotMatch(source, /Get-Command msbuild\.exe/);
  assert.match(source, /Start-Process -FilePath \$msbuild/);
  assert.match(source, /-RedirectStandardOutput \$stdout/);
  assert.match(source, /-RedirectStandardError \$stderr/);
  assert.match(source, /stageResult\.exitCode = \[int\]\$process\.ExitCode/);
  assert.match(source, /stageResult\.launchFailure = \$_\.Exception\.Message/);
  assert.match(source, /\/nodeReuse:false/);
  assert.match(source, /\$report\.buildStages = \$results/);
  assert.match(source, /build-result\.json/);
  assert.match(source, /\$report\.failureReason = \$_\.Exception\.Message/);
});

test('VS discovery captures exit status in a fresh strict PowerShell session', t => {
  const vswhere = join(process.env['ProgramFiles(x86)'] || 'C:/Program Files (x86)',
    'Microsoft Visual Studio/Installer/vswhere.exe');
  if (process.platform !== 'win32' || !existsSync(vswhere)) {
    t.skip('Windows VS discovery executable unavailable');
    return;
  }
  const discovery = source.slice(source.indexOf('$installations ='),
    source.indexOf('$installationRoot ='));
  const script = `Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Remove-Variable LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue
$vswhere = '${vswhere.replaceAll("'", "''")}'
${discovery}
if (-not $installation) { throw 'Missing installation' }
Write-Output 'DISCOVERY_OK'`;
  const result = spawnSync('powershell.exe', ['-NoLogo', '-NoProfile', '-NonInteractive',
    '-EncodedCommand', Buffer.from(script, 'utf16le').toString('base64')],
  { encoding: 'utf8', timeout: 30000 });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /DISCOVERY_OK/);
});

test('compiler wait captures a real nonzero exit without waiting for a process tree', t => {
  if (process.platform !== 'win32') { t.skip('Windows process launch unavailable'); return; }
  const fixture = mkdtempSync(join(tmpdir(), 'mivubi-compiler-exit-'));
  try {
    const launch = source.slice(source.indexOf('      $process = Start-Process'),
      source.indexOf('\n    }\n    catch', source.indexOf('      $process = Start-Process')));
    const script = `Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$msbuild = $env:ComSpec
$source = '${fixture.replaceAll("'", "''")}'
$stdout = Join-Path $source 'stdout.log'
$stderr = Join-Path $source 'stderr.log'
$arguments = @('/d', '/c', 'exit 7')
$stageResult = @{exitCode=$null}
${launch}
if ($stageResult.exitCode -ne 7) { throw 'Compiler exit was lost' }
Write-Output 'EXIT_CAPTURE_OK'`;
    const result = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-EncodedCommand',
      Buffer.from(script, 'utf16le').toString('base64')], {encoding:'utf8', timeout:30000});
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /EXIT_CAPTURE_OK/);
  } finally { rmSync(fixture, {recursive:true, force:true}); }
});

test('build stages match the actual v1.18.6 solution projects and output directories', () => {
  const root = new URL('../../engine/isolation-owned/Sandboxie/', import.meta.url);
  const read = name => readFileSync(new URL(name, root), 'utf8');
  const stages = [
    ['SandboxDll.sln', 'Win32', ['SboxSvc', 'SboxDll']],
    ['Sandbox.sln', 'x64', [
      'SboxSvc', 'SboxDll', 'Start', 'SbieIni', 'KmdUtil', 'SboxMsg',
      'SandboxRpcSs', 'SboxDcomLaunch', 'SandboxCrypto', 'SandboxBITS',
      'SandboxWUAU', 'SbieControl', 'SboxHostDll',
    ]],
    ['SandboxDrv.sln', 'x64', ['SboxDrv']],
  ];
  for (const [solution, platform, projectNames] of stages) {
    const contents = read(solution);
    assert.ok(contents.includes('SbieRelease|'+platform),
      'missing stage configuration: '+solution);
    for (const project of projectNames) {
      const line = contents.split(/\r?\n/).find(item => item.includes(' = "'+project+'", '));
      assert.ok(line, 'solution omits '+project+': '+solution);
      const id = line.match(/, "\{([0-9a-fA-F-]{36})\}"\s*$/)?.[1];
      assert.ok(id, 'unrecognized project GUID: '+project);
      assert.ok(contents.includes('{'+id+'}.SbieRelease|'+platform+'.Build.0'),
        project+' is not scheduled to build in '+platform);
    }
  }
  for (const propsFile of ['Sandbox32.props', 'Sandbox64.props']) {
    const props = read(propsFile);
    assert.ok(props.includes('<OutDir>$(SolutionDir)Bin\\$(PlatformName)\\$(Configuration)\\</OutDir>'),
      'upstream output path drift: '+propsFile);
  }
  assert.ok(profile.requiredArtifacts.includes('Sandboxie/Bin/Win32/SbieRelease/SbieSvc.exe'));
  assert.ok(profile.requiredArtifacts.includes('Sandboxie/Bin/x64/SbieRelease/SbieDrv.sys'));
  assert.equal(new Set(profile.requiredArtifacts).size, profile.requiredArtifacts.length);
});

test('build-only stages reject incomplete outputs before starting the next expensive stage', () => {
  const dirs = {
    'win32-support': profile.requiredArtifacts.filter(path =>
      path.startsWith('Sandboxie/Bin/Win32/SbieRelease/')),
    'x64-user-core': profile.requiredArtifacts.filter(path =>
      path.startsWith('Sandboxie/Bin/x64/SbieRelease/') &&
      !path.endsWith('/SbieDrv.sys')),
    'x64-wdk-kernel': profile.requiredArtifacts.filter(path =>
      path.endsWith('/SbieDrv.sys')),
  };
  assert.deepEqual(Object.fromEntries(
    Object.entries(dirs).map(([name, files]) => [name, files.length])),
    { 'win32-support': 2, 'x64-user-core': 13, 'x64-wdk-kernel': 1 });
  assert.equal(new Set(Object.values(dirs).flat()).size, profile.requiredArtifacts.length);
  assert.match(source, /\$stageRequired = @\(\$required \| Where-Object/);
  assert.match(source, /\$expectedStageCount = switch \(\$stage\.id\)/);
  assert.match(source, /\$stageRequired\.Count -ne \$expectedStageCount/);
  assert.match(source, /Native stage \$\(\$stage\.id\) succeeded but required output is missing/);
  assert.match(source, /Native stage \$\(\$stage\.id\) produced an empty required output/);
  assert.ok(source.indexOf('foreach ($rel in $stageRequired)') <
    source.indexOf('  $artifacts = @()'),
    'fail-fast stage gate must precede full payload artifact collection');
  // The full output hash loop remains a separate final integrity check.
  assert.match(source, /\$digest = \(Get-FileHash -LiteralPath \$path -Algorithm SHA256\)/);
  // The upstream x64 main solution intentionally does not build the driver.
  const main = readFileSync(
    new URL('../../engine/isolation-owned/Sandboxie/Sandbox.sln', import.meta.url), 'utf8');
  const driver = main.split(/\r?\n/).find(line =>
    line.includes(' = "SboxDrv", '));
  assert.ok(driver, 'upstream driver project not found');
  const guid = driver.match(/, "\{([0-9a-fA-F-]{36})\}"\s*$/)?.[1];
  assert.ok(guid, 'unrecognized upstream driver project GUID');
  assert.ok(!main.includes('{'+guid+'}.SbieRelease|x64.Build.0'),
    'x64 stage unexpectedly builds driver; reconsider stage ownership');
});

test('native build report is restricted to ignored workspace/builds without junction ancestors', () => {
  const guard = source.indexOf("$reportAbsolute = [IO.Path]::GetFullPath($ReportDirectory)");
  const preflight = source.indexOf("Build report must be a new child of workspace/builds within the repository.");
  const gitCheck = source.indexOf("$dirty = (& $git -C $repo status");
  const reportCreation = source.indexOf("New-Item -ItemType Directory -Path $reportFolder");
  const compile = source.indexOf("foreach ($stage in $stages)");
  assert.ok(guard >= 0 && guard < preflight && preflight < gitCheck);
  assert.ok(preflight < reportCreation && reportCreation < compile,
    'a dirtying report directory must be rejected before either Git proof or MSBuild');
  assert.match(source, /\$reportRoot = \[IO\.Path\]::GetPathRoot\(\$reportAbsolute\)/);
  assert.match(source, /Build report destination must not be a drive root/);
  assert.match(source, /\$workspaceRoot = Join-Path \$repoFolder 'workspace'/);
  assert.match(source, /\$reportAllowed = Join-Path \$workspaceRoot 'builds'/);
  assert.match(source, /\$reportFolder\.StartsWith\(\$reportAllowed \+ /);
  assert.match(source, /\$ancestor = \$reportParent/);
  assert.match(source, /while \(\$ancestor\) \{/);
  assert.match(source, /\$parentItem = Get-Item -LiteralPath \$ancestor -Force/);
  assert.match(source, /\$next = Split-Path -Parent \$ancestor/);
  assert.match(source, /Build report parent must not be a Windows reparse point/);
  assert.equal((source.match(/Build report must be a new child of workspace\/builds within the repository/g) || []).length, 1);
  // The optional package output must remain separately preflighted.
  assert.match(source, /Engine package output must be a new child of workspace\/packages within the repository/);
  assert.doesNotMatch(source, /Build report may not overlap native source/);
});

test('optional engine package target is refused early before expensive MSBuild work', () => {
  const gate = source.indexOf('$packageAbsolute = [IO.Path]::GetFullPath($PackageDirectory)');
  const compiler = source.indexOf('foreach ($stage in $stages)');
  assert.ok(gate > 0 && gate < compiler, 'package path admission must precede MSBuild');
  assert.ok(gate < source.indexOf('New-Item -ItemType Directory -Path $reportFolder'),
    'bad output must not create a report folder');
  for (const reason of [
    'Engine package output must not be a drive root.',
    'Engine package output must be a new child of workspace/packages within the repository.',
    'Engine package destination already exists; refuse replacement.',
    'Engine package destination parent must exist before native compilation.',
    'Engine package parent must not be a Windows reparse point.',
  ]) {
    assert.ok(source.includes(reason), 'missing non-destructive path refusal: '+reason);
  }
  assert.match(source, /\$packageAllowed = Join-Path \$workspaceRoot 'packages'/);
  assert.match(source, /\$packageFolder\.StartsWith\(\$packageAllowed \+ /);
  assert.match(source, /\$parentItem\.Attributes -band \[IO\.FileAttributes\]::ReparsePoint/);
  assert.match(source, /\$ancestor = \$packageParent/);
  assert.match(source, /while \(\$ancestor\)/);
  assert.match(source, /\$parentItem = Get-Item -LiteralPath \$ancestor/);
  assert.match(source, /\$next = Split-Path -Parent \$ancestor/);
  assert.ok(source.indexOf('while ($ancestor)') <
    source.indexOf('New-Item -ItemType Directory -Path $reportFolder'),
    'junction-ancestor checks must precede all native work');
  assert.match(source, /\$node \$assembler --build-report \$output --output \$PackageDirectory/);
});

test('ignored native build artifacts cannot evade the clean-tree preflight', () => {
  const fixture = mkdtempSync(join(tmpdir(), 'mivubi-native-build-'));
  const command = (...args) => {
    const result = spawnSync('git', args, { cwd: fixture, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr || 'git fixture failed');
    return result.stdout.trim();
  };
  try {
    command('init', '-q');
    command('config', 'user.email', 'test@example.invalid');
    command('config', 'user.name', 'MIVUBI build test');
    const core = join(fixture, 'engine', 'isolation-owned', 'Sandboxie', 'core');
    mkdirSync(core, { recursive: true });
    writeFileSync(join(fixture, '.gitignore'), '*.obj\n');
    writeFileSync(join(core, 'test.c'), 'int native_source;\n');
    command('add', '.gitignore', 'engine/isolation-owned/Sandboxie/core/test.c');
    command('commit', '-qm', 'fixture');
    writeFileSync(join(core, 'stale.obj'), 'stale compiler data');
    assert.equal(command('status', '--porcelain=v1', '--untracked-files=all'), '');
    assert.match(command('ls-files', '--others', '--ignored', '--exclude-standard',
      '--', 'engine/isolation-owned'), /stale\.obj/);
    assert.match(source, /ls-files --others --ignored --exclude-standard -- engine\/isolation-owned/);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
