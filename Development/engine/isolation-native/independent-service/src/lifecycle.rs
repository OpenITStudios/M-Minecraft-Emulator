//! Service-private Windows SCM state transitions only.
//! Does NOT own Minecraft sessions, IPC admission, providers or data.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Phase {
    Cold,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Signal {
    Begin,
    Ready,
    RequestStop,
    CompleteStop,
    Fatal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvalidTransition;

pub(crate) struct HostLifecycle {
    phase: Phase,
}
impl HostLifecycle {
    pub(crate) fn new() -> Self {
        Self { phase: Phase::Cold }
    }
    #[cfg(test)]
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
    pub(crate) fn advance(&mut self, event: Signal) -> Result<Phase, InvalidTransition> {
        use Phase::*;
        use Signal::*;
        let next = match (self.phase, event) {
            (Cold, Begin) => Starting,
            (Starting, Ready) => Running,
            (Starting | Running, RequestStop) => Stopping,
            (Stopping, CompleteStop) => Stopped,
            (Starting | Running | Stopping, Fatal) => Failed,
            _ => return Err(InvalidTransition),
        };
        self.phase = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_service_start_and_stop_are_ordered() {
        let mut host = HostLifecycle::new();
        assert_eq!(host.advance(Signal::Begin), Ok(Phase::Starting));
        assert_eq!(host.advance(Signal::Ready), Ok(Phase::Running));
        assert_eq!(host.advance(Signal::RequestStop), Ok(Phase::Stopping));
        assert_eq!(host.advance(Signal::CompleteStop), Ok(Phase::Stopped));
        assert_eq!(host.advance(Signal::Begin), Err(InvalidTransition));
    }

    #[test]
    fn invalid_order_cannot_claim_scm_running() {
        let mut host = HostLifecycle::new();
        assert_eq!(host.advance(Signal::Ready), Err(InvalidTransition));
        assert_eq!(host.advance(Signal::RequestStop), Err(InvalidTransition));
        assert_eq!(host.advance(Signal::Begin), Ok(Phase::Starting));
        assert_eq!(host.advance(Signal::CompleteStop), Err(InvalidTransition));
        assert_eq!(host.advance(Signal::Begin), Err(InvalidTransition));
        assert_eq!(host.phase(), Phase::Starting);
    }

    #[test]
    fn fatal_transitions_are_terminal_and_never_ready() {
        for initial in [Phase::Starting, Phase::Running, Phase::Stopping] {
            let mut host = HostLifecycle { phase: initial };
            assert_eq!(host.advance(Signal::Fatal), Ok(Phase::Failed));
            assert_eq!(host.advance(Signal::Ready), Err(InvalidTransition));
            assert_eq!(host.advance(Signal::RequestStop), Err(InvalidTransition));
        }
    }
}
