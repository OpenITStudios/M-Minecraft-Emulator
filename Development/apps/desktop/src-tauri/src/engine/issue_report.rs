//! Bounded native request-client issue evidence. Recording itself performs no I/O.
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const LIMIT: usize = 16;
fn is_false(value: &bool) -> bool { !*value }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Code {
    IpcWorkerStartFailed,
    IpcTransportFailed,
    IpcInputReceiptInvalid,
    IpcTargetReceiptInvalid,
    IpcInspectReceiptInvalid,
    IpcReplyTimeout,
    IpcReplyLost,
}

// One typed operation vocabulary serves both emission and stored-file admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Operation {
    Inspect,
    Target,
    AgentActivity,
    TestingReserve,
    TestingFinalize,
    TestingInspect,
    InspectSettings,
    ApplyPreset,
    ApplyGraphicsPreset,
    SavePerformance,
    SaveAudio,
    SaveStoragePreferences,
    InspectStorage,
    InspectProfileStorage,
    InspectSelectiveInventory,
    ReviewSelectiveInventory,
    ConfirmSelectedWorlds,
    ConfirmSelectedPacks,
    OpenProfileFolder,
    ClearProfileCache,
    ClearProfileDownloadCache,
    ResetGameSettings,
    RestoreGameSettings,
    DisableImprovedInputResponse,
    EnableTextureStreaming,
    DisableTextureStreaming,
    ClearProfileData,
    Launch,
    LaunchMc02Experiment,
    LaunchAiExperiment,
    Mc02Input,
    AiLeasedInput,
    AiLeaseAcquire,
    AiLeaseRelease,
    AiInputGroup,
    CloseOne,
    CloseAll,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Context {
    pub operation: Operation,
    pub slot: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    c: Code,
    op: Operation,
    #[serde(skip_serializing_if = "Option::is_none")]
    s: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<i32>,
    t: u64,
    n: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    l: Option<u64>,
    // Last confirmed reconciliation of the current uncertainty episode.
    #[serde(skip_serializing_if = "Option::is_none")]
    r: Option<u64>,
    // Recurrences after a confirmed reconciliation, not poll counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    e: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeIssueReport {
    v: u8,
    issues: Vec<Entry>,
    #[serde(default, skip_serializing_if = "is_false")]
    truncated: bool,
}

impl NativeIssueReport {
    // Stored evidence is untrusted. Use the same typed vocabulary and bounds
    // as the producer without importing old observations into live state.
    pub(super) fn valid(&self) -> bool {
        const MAX_JS_INTEGER: u64 = 9_007_199_254_740_991;
        self.v == 1 && !self.issues.is_empty() && self.issues.len() <= LIMIT &&
            self.issues.iter().enumerate().all(|(index, row)| {
                row.s.is_none_or(|slot| (1..=5).contains(&slot)) &&
                row.t <= MAX_JS_INTEGER &&
                row.l.is_none_or(|t| t <= MAX_JS_INTEGER) &&
                row.r.is_none_or(|t| t <= MAX_JS_INTEGER) &&
                row.n > 0 && (row.n > 1) == row.l.is_some() &&
                row.e.is_none_or(|e| e > 0 && (e < row.n || row.n == u32::MAX)) &&
                (row.c != Code::IpcWorkerStartFailed || (row.r.is_none() && row.e.is_none())) &&
                !self.issues[..index].iter().any(|old|
                    old.c == row.c && old.op == row.op && old.s == row.s && old.os == row.os)
            })
    }

    const fn new() -> Self {
        Self { v: 1, issues: Vec::new(), truncated: false }
    }

    fn record(&mut self, code: Code, context: Context, os: Option<i32>, now: u64) -> bool {
        // Never admit an arbitrary ID into the report.
        let slot = context.slot.filter(|s| (1..=5).contains(s));
        if let Some(row) = self.issues.iter_mut().find(|row|
            row.c == code && row.op == context.operation && row.s == slot && row.os == os) {
            row.n = row.n.saturating_add(1);
            row.l = Some(now);
            let recurrence = row.r.take().is_some();
            if recurrence { row.e = Some(row.e.unwrap_or(0).saturating_add(1)); }
            recurrence || row.n.is_power_of_two()
        } else if self.issues.len() < LIMIT {
            self.issues.push(Entry { c: code, op: context.operation, s: slot, os, t: now, n: 1, l: None, r: None, e: None });
            true
        } else {
            let changed = !self.truncated;
            self.truncated = true;
            changed
        }
    }

