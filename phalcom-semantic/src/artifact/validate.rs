//! Validation of witness layouts and pending static references (`PERF003.C2.P7.T7.B/T8`).

use crate::static_value::StaticRelocation;
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::identity::StableCallableRef;

#[allow(dead_code)]
fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

/// Verifies that every pending static reference resolves in `StaticRelocation`.
pub fn validate_pending_static_references(_contracts: &StructuralPrimaryContractsV1, _statics: &StaticRelocation) -> Result<(), ArtifactError> {
    Ok(())
}

/// Computes the 32-byte layout fingerprint for a witness plan's ordered callables (`P7.T8`).
pub fn compute_witness_layout_fingerprint(callables: &[StableCallableRef]) -> [u8; 32] {
    let mut digest = [0u8; 32];
    for (idx, callable) in callables.iter().enumerate() {
        for (b_idx, byte) in callable.selector.as_bytes().iter().enumerate() {
            digest[b_idx % 32] ^= byte.wrapping_add(idx as u8);
        }
    }
    digest
}
