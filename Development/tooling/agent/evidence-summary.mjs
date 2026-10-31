// Read-only, bounded compression of existing Skills Probe analytics/game results.
// No file watcher, debugger listener, account lookup or gameplay control.
// All player/world attribution remains operator-named and UNVERIFIED.
const EVIDENCE_EVENTS_PER_SLOT = 20;
const MAX_TIMELINE = 30;
const MAX_SIGNAL_COUNT = 12;
const SLOT_SET = new Set([1,2,3,4,5]);

function record(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
function positiveCounts(value) {
  if (!record(value)) return 0;
  return Object.values(value).reduce((total,n) =>
    total + (Number.isSafeInteger(n) && n > 0 ? Math.min(n,1000000) : 0),0);
}
function safeText(value,max=96) {
  return typeof value === 'string' ?
    value.replace(/[\u0000-\u001f\u007f]/g,' ').slice(0,max) : null;
}
function boundedTick(value) {
  return Number.isSafeInteger(value) && value >= 0 ? value : null;
}
export function normalizeEvidenceSlots(slots) {
  if (!Array.isArray(slots) || slots.length < 1 || slots.length > 5 ||
      slots.some(slot => !SLOT_SET.has(slot)) ||
      new Set(slots).size !== slots.length) {
    throw new TypeError('Choose 1-5 distinct Minecraft profile slots');
  }
  return [...slots].sort((a,b)=>a-b);
}

function toEvent(slot,event) {
  if (!record(event) || !record(event.data) ||
      (event.type !== 'analytics' && event.type !== 'game_results')) return null;
  const data=event.data;
  const kind=event.type === 'analytics' ? data.type : data.record_type;
  // The summary can be called by internal evidence fixtures without the
  // file parser. Revalidate here rather than trusting a caller-provided
  // event.type or promoting an arbitrary object to RESULT.
  if (typeof kind!=='string' || !kind.trim() || kind.length>80 ||
      (data.schema!==undefined && data.schema!==1) ||
      (event.type==='game_results' &&
        (data.schema!==1 || kind!=='gameplay_result'))) return null;
  const eventType=safeText(kind,60);
  const endReason=safeText(data.end_reason,24);
  const dropped=positiveCounts(data.dropped_counts);
  const failed=positiveCounts(data.failed_counts);
  const recordId=safeText(data.record_id ?? data.data?.canonical?.record_id,128);
  const sessionId=safeText(data.session_id,80);
  const part=boundedTick(data.data?.part);
  const severity=event.level === 'error' ||
      (event.type === 'analytics' && eventType === 'death') ||
      ['fail','abandoned'].includes(endReason) ||
      dropped > 0 || failed > 0 ? 'ATTENTION' :
    event.type === 'game_results' ? 'RESULT' : 'INFO';
  const observedAt=typeof event.observedAt === 'string' &&
    Number.isFinite(Date.parse(event.observedAt)) ?
      event.observedAt : null;
  return {
    slot, source: event.type, eventType, observedAt,
    sessionId, recordId, stage: safeText(data.stage,80),
    gameTick: boundedTick(data.t?.tick),
    endReason, droppedCount:dropped, failedCount:failed,
    score: typeof data.total_score === 'number' && Number.isFinite(data.total_score) ?
      data.total_score : null,
    map: safeText(data.map?.name,80),
    severity,
    part,
  };
}
function dedupeKey(event) {
  // An operator-assigned MC slot is NOT a game-session identity. Reused
  // record IDs across restarted worlds/players must remain distinct.
  // Without a session ID there is no safe basis for deduplication.
  if (!event.sessionId) return null;
  const scope=[event.slot,event.source,event.sessionId];
  if (event.recordId) {
    return JSON.stringify([...scope,event.recordId,event.part ?? 0]);
  }
  if (event.eventType === 'analytics_summary' ||
      event.eventType === 'export_summary') {
    return JSON.stringify([...scope,event.eventType]);
  }
  return null;
}
export function summarizeEvidenceReports(slots,reports) {
  const selected=normalizeEvidenceSlots(slots);
  if (!Array.isArray(reports) || reports.length !== selected.length) {
    throw new TypeError('Evidence source count does not match requested slots');
  }
  const sources=[];
  const events=[];
  let duplicates=0;
  const seen=new Map();
  for (let index=0;index<selected.length;index++) {
    const slot=selected[index];
    const result=reports[index];
    const valid=record(result) && result.slot === slot &&
      result.status === 'CAPTURE_FILE_PRESENT' && Array.isArray(result.events);
    const status=valid ? 'CAPTURE_FILE_PRESENT' :
      result?.status === 'NO_CAPTURE_FILE' ? 'NO_CAPTURE_FILE' :
      result?.status === 'NOT_CONFIGURED' ? 'NOT_CONFIGURED' :
      result?.status === 'CAPTURE_FILE_UNSTABLE' ? 'CAPTURE_FILE_UNSTABLE' :
      'UNAVAILABLE';
    const truncated=valid && result.truncated === true;
    const sourceEvents=valid ? result.events.slice(-EVIDENCE_EVENTS_PER_SLOT) : [];
    let parsed=0;
    for (const raw of sourceEvents) {
      const event=toEvent(slot,raw);
      if (!event) continue;
      parsed++;
      const key=dedupeKey(event);
      if (key && seen.has(key)) {
        // Repeated pulls of a stable record do not double-count a result.
        // The latest version carries end_reason/duration updates.
        const position=seen.get(key);
        events[position]=event;
        duplicates++;
      } else {
        if (key) seen.set(key,events.length);
        events.push(event);
      }
    }
    sources.push({
      slot, status, parsedRecords:parsed,
      truncated,
      incompleteTrailingLineIgnored:valid &&
        result.incompleteTrailingLineIgnored===true,
      attribution:'OPERATOR_NAMED_UNVERIFIED',
    });
  }
  events.sort((a,b) =>
    (a.observedAt ?? '').localeCompare(b.observedAt ?? '') ||
    a.slot-b.slot);
  // Prioritize actionable anomalies while retaining recent context.
  const selectedPositions=new Set();
  for (let i=Math.max(0,events.length-20);i<events.length;i++) selectedPositions.add(i);
  const important=[];
  for (let i=0;i<events.length;i++) if(events[i].severity==='ATTENTION') important.push(i);
  for (const i of important.slice(-MAX_SIGNAL_COUNT)) selectedPositions.add(i);
  const timeline=[...selectedPositions].sort((a,b)=>a-b).slice(-MAX_TIMELINE)
    .map(i=>events[i]);
  const attention=events.filter(event=>event.severity==='ATTENTION');
  const results=events.filter(event=>event.severity==='RESULT');
  const missing=sources.filter(s=>s.status!=='CAPTURE_FILE_PRESENT').map(s=>s.slot);
  return {
    schema:1, source:'SKILLS_PROBE_CAPTURE_MARKERS',
    requestedSlots:selected, sources,
    records:events.length, duplicatesSuppressed:duplicates,
    timeline, attentionCount:attention.length, resultCount:results.length,
    missingSlots:missing,
    verdict:'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE',
    worldIdentity:'NOT_VERIFIED', playerToProfileIdentity:'NOT_VERIFIED',
    note:'resultCount counts recognized but untrusted gameplay_result records, NEVER a gameplay success. Log file names, player/world identities and map data remain unverified.',
  };
}
export async function summarizeGameEvidence(directory,slots,readEvidence) {
  const selected=normalizeEvidenceSlots(slots);
  if (typeof readEvidence !== 'function') throw new TypeError('Evidence reader unavailable');
  const reports=await Promise.all(selected.map(async slot => {
    try {return await readEvidence(directory,slot);}
    catch {return {slot,status:'UNAVAILABLE',events:[]};}
  }));
  return summarizeEvidenceReports(selected,reports);
}
