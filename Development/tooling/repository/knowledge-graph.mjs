// Read-only repository knowledge navigation, adapted from the pinned
// M-Lazy-Developer Router -> Resource Catalog -> Graph architecture.
// No external Obsidian/Pele/Graphify runtime or code-dependency claim.
import { existsSync, lstatSync, readFileSync, readdirSync } from 'node:fs';
import { dirname, isAbsolute, posix, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const DEFAULT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const TICK = String.fromCharCode(96);
const MARKDOWN_LINK = /\[([^\]\n]*)\]\(([^)\n]+)\)/g;
const WIKI_LINK = /(!?)\[\[([^\]\n]+)\]\]/g;
const FENCE = new RegExp('^ {0,3}(' + TICK + '{3,}|~{3,})(.*)$');
const DOC_ID = /^document\.[a-z0-9]+(?:[.-][a-z0-9]+)*$/;
const RESOURCE_ID = /^(document|source)\.[a-z0-9]+(?:[.-][a-z0-9]+)*$/;

// Pointers, not a second ownership registry. implementation-map.md and
// the actual source files remain the authority for module ownership.
export const NAVIGATION_SOURCES = Object.freeze([
  ['source.rules.agent-policy', 'AGENTS.md'],
  ['source.rules.github-policy', 'GITHUB_RULES.md'],
  ['source.rules.emulator-development-skill', '.agents/skills/emulator-development/SKILL.md'],
  ['source.tooling.knowledge-graph', 'Development/tooling/repository/knowledge-graph.mjs'],
  ['source.tooling.knowledge-query', 'Development/tooling/repository/knowledge-query.mjs'],
  ['source.tooling.agent-mcp', 'Development/tooling/agent/mcp.mjs'],
  ['source.native.session-controller', 'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs'],
  ['source.native.session-transport', 'Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs'],
  ['source.desktop.presentation', 'Development/apps/desktop/src/LazyTesterApp.svelte'],
  ['source.engine.runtime-core', 'Development/engine/runtime-core/src/independent_engine/mod.rs'],
]);

function secureFile(root, path) {
  if (typeof path !== 'string' || !path || path.includes('\0') ||
      isAbsolute(path) || /^[A-Za-z]:/.test(path) || path.includes('\\')) return undefined;
  const full = resolve(root, path);
  const back = relative(root, full);
  if (back === '..' || back.startsWith('..' + sep) || isAbsolute(back)) return undefined;
  // Do not follow a link into ignored account, world, or installed runtime data.
  let cursor = root;
  for (const part of back.split(sep)) {
    cursor = resolve(cursor, part);
    if (!existsSync(cursor) || lstatSync(cursor).isSymbolicLink()) return undefined;
  }
  return full;
}

function metadata(text, path) {
  const normalized = text.replace(/\r\n/g, '\n');
  if (!normalized.startsWith('---\n')) throw new Error(path + ': missing document metadata');
  const end = normalized.indexOf('\n---\n', 4);
  if (end < 0) throw new Error(path + ': incomplete document metadata');
  const data = {};
  for (const original of normalized.slice(4, end).split('\n')) {
    const line = original.trim();
    if (!line) continue;
    const split = line.indexOf(':');
    if (split < 1) throw new Error(path + ': invalid metadata line ' + line);
    const key = line.slice(0, split).trim(), value = line.slice(split + 1).trim();
    if (!value || Object.hasOwn(data, key)) throw new Error(path + ': duplicate/empty field ' + key);
    data[key] = value;
  }
  for (const name of ['id', 'class', 'domain', 'role', 'authority', 'lifecycle']) {
    if (!data[name]) throw new Error(path + ': missing ' + name);
  }
  if (!DOC_ID.test(data.id) || data.class !== 'DOCUMENT' ||
      !/^[a-z0-9-]+$/.test(data.domain) || !/^[A-Z_]+$/.test(data.role) ||
      !['CANONICAL','REFERENCE','WORKING','DERIVED','HISTORICAL'].includes(data.authority) ||
      !['ACTIVE','HISTORICAL','RETIRED'].includes(data.lifecycle)) {
    throw new Error(path + ': unsupported document metadata');
  }
  // Legacy WORKING does not imply CANONICAL. Historical records cannot
  // become active by following a link from current documentation.
  const authority = data.lifecycle !== 'ACTIVE' ? 'HISTORICAL'
    : data.authority === 'WORKING' ? 'REFERENCE' : data.authority;
  return {id:data.id, class:'DOCUMENT', domain:data.domain,
    role:data.role, authority, declaredAuthority:data.authority,
    lifecycle:data.lifecycle, path};
}

