// Host-side API; not a guest sandbox or execution permission.
export type Slot=1|2|3|4|5;
export type Json= null|boolean|number|string|Json[]|{[key:string]:Json};
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
type Scope='full'|'smoke'|'affected';
export interface Tester {
  profiles:{inspect(args?:Record<string,never>):Result};
  testing:{
    context(args?:{slots?:Slot[]}):Result;
    probe(args?:{slot?:Slot;scope?:Scope}):Result;
    inspectRun(args?:{slot?:Slot}):Result;
    readArchived(args:{archivedRunId:string}):Result;
  };
  evidence:{summarize(args?:{slots?:Slot[]}):Result;read(args:{slot:Slot}):Result};
  diagnostics:{
    overview(args?:{run?:string}):Result;
    list(args:{run:string;cursor?:number;limit?:number}):Result;
    details(args:{run:string;id:string}):Result;
    evidence(args:{run:string;id:string;cursor?:number;limit?:number}):Result;
  };
  observation:{state(args?:{slots?:Slot[];since?:Json[]}):Result};
  input:{
    plan(args:({actions:Action[];groups?:never}|{groups:ActionGroup[];actions?:never})&{includeTimeline?:boolean;includeClientFrames?:boolean}):Result;
    assess(args:{actions:Action[]}):Result;
  };
  capabilities(args?:Record<string,never>):Result;
}
export function createCodeModeClient(core:{
  isClosed():boolean;
  callTool(name:string,args:Record<string,unknown>):Promise<unknown>;
}):Readonly<{tester:Tester;close():void}>;

// Trusted adapter helper, not a model-facing generic dispatcher.
export interface CodeModeToolDescriptor {
  name:string;
  description?:string;
  inputSchema:{properties?:Record<string,Json>;required?:string[];additionalProperties?:boolean;[key:string]:Json|undefined};
}
export function createCodeModeSurface(
  descriptors:CodeModeToolDescriptor[],
  callTool:(name:string,args:Record<string,unknown>)=>Promise<unknown>,
  surface?:'code-mode'|'playtest',
):{listTools():CodeModeToolDescriptor[];callTool(name:string,args?:Record<string,unknown>):Promise<unknown>};
