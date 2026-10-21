//! Pure, non-authoritative proof classification for a future independently owned
//! Windows session provider. No process, account, file or driver operations.
//!
//! The existing desktop admission remains the ONLY launch authority. These
//! observations MUST come from future trusted collectors, never UI switches,
//! configuration data, or a self-reported "ready" field.

/// The context in which an observation was actually verified.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProofContext {
    SyntheticFixture,
    WindowsMechanism,
    MinecraftRuntime,
}

/// Unknown and failed findings are never interpreted as success.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Finding {
    #[default]
    Unknown,
    Passed(ProofContext),
    Failed,
}

/// Evidence domains are deliberately independent. Four directories, four PIDs,
/// or separate Windows user SIDs do not establish these claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IsolationCheck {
    DistinctObjectNamespaces,
    PrivateLocalStorage,
    PeerStorageDenied,
    HostStorageDenied,
    IndependentProcessLifetimes,
    PrivatePersistentRoaming,
    /// Distinct game process-local FOLDERID_RoamingAppData, not just APPDATA.
    PrivateRoamingKnownFolder,
    /// Actual private registry access for the target game process, not a
    /// RegLoadAppKey fixture, UAC virtualization flag or chosen hive file.
    PrivateRegistryView,
    IndependentMinecraftLifetimes,
    SeparateLicensedSignIns,
    /// Runtime confirmation that switching/restarting one client does not
    /// attach another slot's Windows broker/MSAL account. No token reads.
    DistinctBrokerAccountContexts,
    RenderAndInputUsable,
    FourConcurrentPlayableClients,
}

impl IsolationCheck {
    const fn index(self) -> usize {
        match self {
            Self::DistinctObjectNamespaces => 0,
            Self::PrivateLocalStorage => 1,
            Self::PeerStorageDenied => 2,
            Self::HostStorageDenied => 3,
            Self::IndependentProcessLifetimes => 4,
            Self::PrivatePersistentRoaming => 5,
            Self::PrivateRoamingKnownFolder => 6,
            Self::PrivateRegistryView => 7,
            Self::IndependentMinecraftLifetimes => 8,
            Self::SeparateLicensedSignIns => 9,
            Self::DistinctBrokerAccountContexts => 10,
            Self::RenderAndInputUsable => 11,
            Self::FourConcurrentPlayableClients => 12,
        }
    }
}

/// This is an in-memory proof summary, NOT persistent lifecycle state, a
/// credential cache, an attestation, or permission to launch an executable.
#[derive(Clone, Debug, Default)]
pub struct IsolationFindings {
    entries: [Finding; 13],
}

impl IsolationFindings {
    pub fn record(&mut self, check: IsolationCheck, finding: Finding) {
        self.entries[check.index()] = finding;
    }

    pub fn finding(&self, check: IsolationCheck) -> Finding {
        self.entries[check.index()]
    }

