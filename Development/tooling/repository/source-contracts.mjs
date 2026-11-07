// Read-only, exact-name Tauri IPC declaration witness. NOT a runtime,
// payload compatibility, OS/feature availability or approval validator.
// This is NOT a new Tauri router; it only observes the existing owners.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';

const BOOTSTRAP='Development/apps/desktop/src-tauri/src/app_bootstrap.rs';
const COMMANDS='Development/apps/desktop/src-tauri/src/commands/native_clients.rs';
const FRONTEND='Development/apps/desktop/src/';

function position(text,at){
  const prior=text.slice(0,at);
  return {line:prior.split('\n').length,column:at-prior.lastIndexOf('\n')};
}
function read(root,path){return readFileSync(resolve(root,path),'utf8');}

// Source-token-based Rust registration parser: comments and string contents
// were removed by the shared conservative Rust lexer before any admission.
export function registeredHandlers(source,lex){
  const tokens=lex(source);
  const result=[],unresolved=[];
  let group=0;
  for(let i=0;i<tokens.length-4;i++){
    if(!['tauri','::','generate_handler','!','['].every((v,k)=>tokens[i+k]?.value===v))continue;
    group++;
    let depth=1,start=i+5,closed=false;
    for(let j=start;j<tokens.length;j++){
      const val=tokens[j].value;
      if(val==='[')depth++;
      if(val===']'&&--depth===0){
        closed=true;
        start=j+1;break;
      }
      if(depth!==1)continue;
      if(tokens[j].kind==='id'&&tokens[j].value==='commands'&&
         tokens[j+1]?.value==='::'&&tokens[j+2]?.value==='native_clients'&&
         tokens[j+3]?.value==='::'&&tokens[j+4]?.kind==='id'){
        const cmd=tokens[j+4].value;
        result.push({command:cmd,group,
          registration:{path:BOOTSTRAP,...position(source,tokens[j+4].at),
            syntax:'TAURI_GENERATE_HANDLER_LITERAL',target:cmd},
          conditional:'BUILD_CFG_NOT_EVALUATED'});
        j+=4;
      }
    }
    if(!closed)unresolved.push({path:BOOTSTRAP,reason:'UNTERMINATED_GENERATE_HANDLER_DECLARATION',
      ...position(source,tokens[i].at)});
  }
  return {result,unresolved,groups:group};
}

export function declaredCommands(source,lex){
  const tokens=lex(source),result=[],unresolved=[];
  for(let i=0;i<tokens.length-4;i++){
    if(!['#','[','tauri','::','command',']'].every((v,k)=>tokens[i+k]?.value===v))continue;
    let j=i+6,found=-1;
    for(;j<tokens.length&&j<i+36;j++){
      if(tokens[j].value==='fn'&&tokens[j+1]?.kind==='id'){found=j+1;break;}
      if(tokens[j].value===';'||tokens[j].value==='{')break;
    }
    if(found<0){
      unresolved.push({path:COMMANDS,reason:'UNPARSED_TAURI_COMMAND_DECLARATION',
        ...position(source,tokens[i].at)});continue;
    }
    const cmd=tokens[found].value;
    result.push({command:cmd,definition:{path:COMMANDS,
      ...position(source,tokens[found].at),syntax:'TAURI_COMMAND_FUNCTION',target:cmd}});
  }
  return {result,unresolved};
}

