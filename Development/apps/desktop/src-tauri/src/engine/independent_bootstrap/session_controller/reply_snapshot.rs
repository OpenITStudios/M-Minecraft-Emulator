//! Controller request-to-receipt scope; no independent state or dispatch.
use super::Sessions;
use crate::engine::native_clients_session::{SessionRequest, SessionSnapshot};

// Keep reply work in one policy: coordinated close already reaps the selected
// slots; status and input receipts must not flush unrelated journals or
// rewrite window labels. Desktop folder and selective-read consumers use
// only their action receipt/payload; selective reads already reap and admit
// the target before inspection. Keep full observation for lifecycle,
// settings and storage reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplySnapshot {
    Full,
    Close,
    Receipt,
    AgentReceipt,
    #[cfg(feature="mc02-native-input")]
    InputReceipt,
    Selected(u32),
    AgentTarget(u32),
}
impl ReplySnapshot {
    pub(super) fn for_request(request: &SessionRequest) -> Self {
        #[cfg(feature="mc02-native-input")]
        if request.expects_input_receipt(){return Self::InputReceipt;}
        match request {
            SessionRequest::CloseOne { .. } | SessionRequest::CloseAll => Self::Close,
            SessionRequest::InspectAgentTarget {slot} => Self::AgentTarget(*slot),
            SessionRequest::OpenProfileFolder { slot, selected_profile } |
            SessionRequest::InspectSelectiveInventory { slot, selected_profile, .. } |
            SessionRequest::ReviewSelectiveInventory { slot, selected_profile, .. } => {
                if *selected_profile == Some(true) && (1..=5).contains(slot) {
                    Self::Selected(*slot)
                } else { Self::Receipt }
            }
            SessionRequest::PublishAgentActivity { .. } |
            SessionRequest::TestingReserve { .. } |
            SessionRequest::TestingFinalize { .. } |
            SessionRequest::TestingInspect { .. } => Self::AgentReceipt,
            _ => Self::Full,
        }
    }

    pub(super) fn collect(self, sessions: &mut Sessions) -> SessionSnapshot {
        match self {
            Self::Full => sessions.snapshot(),
            Self::Close => sessions.snapshot_without_reaping(true),
            Self::Receipt => sessions.snapshot_without_reaping(false),
            Self::AgentReceipt => SessionSnapshot::empty_receipt(),
            #[cfg(feature="mc02-native-input")]
            Self::InputReceipt => SessionSnapshot::empty_receipt(),
            Self::Selected(slot) | Self::AgentTarget(slot) => sessions.snapshot_slots(false, (slot as usize - 1)..slot as usize),
        }
    }
}