    /// Reports which evidence is absent at the EXACT required proof context.
    /// Game-runtime testimony cannot replace Win32 mechanistic proof, and
    /// Windows mechanics cannot replace actual game sign-in.
    /// This is diagnostic only: it never selects or activates a provider.
    pub fn gaps(&self, goal: ProofGoal) -> Vec<IsolationCheck> {
        requirements(goal)
            .iter()
            .filter_map(|&(check, context)| match self.finding(check) {
                Finding::Passed(observed) if observed == context => None,
                _ => Some(check),
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofGoal {
    /// A native synthetic fixture, not a Minecraft or auth claim.
    SyntheticIsolation,
    /// Native OS mechanism evidence, including persistent roaming.
    WindowsStorageFeasibility,
    /// Two licensed, interactive and independent Minecraft clients.
    TwoMinecraftClients,
    /// Final four-client product acceptance (still a separate human gate).
    FourMinecraftClients,
}

type Requirement = (IsolationCheck, ProofContext);
use self::IsolationCheck as Check;
use self::ProofContext as Tier;

const SYNTHETIC: [Requirement; 5] = [
    (Check::DistinctObjectNamespaces, Tier::SyntheticFixture),
    (Check::PrivateLocalStorage, Tier::SyntheticFixture),
    (Check::PeerStorageDenied, Tier::SyntheticFixture),
    (Check::HostStorageDenied, Tier::SyntheticFixture),
    (Check::IndependentProcessLifetimes, Tier::SyntheticFixture),
];

const WINDOWS: [Requirement; 8] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
    (Check::PrivateRoamingKnownFolder, Tier::WindowsMechanism),
    (Check::PrivateRegistryView, Tier::WindowsMechanism),
];

const TWO_GAME: [Requirement; 12] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
    (Check::PrivateRoamingKnownFolder, Tier::WindowsMechanism),
    (Check::PrivateRegistryView, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::IndependentMinecraftLifetimes, Tier::MinecraftRuntime),
    (Check::SeparateLicensedSignIns, Tier::MinecraftRuntime),
    (Check::DistinctBrokerAccountContexts, Tier::MinecraftRuntime),
    (Check::RenderAndInputUsable, Tier::MinecraftRuntime),
];

const FOUR_GAME: [Requirement; 13] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
    (Check::PrivateRoamingKnownFolder, Tier::WindowsMechanism),
    (Check::PrivateRegistryView, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::IndependentMinecraftLifetimes, Tier::MinecraftRuntime),
    (Check::SeparateLicensedSignIns, Tier::MinecraftRuntime),
    (Check::DistinctBrokerAccountContexts, Tier::MinecraftRuntime),
    (Check::RenderAndInputUsable, Tier::MinecraftRuntime),
    (Check::FourConcurrentPlayableClients, Tier::MinecraftRuntime),
];

const fn requirements(goal: ProofGoal) -> &'static [Requirement] {
    match goal {
        ProofGoal::SyntheticIsolation => &SYNTHETIC,
        ProofGoal::WindowsStorageFeasibility => &WINDOWS,
        ProofGoal::TwoMinecraftClients => &TWO_GAME,
        ProofGoal::FourMinecraftClients => &FOUR_GAME,
    }
}


