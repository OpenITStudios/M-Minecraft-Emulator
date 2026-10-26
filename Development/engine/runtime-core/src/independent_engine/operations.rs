//! Static effect classification for planning and reviews only.
//! This does not authorize execution: existing lifecycle/admission owners
//! revalidate identity, availability and supported provider at action time.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionOperation {
    Inspect,
    Select,
    PlaceWindow,
    RestoreWindow,
    Launch,
    RequestNormalClose,
    ProvisionProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationClass {
    Observation,
    Presentation,
    ProcessLifecycle,
    ProfileMutation,
}

impl SessionOperation {
    pub const ALL: [Self; 7] = [
        Self::Inspect,
        Self::Select,
        Self::PlaceWindow,
        Self::RestoreWindow,
        Self::Launch,
        Self::RequestNormalClose,
        Self::ProvisionProfile,
    ];

    pub const fn class(self) -> OperationClass {
        match self {
            Self::Inspect => OperationClass::Observation,
            Self::Select | Self::PlaceWindow | Self::RestoreWindow => OperationClass::Presentation,
            Self::Launch | Self::RequestNormalClose => OperationClass::ProcessLifecycle,
            Self::ProvisionProfile => OperationClass::ProfileMutation,
        }
    }

    pub const fn requires_existing_process(self) -> bool {
        matches!(
            self,
            Self::Select | Self::PlaceWindow | Self::RestoreWindow | Self::RequestNormalClose
        )
    }

    /// A presentation request MUST NOT implicitly launch, terminate, or
    /// reprovision an unrelated session. No secondary lifecycle manager.
    pub const fn is_presentation_only(self) -> bool {
        matches!(self.class(), OperationClass::Presentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_is_never_a_process_or_profile_mutation() {
        for operation in SessionOperation::ALL {
            if operation.is_presentation_only() {
                assert_eq!(operation.class(), OperationClass::Presentation);
                assert!(operation.requires_existing_process());
            }
        }
        assert_eq!(SessionOperation::Inspect.class(), OperationClass::Observation);
        assert_eq!(SessionOperation::Launch.class(), OperationClass::ProcessLifecycle);
        assert_eq!(SessionOperation::RequestNormalClose.class(), OperationClass::ProcessLifecycle);
        assert_eq!(SessionOperation::ProvisionProfile.class(), OperationClass::ProfileMutation);
        assert_ne!(SessionOperation::Select.class(), SessionOperation::RequestNormalClose.class());
    }
}
