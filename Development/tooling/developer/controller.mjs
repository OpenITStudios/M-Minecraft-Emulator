// Single AI-facing developer entrypoint. Existing PowerShell/npm/Cargo/Tauri
// owners remain the only command executors. No arbitrary shell execution.
import {lstatSync, fstatSync, readFileSync, openSync, closeSync, unlinkSync, writeSync, constants} from 'node:fs';
import {spawn, spawnSync} from 'node:child_process';
import {dirname, join, resolve, isAbsolute, posix, win32} from 'node:path';
import {fileURLToPath} from 'node:url';
import {beginSmoke,readSmoke,recordSmoke} from '../local-test/session.mjs';

export const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const REGISTRY = resolve(dirname(fileURLToPath(import.meta.url)), 'tasks.json');
const DEV_SCRIPT = join(REPO, 'Development/tooling/windows-toolchain/dev.ps1');
const REPOSITORY_CHECK = join(REPO, 'Development/tooling/repository/check-structure.mjs');
const ARCHIVE_RETIRE = join(REPO, 'Development/tooling/storage/archive-retire.mjs');
const NODE_EXE = join(REPO, 'workspace/toolchains/node/node.exe');
const PWSH_EXE = join(REPO, 'workspace/toolchains/powershell/pwsh.exe');
const LOCK_FILE = join(REPO, 'workspace/tmp/developer-controller.lock');

