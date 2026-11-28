use super::*;

#[test]
fn pair_comparison_rejects_same_slot_or_pid_before_native_calls() {
    let first = ProcessIdentity::checked(71, 133400000000000071).unwrap();
    let second = ProcessIdentity::checked(72, 133400000000000072).unwrap();
    assert_eq!(
        inspect_independent_process_pair_source(
            SessionSlot::Mc02, first, SessionSlot::Mc02, second
        ).unwrap_err(),
        NativePortError::InvalidIdentity
    );
    assert_eq!(
        inspect_independent_process_pair_source(
            SessionSlot::Mc02, first, SessionSlot::Mc03, first
        ).unwrap_err(),
        NativePortError::InvalidIdentity
    );
    let recycled = ProcessIdentity::checked(71, first.created_file_time + 1).unwrap();
    assert_eq!(
        inspect_independent_process_pair_source(
            SessionSlot::Mc02, first, SessionSlot::Mc03, recycled
        ).unwrap_err(),
        NativePortError::InvalidIdentity
    );
    let invalid = ProcessIdentity { pid: 0, created_file_time: 1 };
    assert_eq!(
        inspect_independent_process_pair_source(
            SessionSlot::Mc01, invalid, SessionSlot::Mc02, second
        ).unwrap_err(),
        NativePortError::InvalidIdentity
    );
}

#[cfg(not(windows))]
#[test]
fn pair_comparison_is_unavailable_without_windows_process_handles() {
    let first = ProcessIdentity::checked(71, 133400000000000071).unwrap();
    let second = ProcessIdentity::checked(72, 133400000000000072).unwrap();
    assert_eq!(
        inspect_independent_process_pair_source(
            SessionSlot::Mc01, first, SessionSlot::Mc02, second
        ).unwrap_err(),
        NativePortError::NotImplemented
    );
}



#[test]
fn provenance_policy_remains_pending_even_with_mock_trusted_file_signature() {
    use virtual_clients_core::independent_engine::{
        review_image_provenance, ImageProvenanceGap,
        PackageIdentityEvidence, PinnedImageProvenanceClaim,
        ReviewedPublisherPolicy, RunningImageBinding, StorageEvidenceSource,
    };
    let id = ProcessIdentity::checked(51, 133400000000000051).unwrap();
    let candidate = unpromoted_image_trust(id, true, None);
    let claim = PinnedImageProvenanceClaim {
        process: id,
        observed_process: Some(id),
        source: StorageEvidenceSource::WindowsProcess,
        authenticode_trusted_on_disk: candidate.image_file_signature
            == ImageFileSignatureTrust::AuthenticodeTrustedOnDisk,
        signer_certificate: None,
        image_binding: RunningImageBinding::QueriedExecutablePathOnly,
        package: PackageIdentityEvidence::Unknown,
    };
    let reviewed = review_image_provenance(claim, ReviewedPublisherPolicy::default()).unwrap();
    assert!(reviewed.gaps.contains(&ImageProvenanceGap::LeafCertificateNotObserved));
    assert!(reviewed.gaps.contains(&ImageProvenanceGap::OfficialSignerPolicyNotReviewed));
    assert!(reviewed.gaps.contains(&ImageProvenanceGap::RunningImageNotBoundToVerifiedFile));
}

#[test]
fn invalid_image_provenance_identity_refuses_any_native_inspection() {
    let invalid = ProcessIdentity { pid: 11, created_file_time: 0 };
    assert_eq!(inspect_process_image_provenance_source(invalid),
        Err(NativePortError::InvalidIdentity));
}

#[cfg(not(windows))]
#[test]
fn image_provenance_is_unavailable_without_windows_native_observation() {
    let key = ProcessIdentity::checked(52, 133400000000000052).unwrap();
    assert_eq!(inspect_process_image_provenance_source(key),
        Err(NativePortError::NotImplemented));
}

#[cfg(windows)]
#[test]
fn pending_root_cannot_bind_host_zero_or_stale_process() {
    let valid = ProcessIdentity::checked(700, 133400000000000700).unwrap();
    let observed = ProcessObservation {
        identity: valid,
        containment: ProcessContainment::AppContainerTokenObserved,
        official_image_signature_observed: false,
        visible_window_observed: false,
    };
    for slot in [SessionSlot::Mc01, SessionSlot::Mc02] {
        for invalid in [
            ProcessIdentity { pid: 0, created_file_time: valid.created_file_time },
            ProcessIdentity { pid: valid.pid, created_file_time: 0 },
        ] {
            assert_eq!(
                validate_private_root_observation(slot, invalid, observed, observed),
                Err(PrivateRootObservationError::InvalidSlotOrProcess),
            );
        }
    }
    assert_eq!(
        validate_private_root_observation(SessionSlot::Mc01, valid, observed, observed),
        Err(PrivateRootObservationError::InvalidSlotOrProcess),
    );
    let stale = ProcessObservation {
        identity: ProcessIdentity::checked(valid.pid, valid.created_file_time + 1).unwrap(),
        ..observed
    };
    assert_eq!(
        validate_private_root_observation(SessionSlot::Mc02, valid, observed, stale),
        Err(PrivateRootObservationError::StaleProcess),
    );
}