export function frontendCommands(path,source,ts){
  if(!ts)return {requests:[],unresolved:[{path,reason:'TYPESCRIPT_PARSER_UNAVAILABLE'}]};
  const ast=ts.createSourceFile(path,source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
  if(ast.parseDiagnostics?.length)return {requests:[],unresolved:[{
    path,reason:'FRONTEND_TAURI_BRIDGE_SYNTAX_INVALID',
    ...position(source,ast.parseDiagnostics[0].start??0)}]};
  const bindings=new Set();
  const requests=[],unresolved=[];
  for(const node of ast.statements){
    if(!ts.isImportDeclaration(node)||!ts.isStringLiteral(node.moduleSpecifier))continue;
    if(!node.moduleSpecifier.text.endsWith('/invokeRuntime.js')&&
       node.moduleSpecifier.text!=='./invokeRuntime.js')continue;
    const clause=node.importClause?.namedBindings;
    if(clause&&ts.isNamedImports(clause))
      for(const element of clause.elements)
        if((element.propertyName?.text??element.name.text)==='invokeRuntime')
          bindings.add(element.name.text);
  }
  function visit(node){
    if(ts.isCallExpression(node)&&ts.isIdentifier(node.expression)&&
       bindings.has(node.expression.text)){
      const argument=node.arguments[0];
      if(!argument||!ts.isStringLiteralLike(argument)){
        unresolved.push({path,reason:'DYNAMIC_TAURI_COMMAND_NAME',
          ...position(source,node.getStart(ast))});
      }else {
        const command=argument.text;
        requests.push({command,bridge:{path,
          ...position(source,argument.getStart(ast)),
          syntax:'TAURI_FRONTEND_LITERAL_INVOKE',target:command}});
      }
    }
    ts.forEachChild(node,visit);
  }
  visit(ast);
  return {requests,unresolved};
}

export function buildTauriContractProjection({root,resources,byPath,typescript,rustTokens}){
  const necessary=[BOOTSTRAP,COMMANDS];
  if(!necessary.every(path=>byPath.has(path)))
    return {mode:'TAURI_DECLARED_COMMAND_BINDING',status:'TAURI_SOURCE_OWNER_UNAVAILABLE',
      edges:[],bindings:[],gaps:[],registrations:0,bridgeRequests:0};
  if(typeof rustTokens!=='function')
    throw new Error('Tauri declaration projection requires the reviewed Rust tokenizer');
  const registration=registeredHandlers(read(root,BOOTSTRAP),rustTokens);
  const definitions=declaredCommands(read(root,COMMANDS),rustTokens);
  const byRegistration=new Map(),byDeclaration=new Map();
  for(const r of registration.result){
    const all=byRegistration.get(r.command)??[];
    all.push(r);byRegistration.set(r.command,all);
  }
  for(const r of definitions.result){
    const list=byDeclaration.get(r.command)??[];
    list.push(r);byDeclaration.set(r.command,list);
  }
  const requests=[],gaps=[...registration.unresolved,...definitions.unresolved];
  for(const entry of resources){
    if(!entry.path.startsWith(FRONTEND)||!entry.path.endsWith('.ts'))continue;
    const extracted=frontendCommands(entry.path,read(root,entry.path),typescript);
    requests.push(...extracted.requests);
    gaps.push(...extracted.unresolved);
  }
  const bindings=[],edges=[];
  for(const request of requests){
    const reg=byRegistration.get(request.command)??[];
    const defs=byDeclaration.get(request.command)??[];
    if(reg.length===0){
      gaps.push({...request.bridge,reason:'FRONTEND_COMMAND_NOT_REGISTERED'});continue;
    }
    if(defs.length!==1){
      gaps.push({...request.bridge,reason:defs.length?'MULTIPLE_COMMAND_DEFINITIONS':'REGISTERED_COMMAND_DEFINITION_MISSING'});continue;
    }
    const from=byPath.get(request.bridge.path)?.id;
    const to=byPath.get(COMMANDS)?.id;
    const registrationOwner=byPath.get(BOOTSTRAP)?.id;
    if(!from||!to||!registrationOwner)throw new Error('Missing indexed Tauri source owner');
    const witnesses={bridge:request.bridge,registered:reg.map(r=>r.registration),
      definition:defs[0].definition};
    bindings.push({command:request.command,from,
      to,registrationFile:registrationOwner,
      registrationGroupCount:reg.length,
      totalHandlerGroups:registration.groups,
      conditional:'BUILD_CFG_NOT_EVALUATED',witnesses});
    for(const [kind,target] of [
      ['TAURI_COMMAND_NAME_BINDING',to],
      ['TAURI_COMMAND_REGISTRATION',registrationOwner]
    ]){
      edges.push({from,to:target,type:kind,basis:'EXTRACTED_DECLARED_NAME_ONLY',
        command:request.command,sourceSites:[request.bridge],
        contractWitnesses:witnesses,evidence:request.bridge,
        confidence:'STATIC_MATCH_NAME_ONLY'});
    }
  }
  // A Rust handler registered with no current frontend consumer may still be
  // intentionally exposed. That is inventory only, not a missing dependency.
  for(const [command,reg] of byRegistration)
    if(!byDeclaration.has(command))gaps.push({reason:'REGISTERED_COMMAND_NOT_DECLARED',
      command,registration:reg.map(x=>x.registration)});
  return {mode:'TAURI_DECLARED_COMMAND_BINDING',
    status:gaps.length?'PARTIAL_DECLARATION_COVERAGE':'DECLARED_NAMES_MATCH',
    proofBoundary:'TAURI_COMMAND_NAME_MATCH_NOT_PAYLOAD_FEATURE_EXECUTION_OR_AUTHORITY_PROOF',
    registrations:registration.result.length,handlerGroups:registration.groups,
    bridgeRequests:requests.length,
    definitions:definitions.result.length,bindings,edges,gaps};
}
