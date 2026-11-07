// Optional, read-only SOURCE graph: file-level literal import/module declarations.
// Deliberately separate from documentation Graph and native Minecraft runtime.
// TypeScript and Svelte use the ALREADY DECLARED desktop parser dependencies;
// Rust uses a conservative source lexer (NOT a full Rust AST or Cargo resolver).
import {createRequire} from 'node:module';
import {existsSync, lstatSync, readFileSync, readdirSync} from 'node:fs';
import {dirname, extname, isAbsolute, posix, relative, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';

export const SOURCE_ROOT=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const desktopRequire=createRequire(new URL('../../apps/desktop/package.json',import.meta.url));
const CODE_EXTS=new Set(['.rs','.ts','.tsx','.mts','.cts','.js','.jsx','.mjs','.cjs','.svelte']);
const JS_EXTS=['.ts','.tsx','.mts','.cts','.js','.jsx','.mjs','.cjs','.svelte'];
export const SOURCE_DIRS=Object.freeze([
  'Development/apps/desktop/src',
  'Development/apps/desktop/test',
  'Development/apps/desktop/src-tauri/src',
  'Development/apps/desktop/scripts',
  'Development/engine/runtime-core/src',
  'Development/tooling',
]);
const EXTRA_FILES=[
  'Development/apps/desktop/src-tauri/build.rs',
  'Development/apps/desktop/vite.config.ts',
];
const SKIP=new Set(['node_modules','target','dist','coverage','.git','generated','.svelte-kit',
  'binaries','gen','icons','package-resources','test-results','__snapshots__']);
export const SOURCE_PROOF='STATIC_DECLARATION_POTENTIAL_DEPENDENCY_NOT_CALLSITE_BUILD_PLATFORM_OR_RUNTIME_PROOF';

function safeRelative(root,input){
  if(typeof input!=='string'||!input||input.includes('\0')||input.includes('\\')||
    isAbsolute(input)||/^[a-zA-Z]:/.test(input)||input.includes('://'))return undefined;
  const parts=input.split('/');
  if(parts.includes('..')||parts.includes('.'))return undefined;
  const output=posix.normalize(input);
  if(output==='.'||output.startsWith('../')||output.startsWith('/'))return undefined;
  const absolute=resolve(root,output),back=relative(root,absolute);
  if(back==='..'||back.startsWith('..'+sep)||isAbsolute(back))return undefined;
  let cursor=root;
  for(const segment of back.split(sep)){
    cursor=resolve(cursor,segment);
    if(!existsSync(cursor)||lstatSync(cursor).isSymbolicLink())return undefined;
  }
  return output;
}

export function listOwnedSourceFiles(root=SOURCE_ROOT){
  root=resolve(root);
  const paths=[];
  function walk(rel){
    const validated=safeRelative(root,rel);
    if(!validated)return;
    const directory=resolve(root,validated);
    if(!lstatSync(directory).isDirectory())return;
    for(const entry of readdirSync(directory,{withFileTypes:true})
      .sort((a,b)=>a.name.localeCompare(b.name))){
      if(SKIP.has(entry.name)||entry.isSymbolicLink())continue;
      const path=posix.join(validated,entry.name);
      if(entry.isDirectory())walk(path);
      else if(entry.isFile()&&CODE_EXTS.has(extname(entry.name)))paths.push(path);
    }
  }
  for(const dir of SOURCE_DIRS)walk(dir);
  for(const path of EXTRA_FILES){
    const selected=safeRelative(root,path);
    if(selected && lstatSync(resolve(root,selected)).isFile())paths.push(selected);
  }
  return [...new Set(paths)].sort();
}
function sourceId(path){
  return 'source.file.'+path.toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,'');
}
function sourceKind(path){
  if(/(?:^|\/)(?:test|tests|fixtures)\//.test(path)||/\.(?:test|spec)\.[cm]?[jt]sx?$/.test(path))
    return 'AUTHORED_TEST_NOT_EXECUTION_PROOF';
  if(path.endsWith('.rs'))return 'RUST_DECLARATION_SOURCE';
  if(path.endsWith('.svelte'))return 'SVELTE_SOURCE';
  return 'JAVASCRIPT_TYPESCRIPT_SOURCE';
}
function lineColumn(content,offset){
  const before=content.slice(0,offset);
  const last=before.lastIndexOf('\n');
  return {line:before.split('\n').length,column:offset-last};
}
function parser(name){
  try{return {module:desktopRequire(name),status:'DECLARED_DESKTOP_DEPENDENCY'};}
  catch{return {module:undefined,status:'DECLARED_PARSER_NOT_INSTALLED'};}
}
function compilerSites(path,content,ts,svelte){
  if(!ts)return {sites:[],gaps:[{path,reason:'TYPESCRIPT_PARSER_UNAVAILABLE'}]};
  let fragments=[{text:content,offset:0}];
  if(path.endsWith('.svelte')){
    if(!svelte)return {sites:[],gaps:[{path,reason:'SVELTE_PARSER_UNAVAILABLE'}]};
    let ast;
    try{ast=svelte.parse(content,{modern:true});}
    catch{return {sites:[],gaps:[{path,reason:'SVELTE_PARSE_FAILED'}]};}
    fragments=[];
    for(const scope of [ast.module,ast.instance]){
      if(!scope)continue;
      const start=scope.content?.start,end=scope.content?.end;
      if(!Number.isSafeInteger(start)||!Number.isSafeInteger(end)||
        start<0||end<=start||end>content.length){
        return {sites:[],gaps:[{path,reason:'SVELTE_SCRIPT_OFFSETS_UNAVAILABLE'}]};
      }
      fragments.push({text:content.slice(start,end),offset:start});
    }
    if(!fragments.length && /<script(?:\s|>)/i.test(content))
      return {sites:[],gaps:[{path,reason:'SVELTE_SCRIPT_NOT_RECOGNIZED'}]};
  }
  const sites=[],gaps=[];
  for(const frag of fragments){
    const kind=path.endsWith('.tsx')||path.endsWith('.jsx')?ts.ScriptKind.TSX:
      path.endsWith('.js')||path.endsWith('.mjs')||path.endsWith('.cjs')?
      ts.ScriptKind.JS:ts.ScriptKind.TS;
    const ast=ts.createSourceFile(path,frag.text,ts.ScriptTarget.Latest,true,kind);
    if(ast.parseDiagnostics?.length){
      gaps.push({path,reason:'TYPESCRIPT_SYNTAX_PARSE_FAILED',
        line:lineColumn(content,frag.offset+ast.parseDiagnostics[0].start).line});
      continue;
    }
    function add(node,text,syntax,runtime){
      const at=frag.offset+node.getStart(ast);
      sites.push({specifier:text,syntax,runtime,...lineColumn(content,at),path});
    }
    function visit(node){
      if((ts.isImportDeclaration(node)||ts.isExportDeclaration(node))&&
        node.moduleSpecifier && ts.isStringLiteralLike(node.moduleSpecifier)){
        const clause=node.importClause;
        const typeOnly=ts.isImportDeclaration(node)?
          (clause?.isTypeOnly===true||
           (clause?.namedBindings&&ts.isNamedImports(clause.namedBindings)&&
            clause.namedBindings.elements.length>0&&
            clause.namedBindings.elements.every(item=>item.isTypeOnly))):
          (node.isTypeOnly===true||
           (node.exportClause&&ts.isNamedExports(node.exportClause)&&
            node.exportClause.elements.length>0&&
            node.exportClause.elements.every(item=>item.isTypeOnly)));
        add(node.moduleSpecifier,node.moduleSpecifier.text,
          ts.isImportDeclaration(node)?'STATIC_IMPORT':'RE_EXPORT',!typeOnly);
      }
      if(ts.isImportTypeNode(node)&&ts.isLiteralTypeNode(node.argument)&&
        ts.isStringLiteralLike(node.argument.literal))
        add(node.argument.literal,node.argument.literal.text,'IMPORT_TYPE',false);
      if(ts.isImportEqualsDeclaration(node)&&ts.isExternalModuleReference(node.moduleReference)&&
        node.moduleReference.expression&&ts.isStringLiteralLike(node.moduleReference.expression))
        add(node.moduleReference.expression,node.moduleReference.expression.text,
          'IMPORT_EQUALS',!node.isTypeOnly);
      if(ts.isCallExpression(node)&&node.expression.kind===ts.SyntaxKind.ImportKeyword){
        const arg=node.arguments.length===1?node.arguments[0]:undefined;
        if(arg&&ts.isStringLiteralLike(arg))add(arg,arg.text,'DYNAMIC_LITERAL',true);
        else add(node,'', 'DYNAMIC_COMPUTED',true);
      }
      if(ts.isCallExpression(node)&&ts.isIdentifier(node.expression)&&
        node.expression.text==='require'&&node.arguments.length===1&&
        ts.isStringLiteralLike(node.arguments[0]))
        add(node.arguments[0],node.arguments[0].text,'REQUIRE_LITERAL',true);
      ts.forEachChild(node,visit);
    }
    visit(ast);
  }
  return {sites,gaps};
}

// Rust lexical tokens skip nested comments, strings and chars. A Rust
// declaration recorded here is NOT a proof of cfg/feature compilation.
function rustTokens(content){
  const out=[];
  const push=(kind,value,at)=>out.push({kind,value,at});
  let i=0;
  while(i<content.length){
    const begin=i, c=content[i];
    if(/\s/.test(c)){i++;continue;}
    if(content.startsWith('//',i)){
      i=content.indexOf('\n',i+2);
      if(i<0)i=content.length;continue;
    }
    if(content.startsWith('/*',i)){
      i+=2;let depth=1;
      while(i<content.length&&depth>0){
        if(content.startsWith('/*',i)){depth++;i+=2;}
        else if(content.startsWith('*/',i)){depth--;i+=2;}
        else i++;
      }
      continue;
    }
    // Rust raw strings r"", r#"..."#, br#"..."#.
    const raw=content.slice(i).match(/^(?:b)?r(#{0,16})"/);
    if(raw){
      const endMarker='"'+raw[1];i+=raw[0].length;
      const end=content.indexOf(endMarker,i);
      push('string',end<0?'':content.slice(i,end),begin);
      i=end<0?content.length:end+endMarker.length;continue;
    }
    const quote=c==='"'||(c==='b'&&content[i+1]==='"');
    if(quote){
      if(c==='b')i++;i++;
      let value='';
      while(i<content.length){
        if(content[i]==='\\'&&i+1<content.length){value+=content[i+1];i+=2;continue;}
        if(content[i]==='"'){i++;break;}
        value+=content[i++];
      }
      push('string',value,begin);continue;
    }
    // Character literals; lifetimes remain punctuation+identifier.
    if(c==="'"){
      const close=content.indexOf("'",i+1),newline=content.indexOf('\n',i+1);
      if(close>=0&&close-i<=8&&(newline<0||close<newline)){
        i=close+1;continue;
      }
    }
    const ident=content.slice(i).match(/^[A-Za-z_][A-Za-z_0-9]*/);
    if(ident){push('id',ident[0],i);i+=ident[0].length;continue;}
    if(content.startsWith('::',i)){push('punct','::',i);i+=2;continue;}
    push('punct',c,begin);i++;
  }
  return out;
}
function rustModuleSegments(path){
  const after=path.split('/src/')[1];
  if(!after)return undefined;
  const segs=after.split('/');
  const name=segs.pop();
  if(name==='lib.rs'||name==='main.rs'||name==='mod.rs')return segs;
  return [...segs,name.replace(/\.rs$/,'')];
}
function rustCrateRoot(path){return path.split('/src/')[0]+'/src';}
function splitTopLevel(tokens){
  let depth=0,start=0;
  const result=[];
  for(let i=0;i<tokens.length;i++){
    const v=tokens[i].value;
    if(v==='{')depth++;
    if(v==='}')depth--;
    if(depth===0&&v===','){result.push(tokens.slice(start,i));start=i+1;}
  }
  result.push(tokens.slice(start));
  return result.filter(x=>x.length);
}
function rustUsePaths(tokens,prefix=[]){
  const result=[];
  for(const clause of splitTopLevel(tokens)){
    const open=clause.findIndex(x=>x.value==='{');
    if(open>=0){
      let depth=0,close=-1;
      for(let i=open;i<clause.length;i++){
        if(clause[i].value==='{')depth++;
        else if(clause[i].value==='}'&&--depth===0){close=i;break;}
      }
      if(close<0)continue;
      const before=clause.slice(0,open).filter(x=>x.kind==='id').map(x=>x.value);
      result.push(...rustUsePaths(clause.slice(open+1,close),[...prefix,...before]));
      continue;
    }
    const alias=clause.findIndex(x=>x.value==='as');
    const kept=alias>=0?clause.slice(0,alias):clause;
    if(kept.some(t=>t.kind!=='id'&&t.value!=='::'&&t.value!=='*'))continue;
    const ids=kept.filter(x=>x.kind==='id').map(x=>x.value);
    if(ids.length)result.push([...prefix,...ids]);
  }
  return result;
}
function knownRustPath(path,segments,known){
  const base=rustCrateRoot(path);
  for(let n=segments.length;n>0;n--){
    const fragment=segments.slice(0,n).join('/');
    const candidates=[base+'/'+fragment+'.rs',base+'/'+fragment+'/mod.rs'];
    const matches=candidates.filter(x=>known.has(x));
    if(matches.length===1)return {status:'RESOLVED',path:matches[0]};
    if(matches.length>1)return {status:'AMBIGUOUS_RUST_MODULE'};
  }
  return {status:'UNRESOLVED_RUST_MODULE'};
}
function rustUseTarget(path,segments,known){
  let root=rustModuleSegments(path);
  if(!root||!segments.length)return {status:'UNSUPPORTED_RUST_PATH'};
  let rest=[...segments];
  if(rest[0]==='crate'){root=[];rest.shift();}
  else if(rest[0]==='self'){rest.shift();}
  else if(rest[0]==='super'){
    while(rest[0]==='super'){
      if(!root.length)return {status:'OUTSIDE_RUST_CRATE'};
      root.pop();rest.shift();
    }
  }else return {status:'EXTERNAL_OR_UNQUALIFIED_RUST_USE'};
  if(rest.at(-1)==='self')rest.pop();
  return knownRustPath(path,[...root,...rest],known);
}
function rustModTarget(path,name,known){
  const root=rustCrateRoot(path),parent=rustModuleSegments(path);
  if(!parent)return {status:'UNSUPPORTED_RUST_MODULE'};
  const stem=[...parent,name].join('/');
  const matches=[root+'/'+stem+'.rs',root+'/'+stem+'/mod.rs'].filter(x=>known.has(x));
  return matches.length===1?{status:'RESOLVED',path:matches[0]}:
    {status:matches.length?'AMBIGUOUS_RUST_MODULE':'UNRESOLVED_RUST_MODULE'};
}
function rustPathAttribute(tokens,index){
  // Only an immediately adjacent authored #[path = "literal.rs"] is
  // recognized; complex cfg_attr path selection stays unresolved.
  let end=index-1;
  while(tokens[end]?.value===']'){
    let depth=0,open=-1;
    for(let i=end;i>=0;i--){
      if(tokens[i].value===']')depth++;
      if(tokens[i].value==='['&&--depth===0){open=i;break;}
    }
    if(open<=0||tokens[open-1]?.value!=='#')break;
    const inner=tokens.slice(open+1,end);
    if(inner[0]?.value==='path'){
      if(inner[1]?.value==='='&&inner[2]?.kind==='string')
        return {status:'LITERAL',path:inner[2].value};
      return {status:'COMPUTED'};
    }
    if(inner[0]?.value==='cfg_attr'&&inner.some(x=>x.value==='path'))
      return {status:'COMPUTED'};
    end=open-2;
  }
  return undefined;
}
function rustSites(path,content,known){
  const tokens=rustTokens(content),sites=[],gaps=[];
  for(let i=0;i<tokens.length;i++){
    const t=tokens[i];
    if(t.kind!=='id')continue;
    if(t.value==='mod'&&tokens[i+1]?.kind==='id'&&tokens[i+2]?.value===';'){
      const attribute=rustPathAttribute(tokens,i);
      let target;
      if(attribute){
        const candidate=attribute.status==='LITERAL'?
          posix.normalize(posix.join(posix.dirname(path),attribute.path)):undefined;
        target=attribute.status!=='LITERAL'?
          {status:'COMPUTED_RUST_MODULE_PATH'}:
          !candidate||candidate.startsWith('../')||candidate.startsWith('/')?
          {status:'OUTSIDE_REPOSITORY'}:
          known.has(candidate)?{status:'RESOLVED',path:candidate}:
          {status:'UNRESOLVED_RUST_MODULE_PATH'};
      }else target=rustModTarget(path,tokens[i+1].value,known);
      const at=lineColumn(content,tokens[i+1].at);
      sites.push({path,syntax:'RUST_MOD',specifier:tokens[i+1].value,
        conditional:'CFG_NOT_EVALUATED',...at,resolution:target});
    }
    if(t.value==='include'&&tokens[i+1]?.value==='!'&&tokens[i+2]?.value==='('){
      if(tokens[i+3]?.kind==='string'&&tokens[i+4]?.value===')'){
        const requested=tokens[i+3].value;
        const candidate=posix.normalize(posix.join(posix.dirname(path),requested));
        const target=candidate.startsWith('../')||candidate.startsWith('/')?
          {status:'OUTSIDE_REPOSITORY'}:
          known.has(candidate)?{status:'RESOLVED',path:candidate}:
          {status:'UNRESOLVED_RUST_INCLUDE'};
        sites.push({path,syntax:'RUST_INCLUDE_LITERAL',specifier:requested,
          conditional:'CFG_NOT_EVALUATED',
          ...lineColumn(content,tokens[i+3].at),resolution:target});
      }else gaps.push({path,reason:'COMPUTED_RUST_INCLUDE_UNTRACED',
        ...lineColumn(content,t.at)});
    }
    if(t.value==='use'){
      const collected=[];let j=i+1,depth=0;
      for(;j<tokens.length&&j-i<400;j++){
        if(tokens[j].value==='{')depth++;
        if(tokens[j].value==='}')depth--;
        if(tokens[j].value===';'&&depth===0)break;
        collected.push(tokens[j]);
      }
      if(tokens[j]?.value!==';'){
        gaps.push({path,reason:'UNSUPPORTED_RUST_USE',...lineColumn(content,t.at)});
        continue;
      }
      const paths=rustUsePaths(collected);
      for(const segments of paths){
        const target=rustUseTarget(path,segments,known);
        if(target.status==='EXTERNAL_OR_UNQUALIFIED_RUST_USE')continue;
        sites.push({path,syntax:'RUST_USE',specifier:segments.join('::'),
          conditional:'CFG_NOT_EVALUATED',
          ...lineColumn(content,t.at),resolution:target});
      }
      i=j;
    }
  }
  return {sites,gaps};
}
function resolveJsImport(from,specifier,known){
  if(!specifier)return {status:'DYNAMIC_COMPUTED'};
  if(!specifier.startsWith('.'))return {status:'EXTERNAL_OR_ALIAS'};
  if(specifier.includes('\0')||specifier.includes('\\')||specifier.includes('?')||
      specifier.includes('#'))return {status:'UNSUPPORTED_SOURCE_SPECIFIER'};
  const target=posix.normalize(posix.join(posix.dirname(from),specifier));
  if(target.startsWith('../')||target==='..'||target.startsWith('/'))
    return {status:'OUTSIDE_REPOSITORY'};
  const ext=posix.extname(target);
  if(ext&&!JS_EXTS.includes(ext))return {status:'NON_CODE_RESOURCE'};
  let candidates=[];
  if(ext){
    candidates=[target];
    const stem=target.slice(0,-ext.length);
    if(ext==='.js')candidates.push(stem+'.ts',stem+'.tsx',stem+'.mts',stem+'.cts');
    if(ext==='.mjs')candidates.push(stem+'.mts');
    if(ext==='.cjs')candidates.push(stem+'.cts');
  }else {
    candidates=[target,...JS_EXTS.map(e=>target+e),
      ...JS_EXTS.map(e=>target+'/index'+e)];
  }
  const matched=[...new Set(candidates.filter(x=>known.has(x)))];
  return matched.length===1?{status:'RESOLVED',path:matched[0]}:
    {status:matched.length?'AMBIGUOUS_SOURCE_TARGET':'UNRESOLVED_SOURCE_TARGET'};
}

function localCargoDependencies(root){
  // Small, read-only local path dependency witness. This is not Cargo
  // feature resolution, lockfile inference or package execution.
  const manifests=[
    'Development/apps/desktop/src-tauri/Cargo.toml',
    'Development/engine/runtime-core/Cargo.toml',
  ];
  const registered=new Set(manifests.filter(p=>safeRelative(root,p)));
  const dependencies=[],unresolved=[];
  for(const manifest of registered){
    const lines=readFileSync(resolve(root,manifest),'utf8').split(/\r?\n/);
    let section='';
    for(let i=0;i<lines.length;i++){
      const line=lines[i].split('#',1)[0].trim();
      if(/^\[[^\]]+\]$/.test(line)){section=line;continue;}
      if(!section.endsWith('.dependencies]')&&
         !['[dependencies]','[build-dependencies]'].includes(section))continue;
      const match=line.match(/^([A-Za-z_][A-Za-z_0-9-]*)\s*=\s*\{([^{}]+)\}/);
      if(!match)continue;
      const pathValue=match[2].match(/\bpath\s*=\s*"([^"]+)"/);
      if(!pathValue)continue;
      const resolved=posix.normalize(posix.join(posix.dirname(manifest),
        pathValue[1],'Cargo.toml'));
      const path=registered.has(resolved)?resolved:undefined;
      const evidence={path:manifest,line:i+1,target:pathValue[1]};
      if(!path){
        unresolved.push({from:manifest,packageAlias:match[1],
          reason:'UNREGISTERED_LOCAL_CARGO_PATH',evidence});continue;
      }
      dependencies.push({fromManifest:manifest,toManifest:path,
        packageAlias:match[1],type:'CARGO_LOCAL_PATH_DEPENDENCY',
        basis:'EXTRACTED_MANIFEST_DECLARATION',evidence});
    }
  }
  return {dependencies,unresolved};
}

export function buildSourceDependencyIndex(root=SOURCE_ROOT,{typescript,svelteCompiler}={}){
  root=resolve(root);
  const sourcePaths=listOwnedSourceFiles(root),known=new Set(sourcePaths);
  const resources=sourcePaths.map(path=>({id:sourceId(path),path,class:'SOURCE',
    domain:path.endsWith('.rs')?'rust':path.endsWith('.svelte')?'svelte':'ts-js',
    authority:'CANONICAL',lifecycle:'ACTIVE',kind:sourceKind(path),
    ownerLookup:'docs/system/implementation-map.md'}));
  const byPath=new Map(resources.map(r=>[r.path,r]));
  const byId=new Map();
  for(const r of resources){
    if(byId.has(r.id))throw new Error('Source ID collision: '+r.id);
    byId.set(r.id,r);
  }
  const needsTs=resources.some(r=>r.domain!=='rust');
  const needsSvelte=resources.some(r=>r.domain==='svelte');
  const tsInfo=needsTs&&typescript===undefined?parser('typescript'):
    {module:typescript,status:typescript?'PROVIDED':'NOT_REQUIRED'};
  const svInfo=needsSvelte&&svelteCompiler===undefined?parser('svelte/compiler'):
    {module:svelteCompiler,status:svelteCompiler?'PROVIDED':'NOT_REQUIRED'};
  const edges=new Map(),unresolved=[],unparsed=[];
  let siteCount=0,externalCount=0,resourceCount=0;
  for(const resource of resources){
    const path=resource.path,full=resolve(root,path);
    if(lstatSync(full).size>2_000_000){
      unparsed.push({path,reason:'SOURCE_FILE_EXCEEDS_2MB_BOUND'});
      continue;
    }
    const content=readFileSync(full,'utf8');
    const result=path.endsWith('.rs')?rustSites(path,content,known):
      compilerSites(path,content,tsInfo.module,svInfo.module);
    unparsed.push(...result.gaps);
    for(const site of result.sites){
      siteCount++;
      // Preserve the origin file and line even for resolved Rust sites.
      // Spreading target.path into the site silently overwrote its source path.
      const target=site.resolution??resolveJsImport(path,site.specifier,known);
      if(['EXTERNAL_OR_ALIAS','EXTERNAL_OR_UNQUALIFIED_RUST_USE'].includes(target.status)){
        externalCount++;continue;
      }
      if(target.status==='NON_CODE_RESOURCE'){resourceCount++;continue;}
      if(target.status!=='RESOLVED'){
        unresolved.push({from:resource.id,path,line:site.line,column:site.column,
          specifier:site.specifier,syntax:site.syntax,reason:target.status});
        continue;
      }
      const to=byPath.get(target.path);
      if(!to){
        unresolved.push({from:resource.id,path,line:site.line,column:site.column,
          specifier:site.specifier,syntax:site.syntax,reason:'UNREGISTERED_SOURCE'});
        continue;
      }
      if(to.id===resource.id)continue;
      const pair=resource.id+'|'+to.id;
      const edge=edges.get(pair)||{from:resource.id,to:to.id,type:'DEPENDS_ON',
        basis:'EXTRACTED_SOURCE_DECLARATION',sourceSites:[]};
      edge.sourceSites.push({path,line:site.line,column:site.column,
        syntax:site.syntax,specifier:site.specifier,runtime:site.runtime,
        ...(site.conditional?{conditional:site.conditional}:{})});
      edges.set(pair,edge);
    }
  }
  const cargo=localCargoDependencies(root);
  const ordered=[...edges.values()].map(e=>{
    e.sourceSites.sort((a,b)=>a.path.localeCompare(b.path)||a.line-b.line||a.column-b.column);
    return {...e,evidence:e.sourceSites[0]};
  }).sort((a,b)=>a.from.localeCompare(b.from)||a.to.localeCompare(b.to));
  unresolved.sort((a,b)=>a.path.localeCompare(b.path)||a.line-b.line||a.specifier.localeCompare(b.specifier));
  return {schemaVersion:1,mode:'SOURCE_FILE_DEPENDENCY_PROJECTION',
    proofBoundary:SOURCE_PROOF,resources,byPath,byId,edges:ordered,unresolved,
    unparsed,siteCount,externalCount,resourceCount,
    cargoLocalDependencies:cargo.dependencies,cargoUnresolved:cargo.unresolved,
    coverage:{indexedSourceFiles:resources.length,
      excludedLanguageFamilies:['C_CPP_HEADERS','POWERSHELL','CMAKE','RUST_CFG_MACRO_EXPANSION','TAURI_COMMAND_LINKING'],
      completeness:'NOT_PROVEN_COMPLETE'},
    parserAvailability:{typescript:tsInfo.status,svelte:svInfo.status},
    status:unparsed.length?'INCOMPLETE_PARSER_COVERAGE':
      unresolved.length?'PARTIAL_UNRESOLVED_DECLARATIONS':'STATIC_DECLARATIONS_INDEXED'};
}
