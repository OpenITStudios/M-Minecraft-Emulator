import {invokeRuntime} from './invokeRuntime.js';
import type {ClientId,OwnedProcess} from './ownedProviderApi.js';
export interface WorkspaceRectangle {x:number;y:number;width:number;height:number}
export interface WorkspaceViewport extends WorkspaceRectangle {pixelRatio:number}
export function workspaceViewport(rect:WorkspaceRectangle,pixelRatio=1):WorkspaceViewport {
  const {x,y,width,height}=rect;
  if(![x,y,width,height,pixelRatio].every(Number.isFinite)||x<0||y<0||width<160||height<120||x+width>20000||y+height>20000||pixelRatio<0.5||pixelRatio>8)throw new Error('The game area is too small. Enlarge the manager window and try again.');
  return {x,y,width,height,pixelRatio};
}
export const workspaceSurface={
  present:(sessionDirectory:string,clientId:ClientId,target:OwnedProcess,rect:WorkspaceRectangle)=>invokeRuntime<void>('desktop_present_workspace',{sessionDirectory,clientId,target,viewport:workspaceViewport(rect,window.devicePixelRatio)}),
  update:(rect:WorkspaceRectangle)=>invokeRuntime<void>('desktop_update_workspace',{viewport:workspaceViewport(rect,window.devicePixelRatio)}),
  release:()=>invokeRuntime<void>('desktop_release_workspace'),
};