// Audited command vocabulary, not a way to execute arbitrary programs.
// Task ids, descriptions and routing live in tasks.json only.
const POLICY = Object.freeze({
  'doctor': {risk:'local-setup', approval:false},
  'setup': {risk:'dependency-write', approval:true},
  'lock': {risk:'source-write', approval:true},
  'repository-structure': {risk:'read', approval:false},
  'check:frontend': {risk:'build', approval:false},
  'check:rust': {risk:'build', approval:false},
  'check': {risk:'build', approval:false},
  'check:quick': {risk:'build', approval:false},
  'preflight': {risk:'dependency-write', approval:true},
  'test': {risk:'build', approval:false},
  'dev': {risk:'interactive', approval:false},
  'package': {risk:'package-write', approval:true},
  'storage': {risk:'read', approval:false},
  'retire-archive': {risk:'archive-write', approval:true},
  'bench:frontend': {risk:'build', approval:false},
  'bench:rust': {risk:'build', approval:false},
});
const STORAGE_MODES = new Set(['report','explain','plan','cargo']);
const ID = /^[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*$/;
const SHA = /^[a-f0-9]{40}$/;
const SCOPE = new Set(['environment','repository','frontend','rust','source','desktop','package','storage','benchmark']);
const PLAN_PROFILES = new Set(['fast','local-smoke','five-client','release','experimental']);
const LOCAL_SMOKE_CHECKPOINTS = Object.freeze([
  Object.freeze({id:"LT-01",phase:"readiness",action:"Pin clean Local SHA, Windows x64, data backup policy and reviewed standard DLL paths",passCondition:"Exact revision and prerequisites; do not treat missing payload as PASS",evidenceKind:"METADATA",stopOnFail:true,legacyCases:"STD-SRC-01..05;STD-SRC-13"}),
  Object.freeze({id:"LT-02",phase:"source",action:"Execute approved check:quick on clean SHA",passCondition:"Real exit 0 and source-quick receipt; full source verification remains NOT_EXECUTED",evidenceKind:"SOURCE_QUICK",stopOnFail:true,legacyCases:"STD-SRC-05;STD-SRC-11 (partial only)"}),
  Object.freeze({id:"LT-03",phase:"desktop",action:"Manually launch the approved Tauri manager",passCondition:"One rendered responsive manager window",evidenceKind:"WINDOW_OBSERVATION",stopOnFail:true,legacyCases:"STD-UI-01"}),
  Object.freeze({id:"LT-04",phase:"desktop",action:"Inspect fixed cards MC01–MC05 without launching game",passCondition:"Exactly five stable slots; no automatic game, Microsoft sign-in or AI input",evidenceKind:"WINDOW_OBSERVATION",stopOnFail:true,legacyCases:"STD-UI-02..04"}),
  Object.freeze({id:"LT-05",phase:"desktop",action:"Use Refresh then close/reopen an idle manager",passCondition:"No duplicate native controller, fabricated running state or data rewrite",evidenceKind:"WINDOW_OBSERVATION",stopOnFail:true,legacyCases:"STD-UI-05;STD-UI-11"}),
  Object.freeze({id:"LT-06",phase:"mc01",action:"Manually sign into licensed MC01 and open dummy world",passCondition:"Rendered playable world with physical movement/camera/interaction",evidenceKind:"MINECRAFT_RUNTIME",stopOnFail:true,legacyCases:"STD-GAME-01..03"}),
  Object.freeze({id:"LT-07",phase:"mc01",action:"Normal save/close and reopen MC01",passCondition:"Saved world and original account/profile survive a normal close",evidenceKind:"MINECRAFT_RUNTIME",stopOnFail:true,legacyCases:"STD-GAME-04..05"}),
  Object.freeze({id:"LT-08",phase:"mc02",action:"Launch MC02 while MC01 remains playable",passCondition:"Distinct licensed account and PID/birth/HWND slot; two independently playable worlds",evidenceKind:"MINECRAFT_RUNTIME",stopOnFail:true,legacyCases:"STD-GAME-06..08;STD-GAME-12"}),
  Object.freeze({id:"LT-09",phase:"two-client",action:"Switch selected game, focus and workspace surface",passCondition:"Correct window/slot, no accidental cross-client input",evidenceKind:"WINDOW_IDENTITY",stopOnFail:true,legacyCases:"STD-WIN-01;STD-WIN-03;STD-WIN-05"}),
  Object.freeze({id:"LT-10",phase:"two-client",action:"Normal CloseOne on MC02 while MC01 remains active",passCondition:"Only MC02 closes after game confirmation; MC01 remains interactive",evidenceKind:"NATIVE_LIFECYCLE",stopOnFail:true,legacyCases:"STD-LIFE-01..04"}),
  Object.freeze({id:"LT-11",phase:"two-client",action:"Close/reopen manager with an owned game still active",passCondition:"Original session is correctly recovered or explicitly UNKNOWN; never fake STOPPED",evidenceKind:"NATIVE_LIFECYCLE",stopOnFail:true,legacyCases:"STD-LIFE-12..13"}),
  Object.freeze({id:"LT-12",phase:"data",action:"Inspect only bounded storage metadata across stopped slots",passCondition:"No reset, account/world disclosure or cross-slot folder confusion",evidenceKind:"STORAGE_METADATA",stopOnFail:true,legacyCases:"STD-DATA-01..03"}),
  Object.freeze({id:"LT-13",phase:"ui-smoke",action:"Paste negative/fraction XYZ and inspect read-only F3",passCondition:"Correct manual formatting; live XYZ/Copy unavailable until actual source exists",evidenceKind:"UI_OBSERVATION",stopOnFail:false,legacyCases:"STD-SET-12..13;STD-SET-15..16"})
]);
const MANUAL_PLANS = Object.freeze({
  'local-smoke': Object.freeze({
    taskIds:['source.quick'],
    manualCaseIds:LOCAL_SMOKE_CHECKPOINTS.map(step=>step.id),
    prerequisites:['Windows x64 and reviewed standard DLLs','Approved pinned toolchain and locked npm setup','Operator-approved test profiles and disposable Minecraft world'],
    reason:'First same-revision baseline is independent of the last commit diff; quick source screening is not full source acceptance.'
  }),
  'five-client': Object.freeze({
    taskIds:[],
    manualCaseIds:['FC-01','FC-02','FC-03','FC-04'],
    prerequisites:['Local smoke PASS on the same exact SHA','Five distinct licensed Education accounts','Explicit Windows/Minecraft runtime authorization'],
    reason:'Extend an accepted two-client baseline to five live clients; no repeated source work without new changes.'
  }),
  'release': Object.freeze({
    taskIds:['source.preflight','build.candidate'],
    manualCaseIds:['RL-01','RL-02','RL-03'],
    prerequisites:['Separate approval for full source verification and package staging','Reviewed standard DLLs','Disposable Windows user/PC and data backup'],
    reason:'Full verification and installer/preservation are separate, explicitly consent-gated steps; this plan never dispatches them.'
  }),
  'experimental': Object.freeze({
    taskIds:[],
    manualCaseIds:['EX-01','EX-02','EX-03','EX-04'],
    prerequisites:['Separate experimental permission','Reviewed AI feature/ABI/digest and native payload (not present in standard package)','Prior same-revision standard game acceptance'],
    reason:'AI input, visual observation and Skills Probe are isolated from standard product smoke; missing AI binary remains BLOCKED.'
  })
});
export function planForProfile(profile,revision,paths=[]) {
  if(!PLAN_PROFILES.has(profile)) throw Error('Unknown local-test profile');
  if(profile==='fast') return {...planForPaths(paths,revision),profile:'fast',manualCaseIds:[],
    manualGuide:'docs/validation/efficient-local-testing.md'};
  if(paths.length!==0) throw Error('Manual test profiles never route changed paths or automatic execution');
  const cfg=MANUAL_PLANS[profile];
  return {status:'READY',sourceSha:revision,profile,taskIds:[...cfg.taskIds],
    manualCaseIds:[...cfg.manualCaseIds],prerequisites:[...cfg.prerequisites],
    manualGuide:'docs/validation/efficient-local-testing.md',manualExecution:'OPERATOR_ONLY',
    checkpoints:profile==='local-smoke'?LOCAL_SMOKE_CHECKPOINTS.map(step=>({...step})):[],
    reason:cfg.reason};
}

export function validatedRegistry(input) {
  if(!input || input.schema!==1 || !Array.isArray(input.tasks) ||
     input.tasks.length<1 || input.tasks.length>50) throw Error('Invalid developer task registry');
  const seen=new Set();
  return Object.freeze(input.tasks.map(value=>{
    if(!value || typeof value.id!=='string' || !ID.test(value.id) || seen.has(value.id) ||
       typeof value.description!=='string' || value.description.length<8 || value.description.length>250 ||
       typeof value.command!=='string' || !Object.hasOwn(POLICY,value.command) ||
       !SCOPE.has(value.scope) || Object.keys(value).some(k=>!['id','description','command','scope','mode'].includes(k))) {
      throw Error('Invalid or duplicate developer task entry');
    }
    if(value.command==='storage' ? !STORAGE_MODES.has(value.mode) : value.mode!==undefined) {
      throw Error('Invalid fixed task mode');
    }
    seen.add(value.id);
    return Object.freeze({...value,risk:POLICY[value.command].risk,
      requiresApproval:POLICY[value.command].approval});
  }));
}

export function classifyPath(path) {
  if(typeof path!=='string' || !path || path.length>512 || path.includes('\\') ||
     isAbsolute(path) || path.startsWith('/') || path.includes('\0')) return 'unknown';
  const clean=posix.normalize(path);
  if(clean!==path || clean==='..' || clean.startsWith('../') || path.startsWith('-')) return 'unknown';
  if(/^(?:Data|App|workspace|\.git)(?:\/|$)/i.test(path)) return 'protected';
  // Markdown guides and skill instructions are not compiled inputs, even when
  // stored beside native source. Do not downgrade executable or unknown files
  // simply because they live beneath docs/, planning/, or Archive/.
  if(/^(?:docs|planning|Archive)\/.*\.md$/.test(path) ||
     /^(?:README|CONTRIBUTING|SECURITY|CONTEXT|GITHUB_RULES|AGENTS)\.md$/.test(path) ||
     /(?:^|\/)(?:README|AGENTS)\.md$/.test(path) ||
     /^\.agents\/skills\/[^/]+\/SKILL\.md$/.test(path)) return 'docs';
  // Build configuration and dependency graph edits can invalidate artifacts
  // across both manifests and profiles; do not classify them as Rust-only.
  if(/(?:^|\/)(?:Cargo\.toml|Cargo\.lock|build\.rs|rust-toolchain(?:\.toml)?)$/.test(path) ||
     path.includes('/.cargo/config') || path.startsWith('.cargo/')) return 'unknown';
  if(path.startsWith('Development/tooling/acceptance/windows/bootstrap-ordering/')) return 'native';
  if(path.startsWith('Development/apps/desktop/src-tauri/src/') ||
     path.startsWith('Development/engine/runtime-core/src/')) return 'rust';
  if(path.startsWith('Development/apps/desktop/src/') ||
     path.startsWith('Development/apps/desktop/test/')) return 'frontend';
  return 'unknown';
}

export function planForPaths(paths,revision) {
  if(!Array.isArray(paths) || !paths.every(x=>typeof x==='string') || paths.length>4096) {
    throw Error('Invalid bounded file selection');
  }
  const kinds=[...new Set(paths.map(classifyPath))].sort();
  if(kinds.includes('protected')) return {
    status:'BLOCKED',sourceSha:revision,changedCount:paths.length,scopes:kinds,
    taskIds:[],reason:'Protected application, account or workspace paths are not eligible for source-routing.'};
  if(kinds.length===0) return {status:'READY',sourceSha:revision,changedCount:0,
    scopes:[],taskIds:[],reason:'No changed source detected; do not rebuild just to create evidence.'};
  // Documentation normally accompanies code changes and must not force a
  // second unnecessary Rust dependency graph or full test suite.
  const executableScopes=kinds.filter(scope=>scope!=='docs');
  if(executableScopes.length===0) return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['repository.validate'],reason:'Documentation-only structural validation.'};
  if(executableScopes.length===1 && executableScopes[0]==='frontend') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['frontend.validate'],reason:'Frontend and documentation validation; no Rust check.'};
  if(executableScopes.length===1 && executableScopes[0]==='rust') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['rust.validate'],reason:'Scoped Rust check; full tests still required for candidate packaging.'};
  return {status:'READY',sourceSha:revision,changedCount:paths.length,scopes:kinds,
    taskIds:['source.validate'],reason:'Mixed, native, configuration or unknown source; use full verification. Native acceptance remains separate.'};
}