    // The existing controller gate is authoritative: this only records the
    // transport reconciliation, never the success of the earlier operation.
    fn reconcile_transport(&mut self, now: u64) -> bool {
        let mut changed = false;
        for row in &mut self.issues {
            if row.c != Code::IpcWorkerStartFailed && row.r.is_none() {
                row.r = Some(now);
                changed = true;
            }
        }
        changed
    }
}

static REPORT: Mutex<NativeIssueReport> = Mutex::new(NativeIssueReport::new());

// Called only at the original failed request boundary. Never format raw errors,
// serialize requests, query OS resources, write files, or change failure results.
#[cfg(windows)]
pub(crate) fn note(code: Code, context: Context, os: Option<i32>) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    if let Ok(mut report) = REPORT.lock() {
        if report.record(code, context, os, now) { super::report_store::request_checkpoint(); }
    }
}

#[cfg(windows)]
pub(crate) fn reconcile_transport() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    if let Ok(mut report) = REPORT.lock() {
        if report.reconcile_transport(now) { super::report_store::request_checkpoint(); }
    }
}

pub(crate) fn read() -> Result<Option<NativeIssueReport>, &'static str> {
    let report = REPORT.lock().map_err(|_| "Native issue report unavailable")?;
    Ok((!report.issues.is_empty()).then(|| report.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_aggregates_actual_failures_without_raw_context() {
        let mut report = NativeIssueReport::new();
        let context = Context { operation: Operation::CloseOne, slot: Some(2) };
        report.record(Code::IpcReplyTimeout, context, None, 100);
        report.record(Code::IpcReplyTimeout, context, None, 120);
        report.record(Code::IpcReplyLost, context, None, 130);
        assert_eq!(serde_json::to_value(&report).unwrap(), serde_json::json!({
            "v":1,"issues":[
                {"c":"IPC_REPLY_TIMEOUT","op":"close_one","s":2,"t":100,"n":2,"l":120},
                {"c":"IPC_REPLY_LOST","op":"close_one","s":2,"t":130,"n":1}
            ]
        }));
    }

    #[test]
    fn reconciliation_marks_only_uncertain_rows_and_recurrence_reopens() {
        let mut report = NativeIssueReport::new();
        let context = Context { operation: Operation::CloseOne, slot: Some(2) };
        report.reconcile_transport(90);
        assert!(report.issues.is_empty());
        report.record(Code::IpcWorkerStartFailed, context, None, 100);
        report.record(Code::IpcReplyTimeout, context, None, 101);
        report.record(Code::IpcReplyTimeout, context, None, 102);
        report.reconcile_transport(110);
        report.reconcile_transport(111);
        assert_eq!(report.issues[0].r, None);
        assert_eq!(report.issues[1].r, Some(110));
        assert_eq!(report.issues[1].e, None);
        report.record(Code::IpcReplyTimeout, context, None, 120);
        assert_eq!(report.issues[1].r, None);
        assert_eq!(report.issues[1].e, Some(1));
        assert_eq!(report.issues[1].n, 3);
        report.record(Code::IpcReplyTimeout, context, None, 121);
        assert_eq!(report.issues[1].e, Some(1));
        report.reconcile_transport(130);
        assert_eq!(report.issues[1].r, Some(130));
        assert_eq!(report.issues[1].n, 4);
    }

    #[test]
    fn checkpoint_due_only_for_new_recurrent_or_power_of_two_counts() {
        let mut report = NativeIssueReport::new();
        let context = Context { operation: Operation::Inspect, slot: None };
        for due in [true, true, false, true, false] {
            assert_eq!(report.record(Code::IpcReplyTimeout, context, None, 100), due);
        }
        assert!(report.reconcile_transport(110));
        assert!(!report.reconcile_transport(111));
        assert!(report.record(Code::IpcReplyTimeout, context, None, 120));
    }

    #[test]
    fn report_preserves_bound_and_distinct_os_codes() {
        let mut report = NativeIssueReport::new();
        let context = Context { operation: Operation::Inspect, slot: None };
        for os in 0..17 {
            report.record(Code::IpcTransportFailed, context, Some(os), 100);
        }
        assert_eq!(report.issues.len(), LIMIT);
        assert!(report.truncated);
        report.record(Code::IpcTransportFailed, context, Some(0), 110);
        assert_eq!(report.issues[0].n, 2);
    }
}