// Per-slot diagnostic evidence. This is NEVER persisted or used as another
// lifecycle/readiness owner. All values are supplied by future trusted
// collectors; a caller-supplied PID alone is not a Windows attestation.
// MC01 remains under the existing official/native owner.
use crate::independent_engine::{ProcessIdentity, SessionSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopedProofError {
    NativeSlotNotOwned,
    InvalidProcess,
    SlotNotBound,
    ProcessChanged,
    ProcessAlreadyBoundToPeer,
    ConflictingEvidence,
}

#[derive(Clone, Debug)]
struct BoundSlotFindings {
    process: ProcessIdentity,
    findings: IsolationFindings,
}

/// Keep MC02, MC03 and MC04 separate even if the same fixture reports PASS
/// for all of them. Any process restart requires a NEW ephemeral ledger:
/// observations from the old PID/creation FILETIME never carry forward.
#[derive(Clone, Debug, Default)]
pub struct ScopedIsolationFindings {
    isolated: [Option<BoundSlotFindings>; 3],
}

fn isolated_index(slot: SessionSlot) -> Result<usize, ScopedProofError> {
    match slot {
        SessionSlot::Mc01 => Err(ScopedProofError::NativeSlotNotOwned),
        SessionSlot::Mc02 => Ok(0),
        SessionSlot::Mc03 => Ok(1),
        SessionSlot::Mc04 => Ok(2),
    }
}

impl ScopedIsolationFindings {
    /// First bind a specific process identity to one logical slot.
    /// This value is still untrusted until a native observer verifies it.
    pub fn bind_process(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
    ) -> Result<(), ScopedProofError> {
        let index = isolated_index(slot)?;
        if process.pid == 0 || process.created_file_time == 0 {
            return Err(ScopedProofError::InvalidProcess);
        }
        // Two concurrently claimed clients may NEVER share one Windows PID.
        // Even a different creation FILETIME on the same PID would describe
        // a reused process, not two concurrently live Minecraft sessions.
        if self.isolated.iter().enumerate().any(|(peer_index, entry)| {
            peer_index != index
                && entry.as_ref().is_some_and(|entry| entry.process.pid == process.pid)
        }) {
            return Err(ScopedProofError::ProcessAlreadyBoundToPeer);
        }
        match &self.isolated[index] {
            Some(existing) if existing.process == process => Ok(()),
            Some(_) => Err(ScopedProofError::ProcessChanged),
            None => {
                self.isolated[index] = Some(BoundSlotFindings {
                    process,
                    findings: IsolationFindings::default(),
                });
                Ok(())
            }
        }
    }

    pub fn record(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        check: IsolationCheck,
        finding: Finding,
    ) -> Result<(), ScopedProofError> {
        let index = isolated_index(slot)?;
        let bound = self.isolated[index]
            .as_mut()
            .ok_or(ScopedProofError::SlotNotBound)?;
        if bound.process != process {
            return Err(ScopedProofError::ProcessChanged);
        }
        // Keep failure sticky and proof context immutable within one
        // process-bound ledger. Corrected measurements require a new,
        // explicitly created ephemeral review instead of overwriting a
        // failure or upgrading a synthetic claim to Windows/game proof.
        let prior = bound.findings.finding(check);
        let may_record = match (prior, finding) {
            (Finding::Unknown, _) => true,
            (Finding::Failed, Finding::Failed) => true,
            (Finding::Passed(old), Finding::Passed(new)) => old == new,
            (Finding::Passed(_), Finding::Failed) => true,
            _ => false,
        };
        if !may_record {
            return Err(ScopedProofError::ConflictingEvidence);
        }
        bound.findings.record(check, finding);
        Ok(())
    }

    pub fn gaps_for_slot(
        &self,
        slot: SessionSlot,
        process: ProcessIdentity,
        goal: ProofGoal,
    ) -> Result<Vec<IsolationCheck>, ScopedProofError> {
        let index = isolated_index(slot)?;
        let bound = self.isolated[index]
            .as_ref()
            .ok_or(ScopedProofError::SlotNotBound)?;
        if bound.process != process {
            return Err(ScopedProofError::ProcessChanged);
        }
        Ok(bound.findings.gaps(goal))
    }

    /// Report every missing check separately for MC02–MC04. This function
    /// does NOT inspect MC01, certify simultaneous gameplay, or grant Launch.
    /// An unbound slot conservatively reports ALL requirements as missing.
    pub fn isolated_slot_gaps(
        &self,
        goal: ProofGoal,
    ) -> Vec<(SessionSlot, IsolationCheck)> {
        [SessionSlot::Mc02, SessionSlot::Mc03, SessionSlot::Mc04]
            .into_iter()
            .enumerate()
            .flat_map(|(index, slot)| {
                let missing = match &self.isolated[index] {
                    Some(bound) => bound.findings.gaps(goal),
                    None => requirements(goal).iter().map(|&(check, _)| check).collect(),
                };
                missing.into_iter().map(move |check| (slot, check))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn same_pid_never_represents_two_live_slots_even_if_creation_time_differs() {
        let mut ledger = ScopedIsolationFindings::default();
        let first = ProcessIdentity::checked(501, 133400000000000501).unwrap();
        let same_key = first;
        let reused = ProcessIdentity::checked(501, first.created_file_time + 1).unwrap();
        ledger.bind_process(SessionSlot::Mc02, first).unwrap();
        assert_eq!(ledger.bind_process(SessionSlot::Mc02, first), Ok(()));
        for key in [same_key, reused] {
            assert_eq!(
                ledger.bind_process(SessionSlot::Mc03, key),
                Err(ScopedProofError::ProcessAlreadyBoundToPeer)
            );
            assert_eq!(
                ledger.bind_process(SessionSlot::Mc04, key),
                Err(ScopedProofError::ProcessAlreadyBoundToPeer)
            );
        }
        let separate = ProcessIdentity::checked(502, first.created_file_time).unwrap();
        ledger.bind_process(SessionSlot::Mc03, separate).unwrap();
        assert!(ledger.isolated_slot_gaps(ProofGoal::FourMinecraftClients)
            .iter().any(|(slot, _)| *slot == SessionSlot::Mc04));
    }

    #[test]
    fn failed_check_cannot_be_overwritten_by_pass_or_unknown_on_same_process() {
        let mut ledger = ScopedIsolationFindings::default();
        let identity = ProcessIdentity::checked(520, 133400000000000520).unwrap();
        ledger.bind_process(SessionSlot::Mc02, identity).unwrap();
        ledger.record(SessionSlot::Mc02, identity, Check::PrivateRegistryView, Finding::Failed)
            .unwrap();
        for changed in [Finding::Unknown, Finding::Passed(Tier::WindowsMechanism)] {
            assert_eq!(
                ledger.record(SessionSlot::Mc02, identity, Check::PrivateRegistryView, changed),
                Err(ScopedProofError::ConflictingEvidence)
            );
        }
        assert!(ledger.gaps_for_slot(SessionSlot::Mc02, identity, ProofGoal::WindowsStorageFeasibility)
            .unwrap().contains(&Check::PrivateRegistryView));
        ledger.record(SessionSlot::Mc02, identity, Check::PrivateRegistryView, Finding::Failed)
            .unwrap();
    }

    #[test]
    fn synthetic_pass_cannot_be_relabelled_as_windows_proof_within_one_ledger() {
        let mut ledger = ScopedIsolationFindings::default();
        let identity = ProcessIdentity::checked(530, 133400000000000530).unwrap();
        ledger.bind_process(SessionSlot::Mc03, identity).unwrap();
        ledger.record(SessionSlot::Mc03, identity, Check::PrivateRegistryView,
            Finding::Passed(Tier::SyntheticFixture)).unwrap();
        for changed in [Finding::Unknown, Finding::Passed(Tier::WindowsMechanism)] {
            assert_eq!(
                ledger.record(SessionSlot::Mc03, identity, Check::PrivateRegistryView, changed),
                Err(ScopedProofError::ConflictingEvidence)
            );
        }
        assert!(ledger.gaps_for_slot(SessionSlot::Mc03, identity, ProofGoal::WindowsStorageFeasibility)
            .unwrap().contains(&Check::PrivateRegistryView));
        ledger.record(SessionSlot::Mc03, identity, Check::PrivateRegistryView, Finding::Failed)
            .unwrap();
        assert_eq!(
            ledger.record(SessionSlot::Mc03, identity, Check::PrivateRegistryView,
                Finding::Passed(Tier::WindowsMechanism)),
            Err(ScopedProofError::ConflictingEvidence)
        );
    }
    #[test]
    fn exact_evidence_context_cannot_substitute_between_native_and_game() {
        let mut facts = IsolationFindings::default();
        facts.record(Check::PrivateRegistryView, Finding::Passed(Tier::SyntheticFixture));
        facts.record(Check::PrivateRoamingKnownFolder, Finding::Passed(Tier::MinecraftRuntime));
        facts.record(Check::DistinctBrokerAccountContexts, Finding::Passed(Tier::WindowsMechanism));
        assert!(facts.gaps(ProofGoal::WindowsStorageFeasibility)
            .contains(&Check::PrivateRegistryView));
        assert!(facts.gaps(ProofGoal::WindowsStorageFeasibility)
            .contains(&Check::PrivateRoamingKnownFolder));
        assert!(facts.gaps(ProofGoal::TwoMinecraftClients)
            .contains(&Check::DistinctBrokerAccountContexts));
    }

    #[test]
    fn one_fixture_cannot_stand_in_for_three_independent_client_slots() {
        let mut scoped = ScopedIsolationFindings::default();
        let id = ProcessIdentity::checked(410, 133400000000000010).unwrap();
        scoped.bind_process(SessionSlot::Mc02, id).unwrap();
        for &(check, tier) in requirements(ProofGoal::FourMinecraftClients) {
            scoped.record(SessionSlot::Mc02, id, check, Finding::Passed(tier)).unwrap();
        }
        assert!(scoped.gaps_for_slot(SessionSlot::Mc02, id, ProofGoal::FourMinecraftClients)
            .unwrap().is_empty());
        let remaining = scoped.isolated_slot_gaps(ProofGoal::FourMinecraftClients);
        assert!(!remaining.iter().any(|(slot, _)| *slot == SessionSlot::Mc02));
        assert!(remaining.iter().any(|(slot, check)| *slot == SessionSlot::Mc03
            && *check == Check::PrivateRegistryView));
        assert!(remaining.iter().any(|(slot, check)| *slot == SessionSlot::Mc04
            && *check == Check::DistinctBrokerAccountContexts));
    }

    #[test]
    fn pid_reuse_and_process_restart_do_not_inherit_old_proof() {
        let mut scoped = ScopedIsolationFindings::default();
        let old = ProcessIdentity::checked(420, 133400000000000010).unwrap();
        let reused = ProcessIdentity::checked(420, old.created_file_time + 1).unwrap();
        scoped.bind_process(SessionSlot::Mc03, old).unwrap();
        scoped.record(SessionSlot::Mc03, old, Check::PrivateRegistryView,
            Finding::Passed(Tier::WindowsMechanism)).unwrap();
        assert_eq!(
            scoped.bind_process(SessionSlot::Mc03, reused),
            Err(ScopedProofError::ProcessChanged)
        );
        assert_eq!(
            scoped.gaps_for_slot(SessionSlot::Mc03, reused, ProofGoal::WindowsStorageFeasibility),
            Err(ScopedProofError::ProcessChanged)
        );
        assert_eq!(
            scoped.record(SessionSlot::Mc03, reused, Check::PrivateRegistryView,
                Finding::Passed(Tier::WindowsMechanism)),
            Err(ScopedProofError::ProcessChanged)
        );
        // A fresh ephemeral ledger can be created for the new process, but
        // no old observation carries forward implicitly.
        let mut fresh = ScopedIsolationFindings::default();
        fresh.bind_process(SessionSlot::Mc03, reused).unwrap();
        assert!(fresh.gaps_for_slot(SessionSlot::Mc03, reused, ProofGoal::WindowsStorageFeasibility)
            .unwrap().contains(&Check::PrivateRegistryView));
    }

    #[test]
    fn unbound_and_native_slots_cannot_receive_isolated_session_proofs() {
        let mut scoped = ScopedIsolationFindings::default();
        let id = ProcessIdentity::checked(430, 133400000000000010).unwrap();
        assert_eq!(
            scoped.bind_process(SessionSlot::Mc01, id),
            Err(ScopedProofError::NativeSlotNotOwned)
        );
        assert_eq!(
            scoped.record(SessionSlot::Mc02, id, Check::SeparateLicensedSignIns,
                Finding::Passed(Tier::MinecraftRuntime)),
            Err(ScopedProofError::SlotNotBound)
        );
        assert_eq!(
            scoped.bind_process(SessionSlot::Mc02,
                ProcessIdentity { pid: 0, created_file_time: 7 }),
            Err(ScopedProofError::InvalidProcess)
        );
        assert_eq!(
            scoped.bind_process(SessionSlot::Mc02,
                ProcessIdentity { pid: 7, created_file_time: 0 }),
            Err(ScopedProofError::InvalidProcess)
        );
        assert_eq!(
            scoped.gaps_for_slot(SessionSlot::Mc04, id, ProofGoal::TwoMinecraftClients),
            Err(ScopedProofError::SlotNotBound)
        );
    }

    #[test]
    fn successful_regloadappkey_fixture_does_not_clear_registry_or_broker_gaps() {
        let mut scoped = ScopedIsolationFindings::default();
        let id = ProcessIdentity::checked(440, 133400000000000010).unwrap();
        scoped.bind_process(SessionSlot::Mc04, id).unwrap();
        // A lock-refusal may be valid synthetic mechanics, but it is NOT
        // actual per-game registry virtualization or broker separation.
        scoped.record(SessionSlot::Mc04, id, Check::PrivateRegistryView,
            Finding::Passed(Tier::SyntheticFixture)).unwrap();
        scoped.record(SessionSlot::Mc04, id, Check::DistinctBrokerAccountContexts,
            Finding::Passed(Tier::SyntheticFixture)).unwrap();
        let gaps = scoped.gaps_for_slot(SessionSlot::Mc04, id, ProofGoal::TwoMinecraftClients)
            .unwrap();
        assert!(gaps.contains(&Check::PrivateRegistryView));
        assert!(gaps.contains(&Check::DistinctBrokerAccountContexts));
    }
    #[test]
    fn empty_and_failed_observations_fail_closed_at_all_tiers() {
        let mut facts = IsolationFindings::default();
        for goal in [
            ProofGoal::SyntheticIsolation,
            ProofGoal::WindowsStorageFeasibility,
            ProofGoal::TwoMinecraftClients,
            ProofGoal::FourMinecraftClients,
        ] {
            assert!(!facts.gaps(goal).is_empty());
        }
        facts.record(Check::DistinctObjectNamespaces, Finding::Failed);
        assert!(facts.gaps(ProofGoal::SyntheticIsolation).contains(&Check::DistinctObjectNamespaces));
    }

    #[test]
    fn synthetic_fixture_cannot_be_promoted_to_native_or_game_acceptance() {
        let mut facts = IsolationFindings::default();
        for &(check, _) in &SYNTHETIC {
            facts.record(check, Finding::Passed(Tier::SyntheticFixture));
        }
        assert!(facts.gaps(ProofGoal::SyntheticIsolation).is_empty());
        assert!(facts.gaps(ProofGoal::WindowsStorageFeasibility).contains(&Check::DistinctObjectNamespaces));
        assert!(facts.gaps(ProofGoal::WindowsStorageFeasibility).contains(&Check::PrivatePersistentRoaming));
        assert!(!facts.gaps(ProofGoal::FourMinecraftClients).is_empty());
    }

    #[test]
    fn native_storage_is_not_equivalent_to_minecraft_sign_in() {
        let mut facts = IsolationFindings::default();
        for &(check, _) in &WINDOWS {
            facts.record(check, Finding::Passed(Tier::WindowsMechanism));
        }
        assert!(facts.gaps(ProofGoal::WindowsStorageFeasibility).is_empty());
        assert_eq!(facts.gaps(ProofGoal::TwoMinecraftClients), vec![
            Check::IndependentMinecraftLifetimes,
            Check::SeparateLicensedSignIns,
            Check::DistinctBrokerAccountContexts,
            Check::RenderAndInputUsable,
        ]);
    }

    #[test]
    fn even_two_playable_clients_do_not_claim_four_client_acceptance() {
        let mut facts = IsolationFindings::default();
        for &(check, context) in &TWO_GAME {
            facts.record(check, Finding::Passed(context));
        }
        assert!(facts.gaps(ProofGoal::TwoMinecraftClients).is_empty());
        assert_eq!(
            facts.gaps(ProofGoal::FourMinecraftClients),
            vec![Check::FourConcurrentPlayableClients],
        );
        facts.record(
            Check::FourConcurrentPlayableClients,
            Finding::Passed(Tier::SyntheticFixture),
        );
        assert_eq!(
            facts.gaps(ProofGoal::FourMinecraftClients),
            vec![Check::FourConcurrentPlayableClients],
        );
    }
}
