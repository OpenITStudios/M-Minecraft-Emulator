//! Stateless, privacy-minimal checklist for future manual Minecraft Education
//! acceptance. Never gathers credentials, authenticates an account, scans
//! processes, launches a game, or approves an independent Windows provider.
//!
//! Inputs are UNATTESTED human/collector reports. ReportedPass means only
//! "a later operator claims this check was observed"; even every reported pass
//! yields PendingHumanReview, not runtime readiness.

use super::{ProcessIdentity, SessionSlot, StorageEvidenceSource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameTrial {
    TwoClients,
    FourClients,
}

impl GameTrial {
    pub const fn required_slots(self) -> &'static [SessionSlot] {
        const TWO: &[SessionSlot] = &[SessionSlot::Mc01, SessionSlot::Mc02];
        const FOUR: &[SessionSlot] = &[
            SessionSlot::Mc01, SessionSlot::Mc02,
            SessionSlot::Mc03, SessionSlot::Mc04,
        ];
        match self {
            Self::TwoClients => TWO,
            Self::FourClients => FOUR,
        }
    }
}

/// Required client-local observations. No account name, UUID, username,
/// world file, token, registry key or raw executable path is recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientGameCheck {
    OfficialPublisherSignature,
    InteractiveVideoAndInput,
    LicensedSignInInClient,
    SessionSurvivesTabSwitch,
    SignInSurvivesReconnect,
    OwnWorldPersistsAfterRestart,
    MultiplayerInteractive,
    NoObservedGameSingletonConflict,
}

impl ClientGameCheck {
    pub const ALL: [Self; 8] = [
        Self::OfficialPublisherSignature,
        Self::InteractiveVideoAndInput,
        Self::LicensedSignInInClient,
        Self::SessionSurvivesTabSwitch,
        Self::SignInSurvivesReconnect,
        Self::OwnWorldPersistsAfterRestart,
        Self::MultiplayerInteractive,
        Self::NoObservedGameSingletonConflict,
    ];
    fn source(self) -> StorageEvidenceSource {
        match self {
            Self::OfficialPublisherSignature => StorageEvidenceSource::WindowsProcess,
            _ => StorageEvidenceSource::MinecraftRuntime,
        }
    }
}

/// Group observations must describe a simultaneous trial for the supplied
/// process set. Independent per-client "PASS" samples cannot substitute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupGameCheck {
    AllRequiredWindowsPlayableTogether,
    DistinctLicensedAccountsAtSameTime,
    OtherClientsContinueOnTabSwitch,
    OtherClientsContinueOnReconnect,
    CrossClientMultiplayerWorks,
}

impl GroupGameCheck {
    pub const ALL: [Self; 5] = [
        Self::AllRequiredWindowsPlayableTogether,
        Self::DistinctLicensedAccountsAtSameTime,
        Self::OtherClientsContinueOnTabSwitch,
        Self::OtherClientsContinueOnReconnect,
        Self::CrossClientMultiplayerWorks,
    ];
}

/// The evidence source is a label, not independent attestation. A missing
/// observation has no source and must not silently turn into success.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReportedGameFinding {
    #[default]
    Unobserved,
    ReportedPass(StorageEvidenceSource),
    ReportedFailure(StorageEvidenceSource),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientGameSample {
    pub slot: SessionSlot,
    pub process: ProcessIdentity,
    pub checks: [ReportedGameFinding; 8],
}

