// Read-only source preflight for the optional Minecraft Education HUD experiment.
// Not part of CI, normal app start, package steps or any automatic workflow.
// Source shape / pinned Bedrock sample is NOT Minecraft Education runtime proof.
import {createHash} from 'node:crypto';
import {lstat,readFile,readdir} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const SOURCE_DIR=new URL('./education-native-hud-prototype/',import.meta.url);
const VANILLA_BLOB='e2037b2d0d789a1431610a8f639ccd16e63686c3';
const MAX_SOURCE_BYTES=16384;
const SOURCE_FILE_NAMES=['README.md','manifest.json','ui'];
const UI_FILE_NAMES=['hud_screen.json'];
const LABEL='LT: XYZ notes in app';
const ID='d1de865d-59c8-4f1e-a4f7-832a169fa150';
const MODULE_ID='c65a874c-bbbf-4e53-af2e-197332ccbb21';

function object(value) {
  return value!==null&&typeof value==='object'&&!Array.isArray(value);
}
function exactly(value,keys) {
  return object(value)&&Object.keys(value).length===keys.length&&
    keys.every(key=>Object.hasOwn(value,key));
}
function requireContract(ok,message) {
  if(!ok)throw new TypeError('HUD_SOURCE_REFUSED: '+message);
}
function same(x,y){return JSON.stringify(x)===JSON.stringify(y);}

export function validateHudSource(manifest,hud) {
  requireContract(exactly(manifest,['format_version','header','modules','metadata'])&&
    manifest.format_version===2,'manifest must be a version-2 resources-only schema');
  const h=manifest.header;
  requireContract(exactly(h,['name','description','uuid','version','min_engine_version','pack_scope'])&&
    h.name==='Lazy Tester - Native Coordinates HUD (Experimental)'&&
    typeof h.description==='string'&&h.description.length<=240&&
    h.uuid===ID&&h.pack_scope==='world'&&
    same(h.version,[0,3,0])&&same(h.min_engine_version,[1,26,30]),
    'manifest must be versioned v0.3.0 and restricted to one world');
  requireContract(Array.isArray(manifest.modules)&&manifest.modules.length===1&&
    exactly(manifest.modules[0],['type','uuid','version'])&&
    manifest.modules[0].type==='resources'&&
    manifest.modules[0].uuid===MODULE_ID&&
    same(manifest.modules[0].version,[0,3,0])&&
    exactly(manifest.metadata,['authors'])&&
    same(manifest.metadata.authors,['OpenITStudios']),
    'no scripts, telemetry, dependencies or second module are permitted');

  requireContract(exactly(hud,['namespace','lt_native_coordinates_hint','root_panel/chat_stack'])
    &&hud.namespace==='hud','HUD must have exactly one isolated nested change');
  const hint=hud.lt_native_coordinates_hint;
  requireContract(exactly(hint,[
    'type','text','localize','font_size','shadow','layer','color','offset','bindings',
  ])&&hint.type==='label'&&hint.text===LABEL&&hint.localize===false&&
    hint.font_size==='small'&&hint.shadow===true&&hint.layer===1&&
    same(hint.color,[0.78,0.88,0.98])&&same(hint.offset,[2,0]),
    'hint must remain inert, short and without a custom position size');
  requireContract(Array.isArray(hint.bindings)&&hint.bindings.length===1&&
    exactly(hint.bindings[0],['binding_name','binding_name_override','binding_type'])&&
    hint.bindings[0].binding_name==='#player_position_visible'&&
    hint.bindings[0].binding_name_override==='#visible'&&
    hint.bindings[0].binding_type==='global',
    'hint may read only the existing vanilla position visibility');
  const tree=hud['root_panel/chat_stack'];
  requireContract(exactly(tree,['modifications'])&&
    Array.isArray(tree.modifications)&&tree.modifications.length===1,
    'only the previously reviewed chat stack insertion is allowed');
  const step=tree.modifications[0];
  requireContract(exactly(step,['control_name','operation','value'])&&
    step.control_name==='player_position'&&step.operation==='insert_after'&&
    same(step.value,[{'lt_native_coordinates_hint@hud.lt_native_coordinates_hint':{}}]),
    'no vanilla player_position, root_panel or game controls can be replaced');
  return Object.freeze({
    sourceStatus:'SOURCE_ONLY_SHAPE_ACCEPTED_NOT_EXECUTED',
    module:'resources',
    packScope:'world',
    packVersion:'0.3.0',
    vanillaXyzOverridden:false,
    originalTeamModulesModified:false,
    minecraftEducationCompatibility:'NOT_VERIFIED',
    executableGameplayFeatures:false,
  });
}

