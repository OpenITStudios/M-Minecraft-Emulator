import {invokeRuntime} from './invokeRuntime.js';
export interface WorkspaceRectangle {x:number;y:number;width:number;height:number}
export interface WorkspaceViewport extends WorkspaceRectangle {pixelRatio:number}
export function workspaceViewport(rect:WorkspaceRectangle,pixelRatio=1):WorkspaceViewport {
  return validatedViewport(rect,pixelRatio,160,120);
}
export function previewViewport(rect:WorkspaceRectangle,pixelRatio=1):WorkspaceViewport {
  return validatedViewport(rect,pixelRatio,32,24);
}
function validatedViewport(rect:WorkspaceRectangle,pixelRatio:number,minimumWidth:number,minimumHeight:number):WorkspaceViewport {
  const {x,y,width,height}=rect;
  if(![x,y,width,height,pixelRatio].every(Number.isFinite)||x<0||y<0||width<minimumWidth||height<minimumHeight||x+width>20000||y+height>20000||pixelRatio<0.5||pixelRatio>8)throw new Error('The game area is too small. Enlarge the manager window and try again.');
  return {x,y,width,height,pixelRatio};
}
export const workspaceSurface={
  update:(rect:WorkspaceRectangle)=>invokeRuntime<void>('desktop_update_workspace',{viewport:workspaceViewport(rect,window.devicePixelRatio)}),
  release:()=>invokeRuntime<void>('desktop_release_workspace',{target:null}),
  suspend:()=>invokeRuntime<boolean>('desktop_suspend_workspace'),
};