impl ClientGameSample {
    pub fn empty(slot: SessionSlot, process: ProcessIdentity) -> Self {
        Self { slot, process, checks: [ReportedGameFinding::Unobserved; 8] }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JointGameSample {
    /// Process keys in canonical MC01–MC04 order. For a two-client run the
    /// MC03/MC04 slots MUST be absent. A report from another run, reused PID
    /// or newly restarted process cannot be silently applied to this trial.
    pub processes: [Option<ProcessIdentity>; 4],
    pub checks: [ReportedGameFinding; 5],
}

impl Default for JointGameSample {
    fn default() -> Self {
        Self {
            processes: [None; 4],
            checks: [ReportedGameFinding::Unobserved; 5],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameTrialError {
    UnexpectedSlot,
    DuplicateSlot,
    MissingOrInvalidProcessIdentity,
    ProcessUsedByMultipleSlots,
    WrongEvidenceSource,
    WrongJointProcessSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingGameAcceptance {
    pub trial: GameTrial,
    pub missing_clients: Vec<SessionSlot>,
    pub missing_client_checks: Vec<(SessionSlot, ClientGameCheck)>,
    pub failed_client_checks: Vec<(SessionSlot, ClientGameCheck)>,
    pub missing_joint_checks: Vec<GroupGameCheck>,
    pub failed_joint_checks: Vec<GroupGameCheck>,
}

/// Review future MANUAL observations without side effects or readiness state.
/// Missing slots/checks remain missing. False source elevation is rejected.
/// Even an all-positive report is PendingGameAcceptance and never LaunchAllowed.
pub fn review_game_acceptance(
    trial: GameTrial,
    clients: &[ClientGameSample],
    joint: JointGameSample,
) -> Result<PendingGameAcceptance, GameTrialError> {
    let required = trial.required_slots();
    let mut occupied = [false; 4];
    for client in clients {
        let index = SessionSlot::ALL.iter().position(|slot| *slot == client.slot)
            .expect("all SessionSlot values must be included in ALL");
        if !required.contains(&client.slot) {
            return Err(GameTrialError::UnexpectedSlot);
        }
        if occupied[index] {
            return Err(GameTrialError::DuplicateSlot);
        }
        if client.process.pid == 0 || client.process.created_file_time == 0 {
            return Err(GameTrialError::MissingOrInvalidProcessIdentity);
        }
        if clients.iter().any(|other| {
            other.slot != client.slot && other.process.pid == client.process.pid
        }) {
            return Err(GameTrialError::ProcessUsedByMultipleSlots);
        }
        occupied[index] = true;
    }

    // A joint observation is meaningful only if it names the exact clients
    // in the same trial, including the native MC01 baseline. Never inherit
    // positive observations from a prior two-client or pre-restart run.
    let any_joint_observed = joint.checks.iter().any(|finding| {
        *finding != ReportedGameFinding::Unobserved
    });
    if any_joint_observed {
        for (index, slot) in required.iter().enumerate() {
            let client = clients.iter().find(|client| client.slot == *slot);
            if client.map(|item| item.process) != joint.processes[index] || client.is_none() {
                return Err(GameTrialError::WrongJointProcessSet);
            }
        }
        if joint.processes[required.len()..].iter().any(Option::is_some) {
            return Err(GameTrialError::WrongJointProcessSet);
        }
    } else if joint.processes.iter().any(Option::is_some) {
        // A claimed binding with no actual joint observation is ambiguous.
        return Err(GameTrialError::WrongJointProcessSet);
    }

    // No check can be promoted from a fixture or manager-side Windows
    // metadata to an actual licensed account or gameplay observation.
    for client in clients {
        for (index, finding) in client.checks.iter().enumerate() {
            let expected = ClientGameCheck::ALL[index].source();
            if let ReportedGameFinding::ReportedPass(source)
                | ReportedGameFinding::ReportedFailure(source) = finding
            {
                if *source != expected {
                    return Err(GameTrialError::WrongEvidenceSource);
                }
            }
        }
    }
    for finding in joint.checks {
        if let ReportedGameFinding::ReportedPass(source)
            | ReportedGameFinding::ReportedFailure(source) = finding
        {
            if source != StorageEvidenceSource::MinecraftRuntime {
                return Err(GameTrialError::WrongEvidenceSource);
            }
        }
    }

    let mut report = PendingGameAcceptance {
        trial,
        missing_clients: Vec::new(),
        missing_client_checks: Vec::new(),
        failed_client_checks: Vec::new(),
        missing_joint_checks: Vec::new(),
        failed_joint_checks: Vec::new(),
    };
    for slot in required {
        let sample = clients.iter().find(|client| client.slot == *slot);
        if sample.is_none() {
            report.missing_clients.push(*slot);
        }
        for (index, check) in ClientGameCheck::ALL.iter().enumerate() {
            let finding = sample.map(|client| client.checks[index])
                .unwrap_or(ReportedGameFinding::Unobserved);
            match finding {
                ReportedGameFinding::Unobserved => {
                    report.missing_client_checks.push((*slot, *check));
                }
                ReportedGameFinding::ReportedFailure(_) => {
                    report.failed_client_checks.push((*slot, *check));
                }
                ReportedGameFinding::ReportedPass(_) => {}
            }
        }
    }
    for (index, check) in GroupGameCheck::ALL.iter().enumerate() {
        match joint.checks[index] {
            ReportedGameFinding::Unobserved => report.missing_joint_checks.push(*check),
            ReportedGameFinding::ReportedFailure(_) => report.failed_joint_checks.push(*check),
            ReportedGameFinding::ReportedPass(_) => {}
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(pid: u32) -> ProcessIdentity {
        ProcessIdentity::checked(pid, 133400000000000000 + u64::from(pid)).unwrap()
    }
    fn capture(slot: SessionSlot, pid: u32) -> ClientGameSample {
        ClientGameSample::empty(slot, key(pid))
    }
    fn complete(slot: SessionSlot, pid: u32) -> ClientGameSample {
        let mut client = capture(slot, pid);
        for (i, check) in ClientGameCheck::ALL.iter().enumerate() {
            client.checks[i] = ReportedGameFinding::ReportedPass(check.source());
        }
        client
    }
    fn all_joint(clients: &[ClientGameSample]) -> JointGameSample {
        let mut joint = JointGameSample::default();
        for client in clients {
            let index = SessionSlot::ALL.iter().position(|slot| *slot == client.slot).unwrap();
            joint.processes[index] = Some(client.process);
        }
        joint.checks = [ReportedGameFinding::ReportedPass(
            StorageEvidenceSource::MinecraftRuntime); 5];
        joint
    }

    #[test]
    fn missing_sessions_and_group_checks_are_explicit() {
        let result = review_game_acceptance(
            GameTrial::FourClients, &[], JointGameSample::default()).unwrap();
        assert_eq!(result.missing_clients, SessionSlot::ALL);
        assert_eq!(result.missing_client_checks.len(), 32);
        assert_eq!(result.missing_joint_checks, GroupGameCheck::ALL);
    }

    #[test]
    fn two_client_run_cannot_claim_four_client_concurrency() {
        let clients = [complete(SessionSlot::Mc01, 1), complete(SessionSlot::Mc02, 2)];
        let two = review_game_acceptance(GameTrial::TwoClients, &clients, all_joint(&clients)).unwrap();
        assert!(two.missing_clients.is_empty());
        assert!(two.missing_client_checks.is_empty());
        assert!(two.missing_joint_checks.is_empty());
        let four = review_game_acceptance(
            GameTrial::FourClients, &clients, JointGameSample::default()).unwrap();
        assert_eq!(four.missing_clients, vec![SessionSlot::Mc03, SessionSlot::Mc04]);
        assert_eq!(four.missing_client_checks.len(), 16);
    }

    #[test]
    fn wrong_slot_duplicate_slot_and_process_alias_are_rejected() {
        let unknown = [capture(SessionSlot::Mc03, 3)];
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &unknown, JointGameSample::default()),
            Err(GameTrialError::UnexpectedSlot)
        );
        let duplicates = [capture(SessionSlot::Mc02, 2), capture(SessionSlot::Mc02, 3)];
        assert_eq!(
            review_game_acceptance(GameTrial::FourClients, &duplicates, JointGameSample::default()),
            Err(GameTrialError::DuplicateSlot)
        );
        let alias = [capture(SessionSlot::Mc01, 1), capture(SessionSlot::Mc02, 1)];
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &alias, JointGameSample::default()),
            Err(GameTrialError::ProcessUsedByMultipleSlots)
        );
    }

    #[test]
    fn zero_identity_and_reused_pid_with_new_filetime_are_not_live_clients() {
        let invalid = [ClientGameSample {
            process: ProcessIdentity { pid: 0, created_file_time: 10 },
            ..capture(SessionSlot::Mc02, 2)
        }];
        assert_eq!(
            review_game_acceptance(GameTrial::FourClients, &invalid, JointGameSample::default()),
            Err(GameTrialError::MissingOrInvalidProcessIdentity)
        );
        let alias = [capture(SessionSlot::Mc01, 4), ClientGameSample {
            process: ProcessIdentity::checked(4, key(4).created_file_time + 1).unwrap(),
            ..capture(SessionSlot::Mc02, 5)
        }];
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &alias, JointGameSample::default()),
            Err(GameTrialError::ProcessUsedByMultipleSlots)
        );
    }

    #[test]
    fn synthetic_signature_and_windows_login_reports_are_refused() {
        let mut game = capture(SessionSlot::Mc01, 1);
        game.checks[0] = ReportedGameFinding::ReportedPass(
            StorageEvidenceSource::SyntheticFixture);
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &[game], JointGameSample::default()),
            Err(GameTrialError::WrongEvidenceSource)
        );
        let mut game = capture(SessionSlot::Mc01, 1);
        game.checks[2] = ReportedGameFinding::ReportedPass(
            StorageEvidenceSource::WindowsProcess);
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &[game], JointGameSample::default()),
            Err(GameTrialError::WrongEvidenceSource)
        );
    }

    #[test]
    fn windows_or_fixture_group_claims_cannot_prove_broker_continuity() {
        let clients = [capture(SessionSlot::Mc01, 1), capture(SessionSlot::Mc02, 2)];
        for source in [StorageEvidenceSource::WindowsProcess,
            StorageEvidenceSource::SyntheticFixture] {
            let mut joint = all_joint(&clients);
            joint.checks[1] = ReportedGameFinding::ReportedPass(source);
            assert_eq!(
                review_game_acceptance(GameTrial::TwoClients, &clients, joint),
                Err(GameTrialError::WrongEvidenceSource)
            );
        }
    }

    #[test]
    fn failed_client_and_group_checks_never_disappear_into_missing() {
        let mut client = capture(SessionSlot::Mc02, 2);
        client.checks[4] = ReportedGameFinding::ReportedFailure(
            StorageEvidenceSource::MinecraftRuntime);
        let clients = [capture(SessionSlot::Mc01, 1), client];
        let mut joint = all_joint(&clients);
        joint.checks[3] = ReportedGameFinding::ReportedFailure(
            StorageEvidenceSource::MinecraftRuntime);
        let result = review_game_acceptance(GameTrial::TwoClients, &clients, joint).unwrap();
        assert_eq!(result.failed_client_checks, vec![(
            SessionSlot::Mc02, ClientGameCheck::SignInSurvivesReconnect)]);
        assert_eq!(result.failed_joint_checks,
            vec![GroupGameCheck::OtherClientsContinueOnReconnect]);
        assert!(!result.missing_client_checks.contains(&(
            SessionSlot::Mc02, ClientGameCheck::SignInSurvivesReconnect)));
    }


    #[test]
    fn joint_results_without_exact_client_scope_are_refused() {
        let clients = [complete(SessionSlot::Mc01, 1), complete(SessionSlot::Mc02, 2)];
        let mut unbound = JointGameSample::default();
        unbound.checks[0] = ReportedGameFinding::ReportedPass(
            StorageEvidenceSource::MinecraftRuntime);
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &clients, unbound),
            Err(GameTrialError::WrongJointProcessSet)
        );
        let mut stale = all_joint(&clients);
        stale.processes[1] = ProcessIdentity::checked(
            2, clients[1].process.created_file_time + 1
        );
        assert_eq!(
            review_game_acceptance(GameTrial::TwoClients, &clients, stale),
            Err(GameTrialError::WrongJointProcessSet)
        );
    }

    #[test]
    fn two_client_joint_report_cannot_claim_four_client_runtime_scope() {
        let all = [
            complete(SessionSlot::Mc01, 1), complete(SessionSlot::Mc02, 2),
            complete(SessionSlot::Mc03, 3), complete(SessionSlot::Mc04, 4),
        ];
        let old_two = all_joint(&all[..2]);
        assert_eq!(
            review_game_acceptance(GameTrial::FourClients, &all, old_two),
            Err(GameTrialError::WrongJointProcessSet)
        );
        let full = all_joint(&all);
        assert!(review_game_acceptance(GameTrial::FourClients, &all, full)
            .unwrap().missing_joint_checks.is_empty());
        let mut extra = full;
        extra.processes[3] = None;
        assert_eq!(
            review_game_acceptance(GameTrial::FourClients, &all, extra),
            Err(GameTrialError::WrongJointProcessSet)
        );
    }

    #[test]
    fn even_all_claimed_passes_only_produce_pending_manual_review() {
        let clients = [
            complete(SessionSlot::Mc01, 1), complete(SessionSlot::Mc02, 2),
            complete(SessionSlot::Mc03, 3), complete(SessionSlot::Mc04, 4),
        ];
        let report = review_game_acceptance(
            GameTrial::FourClients, &clients, all_joint(&clients)).unwrap();
        assert!(report.missing_clients.is_empty());
        assert!(report.missing_client_checks.is_empty());
        assert!(report.failed_client_checks.is_empty());
        assert!(report.missing_joint_checks.is_empty());
        assert!(report.failed_joint_checks.is_empty());
        // There is no production-ready, login, account or launch state.
    }
}
