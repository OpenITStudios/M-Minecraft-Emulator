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
    IndependentMinecraftLifetimes,
    SeparateLicensedSignIns,
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
            Self::IndependentMinecraftLifetimes => 6,
            Self::SeparateLicensedSignIns => 7,
            Self::RenderAndInputUsable => 8,
            Self::FourConcurrentPlayableClients => 9,
        }
    }
}

/// This is an in-memory proof summary, NOT persistent lifecycle state, a
/// credential cache, an attestation, or permission to launch an executable.
#[derive(Clone, Debug, Default)]
pub struct IsolationFindings {
    entries: [Finding; 10],
}

impl IsolationFindings {
    pub fn record(&mut self, check: IsolationCheck, finding: Finding) {
        self.entries[check.index()] = finding;
    }

    pub fn finding(&self, check: IsolationCheck) -> Finding {
        self.entries[check.index()]
    }

    /// Reports which evidence is absent at the REQUIRED proof tier.
    /// This assessment is diagnostic: it never selects or activates a provider.
    pub fn gaps(&self, goal: ProofGoal) -> Vec<IsolationCheck> {
        requirements(goal)
            .iter()
            .filter_map(|&(check, context)| match self.finding(check) {
                Finding::Passed(observed) if observed >= context => None,
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

const WINDOWS: [Requirement; 6] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
];

const TWO_GAME: [Requirement; 9] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::IndependentMinecraftLifetimes, Tier::MinecraftRuntime),
    (Check::SeparateLicensedSignIns, Tier::MinecraftRuntime),
    (Check::RenderAndInputUsable, Tier::MinecraftRuntime),
];

const FOUR_GAME: [Requirement; 10] = [
    (Check::DistinctObjectNamespaces, Tier::WindowsMechanism),
    (Check::PrivateLocalStorage, Tier::WindowsMechanism),
    (Check::PeerStorageDenied, Tier::WindowsMechanism),
    (Check::HostStorageDenied, Tier::WindowsMechanism),
    (Check::PrivatePersistentRoaming, Tier::WindowsMechanism),
    (Check::IndependentProcessLifetimes, Tier::WindowsMechanism),
    (Check::IndependentMinecraftLifetimes, Tier::MinecraftRuntime),
    (Check::SeparateLicensedSignIns, Tier::MinecraftRuntime),
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

#[cfg(test)]
mod tests {
    use super::*;

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