// Source-only quality review: a deterministic checklist, NOT an LLM runner,
// binary investigator, gameplay agent, or grading/acceptance authority.
const QUALITY_AREAS=Object.freeze({
  'education-hud':Object.freeze({
    owner:'Development/tooling/agent/check-education-native-hud-source.mjs and education-native-hud-prototype/; review the existing HUD fixture and docs/system/education-native-coordinate-hud.md as direct companions',
    checks:[
      'Verify JSONC structural parent/child path and after-player_position sibling; never insert into auto-sized native image.',
      'Review the pinned Bedrock coordinate label type/text, binding conditions, duplicate JSON keys, and source file-handle admission with the existing negative fixtures.',
      'Keep world-only manifest scope, original XYZ/Agent Pos/visibility bindings and no Script API or button action.',
      'Inspect the existing HUD source fixture and HUD contract guide even when only one is in the selected diff; changed-path selection is not proof of test coverage.',
      'Keep Education 26.33 rendering, layout, team forms and pack interaction explicitly UNVERIFIED.'
    ]
  }),
  'agent-mcp':Object.freeze({
    owner:'Development/tooling/agent/mcp.mjs and the existing specialized agent modules',
    checks:[
      'Preserve the single authenticated Rust controller, per-slot owner and default-disabled AI input.',
      'Check native target PID/birth/session/HWND, frame delivery, stale cache and fail-closed writer lease paths.',
      'Do not reinterpret original Skills Probe/team results as verified map or Minecraft gameplay.'
    ]
  }),
  'native-input-v4':Object.freeze({
    owner:'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/mc02_input.rs (Rust 96-byte writer); Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp (opt-in C++ reader)',
    checks:[
      'Compare the 15 Rust/C++ ABI v4 field offsets, 96-byte size, alignment and eight independent 8-bit press counters.',
      'Trace exact original PID/birth/session/channel binding, 500-ms packet TTL, 350-ms scheduled frames and 1-ms native neutral writes.',
      'Review expired packet refusal, P40-P42 pending UPs/partial buffer cursors and P43 counter overflow without manufacturing gameplay evidence.',
      'Keep the original Rust controller and compile-OFF C++ consumer unchanged in authority; no new DLL hook or verified Minecraft acceptance.'
    ]
  }),
  'native-session':Object.freeze({
    owner:'Development/apps/desktop/src-tauri/src/engine/; Development/engine/runtime-core/',
    checks:[
      'Trace original native lifecycle/identity/IPC owner and all directly affected callers.',
      'Check PID reuse, timeout, interrupted/unknown mutations, release ordering and per-profile preservation.',
      'Refuse second game controller, authentication bypass or unsupported multi-session fallback.'
    ]
  }),
  'desktop-ui':Object.freeze({
    owner:'Development/apps/desktop/src/ and typed Tauri bridge',
    checks:[
      'Check selection, keyboard/mouse presentation and user-visible state against retained native facts.',
      'Check empty/loading/error/replacement states without inventing game readiness or coordinates.',
      'Preserve five fixed profile cards and existing native window/preview authority.'
    ]
  }),
  'developer-tooling':Object.freeze({
    owner:'DEV.cmd; Development/tooling/developer/ and repository checks',
    checks:[
      'Keep one allowlisted developer command owner, exact Local SHA and non-executable read-only planning.',
      'Reject arbitrary shell/tool/plugin activation, CI, implicit approval and unscoped deletion.',
      'Check affected consumer paths and authored positive/negative regressions; do not label them executed.'
    ]
  }),
  'documentation':Object.freeze({
    owner:'Existing canonical AGENTS, docs/system and planning/development.md',
    checks:[
      'Resolve references to current source and remove stale claims rather than duplicate policy or status owners.',
      'Keep authored tests distinct from executed tests and historical evidence distinct from exact-HEAD proof.'
    ]
  }),
  'cross-cutting':Object.freeze({
    owner:'Resolve through docs/system/implementation-map.md before proposing any new component',
    checks:[
      'Treat unmatched source/configuration as full-scope review; identify canonical owner and consumers first.',
      'Check source, dependency, package, data preservation and negative regression boundaries.'
    ]
  })
});
function qualityArea(path) {
  if(path.startsWith('Development/tooling/agent/education-native-hud-prototype/') ||
      path==='Development/tooling/agent/check-education-native-hud-source.mjs' ||
      path==='Development/tooling/agent/education-native-hud-prototype.source.test.mjs' ||
      path==='docs/system/education-native-coordinate-hud.md') return 'education-hud';
  if(path.startsWith('Development/tooling/agent/')) return 'agent-mcp';
  // The C++ game-side input reader is a source ABI consumer, not just
  // developer tooling. Its edits require review of the retained Rust
  // shared-memory writer even when the writer was not in the diff.
  if(path==='Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp' ||
      path==='Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.h') {
    return 'native-input-v4';
  }
  if(path.startsWith('Development/apps/desktop/src-tauri/') ||
      path.startsWith('Development/engine/runtime-core/')) return 'native-session';
  if(path.startsWith('Development/apps/desktop/src/') ||
      path.startsWith('Development/apps/desktop/test/')) return 'desktop-ui';
  if(path.startsWith('Development/tooling/') || path==='DEV.cmd') return 'developer-tooling';
  if(classifyPath(path)==='docs') return 'documentation';
  return 'cross-cutting';
}
function safeReviewPath(path) {
  return typeof path==='string' && path.length>0 && path.length<=512 &&
    !path.includes('\\') && !path.includes('\0') && !path.startsWith('-') &&
    !isAbsolute(path) && !win32.isAbsolute(path) &&
    posix.normalize(path)===path && path!=='..' && !path.startsWith('../');
}