#[cfg(windows)]
#[test]
fn pending_root_requires_appcontainer_and_stable_observation() {
    let valid = ProcessIdentity::checked(704, 133400000000000704).unwrap();
    let isolated = ProcessObservation {
        identity: valid,
        containment: ProcessContainment::AppContainerTokenObserved,
        official_image_signature_observed: false,
        visible_window_observed: false,
    };
    let normal = ProcessObservation {
        containment: ProcessContainment::NoAppContainerToken,
        ..isolated
    };
    assert_eq!(
        validate_private_root_observation(SessionSlot::Mc02, valid, normal, normal),
        Err(PrivateRootObservationError::NotAppContainer),
    );
    assert_eq!(
        validate_private_root_observation(SessionSlot::Mc03, valid, isolated, normal),
        Err(PrivateRootObservationError::StaleProcess),
    );
    assert!(validate_private_root_observation(
        SessionSlot::Mc04, valid, isolated, isolated
    ).is_ok());
    // A successful predicate is only one data-point, never ACL/owner proof.
}

#[test]
fn trusted_generic_signature_never_identifies_official_publisher() {
    let id = ProcessIdentity::checked(501, 133400000000000501).unwrap();
    for trusted in [false, true] {
        let observation = unpromoted_image_trust(id, trusted, None);
        assert_eq!(observation.process, id);
        assert_eq!(observation.publisher, OfficialGamePublisher::NotAttested);
        assert_eq!(observation.signer_leaf_sha256, None);
        assert_eq!(observation.image_file_signature, if trusted {
            ImageFileSignatureTrust::AuthenticodeTrustedOnDisk
        } else {
            ImageFileSignatureTrust::NotVerified
        });
    }
}


#[test]
fn wintrust_signer_review_fails_closed_on_missing_or_unreleased_state() {
    let hash = [0x51; 32];
    let positive = finish_file_signature_review(true, true, true, Ok(Some(hash)));
    assert_eq!(positive, Ok((true, Some(hash))));
    assert_eq!(
        finish_file_signature_review(true, false, true, Ok(Some(hash))),
        Err("WinTrust did not provide successful verification state"),
    );
    for verified in [true, false] {
        assert_eq!(
            finish_file_signature_review(verified, true, false, Ok(Some(hash))),
            Err("WinTrust provider state could not be closed"),
        );
    }
    for invalid in [Ok(None), Ok(Some([0; 32])), Err(())] {
        assert_eq!(
            finish_file_signature_review(true, true, true, invalid),
            Err("Verified WinTrust signer certificate is unavailable"),
        );
    }
    // Rejected signatures cannot expose an alleged certificate, even if
    // a collector supplied a nonzero hash.
    assert_eq!(
        finish_file_signature_review(false, true, true, Ok(Some(hash))),
        Ok((false, None)),
    );
    assert_eq!(
        finish_file_signature_review(false, false, true, Err(())),
        Ok((false, None)),
    );
}
#[test]
fn leaf_digest_remains_audit_only_with_publisher_and_mapping_unattested() {
    use virtual_clients_core::independent_engine::{
        review_image_provenance, ImageProvenanceGap, PackageIdentityEvidence,
        PinnedImageProvenanceClaim, PublisherCertificateClaim, ReviewedPublisherPolicy,
        RunningImageBinding, StorageEvidenceSource,
    };
    let id = ProcessIdentity::checked(703, 133400000000000703).unwrap();
    let digest = [0x42; 32];
    let candidate = unpromoted_image_trust(id, true, Some(digest));
    assert_eq!(candidate.signer_leaf_sha256, Some(digest));
    assert_eq!(candidate.publisher, OfficialGamePublisher::NotAttested);
    let report = review_image_provenance(PinnedImageProvenanceClaim {
        process: id, observed_process: Some(id),
        source: StorageEvidenceSource::WindowsProcess,
        authenticode_trusted_on_disk: true,
        signer_certificate: Some(PublisherCertificateClaim {
            source: StorageEvidenceSource::WindowsProcess,
            signer_leaf_sha256: digest,
        }),
        image_binding: RunningImageBinding::QueriedExecutablePathOnly,
        package: PackageIdentityEvidence::Unknown,
    }, ReviewedPublisherPolicy::default()).unwrap();
    assert!(!report.gaps.contains(&ImageProvenanceGap::LeafCertificateNotObserved));
    assert!(report.gaps.contains(&ImageProvenanceGap::OfficialSignerPolicyNotReviewed));
    assert!(report.gaps.contains(&ImageProvenanceGap::RunningImageNotBoundToVerifiedFile));
    assert!(report.gaps.contains(&ImageProvenanceGap::PackageOriginNotReviewed));
    assert_eq!(unpromoted_image_trust(id, false, Some(digest)).signer_leaf_sha256, None);
}

