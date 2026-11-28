// Operator-selected agent configuration; no native operation or lifecycle ownership.
import path from 'node:path';
import {assertAgentSurface} from './agent-contract.mjs';
import {validateDiagnosticsRoot} from './diagnostics-local.mjs';
import {validateTestingArchiveRoot} from './testing-result-store.mjs';
import {validateCuaExecutable} from './visual-capture.mjs';
import {validateGpuExecutable} from './gpu-frame-source.mjs';

export function parseConfiguration(argv, env = process.env) {
  let mode = 'human-plus-four-ai';
  let surface = 'legacy';
  let surfaceSelected=false;
  let executable = env.LAZY_TESTER_AGENT_EXE ?? '';
  let evidenceDirectory = env.LAZY_TESTER_EVIDENCE_DIR ?? '';
  let diagnosticsDirectory = env.LAZY_TESTER_DIAGNOSTICS_DIR ?? '';
  let testingArchiveDirectory = env.LAZY_TESTER_TESTING_ARCHIVE_DIR ?? '';
  let probeZip = env.LAZY_TESTER_PROBE_ZIP ?? '';
  let mapContractPath = env.LAZY_TESTER_MAP_CONTRACT ?? '';
  let cuaExecutable = env.LAZY_TESTER_CUA_EXE ?? '';
  let gpuExecutable = env.LAZY_TESTER_GPU_CAPTURE_EXE ?? '';
  let experimentalMc02InputEnabled=false;
  let experimentalMultiClientInputEnabled=false;
  for (let i=0;i<argv.length;i++) {
    const flag=argv[i];
    if (!['--surface','--mode','--exe','--evidence-dir','--diagnostics-dir','--testing-archive-dir','--probe-zip','--map-contract','--cua-exe','--gpu-capture-exe','--mc02-input-experimental','--multi-client-input-experimental'].includes(flag) || i===argv.length-1) {
      throw new Error('Accepted arguments: [--surface legacy|code-mode|playtest] --mode human-plus-four-ai|five-ai --exe <lazy-tester.exe> [--evidence-dir <folder>] [--diagnostics-dir <project|Data/Diagnostics>] [--testing-archive-dir <project|Data/Testing>] [--map-contract <map-test-contract.json>] [--cua-exe <cua-driver.exe>] [--gpu-capture-exe <lazy-tester-gpu-capture.exe>]');
    }
    const value=argv[++i];
    if (flag==='--mode') mode=value;
    if (flag==='--surface') {
      // Optional connection profile selection must never silently override
      // an earlier restricted profile.
      if(surfaceSelected)throw new Error('Select exactly one agent surface');
      surfaceSelected=true;
      surface=value;
    }
    if (flag==='--exe') executable=value;
    if (flag==='--evidence-dir') evidenceDirectory=value;
    if (flag==='--diagnostics-dir') diagnosticsDirectory=value;
    if (flag==='--testing-archive-dir') testingArchiveDirectory=value;
    if (flag==='--probe-zip') probeZip=value;
    if (flag==='--map-contract') mapContractPath=value;
    if (flag==='--cua-exe') cuaExecutable=value;
    if (flag==='--gpu-capture-exe') gpuExecutable=value;
    if (flag==='--mc02-input-experimental') {
      if(value!=='on') throw new Error('MC02 experimental input requires explicit on');
      experimentalMc02InputEnabled=true;
    }
    if (flag==='--multi-client-input-experimental') {
      if(value!=='on')throw new Error('Multi-client input opt-in requires explicit on');
      experimentalMultiClientInputEnabled=true;
    }
  }
  assertAgentSurface(surface,experimentalMc02InputEnabled||experimentalMultiClientInputEnabled);
  if(experimentalMc02InputEnabled&&experimentalMultiClientInputEnabled) {
    throw new Error('Select only one experimental native input mode');
  }
  if (!['human-plus-four-ai','five-ai'].includes(mode)) throw new Error('Unknown test mode');
  if(experimentalMc02InputEnabled &&
     (mode!=='human-plus-four-ai' ||
      env.LAZY_TESTER_MC02_VIRTUAL_INPUT!=='I_ACCEPT_EXPERIMENTAL_MC02_INPUT')) {
    throw new Error('MC02 experiment needs human-plus-four-ai mode and explicit native opt-in');
  }
  if(experimentalMultiClientInputEnabled &&
     env.LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT!=='I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT') {
    throw new Error('Multi-client input requires explicit native opt-in');
  }
  if (executable && (!path.isAbsolute(executable) || !/\.exe$/i.test(executable))) {
    throw new Error('Set --exe to the absolute path of the installed lazy-tester.exe');
  }
  if (evidenceDirectory && !path.isAbsolute(evidenceDirectory)) {
    throw new Error('Evidence directory must be absolute');
  }
  if (diagnosticsDirectory || testingArchiveDirectory) {
    // Match the native App/../Data layout only. Do not guess a root for a
    // developer build, legacy flat install, or archive-only invocation.
    const appFolder=executable ? path.dirname(executable) : '';
    const projectData=appFolder && path.basename(appFolder).toLowerCase()==='app'
      && path.basename(executable).toLowerCase()==='lazy-tester.exe'
      && path.normalize(executable)===executable
      ? path.join(path.dirname(appFolder),'Data') : '';
    const archivePath=(value,child)=>{
      if(value!=='project') return value;
      if(!projectData)throw new Error('Project archive selection requires an absolute App/lazy-tester.exe path');
      return path.join(projectData,child);
    };
    diagnosticsDirectory=archivePath(diagnosticsDirectory,'Diagnostics');
    testingArchiveDirectory=archivePath(testingArchiveDirectory,'Testing');
    if (diagnosticsDirectory) validateDiagnosticsRoot(diagnosticsDirectory);
    if (testingArchiveDirectory)validateTestingArchiveRoot(testingArchiveDirectory);
    // Configuration consistency, not filesystem identity or access authority.
    // Existing archive readers/writers still own their per-operation checks.
    const roots=[diagnosticsDirectory,testingArchiveDirectory].filter(Boolean).map(dir=>path.dirname(dir));
    const expected=projectData || roots[0];
    if(roots.some(root=>path.relative(expected,root)!=='')) {
      throw new Error('Diagnostics and Testing must use the same Project Data folder');
    }
  }
  if (probeZip && (!path.isAbsolute(probeZip) || !/\.zip$/i.test(probeZip))) {
    throw new Error('Probe ZIP must be an absolute .zip file path');
  }
  if (mapContractPath && (!path.isAbsolute(mapContractPath) ||
      path.extname(mapContractPath).toLowerCase()!=='.json')) {
    throw new Error('Map contract must be an absolute .json file path');
  }
  if (cuaExecutable) validateCuaExecutable(cuaExecutable);
  if (gpuExecutable) validateGpuExecutable(gpuExecutable);
  return Object.freeze({mode, surface, executable, evidenceDirectory,diagnosticsDirectory,
    testingArchiveDirectory,probeZip,mapContractPath,
    cuaExecutable,gpuExecutable,experimentalMc02InputEnabled,
    experimentalMultiClientInputEnabled});
}