// These are the existing, direct source authority boundaries. They do not
// inspect callers, prove runtime compatibility, or grant tool permissions.
// A changed endpoint requires a human/source reviewer to examine all parties.
const CRITICAL_REVIEW_SEAMS=Object.freeze([
  Object.freeze({
    id:'agent-input-lease',
    parties:[
      {id:'agent-mcp',sourcePaths:[
        'Development/tooling/agent/mcp.mjs',
        'Development/tooling/agent/mcp.test.mjs',
        'Development/tooling/agent/input-route.mjs',
        'Development/tooling/agent/input-route.test.mjs',
        'Development/tooling/agent/input-delivery.mjs',
        'Development/tooling/agent/input-delivery.test.mjs',
        'Development/tooling/agent/input-timeline.mjs',
        'Development/tooling/agent/input-timeline.test.mjs',
        'Development/tooling/agent/visual-observer.mjs',
        'Development/tooling/agent/visual-observer.test.mjs'
      ],inspectFiles:[
        'Development/tooling/agent/mcp.mjs',
        'Development/tooling/agent/input-delivery.mjs'
      ]},
      {id:'native-controller',sourcePaths:[
        'Development/apps/desktop/src-tauri/src/engine/agent_cli.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session/pipe.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs'
      ],inspectFiles:[
        'Development/apps/desktop/src-tauri/src/engine/agent_cli.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs'
      ]}
    ],
    checks:[
      'Follow MCP request through same-image agent CLI and authenticated original Rust IPC; neither owns a second Minecraft lifecycle.',
      'Review per-slot PID/birth/session/HWND and fenced lease-epoch admission, interrupted writer cleanup and fail-closed recovery.',
      'Native transport ACK and a delivered visual frame do not prove Minecraft accepted keyboard/mouse input.'
    ]
  }),
  Object.freeze({
    id:'native-desktop-session',
    parties:[
      {id:'native-controller',sourcePaths:[
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session/pipe.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs',
        'Development/apps/desktop/src-tauri/src/engine/instance_identity.rs',
        'Development/apps/desktop/src-tauri/src/engine/process_identity.rs',
        'Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs',
        'Development/apps/desktop/src-tauri/src/engine/workspace_surface/controller.rs',
        'Development/apps/desktop/src-tauri/src/engine/workspace_surface/previews.rs',
        'Development/apps/desktop/src-tauri/src/commands/native_clients.rs'
      ],inspectFiles:[
        'Development/apps/desktop/src-tauri/src/commands/native_clients.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs'
      ]},
      {id:'desktop-ui',sourcePaths:[
        'Development/apps/desktop/src/LazyTesterApp.svelte',
        'Development/apps/desktop/src/app/bridge/lazyTester.ts',
        'Development/apps/desktop/src/app/bridge/invokeRuntime.ts',
        'Development/apps/desktop/src/app/bridge/workspaceSurface.ts',
        'Development/apps/desktop/src/app/presentation/sessionFeedback.ts',
        'Development/apps/desktop/test/lazy-tester.test.ts',
        'Development/apps/desktop/test/workspace-surface.test.ts',
        'Development/apps/desktop/test/observed-process.test.ts'
      ],inspectFiles:[
        'Development/apps/desktop/src/LazyTesterApp.svelte',
        'Development/apps/desktop/src/app/bridge/lazyTester.ts',
        'Development/apps/desktop/src/app/bridge/invokeRuntime.ts'
      ]}
    ],
    checks:[
      'Review typed bridge, Tauri command admission and original session identity rather than copying lifecycle authority into Svelte.',
      'Check replaced/unknown process and HWND, stale selection, pending CloseOne/CloseAll, and five-slot DWM preview recovery.',
      'Missing controller data must not silently become STOPPED, authenticated world, successful sign-in or gameplay.'
    ]
  }),
  Object.freeze({
    id:'agent-activity-mirror',
    parties:[
      {id:'agent-mcp',sourcePaths:[
        'Development/tooling/agent/mcp.mjs',
        'Development/tooling/agent/mcp.test.mjs',
        'Development/tooling/agent/activity-status.mjs',
        'Development/tooling/agent/activity-status.test.mjs'
      ],inspectFiles:[
        'Development/tooling/agent/activity-status.mjs',
        'Development/tooling/agent/mcp.mjs'
      ]},
      {id:'native-controller',sourcePaths:[
        'Development/apps/desktop/src-tauri/src/engine/agent_cli.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session/pipe.rs',
        'Development/apps/desktop/src-tauri/src/commands/native_clients.rs'
      ],inspectFiles:[
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/commands/native_clients.rs'
      ]},
      {id:'desktop-ui',sourcePaths:[
        'Development/apps/desktop/src/app/presentation/agentActivity.ts',
        'Development/apps/desktop/src/AgentActivityNotice.svelte',
        'Development/apps/desktop/src/app/bridge/lazyTester.ts',
        'Development/apps/desktop/src/LazyTesterApp.svelte',
        'Development/apps/desktop/test/lazy-tester.test.ts'
      ],inspectFiles:[
        'Development/apps/desktop/src/app/presentation/agentActivity.ts',
        'Development/apps/desktop/src/AgentActivityNotice.svelte',
        'Development/apps/desktop/src/app/bridge/lazyTester.ts'
      ]}
    ],
    checks:[
      'Trace only the explicitly published, bounded MCP activity projection through same-image native admission to the UI.',
      'Preserve native rejection of webview activity publication, stale ATTENTION semantics, fixed profile slots and validated Probe counts.',
      'The UI must say reported by local agent: no verified PC control, native game input, world attribution or map PASS.'
    ]
  }),
  Object.freeze({
    id:'native-input-v4',
    parties:[
      {id:'agent-scheduler',sourcePaths:[
        'Development/tooling/agent/mcp.mjs',
        'Development/tooling/agent/mcp.test.mjs',
        'Development/tooling/agent/mc02-action-sequence.mjs',
        'Development/tooling/agent/mc02-action-sequence.test.mjs',
        'Development/tooling/agent/input-timeline.mjs',
        'Development/tooling/agent/input-timeline.test.mjs',
        'Development/apps/desktop/src-tauri/src/engine/agent_cli.rs'
      ],inspectFiles:[
        'Development/tooling/agent/mcp.mjs',
        'Development/tooling/agent/input-timeline.mjs',
        'Development/apps/desktop/src-tauri/src/engine/agent_cli.rs'
      ]},
      {id:'native-rust-writer',sourcePaths:[
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/mc02_input.rs'
      ],inspectFiles:[
        'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs',
        'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/mc02_input.rs'
      ]},
      {id:'optin-cpp-reader',sourcePaths:[
        'Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp',
        'Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.h'
      ],inspectFiles:[
        'Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp',
        'Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.h'
      ]}
    ],
    checks:[
      'Pin both 96-byte ABI v4 definitions: Rust and C++ must match all 15 field offsets, slot/PID/birth/session and 8x8-bit rising-edge counts.',
      'Trace exact MCP/CLI to original Rust controller admission: input frames 350 ms, native maximum TTL 500 ms, neutral write 1 ms and epoch/owner fencing.',
      'Inspect C++ GetTickCount expiry, original HWND/game-thread/foreground checks, Raw Input pending key-up/partial buffer, pre/post-copy revision gates and counter rollover refusal.',
      'Never conflate source/static_assert fixtures with executed tests, real Windows Win32 polling, Minecraft key-up delivery or five-client acceptance; compile-OFF by default.'
    ]
  })
]);
function crossModuleReviewForPaths(paths) {
  const selected=new Set(paths);
  return CRITICAL_REVIEW_SEAMS.flatMap(seam=>{
    const parties=seam.parties.map(p=>({
      id:p.id,
      selectedInChangeSet:p.sourcePaths.some(file=>selected.has(file)),
      inspectFiles:[...p.inspectFiles],
      status:'NOT_REVIEWED'
    }));
    const touched=parties.filter(p=>p.selectedInChangeSet).map(p=>p.id);
    if(!touched.length)return [];
    return [{
      id:seam.id,status:'REVIEW_REQUIRED',
      selectedParties:touched,
      counterpartsNotSelected:parties.filter(p=>!p.selectedInChangeSet).map(p=>p.id),
      reviewParties:parties,checks:[...seam.checks],
      provenance:'SOURCE_PATH_SELECTION_ONLY',
      execution:'NOT_EXECUTED'
    }];
  });
}