// Mojang's raw Bedrock HUD is JSONC (// comments and optional trailing
// commas). Parse read-only rather than trusting string order anywhere in the
// file. This is a source-structure witness, NOT Education/UI runtime proof.
function parseVanillaJsonc(text) {
  requireContract(typeof text==='string' && text.length<500000,
    'invalid Bedrock JSONC input');
  let clear='';
  let quote=false,escape=false,line=false,block=false;
  for(let i=0;i<text.length;i++){
    const char=text[i],next=text[i+1];
    if(line){
      if(char==='\n'||char==='\r'){line=false;clear+=char;}
      else clear+=' ';
      continue;
    }
    if(block){
      if(char==='*'&&next==='/'){block=false;clear+='  ';i++;}
      else clear+=char==='\n'?'\n':' ';
      continue;
    }
    if(quote){
      clear+=char;
      if(escape)escape=false;
      else if(char==='\\')escape=true;
      else if(char==='"')quote=false;
      continue;
    }
    if(char==='"'){quote=true;clear+=char;continue;}
    if(char==='/'&&next==='/'){line=true;clear+='  ';i++;continue;}
    if(char==='/'&&next==='*'){block=true;clear+='  ';i++;continue;}
    clear+=char;
  }
  requireContract(!quote&&!block,'incomplete Bedrock JSONC input');
  const characters=clear.split('');
  quote=false;escape=false;
  for(let i=0;i<characters.length;i++){
    const char=characters[i];
    if(quote){
      if(escape)escape=false;
      else if(char==='\\')escape=true;
      else if(char==='"')quote=false;
      continue;
    }
    if(char==='"'){quote=true;continue;}
    if(char!==',')continue;
    let j=i+1;
    while(j<characters.length&&/\s/u.test(characters[j]))j++;
    if(characters[j]===']'||characters[j]==='}')characters[i]=' ';
  }
  try{return JSON.parse(characters.join(''));}
  catch{throw new TypeError('HUD_SOURCE_REFUSED: Bedrock JSONC cannot be parsed safely');}
}

/**
 * Structural diagnostic, untrusted until validatePinnedVanillaSample hashes
 * the exact original Mojang Git blob. Caller must not treat a synthetic
 * input as Minecraft/Education acceptance.
 */
export function inspectVanillaHudStructure(text) {
  const vanilla=parseVanillaJsonc(text);
  requireContract(object(vanilla)&&object(vanilla.root_panel)&&
    Array.isArray(vanilla.root_panel.controls),
    'Bedrock root_panel controls unavailable');
  const possible=vanilla.root_panel.controls.filter(item=>
    object(item)&&Object.keys(item).length===1&&
    Object.hasOwn(item,'chat_stack'));
  requireContract(possible.length===1,'expected one direct root_panel/chat_stack');
  const stack=possible[0].chat_stack;
  requireContract(object(stack)&&stack.type==='stack_panel'&&
    stack.orientation==='vertical'&&Array.isArray(stack.controls),
    'expected vertical chat_stack controls');
  const names=stack.controls.map(item=>
    object(item)&&Object.keys(item).length===1
      ? Object.keys(item)[0] : null);
  const xyz='player_position@hud.player_position';
  const days='number_of_days_played@hud.number_of_days_played';
  const index=names.indexOf(xyz);
  requireContract(index>=0&&names.lastIndexOf(xyz)===index&&
    names.indexOf(days)===index+1&&names.lastIndexOf(days)===index+1,
    'player_position must occur once immediately before days row');
  const panel=vanilla.player_position;
  requireContract(object(panel)&&panel.type==='image'&&
    same(panel.size,['100%c + 6px','100%c + 2px'])&&
    Array.isArray(panel.controls)&&panel.controls.length===1&&
    object(panel.controls[0])&&
    Object.hasOwn(panel.controls[0],'player_position_text'),
    'native player_position image/text layout drift');
  const textLabel=panel.controls[0].player_position_text;
  const nativeText=Array.isArray(textLabel?.bindings)
    ?textLabel.bindings.filter(b=>
      b?.binding_name==='#player_position_text' &&
      b.binding_name_override==='#text' &&
      b.binding_type==='global'): [];
  const nativeVisible=Array.isArray(panel.bindings)
    ?panel.bindings.filter(b=>
      b?.binding_name==='#player_position_visible' &&
      b.binding_name_override==='#visible'&&
      b.binding_type==='global'): [];
  requireContract(nativeText.length===1&&nativeVisible.length===1,
    'native position text or visibility binding unavailable');
  return Object.freeze({
    sampleStatus:'STRUCTURAL_REVIEW_UNTRUSTED_UNTIL_SHA_CHECK',
    chatStackChildren:names.length,
    nativePositionIndex:index,
    nextControl:days,
    nativeXyzBinding:'#player_position_text',
    nativeVisibilityBinding:'#player_position_visible',
    resourcePatchTarget:'root_panel/chat_stack -> insert_after(player_position)',
    educationCompatibility:'NOT_VERIFIED',
  });
}