function maskLine(line, state) {
  const out = [...line];
  let i = 0;
  const erase = (a,b) => { for(let k=a;k<b;k++) out[k]=' '; };
  while(i < line.length) {
    if(state.comment) {
      const end=line.indexOf('-->',i);
      if(end<0){erase(i,line.length);break;}
      erase(i,end+3);i=end+3;state.comment=false;continue;
    }
    if(state.ticks) {
      if(line[i]===TICK) {
        let end=i;while(line[end]===TICK)end++;
        if(end-i===state.ticks)state.ticks=0;
        erase(i,end);i=end;continue;
      }
      out[i++]=' ';continue;
    }
    if(line.startsWith('<!--',i)) {state.comment=true;continue;}
    if(line[i]===TICK && (i===0 || line[i-1]!=='\\')) {
      let end=i;while(line[end]===TICK)end++;
      state.ticks=end-i;erase(i,end);i=end;continue;
    }
    i++;
  }
  return out.join('');
}

export function* proseLines(markdown) {
  let fence;
  const state={comment:false,ticks:0};
  const lines=markdown.split(/\r?\n/);
  for(let i=0;i<lines.length;i++) {
    const line=lines[i], marker=line.match(FENCE);
    if(fence) {
      if(marker && marker[1][0]===fence.char &&
        marker[1].length>=fence.length && !marker[2].trim()) fence=undefined;
      continue;
    }
    if(!state.comment && !state.ticks && marker){
      fence={char:marker[1][0],length:marker[1].length};continue;
    }
    if(!state.comment && !state.ticks && /^( {4}|\t)/.test(line)) continue;
    yield {line:i+1, text:maskLine(line,state)};
  }
}

export function* authoredLinks(markdown) {
  for(const row of proseLines(markdown)){
    for(const m of row.text.matchAll(MARKDOWN_LINK)){
      if(m.index>0 && row.text[m.index-1]==='!') continue;
      yield {target:m[2],syntax:'MARKDOWN_INLINE',line:row.line};
    }
    for(const m of row.text.matchAll(WIKI_LINK)){
      if(m[1]==='!') continue;
      yield {target:m[2].split('|')[0].trim(),syntax:'OBSIDIAN_WIKILINK',line:row.line};
    }
  }
}

function linkPath(fromPath, raw, syntax) {
  let target=raw.trim().replace(/^<|>$/g,'').split('#',1)[0].split('?',1)[0];
  if(!target || /^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith('//')) return undefined;
  if(syntax==='MARKDOWN_INLINE') target=target.split(/\s+["']/)[0];
  try{target=decodeURIComponent(target);}catch{return undefined;}
  if(target.includes('\0') || target.includes('\\')) return undefined;
  if(syntax==='OBSIDIAN_WIKILINK'){
    if(!/^(docs|Development|\.agents)\//.test(target) &&
       !['AGENTS.md','GITHUB_RULES.md','CONTEXT.md','README.md'].includes(target)) return undefined;
    target=target.replace(/^\//,'');
  } else {
    target=target.startsWith('/')?target.slice(1):
      posix.join(posix.dirname(fromPath),target);
  }
  const resolved=posix.normalize(target);
  if(resolved==='..' || resolved.startsWith('../') || resolved.startsWith('/'))return undefined;
  return resolved;
}

function sectionName(value){
  return value.toLowerCase().replace(/!\[[^\]]*\]\([^)]*\)/g,'')
    .replace(/\[([^\]]+)\]\([^)]*\)/g,'$1')
    .replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,'') || 'section';
}
function docPaths(root) {
  const docs=secureFile(root,'docs');
  if(!docs || !lstatSync(docs).isDirectory())throw new Error('Missing docs/');
  const result=['docs/README.md'];
  for(const domain of readdirSync(docs,{withFileTypes:true}).filter(e=>e.isDirectory())
    .sort((a,b)=>a.name.localeCompare(b.name))){
    for(const file of readdirSync(resolve(docs,domain.name),{withFileTypes:true})
      .filter(e=>e.isFile()&&e.name.endsWith('.md')).sort((a,b)=>a.name.localeCompare(b.name))){
      result.push('docs/'+domain.name+'/'+file.name);
    }
  }
  return result;
}