export function qualityReviewForPaths(paths,revision) {
  if(!SHA.test(revision??'')) throw Error('Quality review requires an exact source SHA');
  const validation=planForPaths(paths,revision);
  if(validation.status==='BLOCKED' ||
      paths.some(path=>!safeReviewPath(path)) ||
      new Set(paths).size!==paths.length) {
    return {status:'BLOCKED',sourceSha:revision,changedCount:paths.length,
      areas:[],crossModuleReviews:[],recommendedValidationTaskIds:[],stages:[],
      reason:'Protected, repeated or unsafe paths cannot be admitted to quality review',
      proof:'NOT_EXECUTED'};
  }
  const areas=[...new Set(paths.map(qualityArea))].sort().map(id=>({
    id,canonicalOwner:QUALITY_AREAS[id].owner,
    reviewChecks:[...QUALITY_AREAS[id].checks],status:'NOT_REVIEWED'
  }));
  return {
    status:paths.length?'REVIEW_REQUIRED':'NO_CHANGES',
    sourceSha:revision,changedCount:paths.length,areas,
    crossModuleReviews:crossModuleReviewForPaths(paths),
    recommendedValidationTaskIds:[...validation.taskIds],
    stages:paths.length?[
      {id:'PROPOSE',status:'NOT_EXECUTED',
        requirement:'Compare no-change, correction in the existing owner, reuse and a justified minimal extension. Cite exact current source for each candidate.'},
      {id:'PROBE',status:'NOT_EXECUTED',
        requirement:'Review each candidate against owner boundaries, negative cases, changed consumers, source provenance and missing runtime evidence. No synthetic PASS.'},
      {id:'COMPOSE',status:'NOT_EXECUTED',
        requirement:'Adopt only a supported, minimal source change; unresolved risk remains explicit and no agent automatically selects or commits a winner.'}
    ]:[],
    externalIntegrations:{
      harness2:'METHODOLOGY_ONLY_NO_MODEL_RUN',
      deepseekHarness:'ORCHESTRATION_PATTERN_ONLY_NO_AGENT_RUNTIME',
      rea:'OPTIONAL_AUTHORIZED_ARTIFACT_INVESTIGATION_NOT_RUN'
    },
    evidence:{sourceReview:'NOT_REVIEWED',sourceTests:'NOT_EXECUTED',
      targetWindows:'NOT_EXECUTED',minecraftRuntime:'NOT_EXECUTED',
      fiveClientAcceptance:'NOT_EXECUTED'},
    reason:paths.length?'Read-only plan; candidate analysis, testing and code changes require separate actual evidence.':
      'No selected changes; nothing was reviewed or executed.',
    proof:'STATIC_REVIEW_PLAN_ONLY'
  };
}

export function legacyArgs(task) {
  if(!task || !Object.hasOwn(POLICY,task.command)) throw Error('Unregistered executor');
  if(task.command==='repository-structure') return {type:'node',args:[REPOSITORY_CHECK]};
  if(task.command==='retire-archive') return {type:'node',args:[ARCHIVE_RETIRE]};
  const args=['-NoLogo','-NoProfile','-NonInteractive','-File',DEV_SCRIPT,task.command];
  if(task.command==='storage') args.push(task.mode);
  return {type:'pwsh',args};
}