#[test]
fn zero_process_identity_refuses_image_trust_inspection() {
    for key in [
        ProcessIdentity { pid: 0, created_file_time: 1 },
        ProcessIdentity { pid: 3, created_file_time: 0 },
    ] {
        assert_eq!(inspect_process_image_trust_source(key),
            Err(NativePortError::InvalidIdentity));
    }
}

#[cfg(not(windows))]
#[test]
fn image_trust_inspection_fails_closed_on_non_windows() {
    let id = ProcessIdentity::checked(11, 133400000000000011).unwrap();
    assert_eq!(inspect_process_image_trust_source(id),
        Err(NativePortError::NotImplemented));
}

#[test]
fn package_family_relation_is_diagnostic_not_a_process_count() {
    use ObservedPackageFamilyRelation as Relation;
    assert_eq!(
        classify_package_family_pair(None, None), Relation::BothUnpackaged
    );
    assert_eq!(
        classify_package_family_pair(Some("Fixture_abcd"), None),
        Relation::OnePackaged
    );
    assert_eq!(
        classify_package_family_pair(None, Some("Fixture_abcd")),
        Relation::OnePackaged
    );
    assert_eq!(
        classify_package_family_pair(
            Some("MinecraftFixture_abcd"), Some("minecraftfixture_ABCD"),
        ),
        Relation::SamePackageFamily
    );
    assert_eq!(
        classify_package_family_pair(
            Some("MinecraftFixture_abcd"), Some("DifferentFixture_abcd"),
        ),
        Relation::DifferentPackageFamilies
    );
}

#[test]
fn package_family_pair_refuses_zero_duplicate_and_reused_process_ids() {
    let first = ProcessIdentity::checked(271, 133400000000000271).unwrap();
    let second = ProcessIdentity::checked(272, 133400000000000272).unwrap();
    for (left, right) in [
        (ProcessIdentity { pid: 0, ..first }, second),
        (first, ProcessIdentity { created_file_time: 0, ..second }),
        (first, first),
        (first, ProcessIdentity { pid: first.pid, ..second }),
    ] {
        assert_eq!(
            inspect_process_package_family_pair_source(left, right),
            Err(NativePortError::InvalidIdentity),
        );
    }
}

#[cfg(not(windows))]
#[test]
fn package_family_pair_requires_windows_native_process_handles() {
    let first = ProcessIdentity::checked(271, 133400000000000271).unwrap();
    let second = ProcessIdentity::checked(272, 133400000000000272).unwrap();
    assert_eq!(
        inspect_process_package_family_pair_source(first, second),
        Err(NativePortError::NotImplemented),
    );
}

#[test]
fn package_form_probe_does_not_promote_unknown_errors_to_unpacked() {
    assert_eq!(classify_package_probe(15700, 0), Ok(None));
    assert_eq!(classify_package_probe(122, 2), Ok(Some(2)));
    assert_eq!(classify_package_probe(122, 1024), Ok(Some(1024)));
    for (status, chars) in [
        (0, 0), (5, 0), (122, 0), (122, 1), (122, 1025),
        (15701, 0),
    ] {
        assert_eq!(
            classify_package_probe(status, chars),
            Err(NativePortError::OperationFailed),
        );
    }
}

#[test]
fn package_form_query_rejects_invalid_pid_filetime_before_windows_io() {
    for identity in [
        ProcessIdentity { pid: 0, created_file_time: 123 },
        ProcessIdentity { pid: 42, created_file_time: 0 },
    ] {
        assert_eq!(
            inspect_process_package_form_source(identity),
            Err(NativePortError::InvalidIdentity),
        );
    }
}

#[cfg(not(windows))]
#[test]
fn package_form_source_unavailable_outside_windows() {
    let identity = ProcessIdentity::checked(42, 123).unwrap();
    assert_eq!(
        inspect_process_package_form_source(identity),
        Err(NativePortError::NotImplemented),
    );
}