export function validatePinnedVanillaSample(raw) {
  requireContract(Buffer.isBuffer(raw)&&raw.length>1000&&raw.length<500000,
    'Mojang reference must be explicit original file bytes');
  const gitBlob=createHash('sha1').update('blob '+raw.length+'\0').update(raw).digest('hex');
  requireContract(gitBlob===VANILLA_BLOB,'reference is not the exact pinned Mojang Bedrock blob');
  const structure=inspectVanillaHudStructure(raw.toString('utf8'));
  return Object.freeze({
    bedrockSample:'PINNED_MOJANG_BLOB_CONFIRMED',
    gitBlob,
    ...structure,
    sampleStatus:'PINNED_BEDROCK_SOURCE_STRUCTURE_CONFIRMED',
    minecraftEducationCompatibility:'NOT_VERIFIED',
  });
}

async function ordinaryFile(url,maxBytes=MAX_SOURCE_BYTES) {
  const st=await lstat(url);
  requireContract(st.isFile()&&!st.isSymbolicLink()&&st.nlink===1&&
    st.size>0&&st.size<=maxBytes,'invalid or unsafe source file');
  const value=await readFile(url);
  const after=await lstat(url);
  requireContract(st.dev===after.dev&&st.ino===after.ino&&
    st.size===after.size&&st.mtimeMs===after.mtimeMs,
    'source changed during read');
  return value;
}
async function ordinaryDir(url) {
  const st=await lstat(url);
  requireContract(st.isDirectory()&&!st.isSymbolicLink(),'source directory must not be a symlink');
}
function expectEntries(actual,expected) {
  requireContract(same([...actual].sort(),[...expected].sort()),
    'unexpected pack source entries; build would include unreviewed assets');
}
export async function reviewHudPrototype(samplePath=null) {
  await ordinaryDir(SOURCE_DIR);
  expectEntries(await readdir(SOURCE_DIR),SOURCE_FILE_NAMES);
  await ordinaryDir(new URL('ui/',SOURCE_DIR));
  expectEntries(await readdir(new URL('ui/',SOURCE_DIR)),UI_FILE_NAMES);
  await ordinaryFile(new URL('README.md',SOURCE_DIR));
  const [manifestBytes,hudBytes]=await Promise.all([
    ordinaryFile(new URL('manifest.json',SOURCE_DIR)),
    ordinaryFile(new URL('ui/hud_screen.json',SOURCE_DIR)),
  ]);
  let manifest,hud;
  try {
    manifest=JSON.parse(manifestBytes.toString('utf8'));
    hud=JSON.parse(hudBytes.toString('utf8'));
  }catch {
    throw new TypeError('HUD_SOURCE_REFUSED: JSON input is malformed');
  }
  const verdict=validateHudSource(manifest,hud);
  const reference=samplePath===null ? {
    bedrockSample:'NOT_SUPPLIED',
    minecraftEducationCompatibility:'NOT_VERIFIED',
  } : validatePinnedVanillaSample(await ordinaryFile(samplePath,500000));
  return {
    schema:1,
    ...verdict,
    pinnedBedrockReference:reference,
    actualMinecraftScreenshots:'NOT_EXECUTED',
    packageCreated:false,
    installerOrWorldChanged:false,
  };
}

// Intentionally no default pack creation, network requests or game interaction.
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  const args=process.argv.slice(2);
  let reference=null;
  if(args.length===2&&args[0]==='--pinned-vanilla'&&path.isAbsolute(args[1])){
    reference=args[1];
  }else if(args.length!==0){
    console.error('Usage: node check-education-native-hud-source.mjs [--pinned-vanilla <absolute Bedrock HUD path>]');
    process.exitCode=2;
  }
  if(process.exitCode!==2){
    reviewHudPrototype(reference)
      .then(result=>console.log(JSON.stringify(result)))
      .catch(error=>{
        console.error(String(error.message).slice(0,260));
        process.exitCode=1;
      });
  }
}