function git(args) {
  const child=spawnSync('git',args,{cwd:REPO,encoding:'utf8',shell:false,
    timeout:10000,maxBuffer:256*1024,windowsHide:true});
  if(child.error || child.status!==0 || typeof child.stdout!=='string') {
    throw Error('Git metadata unavailable; developer control must fail closed');
  }
  return child.stdout;
}
function sourceInfo() {
  const branch=git(['branch','--show-current']).trim();
  const sha=git(['rev-parse','--verify','HEAD']).trim();
  if(!SHA.test(sha)) throw Error('Invalid source SHA');
  return {branch,sha};
}
function changedPaths(readGit=git) {
  const changed=readGit(['diff','--name-only','-z','HEAD']).split('\0').filter(Boolean);
  const untracked=readGit(['ls-files','--others','--exclude-standard','-z']).split('\0').filter(Boolean);
  if(changed.length+untracked.length>4096) throw Error('Too many changes for bounded planning');
  return [...new Set([...changed,...untracked])];
}
// The ordinary quick route inspects the working tree, or HEAD^ on a clean
// checkout. An explicit ancestor baseline covers an entire development batch
// without a second task runner or a persistent "last validated" state file.
export function automaticChangeSet(baseSha=null,readGit=git) {
  const pending=changedPaths(readGit);
  if(baseSha!==null) {
    if(!SHA.test(baseSha)) throw Error('Invalid exact base SHA');
    const ancestor=readGit(['merge-base',baseSha,'HEAD']).trim();
    if(ancestor!==baseSha) throw Error('Base commit must be an ancestor of current Local HEAD');
    const committed=readGit(['diff','--no-renames','--name-only','-z',baseSha,'HEAD','--']).split('\0').filter(Boolean);
    const paths=[...new Set([...committed,...pending])];
    if(paths.length>4096) throw Error('Too many changes for bounded planning');
    return {paths,basis:'base-commit',baseSha};
  }
  if(pending.length) return {paths:pending,basis:'working-tree'};
  const committed=readGit(['diff','--name-only','-z','HEAD^','HEAD','--']).split('\0').filter(Boolean);
  if(committed.length>4096) throw Error('Too many committed changes for bounded routing');
  return {paths:[...new Set(committed)],basis:'latest-commit'};
}
function ordinary(path) {
  try {const info=lstatSync(path);return info.isFile()&&!info.isSymbolicLink();}
  catch {return false;}
}
// Node's version alone does not prove the current executable is the reviewed
// checkout-local runtime. Compare normalized absolute paths, never PATH lookup.
export function isPinnedNodeProcess(actual,expected,platform=process.platform) {
  if(typeof actual!=='string' || typeof expected!=='string') return false;
  if(platform==='win32') return win32.isAbsolute(actual) && win32.isAbsolute(expected) &&
    win32.normalize(actual).toLowerCase()===win32.normalize(expected).toLowerCase();
  return isAbsolute(actual) && isAbsolute(expected) && resolve(actual)===resolve(expected);
}
// A child process exit is not revision-scoped evidence if Local HEAD changed
// while it ran or the final Git check failed.
export function classifyCompletion(before,after,exitCode,spawnError=false,interrupted=false) {
  if(!after) return 'SOURCE_UNVERIFIED';
  if(before.branch!=='Local' || after.branch!=='Local' || before.sha!==after.sha) return 'SOURCE_CHANGED';
  if(interrupted) return 'INTERRUPTED';
  return spawnError || exitCode!==0 ? 'FAILED' : 'COMMAND_SUCCEEDED';
}
function report(operation,data) {return {schema:1,operation,...data};}
function usage() {
  return 'Usage: DEV.cmd inspect|catalog|plan|review [--json] [--profile fast|local-smoke|five-client|release|experimental] [--base-sha <ancestor-40-char-SHA>] [--paths <source paths>...]\n'+
    '       DEV.cmd run auto --expect-sha <40-char Local SHA> [--base-sha <ancestor-40-char-SHA>] [--json]\n'+
    '       DEV.cmd run <task-id> --expect-sha <40-char Local SHA> [--approve <task-id>] [--json]\n'+
    '       DEV.cmd smoke start --expect-sha <full-SHA> [--json]\n'+
    '       DEV.cmd smoke record --run <smoke-run-id> --case LT-01 --result PASS --observed --expect-sha <full-SHA> [--json]\n'+
    '       DEV.cmd smoke status --run <smoke-run-id> [--json]\n'+
    '       DEV.cmd bootstrap (existing reviewed opt-in setup)';
}
export function parseFlags(tokens,operation) {
  const result={json:false,paths:[],taskId:null,expectSha:null,approval:null,baseSha:null,profile:null};
  const rest=[...tokens];
  if(operation==='run') {
    result.taskId=rest.shift();
    if(!result.taskId || !ID.test(result.taskId)) throw Error('Invalid task id');
  }
  const seen=new Set();
  for(let i=0;i<rest.length;i++) {
    const token=rest[i];
    if(token==='--json' && !seen.has(token)){result.json=true;seen.add(token);continue;}
    if(operation==='plan' && token==='--profile' && !seen.has(token)) {
      const profile=rest[++i];
      if(!PLAN_PROFILES.has(profile)) throw Error('--profile requires fast, local-smoke, five-client, release or experimental');
      result.profile=profile;seen.add(token);continue;
    }
    if((operation==='plan'||operation==='review'||operation==='run') && token==='--base-sha' && !seen.has(token)) {
      const base=rest[++i];
      if(!SHA.test(base??'')) throw Error('--base-sha requires one exact 40-character commit SHA');
      result.baseSha=base;seen.add(token);continue;
    }
    if((operation==='plan'||operation==='review') && token==='--paths' && !seen.has(token)) {
      result.paths=rest.slice(i+1);
      if(!result.paths.length || result.paths.some(path=>path.startsWith('--'))) throw Error('--paths requires source paths only');
      break;
    }
    if(operation==='run' && token==='--expect-sha' && !seen.has(token)) {
      result.expectSha=rest[++i];seen.add(token);continue;
    }
    if(operation==='run' && token==='--approve' && !seen.has(token)) {
      result.approval=rest[++i];seen.add(token);continue;
    }
    throw Error('Unexpected or repeated developer controller argument');
  }
  if(result.baseSha!==null && result.paths.length) throw Error('--base-sha cannot be combined with --paths');
  if(operation==='plan' && result.profile!==null && result.profile!=='fast' &&
      (result.baseSha!==null || result.paths.length)) {
    throw Error('Manual profiles cannot be combined with --base-sha or --paths');
  }
  if(result.baseSha!==null && operation==='run' && result.taskId!=='auto') {
    throw Error('--base-sha is only supported by run auto');
  }
  return result;
}

export function parseSmokeFlags(tokens) {
  const [action,...rest] = tokens;
  if (!['start','record','status'].includes(action)) throw Error('Invalid smoke action');
  const allowed = {
    start:new Set(['--expect-sha','--json']),
    record:new Set(['--expect-sha','--run','--case','--result','--observed','--json']),
    status:new Set(['--run','--json'])
  }[action];
  const result = {action,json:false,expectSha:null,runId:null,caseId:null,
    reportedStatus:null,observed:false};
  const seen=new Set();
  for(let i=0;i<rest.length;i++){
    const flag=rest[i];
    if(!allowed.has(flag) || seen.has(flag))throw Error('Unexpected/repeated smoke argument');
    seen.add(flag);
    if(flag==='--json'){result.json=true;continue;}
    if(flag==='--observed'){result.observed=true;continue;}
    const value=rest[++i];
    if(!value || value.startsWith('--'))throw Error('Missing smoke argument value');
    if(flag==='--expect-sha') result.expectSha=value;
    if(flag==='--run') result.runId=value;
    if(flag==='--case') result.caseId=value;
    if(flag==='--result') result.reportedStatus=value;
  }
  if(action!=='status' && !SHA.test(result.expectSha??''))
    throw Error('Smoke mutation requires exact --expect-sha');
  if(action!=='start' && (!result.runId ||
      !/^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}-smoke-[a-f0-9]{8}$/.test(result.runId))) {
    throw Error('Smoke status/record requires a bounded run ID');
  }
  if(action==='record' && (!/^LT-(0[1-9]|1[0-3])$/.test(result.caseId??'') ||
      !['PASS','FAIL','BLOCKED','INCONCLUSIVE'].includes(result.reportedStatus) ||
      (result.reportedStatus==='PASS' && !result.observed) ||
      (result.reportedStatus!=='PASS' && result.observed))) {
    throw Error('Smoke record requires one LT-01..LT-13 result; --observed only for operator-confirmed PASS');
  }
  return result;
}