#[test]
fn default_pair_boundary_collector_keeps_both_unknown() {
    use virtual_clients_core::independent_engine::{
        PairBoundaryFinding, PairBoundaryObserverPort,
    };
    let first = ProcessIdentity::checked(101, 133400000000000101).unwrap();
    let second = ProcessIdentity::checked(102, 133400000000000102).unwrap();
    let observation = UnknownPairBoundaryObserver
        .observe_pair_boundaries(SessionSlot::Mc02, first, SessionSlot::Mc03, second)
        .unwrap();
    assert_eq!(observation.singleton.finding, PairBoundaryFinding::Unknown);
    assert_eq!(observation.broker.finding, PairBoundaryFinding::Unknown);
}

#[test]
fn invalid_pair_is_rejected_without_native_process_io() {
    use virtual_clients_core::independent_engine::PairBoundaryError;
    let identity = ProcessIdentity::checked(101, 133400000000000101).unwrap();
    assert_eq!(
        inspect_independent_pair_boundaries_source(
            SessionSlot::Mc02, identity, SessionSlot::Mc03, identity,
        ),
        Err(PairBoundaryError::InvalidPair),
    );
}

#[cfg(not(windows))]
#[test]
fn valid_pair_fails_closed_without_native_windows() {
    use virtual_clients_core::independent_engine::PairBoundaryError;
    let first = ProcessIdentity::checked(101, 133400000000000101).unwrap();
    let second = ProcessIdentity::checked(102, 133400000000000102).unwrap();
    assert_eq!(
        inspect_independent_pair_boundaries_source(
            SessionSlot::Mc02, first, SessionSlot::Mc03, second,
        ),
        Err(PairBoundaryError::Native(NativePortError::NotImplemented)),
    );
}

#[test]
fn independent_storage_adapter_cannot_invent_verified_broker_or_roaming() {
    use virtual_clients_core::independent_engine::{
        StorageFinding, StorageSurface,
    };
    let expected = ProcessIdentity::checked(42, 133400000000000000).unwrap();
    let report = UnknownStorageObserver
        .inspect_boundaries(SessionSlot::Mc02, expected)
        .unwrap();
    for surface in StorageSurface::ALL {
        assert_eq!(report.finding(surface), StorageFinding::Unknown);
    }
    assert_eq!(report.unresolved().len(), StorageSurface::ALL.len());
}

#[cfg(not(windows))]
#[test]
fn non_windows_legacy_registry_query_is_unavailable() {
    let process = ProcessIdentity::checked(2, 1234567).unwrap();
    assert_eq!(
        inspect_legacy_token_virtualization_source(process).unwrap_err(),
        NativePortError::NotImplemented,
    );
}

#[test]
fn static_route_denial_has_no_windows_side_effects() {
    use virtual_clients_core::independent_engine::{
        RouteBoundaryInspection, StorageRouteAction, StorageRouteBlocker,
        StorageRouteDomain, StorageRouteRequest, StorageRouteTarget,
    };
    let identity = ProcessIdentity::checked(2, 133400000000000000).unwrap();
    let request = StorageRouteRequest {
        slot: SessionSlot::Mc02,
        expected_process: identity,
        observed_process: None,
        domain: StorageRouteDomain::CurrentUserRegistry,
        target: StorageRouteTarget::HostAccountData,
        action: StorageRouteAction::Read,
    };
    assert_eq!(
        inspect_independent_storage_route_source(request).unwrap(),
        RouteBoundaryInspection::Denied(StorageRouteBlocker::HostAccountBoundary),
    );
}

#[cfg(not(windows))]
#[test]
fn non_windows_source_observation_fails_closed() {
    let process = ProcessIdentity::checked(1, 12345).unwrap();
    assert!(matches!(
        inspect_independent_source(SessionSlot::Mc02, process),
        Err(NativePortError::NotImplemented)
    ));
}

#[test]
fn accepts_full_width_windows_filetime_without_lossy_float_conversion() {
    assert_eq!(
        parse_window_identity(42, Some("133400000000000000"), Some("0x100")),
        Some(WindowIdentity {
            pid: 42,
            created_file_time: 133400000000000000,
            hwnd: 256
        })
    );
    assert_eq!(
        parse_window_identity(42, Some("18446744073709551615"), Some("0xffffffffffffffff"))
            .map(|key| key.created_file_time),
        Some(u64::MAX)
    );
}
#[test]
fn refuses_missing_zero_reused_or_malformed_identity_pieces() {
    for (pid, created, handle) in [
        (0, Some("10"), Some("0x2")),
        (1, None, Some("0x2")),
        (1, Some("0"), Some("0x2")),
        (1, Some("-10"), Some("0x2")),
        (1, Some("10.5"), Some("0x2")),
        (1, Some("18446744073709551616"), Some("0x2")),
        (1, Some("10"), None),
        (1, Some("10"), Some("0x0")),
        (1, Some("10"), Some("2")),
        (1, Some("10"), Some("0x")),
        (1, Some("10"), Some("0xzz")),
    ] {
        assert_eq!(parse_window_identity(pid, created, handle), None);
    }
}