export function buildResourceCatalog(root=DEFAULT_ROOT){
  root=resolve(root);
  const resources=[],seen=new Set(),contents=new Map();
  function add(r){
    if(!RESOURCE_ID.test(r.id) || seen.has(r.id))throw new Error('Duplicate/invalid Resource ID: '+r.id);
    seen.add(r.id);resources.push(r);
  }
  for(const path of docPaths(root)){
    const full=secureFile(root,path);
    if(!full || !lstatSync(full).isFile())throw new Error('Unsafe document: '+path);
    const content=readFileSync(full,'utf8');
    contents.set(path,content);
    const doc=metadata(content,path);
    if(path==='docs/README.md' && doc.role!=='ROUTER')
      throw new Error('docs/README.md must be a router');
    if(path.startsWith('docs/')&&path!=='docs/README.md' &&
      doc.domain!==path.split('/')[1])
      throw new Error(path+': frontmatter domain does not match directory');
    add(doc);
    if(doc.lifecycle!=='ACTIVE')continue;
    const counts=new Map();
    for(const row of proseLines(content)){
      const heading=row.text.match(/^#{2,3}\s+(.+?)\s*#*\s*$/);
      if(!heading)continue;
      const key=sectionName(heading[1]);
      const count=(counts.get(key)||0)+1;counts.set(key,count);
      add({id:doc.id+'.section.'+key+(count>1?'-'+count:''),
        class:'DOCUMENT',domain:doc.domain,role:'DOMAIN',authority:'DERIVED',
        lifecycle:'ACTIVE',path,locator:'#'+key,sectionLine:row.line,
        parent:doc.id});
    }
  }
  for(const [id,path] of NAVIGATION_SOURCES){
    const file=secureFile(root,path);
    if(file && lstatSync(file).isFile())add({
      id,class:'SOURCE',domain:'source',authority:'CANONICAL',
      lifecycle:'ACTIVE',path
    });
  }
  return {schemaVersion:1,resources:resources.sort((a,b)=>a.id.localeCompare(b.id)),contents};
}

export function buildKnowledgeIndex(root=DEFAULT_ROOT){
  const {resources,contents}=buildResourceCatalog(root);
  const byId=new Map(resources.map(r=>[r.id,r]));
  const byPath=new Map(resources.filter(r=>!r.locator).map(r=>[r.path,r]));
  const edges=[],unresolved=[],seen=new Set();
  function add(edge){
    if(edge.from===edge.to)return;
    const key=edge.from+'|'+edge.type+'|'+edge.to;
    if(!seen.has(key)){seen.add(key);edges.push(edge);}
  }
  for(const resource of resources){
    if(resource.parent){
      add({from:resource.id,to:resource.parent,type:'DERIVED_FROM',
        basis:'EXTRACTED',evidence:{path:resource.path,line:resource.sectionLine,target:resource.locator}});
      continue;
    }
    if(resource.lifecycle!=='ACTIVE' || !resource.path.endsWith('.md') ||
       !contents.has(resource.path)) continue;
    for(const link of authoredLinks(contents.get(resource.path))){
      const path=linkPath(resource.path,link.target,link.syntax);
      if(!path)continue;
      const target=byPath.get(path);
      if(!target){
        unresolved.push({from:resource.id,path:resource.path,line:link.line,
          target:link.target,reason:secureFile(root,path)?'NOT_CATALOGED':'MISSING_OR_UNSAFE'});
        continue;
      }
      if(target.lifecycle!=='ACTIVE')continue;
      add({from:resource.id,to:target.id,
        type:resource.role==='ROUTER'?'ROUTES_TO':'RELATES_TO',
        basis:'EXTRACTED',
        evidence:{path:resource.path,line:link.line,target:link.target,syntax:link.syntax}});
    }
  }
  edges.sort((a,b)=>(a.from+'|'+a.type+'|'+a.to).localeCompare(b.from+'|'+b.type+'|'+b.to));
  unresolved.sort((a,b)=>a.path.localeCompare(b.path)||a.line-b.line);
  return {schemaVersion:1,resources,edges,unresolved,byId,
    proofBoundary:'STATIC_AUTHORED_NAVIGATION_ONLY_NOT_SOURCE_DEPENDENCY_OR_RUNTIME_PROOF'};
}

export function explainResourcePath(index,from,to,{maxHops=4,allowReverse=false,includeHistorical=false}={}){
  const scope={maxHops,allowReverse,includeHistorical};
  const admissible=id=>{
    const r=index.byId.get(id);
    return r && (r.lifecycle==='ACTIVE'||includeHistorical) ? r : undefined;
  };
  if(!admissible(from)||!admissible(to))return {
    status:'UNKNOWN_RESOURCE',nodes:[],steps:[],scope,proofBoundary:index.proofBoundary};
  const visited=new Set([from]),queue=[{id:from,ids:[from],steps:[]}];
  for(let i=0;i<queue.length;i++){
    const current=queue[i];
    if(current.id===to)return {
      status:'FOUND',nodes:current.ids.map(id=>index.byId.get(id)),
      steps:current.steps,scope,proofBoundary:index.proofBoundary};
    if(current.steps.length>=maxHops)continue;
    for(const e of index.edges){
      const forward=e.from===current.id,reverse=allowReverse&&e.to===current.id;
      if(!forward&&!reverse)continue;
      const next=forward?e.to:e.from;
      if(visited.has(next)||!admissible(next))continue;
      visited.add(next);
      queue.push({id:next,ids:[...current.ids,next],
        steps:[...current.steps,{...e,direction:forward?'FORWARD':'REVERSE'}]});
    }
  }
  return {status:'NO_PATH',nodes:[],steps:[],scope,proofBoundary:index.proofBoundary};
}

export function findResources(index,query,{domain,all=false,includeHistorical=false,limit=12}={}){
  if(!domain&&!all)throw new Error('Choose --domain or explicit --all for broad discovery.');
  const tokens=query.toLowerCase().trim().split(/\s+/).filter(Boolean);
  if(!tokens.length)throw new Error('Query must not be empty.');
  return index.resources.filter(r=>(all||r.domain===domain)&&
    (includeHistorical||r.lifecycle==='ACTIVE')&&r.authority!=='DERIVED')
    .map(r=>{
      const field=[r.id,r.path,r.role||'',r.domain].join(' ').toLowerCase();
      const hits=tokens.filter(t=>field.includes(t)).length;
      return {resource:r,score:hits};
    }).filter(x=>x.score===tokens.length)
    .sort((a,b)=>b.score-a.score||a.resource.id.localeCompare(b.resource.id))
    .slice(0,Math.min(30,limit));
}

export function graphSummary(index){
  const count=(items,key)=>Object.fromEntries([...new Set(items.map(key))].sort()
    .map(name=>[name,items.filter(item=>key(item)===name).length]));
  return {schemaVersion:1,mode:'REPOSITORY_KNOWLEDGE_NAVIGATION',
    proofBoundary:index.proofBoundary,
    resources:index.resources.length,
    classes:count(index.resources,r=>r.class),
    authorities:count(index.resources,r=>r.authority),
    lifecycles:count(index.resources,r=>r.lifecycle),
    edgeTypes:count(index.edges,e=>e.type),edges:index.edges.length,
    unresolvedLinkCount:index.unresolved.length,
    unresolved:index.unresolved.slice(0,30)};
}
