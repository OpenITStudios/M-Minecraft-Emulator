// Shared argument and public text boundary for MCP and trusted Code Mode.
export const REQUEST_LIMIT = 64 * 1024;
const MAX_MCP_TEXT_BYTES = 128 * 1024;
export const MAX_MCP_IMAGE_CHARS = 4 * 1024 * 1024; // 3 MiB encoded image -> Base64.

export function mcpImageMime(data,requireWebp=false) {
  if(typeof data.imageBase64!=='string')return null;
  if(data.imageBase64.length>MAX_MCP_IMAGE_CHARS)throw new Error('MCP_IMAGE_RESPONSE_TOO_LARGE');
  const mime=data.metadata?.mimeType??'image/png';
  if(!['image/png','image/webp'].includes(mime)||(requireWebp&&mime!=='image/webp')) {
    throw new Error('VISUAL_FORMAT_MISMATCH');
  }
  return mime;
}

// An explicit opt-in is required to widen an old read/plan connection.
export function assertAgentSurface(surface,experimental=false) {
  if(!['legacy','code-mode','playtest'].includes(surface))throw new Error('Unknown agent surface');
  if(surface!=='legacy'&&experimental) {
    throw new Error('Code Mode surface does not admit experimental input');
  }
  return surface;
}

// Only fixed application errors are exposed over MCP. Filesystem and
// provider messages may contain account paths, native diagnostics or
// untrusted log text. Internal direct callTool() errors remain available
// to source tests; these do not become model-visible error strings.
const PUBLIC_TOOL_ERRORS=new Set([
  'Unknown tool','Invalid tool arguments',
  'slot must be an integer from 1 to 5',
  'Native operation in progress; do not overlap requests',
  'Evidence window operation in progress',
  'MC01 is reserved for the human host in hybrid mode',
  'MC01 human host is protected from AI visual observation',
  'MC01 human host is protected from AI computer-use targeting',
  'Visual provider is NOT_CONFIGURED',
  'WEBP_CAPTURE_REQUIRES_WGC','GPU_PREVIEW_UNAVAILABLE',
  'GPU_PREVIEW_USE_EXISTING_SAMPLER','GPU_WORKER_STOP_PENDING','VISUAL_FORMAT_MISMATCH',
  'Visual provider is NOT_CONFIGURED; set --cua-exe or --gpu-capture-exe',
  'Native integration is not configured; set --exe',
  'Native controller unavailable or operation refused',
  'Native request outcome UNCERTAIN; do not retry blindly',
  'GPU_OBSERVATION_START_CANCELLED',
  'MCP_TEXT_RESPONSE_TOO_LARGE','MCP_IMAGE_RESPONSE_TOO_LARGE',
  'MC02_EXPERIMENT_DISABLED','MC02_INPUT_INVALID_COMMAND',
  'MC02_INPUT_HUMAN_FOREGROUND_REQUIRED',
  'MC02_INPUT_TARGETS_NOT_SEPARATE','MC02_INPUT_NATIVE_RESULT_UNVERIFIED',
  'MC02_INPUT_REQUEST_STALE',
  'MC02_SEQUENCE_INVALID_ACTIONS','MC02_SEQUENCE_NATIVE_UNCONFIRMED',
  'MC02_SEQUENCE_DEADLINE','MC02_SEQUENCE_ABORTED','MC02_RELEASE_UNCONFIRMED',
  'MULTI_CLIENT_EXPERIMENT_DISABLED','MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS',
  'TEAM_OBSERVATION_INCOMPLETE','TEAM_OBSERVATION_STALE_OR_REPLACED',
  'TEAM_OBSERVATION_TIMING_UNAVAILABLE','TEAM_OBSERVATION_IMAGE_NOT_DELIVERED',
  'TEAM_ACTION_RECEIPT_MISMATCH',
  'AI_SEQUENCE_ABORTED','AI_RELEASE_UNCONFIRMED','AI_INPUT_NATIVE_RESULT_UNVERIFIED',
  'AI_WRITER_LEASE_INVALID','AI_WRITER_LEASE_REQUIRED',
  'AI_LEASE_EPOCH_REQUIRED',
  'AI_WRITER_LEASE_UNCONFIRMED',
  'AI_WRITER_LEASE_RELEASE_UNCONFIRMED',
  'AI_INPUT_RECOVERY_REQUIRED','MCP_CONNECTION_CLOSED',
  'PROBE_SOURCE_REVIEW_REQUIRED',
]);
export function publicToolError(error) {
  const message=typeof error?.message==='string' ? error.message : '';
  return PUBLIC_TOOL_ERRORS.has(message) ||
    /^Native request refused: [A-Z_]{1,40}$/.test(message) ?
    message : 'Tool unavailable; details withheld';
}
export function boundedMcpText(value) {
  const encoded=JSON.stringify(value);
  if(typeof encoded!=='string' ||
     Buffer.byteLength(encoded,'utf8')>MAX_MCP_TEXT_BYTES) {
    throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
  }
  return encoded;
}

export function slotOf(value) {
  if (!Number.isInteger(value) || value < 1 || value > 5) throw new Error('slot must be an integer from 1 to 5');
  return value;
}
export function keysExactly(value, allowed) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).some(key=>!allowed.includes(key))) throw new Error('Invalid tool arguments');
  return value;
}
