//! Non-authorizing publisher and running-image provenance review.
//!
//! WinVerifyTrust means an on-disk file passed an Authenticode policy,
//! NOT that it is Minecraft Education or the actual image mapped into an
//! independently identified process. Signer fingerprints are untrusted
//! collector assertions until pinned by a verified Windows trust provider.
//! This module has NO Windows APIs, paths, secrets, system writes, provider
//! registration or external network calls.

use super::{ProcessIdentity, StorageEvidenceSource};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RunningImageBinding {
    #[default]
    Unverified,
    QueriedExecutablePathOnly,
    /// Future verified OS observation: exact mapped section/file identity.
    /// Source claims alone are insufficient for authorization.
    SectionFileObjectMatched,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PackageIdentityEvidence {
    #[default]
    Unknown,
    /// Package claim still needs independent signing/ownership review.
    ProcessPackageMetadataObserved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublisherCertificateClaim {
    pub source: StorageEvidenceSource,
    /// Hash of the DER *leaf signer certificate*, not a content file hash,
    /// issuer name, certificate subject string or Windows account ID.
    pub signer_leaf_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PinnedImageProvenanceClaim {
    pub process: ProcessIdentity,
    pub observed_process: Option<ProcessIdentity>,
    pub source: StorageEvidenceSource,
    pub authenticode_trusted_on_disk: bool,
    pub signer_certificate: Option<PublisherCertificateClaim>,
    pub image_binding: RunningImageBinding,
    pub package: PackageIdentityEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageProvenanceError {
    UnpinnedOrStaleProcess,
    WrongEvidenceSource,
    InvalidCertificateDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageProvenanceGap {
    AuthenticodeNotVerified,
    LeafCertificateNotObserved,
    OfficialSignerPolicyNotReviewed,
    CertificateNotOnReviewedList,
    RunningImageNotBoundToVerifiedFile,
    PackageOriginNotReviewed,
}

/// This allowlist MUST originate from reviewed, legitimately obtained
/// official Minecraft Education signed builds. Empty means "NO POLICY".
/// The model does not embed a guessed Microsoft/Mojang certificate hash:
/// real signers can rotate and app-store package trust is a separate issue.
#[derive(Clone, Copy, Debug)]
pub struct ReviewedPublisherPolicy<'a> {
    pub leaf_certificate_sha256: &'a [[u8; 32]],
    pub official_release_origin_reviewed: bool,
}

impl<'a> Default for ReviewedPublisherPolicy<'a> {
    fn default() -> Self {
        Self { leaf_certificate_sha256: &[], official_release_origin_reviewed: false }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingImageProvenance {
    pub process: ProcessIdentity,
    pub gaps: Vec<ImageProvenanceGap>,
}

/// Compare untrusted source claims with an independently reviewed signer
/// policy. Even zero gaps means only a checklist is claimed complete;
/// an actual OS collector, game identity and authorizing admission are
/// separate. Never return a Ready / OfficialPublisherVerified enum.
pub fn review_image_provenance(
    candidate: PinnedImageProvenanceClaim,
    policy: ReviewedPublisherPolicy<'_>,
) -> Result<PendingImageProvenance, ImageProvenanceError> {
    if candidate.process.pid == 0 || candidate.process.created_file_time == 0
        || candidate.observed_process != Some(candidate.process)
    {
        return Err(ImageProvenanceError::UnpinnedOrStaleProcess);
    }
    if candidate.source != StorageEvidenceSource::WindowsProcess {
        return Err(ImageProvenanceError::WrongEvidenceSource);
    }
    if let Some(signer) = candidate.signer_certificate {
        if signer.source != StorageEvidenceSource::WindowsProcess {
            return Err(ImageProvenanceError::WrongEvidenceSource);
        }
        if signer.signer_leaf_sha256 == [0; 32] {
            return Err(ImageProvenanceError::InvalidCertificateDigest);
        }
    }

    let mut gaps = Vec::new();
    if !candidate.authenticode_trusted_on_disk {
        gaps.push(ImageProvenanceGap::AuthenticodeNotVerified);
    }
    if candidate.signer_certificate.is_none() {
        gaps.push(ImageProvenanceGap::LeafCertificateNotObserved);
    }
    if !policy.official_release_origin_reviewed || policy.leaf_certificate_sha256.is_empty() {
        gaps.push(ImageProvenanceGap::OfficialSignerPolicyNotReviewed);
    } else if !candidate.signer_certificate.is_some_and(|signer| {
        policy.leaf_certificate_sha256.contains(&signer.signer_leaf_sha256)
    }) {
        gaps.push(ImageProvenanceGap::CertificateNotOnReviewedList);
    }
    if candidate.image_binding != RunningImageBinding::SectionFileObjectMatched {
        gaps.push(ImageProvenanceGap::RunningImageNotBoundToVerifiedFile);
    }
    if candidate.package != PackageIdentityEvidence::ProcessPackageMetadataObserved {
        gaps.push(ImageProvenanceGap::PackageOriginNotReviewed);
    }
    Ok(PendingImageProvenance { process: candidate.process, gaps })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> ProcessIdentity {
        ProcessIdentity::checked(443, 133400000000000443).unwrap()
    }
    fn candidate() -> PinnedImageProvenanceClaim {
        PinnedImageProvenanceClaim {
            process: key(),
            observed_process: Some(key()),
            source: StorageEvidenceSource::WindowsProcess,
            authenticode_trusted_on_disk: true,
            signer_certificate: None,
            image_binding: RunningImageBinding::QueriedExecutablePathOnly,
            package: PackageIdentityEvidence::Unknown,
        }
    }

    #[test]
    fn generic_trusted_authenticode_never_satisfies_minecraft_publisher_policy() {
        let result = review_image_provenance(
            candidate(), ReviewedPublisherPolicy::default()).unwrap();
        assert!(result.gaps.contains(&ImageProvenanceGap::LeafCertificateNotObserved));
        assert!(result.gaps.contains(&ImageProvenanceGap::OfficialSignerPolicyNotReviewed));
        assert!(result.gaps.contains(&ImageProvenanceGap::RunningImageNotBoundToVerifiedFile));
        assert!(result.gaps.contains(&ImageProvenanceGap::PackageOriginNotReviewed));
    }

    #[test]
    fn unknown_or_failed_signature_is_never_a_positive_attestation() {
        let result = review_image_provenance(PinnedImageProvenanceClaim {
            authenticode_trusted_on_disk: false, ..candidate()
        }, ReviewedPublisherPolicy::default()).unwrap();
        assert!(result.gaps.contains(&ImageProvenanceGap::AuthenticodeNotVerified));
    }

    #[test]
    fn mismatched_stale_or_zero_pid_filetime_fails_closed() {
        for observed_process in [
            None,
            ProcessIdentity::checked(key().pid, key().created_file_time + 1),
            ProcessIdentity::checked(key().pid + 1, key().created_file_time),
        ] {
            assert_eq!(review_image_provenance(PinnedImageProvenanceClaim {
                observed_process, ..candidate()
            }, ReviewedPublisherPolicy::default()),
            Err(ImageProvenanceError::UnpinnedOrStaleProcess));
        }
        let mut bad = candidate();
        bad.process.created_file_time = 0;
        assert_eq!(review_image_provenance(bad, ReviewedPublisherPolicy::default()),
            Err(ImageProvenanceError::UnpinnedOrStaleProcess));
    }

    #[test]
    fn synthetic_certificates_and_empty_fingerprints_are_rejected() {
        for (source, hash, expected) in [
            (StorageEvidenceSource::SyntheticFixture, [7; 32],
                ImageProvenanceError::WrongEvidenceSource),
            (StorageEvidenceSource::WindowsProcess, [0; 32],
                ImageProvenanceError::InvalidCertificateDigest),
        ] {
            assert_eq!(review_image_provenance(PinnedImageProvenanceClaim {
                signer_certificate: Some(PublisherCertificateClaim {
                    source, signer_leaf_sha256: hash,
                }), ..candidate()
            }, ReviewedPublisherPolicy::default()), Err(expected));
        }
    }

    #[test]
    fn unreviewed_signer_fingerprint_never_matches_publisher_claim() {
        let source = PublisherCertificateClaim {
            source: StorageEvidenceSource::WindowsProcess,
            signer_leaf_sha256: [17; 32],
        };
        let report = review_image_provenance(PinnedImageProvenanceClaim {
            signer_certificate: Some(source),
            ..candidate()
        }, ReviewedPublisherPolicy {
            leaf_certificate_sha256: &[[18; 32]],
            official_release_origin_reviewed: true,
        }).unwrap();
        assert!(report.gaps.contains(&ImageProvenanceGap::CertificateNotOnReviewedList));
    }

    #[test]
    fn even_every_matching_claim_returns_only_pending_not_an_admission_grant() {
        let signer = [17; 32];
        let report = review_image_provenance(PinnedImageProvenanceClaim {
            signer_certificate: Some(PublisherCertificateClaim {
                source: StorageEvidenceSource::WindowsProcess,
                signer_leaf_sha256: signer,
            }),
            image_binding: RunningImageBinding::SectionFileObjectMatched,
            package: PackageIdentityEvidence::ProcessPackageMetadataObserved,
            ..candidate()
        }, ReviewedPublisherPolicy {
            leaf_certificate_sha256: &[signer],
            official_release_origin_reviewed: true,
        }).unwrap();
        assert!(report.gaps.is_empty());
        assert_eq!(report.process, key());
        // Claims alone are not an OS signature, publisher or loaded-PE attestation.
    }
}