function startup() {
  const pinned=JSON.parse(readFileSync(join(REPO,'Development/toolchain.json'),'utf8'));
  if(process.versions.node!==pinned.node.version) throw Error('Pinned workspace Node version required; no global fallback');
  if(!ordinary(NODE_EXE)) throw Error('Workspace-local Node executable is missing or linked');
  if(!isPinnedNodeProcess(process.execPath,NODE_EXE)) throw Error('Controller must run through pinned DEV.cmd Node; global Node is forbidden');
  return validatedRegistry(JSON.parse(readFileSync(REGISTRY,'utf8')));
}
function toolPresence() {
  return {node:ordinary(NODE_EXE),powershell:ordinary(PWSH_EXE),
    cargo:ordinary(join(REPO,'workspace/toolchains/cargo/bin/cargo.exe'))};
}
function emit(payload,json) {console.log(json?JSON.stringify(payload):JSON.stringify(payload,null,2));}
function blocked(operation,reason,json) {
  emit(report(operation,{status:'BLOCKED',reason,proof:'NOT_EXECUTED'}),json);
  return 3;
}
// Owned developer executor only. Never manage the Minecraft process tree.
// Console Ctrl+C reaches the child independently: do not send it twice.
// On Windows child.kill(SIGINT/TERM) is abrupt; only observe termination
// and preserve a stale lock if the child fails to exit.
export function runOwnedChild(executable,args,{cwd,json=false,cancellationSignal=null,forwardHostSignals=true}={}) {
  return new Promise(resolveResult=>{
    let child=null,childError=null,interrupted=false,closed=false;
    const requestStop=(signalName,forwardChild=false)=>{
      if(closed || interrupted) return;
      interrupted=true;
      if(!forwardChild || process.platform==='win32') return; // No duplicate Ctrl+C or forced Windows termination.
      try {
        if(child?.pid && child.exitCode===null && child.signalCode===null) child.kill(signalName);
      } catch {}
    };
    const onInterrupt=()=>requestStop('SIGINT');
    const onTerminate=()=>requestStop('SIGTERM',true);
    const onAbort=()=>requestStop('SIGTERM',true);
    const cleanup=()=>{
      process.removeListener('SIGINT',onInterrupt);
      process.removeListener('SIGTERM',onTerminate);
      cancellationSignal?.removeEventListener('abort',onAbort);
    };
    if(forwardHostSignals){
      process.on('SIGINT',onInterrupt);
      process.on('SIGTERM',onTerminate);
    }
    cancellationSignal?.addEventListener('abort',onAbort,{once:true});
    try {
      child=spawn(executable,args,{cwd,shell:false,windowsHide:false,
        stdio:json?['inherit',process.stderr,'inherit']:'inherit'});
      child.once('error',error=>{childError=error;});
      child.once('close',(code,signal)=>{
        closed=true;cleanup();
        resolveResult({code,signal,error:childError,interrupted});
      });
      if(cancellationSignal?.aborted) onAbort();
    }catch(error){
      closed=true;cleanup();
      resolveResult({code:null,signal:null,error,interrupted});
    }
  });
}

