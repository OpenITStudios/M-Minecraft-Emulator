// Trusted host API; the live core catalog is the runtime schema authority.
export type Slot=1|2|3|4|5;
export type Json=null|boolean|number|string|Json[]|{[key:string]:Json};
export interface Action {
  slot:Slot; kind:'move_forward'|'move_backward'|'strafe_left'|'strafe_right'|'jump'|'sneak'|'attack'|'use'|'look_left'|'look_right'|'look_relative'|'wait';
  atMs:number; durationMs:number; deltaX?:number; deltaY?:number;
}
export interface ActionGroup {
  slot:Slot; atMs:number; durationMs:number;
  hold?:('move_forward'|'move_backward'|'strafe_left'|'strafe_right'|'jump'|'sneak'|'attack'|'use')[];
  look?:{deltaX:number;deltaY:number;steps?:number;curve?:'linear'|'smooth'};
}
type Result=Promise<Json>;
type ActionInput={actions:Action[];groups?:never}|{groups:ActionGroup[];actions?:never};
type VisualFormat={imageFormat?:'png';width?:never;height?:never}|({imageFormat:'webp'}&({width:number;height:number}|{width?:never;height?:never}));
type Scope='full'|'smoke'|'affected';
type ProbeArgs={slot?:Slot;restart?:boolean;scope?:Scope;publishToDesktop?:boolean;coordinateNative?:boolean;inspectRun?:never;archivedRunId?:never}
  |{slot?:Slot;inspectRun:true;restart?:never;scope?:never;publishToDesktop?:never;coordinateNative?:never;archivedRunId?:never}
  |{archivedRunId:string;slot?:never;inspectRun?:never;restart?:never;scope?:never;publishToDesktop?:never;coordinateNative?:never};
export interface VisualResult {metadata:Json;imageBase64:string|null}
export interface Tester {
  agent_capabilities(args?:{inspectProfiles?:boolean;publishToDesktop?:boolean}):Result;
  inspect_profiles(args?:Record<string,never>):Result;
  launch_profile(args:{slot:Slot}):Result;
  plan_peer_session(args?:Record<string,never>):Result;
  get_test_context(args?:{slots?:Slot[]}):Result;
  plan_player_actions(args:ActionInput&{includeTimeline?:boolean;includeClientFrames?:boolean}):Result;
  assess_input_route(args:ActionInput):Result;
  probe_qa(args?:ProbeArgs):Result;
  summarize_test_evidence(args?:{slots?:Slot[];window?:never}|{slots:Slot[];window:'begin'|'read'}):Result;
  read_game_evidence(args:{slot:Slot}):Result;
  get_diagnostics_overview(args?:{run?:string}):Result;
  list_issue_candidates(args:{run:string;cursor?:number;limit?:number}):Result;
  get_issue_details(args:{run:string;id:string}):Result;
  query_diagnostic_evidence(args:{run:string;id:string;cursor?:number;limit?:number}):Result;
  observe_client(args:{slot:Slot;afterFrame?:string}&VisualFormat):Promise<VisualResult>;
  start_visual_observation(args:{slots:Slot[];intervalMs?:number;durationMs?:number}&VisualFormat):Result;
  read_visual_observation(args:{slot:Slot;afterSequence?:number;includeImage?:boolean;imageFormat?:'png'|'webp';screenReview?:Json}):Promise<VisualResult>;
  stop_visual_observation(args:{slots:Slot[]}):Result;
  get_multi_client_visual_state(args?:{slots?:Slot[];since?:Json[]}):Result;
  inspect_client_ui(args:{slot:Slot}):Result;
  assess_client_background_action(args:{slot:Slot;elementIndex:number;action:'invoke'}):Result;
  get_computer_use_target(args:{slot:Slot}):Result;
  revalidate_computer_use_target(args:{slot:Slot;pid:number;windowId:number;createdFileTime:string}):Result;
}
interface Descriptor {name:string;inputSchema:{properties?:Record<string,Json>;required?:string[]}}
interface Core {isClosed():boolean;listTools():Descriptor[];callTool(name:string,args:Record<string,unknown>):Promise<unknown>}
export function createAgentClient(core:Core):Readonly<{tester:Tester;close():void}>;