// One exclusive lock across the entire awaited developer child lifetime.
// The lock belongs to the creator; never take over an unknown stale lock.
export async function withBuildLock(taskId,callback,lockFile=LOCK_FILE) {
  const tmp=dirname(lockFile),workspace=dirname(tmp);
  for(const dir of [workspace,tmp]){
    const stat=lstatSync(dir);
    if(!stat.isDirectory() || stat.isSymbolicLink()) throw Error('Developer workspace lock directory must be ordinary');
  }
  const fd=openSync(lockFile,constants.O_CREAT|constants.O_EXCL|constants.O_WRONLY,0o600);
  let original;
  try {
    original=fstatSync(fd);
    writeSync(fd,JSON.stringify({taskId,pid:process.pid})+'\n');
    return await callback();
  }finally{
    closeSync(fd);
    // If identity admission failed, preserve the lock for explicit inspection.
    if(!original) throw Error('Developer lock identity unavailable; manual review required');
    const current=lstatSync(lockFile);
    if(!current.isFile() || current.isSymbolicLink() || current.nlink!==1 ||
       current.dev!==original.dev || current.ino!==original.ino) {
      throw Error('Developer lock identity changed: preserve it for manual review');
    }
    unlinkSync(lockFile);
  }
}
async function runTask(task,parsed,source,selection=null) {
  if(source.branch!=='Local') return blocked('run','Task execution requires branch Local',parsed.json);
  if(!parsed.expectSha || !SHA.test(parsed.expectSha) || source.sha!==parsed.expectSha) {
    return blocked('run','Exact current Local SHA is required; re-inspect the checkout',parsed.json);
  }
  if(task.requiresApproval && parsed.approval!==task.id) return blocked('run',
    'Operator approval required for this task; supply --approve with the exact task id only after explicit approval',parsed.json);
  if(!task.requiresApproval && parsed.approval!==null) return blocked('run',
    'Approval flag is reserved for explicit high-impact tasks',parsed.json);
  const execution=legacyArgs(task);
  const executable=execution.type==='node'?NODE_EXE:PWSH_EXE;
  if(!ordinary(executable)) return blocked('run','Pinned workspace executor missing or linked',parsed.json);
  let attempted=false;
  const invoke=async()=>{
    const latest=sourceInfo();
    if(latest.branch!=='Local' || latest.sha!==source.sha) {
      return {status:'BLOCKED',taskId:task.id,sourceSha:source.sha,
        reason:'Local HEAD changed before execution; task not started',proof:'NOT_EXECUTED'};
    }
    attempted=true;
    const child=await runOwnedChild(executable,execution.args,{cwd:join(REPO,'Development'),json:parsed.json});
    let after;
    try {after=sourceInfo();} catch {after=null;}
    const status=classifyCompletion(source,after,child.code,Boolean(child.error),child.interrupted);
    return {status,taskId:task.id,sourceSha:source.sha,exitCode:child.code??null,
      proof:status==='COMMAND_SUCCEEDED'?'COMMAND_EXIT_AND_HEAD_ONLY':'NOT_VERIFIED',
      note:'Only the committed HEAD was rechecked. Uncommitted source and Windows/game acceptance are not certified.',
      ...(selection ? {selectionBasis:selection.basis,selectedChangeCount:selection.paths.length,
        ...(selection.baseSha ? {baseSha:selection.baseSha} : {})} : {})};
  };
  let outcome;
  try {outcome=await (task.risk==='read'?invoke():withBuildLock(task.id,invoke));}
  catch {
    if(!attempted) return blocked('run','Task could not start: developer lock or Git metadata is unsafe',parsed.json);
    emit(report('run',{status:'FAILED_UNVERIFIED',taskId:task.id,sourceSha:source.sha,
      proof:'OUTCOME_UNVERIFIED',reason:'Task may have run but completion or lock release failed. Inspect manually; no automatic lock recovery.'}),parsed.json);
    return 1;
  }
  emit(report('run',outcome),parsed.json);
  return outcome.status==='COMMAND_SUCCEEDED'?0:outcome.status==='BLOCKED'?3:1;
}
export async function main(argv=process.argv.slice(2)) {
  const [operation,...arguments_]=argv;
  if(!['inspect','catalog','plan','review','run','smoke'].includes(operation)) {
    console.error(usage());return 2;
  }
  if(operation==='smoke') {
    let parsed;
    try {parsed=parseSmokeFlags(arguments_);}
    catch(error){console.error(error.message+'\n'+usage());return 2;}
    try {startup();}
    catch {return blocked('smoke','Pinned Node or task registry unavailable',parsed.json);}
    let source;
    try {source=sourceInfo();}
    catch(error){return blocked('smoke',error.message,parsed.json);}
    if(source.branch!=='Local')return blocked('smoke','Local branch required',parsed.json);
    const caseIds=LOCAL_SMOKE_CHECKPOINTS.map(step=>step.id);
    if(parsed.action!=='status') {
      if(parsed.expectSha!==source.sha) {
        return blocked('smoke','Exact current Local SHA required',parsed.json);
      }
      try {if(changedPaths().length) return blocked('smoke',
        'Source checkout has uncommitted/untracked changes; do not mix smoke revisions',parsed.json);}
      catch(error){return blocked('smoke',error.message,parsed.json);}
    }
    let output;
    try {
      if(parsed.action==='start') output=beginSmoke(REPO,source.sha,caseIds);
      if(parsed.action==='record') output=recordSmoke(REPO,parsed.runId,source.sha,
        caseIds,parsed.caseId,parsed.reportedStatus);
      if(parsed.action==='status') output=readSmoke(REPO,parsed.runId,caseIds);
    }catch(error){return blocked('smoke',error.message,parsed.json);}
    const staleForCurrentHead=output.sourceSha!==source.sha;
    const step=!staleForCurrentHead&&output.nextCaseId?
      LOCAL_SMOKE_CHECKPOINTS.find(x=>x.id===output.nextCaseId):null;
    emit(report('smoke',{status:staleForCurrentHead?'STALE_REVISION':'OPERATOR_REPORT_ONLY',
      action:parsed.action,runId:output.runId??parsed.runId,
      ...(output.manifestRelativePath?{manifestRelativePath:output.manifestRelativePath}:{}),
      ...output,staleForCurrentHead,
      nextCheckpoint:step?{...step}:null,
      proof:'MANUAL_OPERATOR_REPORT_NOT_RUNTIME_ATTESTATION'}),parsed.json);
    return staleForCurrentHead?3:0;
  }
  let parsed;
  try {parsed=parseFlags(arguments_,operation);}catch(error){console.error(error.message+'\n'+usage());return 2;}
  let tasks;
  try {tasks=startup();}
  catch(error){return blocked(operation,'Pinned Node or developer task registry is unavailable/invalid',parsed.json);}
  if(operation==='catalog') {
    emit(report(operation,{status:'READY',tasks:tasks.map(({id,description,scope,risk,requiresApproval})=>
      ({id,description,scope,risk,requiresApproval})),proof:'REGISTRY_ONLY'}),parsed.json);
    return 0;
  }
  let source;
  try {source=sourceInfo();}
  catch(error){return blocked(operation,error.message,parsed.json);}
  if(operation==='inspect') {
    let paths;
    try {paths=changedPaths();}catch(error){return blocked(operation,error.message,parsed.json);}
    emit(report(operation,{status:source.branch==='Local'?'READY':'BLOCKED',
      source:{branch:source.branch,sha:source.sha},changedCount:paths.length,
      scopes:[...new Set(paths.map(classifyPath))].sort(),toolchainPresence:toolPresence(),
      proof:'GIT_AND_FILE_METADATA_ONLY',note:'Presence is not version or execution proof.'}),parsed.json);
    return source.branch==='Local'?0:3;
  }
  if(operation==='plan') {
    if(source.branch!=='Local') return blocked(operation,'Planning requires branch Local',parsed.json);
    const profile=parsed.profile??'fast';
    let selection;
    try {selection=profile==='fast'
      ?(parsed.paths.length
        ?{paths:parsed.paths,basis:'declared-paths'}:automaticChangeSet(parsed.baseSha))
      :{paths:[],basis:'explicit-profile'};}
    catch(error){return blocked(operation,error.message,parsed.json);}
    const planned=planForProfile(profile,source.sha,selection.paths);
    emit(report(operation,{...planned,basis:selection.basis,
      ...(selection.baseSha ? {baseSha:selection.baseSha} : {}),
      sourceBranch:source.branch,proof:'STATIC_RECOMMENDATION_ONLY'}),parsed.json);
    return source.branch==='Local'&&planned.status!=='BLOCKED'?0:3;
  }
  if(operation==='review') {
    if(source.branch!=='Local') return blocked(operation,'Review requires branch Local',parsed.json);
    let selection;
    try {selection=parsed.paths.length
      ?{paths:parsed.paths,basis:'declared-paths'}:automaticChangeSet(parsed.baseSha);}
    catch(error){return blocked(operation,error.message,parsed.json);}
    const review=qualityReviewForPaths(selection.paths,source.sha);
    emit(report(operation,{...review,basis:selection.basis,
      inputTrust:selection.basis==='declared-paths'?'CALLER_DECLARATION_NOT_GIT_VERIFIED':
        'LOCAL_GIT_METADATA_NOT_CONTENT_REVIEW',
      ...(selection.baseSha?{baseSha:selection.baseSha}:{}),
      sourceBranch:source.branch}),parsed.json);
    return review.status==='BLOCKED'?3:0;
  }
  let requested=parsed.taskId;
  let autoSelection=null;
  if(requested==='auto') {
    if(source.branch!=='Local')return blocked(operation,'Automatic validation requires Local',parsed.json);
    let selection;
    try {selection=automaticChangeSet(parsed.baseSha);}
    catch(error){return blocked(operation,error.message,parsed.json);}
    autoSelection=selection;
    const recommendation=planForPaths(selection.paths,source.sha);
    if(recommendation.status!=='READY') return blocked(operation,
      'Automatic validation blocked by protected or untrusted paths',parsed.json);
    if(!parsed.expectSha || parsed.expectSha!==source.sha || !SHA.test(parsed.expectSha)) {
      return blocked(operation,'Automatic validation requires the exact current Local SHA',parsed.json);
    }
    if(recommendation.taskIds.length===0) {
      emit(report(operation,{status:'SKIPPED_NO_CHANGES',sourceSha:source.sha,
        selectionBasis:selection.basis,...(selection.baseSha ? {baseSha:selection.baseSha} : {}),
        proof:'STATIC_RECOMMENDATION_ONLY',reason:'No changed source; no build required.'}),parsed.json);
      return 0;
    }
    if(recommendation.taskIds.length!==1) return blocked(operation,'Invalid automatic task plan',parsed.json);
    requested=recommendation.taskIds[0];
  }
  const task=tasks.find(t=>t.id===requested);
  if(!task) return blocked(operation,'Unknown task id; use catalog for available capabilities',parsed.json);
  if(parsed.taskId==='auto' && task.requiresApproval)return blocked(operation,'Automatic mode cannot run consent-gated tasks',parsed.json);
  return await runTask(task,parsed,source,autoSelection);
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {process.exitCode=await main();}
  catch(error){console.error('Developer controller failed closed: '+(error?.code??error?.message??'unknown error'));process.exitCode=3;}
}
